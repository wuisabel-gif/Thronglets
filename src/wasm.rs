use wasm_bindgen::prelude::*;

use crate::creature::Activity;
use crate::sim::Sim;
use crate::theme::{self, Palette, Rgb};
use crate::world::{Tile, WORLD_H, WORLD_W};

#[wasm_bindgen]
pub struct ThrongletsWeb {
    sim: Sim,
    cursor_x: usize,
    cursor_y: usize,
    theme: &'static Palette,
}

#[wasm_bindgen]
impl ThrongletsWeb {
    #[wasm_bindgen(constructor)]
    pub fn new(seed: u32, start_pop: usize) -> ThrongletsWeb {
        ThrongletsWeb {
            sim: Sim::new(seed as u64, start_pop),
            cursor_x: WORLD_W / 2,
            cursor_y: WORLD_H / 2,
            theme: theme::default(),
        }
    }

    pub fn world_width(&self) -> usize {
        WORLD_W
    }

    pub fn world_height(&self) -> usize {
        WORLD_H
    }

    pub fn tick(&self) -> u64 {
        self.sim.world.tick
    }

    pub fn population(&self) -> usize {
        self.sim.alive_count()
    }

    pub fn eggs(&self) -> usize {
        self.sim.egg_count()
    }

    pub fn faded(&self) -> usize {
        self.sim.faded_count()
    }

    pub fn ideas(&self) -> usize {
        self.sim.culture.ideas.len()
    }

    pub fn theme_name(&self) -> String {
        self.theme.name.to_string()
    }

    pub fn step(&mut self, ticks: u32) {
        for _ in 0..ticks {
            self.sim.step();
        }
    }

    pub fn move_cursor(&mut self, dx: i32, dy: i32) {
        self.cursor_x = offset_clamped(self.cursor_x, dx, WORLD_W - 1);
        self.cursor_y = offset_clamped(self.cursor_y, dy, WORLD_H - 1);
    }

    pub fn set_cursor(&mut self, x: usize, y: usize) {
        self.cursor_x = x.min(WORLD_W - 1);
        self.cursor_y = y.min(WORLD_H - 1);
    }

    pub fn drop_food(&mut self) {
        self.sim.world.drop_food(self.cursor_x, self.cursor_y);
    }

    pub fn place_egg(&mut self) {
        self.sim.spawn_egg(self.cursor_x, self.cursor_y);
    }

    pub fn seed_idea(&mut self) {
        self.sim.seed_idea(self.cursor_x, self.cursor_y);
    }

    pub fn next_theme(&mut self) {
        self.theme = theme::next(self.theme);
    }

    pub fn render_rgba(&self, width: usize, height: usize) -> Vec<u8> {
        let mut pixels = vec![0; width.saturating_mul(height).saturating_mul(4)];
        if width == 0 || height == 0 {
            return pixels;
        }

        let scale_x = WORLD_W as f32 / width as f32;
        let scale_y = WORLD_H as f32 / height as f32;
        let grade = daylight(self.sim.world.time_of_day());
        let tick = self.sim.world.tick;

        for y in 0..height {
            for x in 0..width {
                let wx = ((x as f32 * scale_x) as usize).min(WORLD_W - 1);
                let wy = ((y as f32 * scale_y) as usize).min(WORLD_H - 1);
                let color = terrain_color(&self.sim, wx, wy, tick, self.theme);
                put_rgba(&mut pixels, width, x, y, apply_grade(color, grade));
            }
        }

        for &(px, py, amt) in &self.sim.world.pellets {
            if amt > 0 {
                draw_block(&mut pixels, width, height, px, py, self.theme.pellet, 4);
            }
        }

        for creature in &self.sim.creatures {
            if matches!(creature.activity, Activity::Hatching) {
                draw_block(
                    &mut pixels,
                    width,
                    height,
                    creature.x,
                    creature.y,
                    self.theme.egg,
                    6,
                );
                continue;
            }
            let body = if creature.faded {
                self.theme.faded
            } else {
                self.theme.body
            };
            let feet = if creature.faded {
                self.theme.faded
            } else {
                self.theme.feet
            };
            draw_block(&mut pixels, width, height, creature.x, creature.y, body, 6);
            draw_block_offset(
                &mut pixels,
                width,
                height,
                creature.x,
                creature.y,
                feet,
                1,
                4,
                2,
            );
        }

        draw_cursor(
            &mut pixels,
            width,
            height,
            self.cursor_x,
            self.cursor_y,
            self.theme.cursor,
        );
        pixels
    }
}

