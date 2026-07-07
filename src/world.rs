//! The world: a pixel grid of terrain plus regrowing food.
//!
//! Terrain is generated once at startup: hash-noise grass shading, a random-walk
//! river, clustered trees, rocks hugging the river, and berry bushes that
//! creatures eat from (and which regrow).

use rand::Rng;

pub const WORLD_W: usize = 220;
pub const WORLD_H: usize = 180;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tile {
    Grass(u8), // shade index 0..4
    Water,
    Rock,
    Tree,
    Bush { berries: u8 }, // 0 = picked clean, regrows
}

impl Tile {
    pub fn walkable(&self) -> bool {
        matches!(self, Tile::Grass(_) | Tile::Bush { .. })
    }
}

pub struct World {
    pub tiles: Vec<Tile>,
    pub tick: u64,
    /// Food pellets dropped by the player: (x, y, amount)
    pub pellets: Vec<(usize, usize, u8)>,
}

impl World {
    pub fn generate(rng: &mut impl Rng) -> Self {
        let mut tiles = vec![Tile::Grass(0); WORLD_W * WORLD_H];

        // Grass shading: coarse blotches via a cheap integer hash on cell blocks.
        for y in 0..WORLD_H {
            for x in 0..WORLD_W {
                let bx = x / 6;
                let by = y / 5;
                let h = hash2(bx as u64, by as u64);
                let shade = (h % 4) as u8;
                tiles[y * WORLD_W + x] = Tile::Grass(shade);
            }
        }

        // River: random walk from the top edge to the bottom-left region.
        let mut rx = rng.gen_range(WORLD_W / 4..WORLD_W / 2) as isize;
        let mut y = 0usize;
        while y < WORLD_H {
            let width = 2 + (hash2(y as u64, 7) % 2) as isize;
            for dx in 0..width {
                let x = rx + dx;
                if x >= 0 && (x as usize) < WORLD_W {
                    tiles[y * WORLD_W + x as usize] = Tile::Water;
                }
            }
            // Drift left-ish with jitter, like the reference vibe.
            let step = rng.gen_range(-1..=1) - if y % 5 == 0 { 1 } else { 0 };
            rx = (rx + step).clamp(1, (WORLD_W - 4) as isize);
            y += 1;
        }

        // Rocks: scattered near water.
        let mut rock_budget = 180;
        while rock_budget > 0 {
            let x = rng.gen_range(0..WORLD_W);
            let yy = rng.gen_range(0..WORLD_H);
            if near_water(&tiles, x, yy, 3) && tiles[yy * WORLD_W + x].walkable() {
                tiles[yy * WORLD_W + x] = Tile::Rock;
                rock_budget -= 1;
            } else {
                rock_budget -= 1; // always make progress
            }
        }

        // Trees: clustered clumps, denser toward the map edges.
        for _ in 0..70 {
            let cx = rng.gen_range(0..WORLD_W) as isize;
            let cy = rng.gen_range(0..WORLD_H) as isize;
            let edge_bias =
                cx < 14 || cx > (WORLD_W as isize - 14) || cy < 8 || cy > (WORLD_H as isize - 8);
            let clump = if edge_bias {
                rng.gen_range(6..14)
            } else {
                rng.gen_range(2..6)
            };
            for _ in 0..clump {
                let x = cx + rng.gen_range(-4..=4);
                let yy = cy + rng.gen_range(-3..=3);
                if x >= 0 && yy >= 0 && (x as usize) < WORLD_W && (yy as usize) < WORLD_H {
                    let i = yy as usize * WORLD_W + x as usize;
                    if tiles[i].walkable() {
                        tiles[i] = Tile::Tree;
                    }
                }
            }
        }

        // Berry bushes: on open grass, away from water.
        let mut bushes = 0;
        let mut guard = 0;
        while bushes < 90 && guard < 12000 {
            guard += 1;
            let x = rng.gen_range(2..WORLD_W - 2);
            let yy = rng.gen_range(2..WORLD_H - 2);
            let i = yy * WORLD_W + x;
            if matches!(tiles[i], Tile::Grass(_)) && !near_water(&tiles, x, yy, 2) {
                tiles[i] = Tile::Bush { berries: 3 };
                bushes += 1;
            }
        }

        World {
            tiles,
            tick: 0,
            pellets: Vec::new(),
        }
    }

    pub fn at(&self, x: usize, y: usize) -> Tile {
        self.tiles[y * WORLD_W + x]
    }

    pub fn walkable(&self, x: isize, y: isize) -> bool {
        x >= 0
            && y >= 0
            && (x as usize) < WORLD_W
            && (y as usize) < WORLD_H
            && self.at(x as usize, y as usize).walkable()
    }

    /// One world tick: berry regrowth.
    pub fn step(&mut self, rng: &mut impl Rng) {
        self.tick += 1;
        if self.tick % 40 == 0 {
            // A few random bushes regrow one berry.
            for t in self.tiles.iter_mut() {
                if let Tile::Bush { berries } = t {
                    if *berries < 3 && rng.gen_bool(0.15) {
                        *berries += 1;
                    }
                }
            }
        }
        self.pellets.retain(|p| p.2 > 0);
    }