fn offset_clamped(value: usize, delta: i32, max: usize) -> usize {
    if delta < 0 {
        value.saturating_sub(delta.unsigned_abs() as usize)
    } else {
        value.saturating_add(delta as usize).min(max)
    }
}

fn terrain_color(sim: &Sim, x: usize, y: usize, tick: u64, palette: &Palette) -> Rgb {
    match sim.world.at(x, y) {
        Tile::Grass(s) => {
            if diagonal_path(x, y) {
                palette.grass_path[(x / 5 + y / 4) % 2]
            } else {
                let stipple = ((x * 13 + y * 7 + tick as usize / 5) % 17 == 0) as usize;
                palette.grass[(s as usize + stipple) % 4]
            }
        }
        Tile::Water => {
            if (x + y + (tick / 8) as usize) % 7 == 0 {
                palette.water_light
            } else {
                palette.water
            }
        }
        Tile::Rock => {
            if (x + y) % 3 == 0 {
                palette.rock_dark
            } else {
                palette.rock
            }
        }
        Tile::Tree => {
            if (x + y * 2) % 11 == 0 {
                palette.tree_trunk
            } else if (x * 3 + y) % 4 == 0 {
                palette.tree_light
            } else {
                palette.tree_canopy
            }
        }
        Tile::Bush { berries } => {
            if berries > 0 && (x + y * 2) % 3 == 0 {
                palette.berry
            } else {
                palette.bush
            }
        }
    }
}

fn diagonal_path(x: usize, y: usize) -> bool {
    let band_a = ((x as isize - y as isize * 2 + 28).abs() % 46) < 13;
    let band_b = ((x as isize + y as isize * 2 - 132).abs() % 58) < 10;
    band_a || band_b
}

fn daylight(t: f32) -> (f32, f32, f32) {
    let lum = if (0.25..0.75).contains(&t) {
        1.0
    } else if (0.2..0.25).contains(&t) {
        0.55 + (t - 0.2) / 0.05 * 0.45
    } else if (0.75..0.8).contains(&t) {
        1.0 - (t - 0.75) / 0.05 * 0.45
    } else {
        0.55
    };
    let blue = if lum < 0.99 { 1.08 } else { 1.0 };
    (lum * 0.95, lum, lum * blue)
}

fn apply_grade(c: Rgb, g: (f32, f32, f32)) -> Rgb {
    (
        (c.0 as f32 * g.0).min(255.0) as u8,
        (c.1 as f32 * g.1).min(255.0) as u8,
        (c.2 as f32 * g.2).min(255.0) as u8,
    )
}

fn put_rgba(pixels: &mut [u8], width: usize, x: usize, y: usize, c: Rgb) {
    let i = (y * width + x) * 4;
    if i + 3 < pixels.len() {
        pixels[i] = c.0;
        pixels[i + 1] = c.1;
        pixels[i + 2] = c.2;
        pixels[i + 3] = 255;
    }
}

fn draw_block(
    pixels: &mut [u8],
    width: usize,
    height: usize,
    wx: usize,
    wy: usize,
    c: Rgb,
    size: usize,
) {
    draw_block_offset(pixels, width, height, wx, wy, c, 0, 0, size);
}

fn draw_block_offset(
    pixels: &mut [u8],
    width: usize,
    height: usize,
    wx: usize,
    wy: usize,
    c: Rgb,
    ox: usize,
    oy: usize,
    size: usize,
) {
    let sx = wx * width / WORLD_W + ox;
    let sy = wy * height / WORLD_H + oy;
    for y in sy..(sy + size).min(height) {
        for x in sx..(sx + size).min(width) {
            put_rgba(pixels, width, x, y, c);
        }
    }
}

fn draw_cursor(pixels: &mut [u8], width: usize, height: usize, wx: usize, wy: usize, c: Rgb) {
    let sx = wx * width / WORLD_W;
    let sy = wy * height / WORLD_H;
    for i in 0..8 {
        if sx + i < width {
            put_rgba(pixels, width, sx + i, sy, c);
            put_rgba(pixels, width, sx + i, (sy + 8).min(height - 1), c);
        }
        if sy + i < height {
            put_rgba(pixels, width, sx, sy + i, c);
            put_rgba(pixels, width, (sx + 8).min(width - 1), sy + i, c);
        }
    }
}