    /// Fraction of the day, 0.0..1.0. One day = 1200 ticks.
    pub fn time_of_day(&self) -> f32 {
        (self.tick % 1200) as f32 / 1200.0
    }

    pub fn day(&self) -> u64 {
        self.tick / 1200
    }

    /// Nearest food source (bush with berries, or a pellet) within `radius`.
    pub fn nearest_food(&self, x: usize, y: usize, radius: isize) -> Option<(usize, usize)> {
        let mut best: Option<(usize, usize, isize)> =
            self.nearest_pellet_with_distance(x, y, radius);
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let nx = x as isize + dx;
                let ny = y as isize + dy;
                if nx < 0 || ny < 0 || nx as usize >= WORLD_W || ny as usize >= WORLD_H {
                    continue;
                }
                if let Tile::Bush { berries } = self.at(nx as usize, ny as usize) {
                    if berries > 0 {
                        let d = dx.abs() + dy.abs();
                        if best.map_or(true, |b| d < b.2) {
                            best = Some((nx as usize, ny as usize, d));
                        }
                    }
                }
            }
        }
        best.map(|(bx, by, _)| (bx, by))
    }

    pub fn nearest_pellet(&self, x: usize, y: usize, radius: isize) -> Option<(usize, usize)> {
        self.nearest_pellet_with_distance(x, y, radius)
            .map(|(px, py, _)| (px, py))
    }

    fn nearest_pellet_with_distance(
        &self,
        x: usize,
        y: usize,
        radius: isize,
    ) -> Option<(usize, usize, isize)> {
        let mut best: Option<(usize, usize, isize)> = None;
        for (px, py, amt) in &self.pellets {
            if *amt == 0 {
                continue;
            }
            let d = (*px as isize - x as isize).abs() + (*py as isize - y as isize).abs();
            if d <= radius && best.map_or(true, |b| d < b.2) {
                best = Some((*px, *py, d));
            }
        }
        best
    }

    /// Try to consume food at (x, y). Dropped pellets disappear after one meal.
    pub fn eat_at(&mut self, x: usize, y: usize) -> bool {
        for p in self.pellets.iter_mut() {
            if p.0 == x && p.1 == y && p.2 > 0 {
                p.2 = 0;
                return true;
            }
        }
        let i = y * WORLD_W + x;
        if let Tile::Bush { berries } = &mut self.tiles[i] {
            if *berries > 0 {
                *berries -= 1;
                return true;
            }
        }
        false
    }

    pub fn drop_food(&mut self, x: usize, y: usize) {
        if self.at(x, y).walkable() {
            self.pellets.push((x, y, 1));
        }
    }

    pub fn food_units(&self) -> u32 {
        let berries = self
            .tiles
            .iter()
            .map(|tile| match tile {
                Tile::Bush { berries } => *berries as u32,
                _ => 0,
            })
            .sum::<u32>();
        let pellets = self
            .pellets
            .iter()
            .map(|(_, _, amt)| *amt as u32)
            .sum::<u32>();
        berries + pellets
    }
}

fn near_water(tiles: &[Tile], x: usize, y: usize, r: isize) -> bool {
    for dy in -r..=r {
        for dx in -r..=r {
            let nx = x as isize + dx;
            let ny = y as isize + dy;
            if nx >= 0
                && ny >= 0
                && (nx as usize) < WORLD_W
                && (ny as usize) < WORLD_H
                && tiles[ny as usize * WORLD_W + nx as usize] == Tile::Water
            {
                return true;
            }
        }
    }
    false
}

fn hash2(a: u64, b: u64) -> u64 {
    let mut h = a.wrapping_mul(0x9E3779B97F4A7C15) ^ b.wrapping_mul(0xBF58476D1CE4E5B9);
    h ^= h >> 27;
    h = h.wrapping_mul(0x94D049BB133111EB);
    h ^ (h >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn dropped_food_disappears_after_one_meal() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(11);
        let mut world = World::generate(&mut rng);
        let (x, y) = (0..WORLD_H)
            .flat_map(|y| (0..WORLD_W).map(move |x| (x, y)))
            .find(|&(x, y)| world.at(x, y).walkable())
            .expect("generated world has walkable ground");

        world.drop_food(x, y);
        assert_eq!(world.pellets.len(), 1);
        assert_eq!(world.pellets[0].2, 1);

        assert!(world.eat_at(x, y));
        assert_eq!(world.pellets[0].2, 0);
        assert!(world.nearest_food(x, y, 1).is_none());
    }

    #[test]
    fn food_units_counts_berries_and_dropped_food() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(12);
        let mut world = World::generate(&mut rng);
        for tile in world.tiles.iter_mut() {
            if let Tile::Bush { berries } = tile {
                *berries = 0;
            }
        }
        world.pellets.clear();
        let (x, y) = (0..WORLD_H)
            .flat_map(|y| (0..WORLD_W).map(move |x| (x, y)))
            .find(|&(x, y)| world.at(x, y).walkable())
            .expect("generated world has walkable ground");

        assert_eq!(world.food_units(), 0);
        world.drop_food(x, y);
        assert_eq!(world.food_units(), 1);
    }
}
