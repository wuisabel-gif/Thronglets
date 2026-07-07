use wasm_bindgen::prelude::*;

use crate::creature::Activity;
use crate::sim::Sim;
use crate::theme::{self, Palette, Rgb};
use crate::world::{Tile, WORLD_H, WORLD_W};

const WEB_VIEW_W: usize = 96;
const WEB_VIEW_H: usize = 64;

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

    pub fn view_width(&self) -> usize {
        WEB_VIEW_W
    }

    pub fn view_height(&self) -> usize {
        WEB_VIEW_H
    }

    pub fn camera_x(&self) -> usize {
        self.camera_origin().0
    }

    pub fn camera_y(&self) -> usize {
        self.camera_origin().1
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

        let mut frame = vec![(0, 0, 0); WEB_VIEW_W * WEB_VIEW_H];
        let (cam_x, cam_y) = self.camera_origin();
        let grade = daylight(self.sim.world.time_of_day());
        let tick = self.sim.world.tick;

        for vy in 0..WEB_VIEW_H {
            for vx in 0..WEB_VIEW_W {
                let wx = cam_x + vx;
                let wy = cam_y + vy;
                let color = terrain_color(&self.sim, wx, wy, tick, self.theme);
                put_frame(
                    &mut frame,
                    vx as isize,
                    vy as isize,
                    apply_grade(color, grade),
                );
            }
        }

        for &(px, py, amt) in &self.sim.world.pellets {
            if amt > 0 {
                let vx = px as isize - cam_x as isize;
                let vy = py as isize - cam_y as isize;
                let pellet = apply_grade(self.theme.pellet, grade);
                put_frame(&mut frame, vx, vy, pellet);
                put_frame(&mut frame, vx + 1, vy, pellet);
                put_frame(&mut frame, vx, vy + 1, pellet);
            }
        }

        for creature in &self.sim.creatures {
            let vx = creature.x as isize - cam_x as isize;
            let vy = creature.y as isize - cam_y as isize;
            if matches!(creature.activity, Activity::Hatching) {
                draw_egg(&mut frame, vx, vy, grade, self.theme);
                continue;
            }
            let creature_palette = if creature.faded {
                CreaturePalette {
                    body: self.theme.faded,
                    light: self.theme.faded,
                    dark: (83, 87, 92),
                    face: (55, 58, 62),
                    feet: (82, 92, 98),
                }
            } else {
                CreaturePalette {
                    body: self.theme.body,
                    light: self.theme.body_light,
                    dark: self.theme.body_dark,
                    face: self.theme.face,
                    feet: self.theme.feet,
                }
            };
            let bob = if ((tick / 10) as u32 + creature.id) % 2 == 0 {
                0
            } else {
                -1
            };
            draw_creature(
                &mut frame,
                vx,
                vy + bob,
                grade,
                creature_palette,
                self.theme.shadow,
            );
            match creature.activity {
                Activity::Sleeping => {
                    put_frame(
                        &mut frame,
                        vx + 3,
                        vy - 3,
                        apply_grade(self.theme.zzz, grade),
                    );
                    put_frame(
                        &mut frame,
                        vx + 4,
                        vy - 4,
                        apply_grade(self.theme.zzz, grade),
                    );
                }
                Activity::Chatting(_) => {
                    if (tick / 6) % 2 == 0 {
                        put_frame(&mut frame, vx + 2, vy - 3, self.theme.chirp);
                    }
                }
                _ => {}
            }
        }

        let cx = self.cursor_x as isize - cam_x as isize;
        let cy = self.cursor_y as isize - cam_y as isize;
        for (dx, dy) in [(-1, -1), (2, -1), (-1, 2), (2, 2)] {
            put_frame(&mut frame, cx + dx, cy + dy, self.theme.cursor);
        }

        for y in 0..height {
            let vy = y * WEB_VIEW_H / height;
            for x in 0..width {
                let vx = x * WEB_VIEW_W / width;
                put_rgba(&mut pixels, width, x, y, frame[vy * WEB_VIEW_W + vx]);
            }
        }
        pixels
    }
}

impl ThrongletsWeb {
    fn camera_origin(&self) -> (usize, usize) {
        let x = self
            .cursor_x
            .saturating_sub(WEB_VIEW_W / 2)
            .min(WORLD_W.saturating_sub(WEB_VIEW_W.min(WORLD_W)));
        let y = self
            .cursor_y
            .saturating_sub(WEB_VIEW_H / 2)
            .min(WORLD_H.saturating_sub(WEB_VIEW_H.min(WORLD_H)));
        (x, y)
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
    if x >= WORLD_W || y >= WORLD_H {
        return palette.grass[(x / 6 + y / 5) % 4];
    }
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

fn put_frame(frame: &mut [Rgb], x: isize, y: isize, c: Rgb) {
    if x >= 0 && y >= 0 && (x as usize) < WEB_VIEW_W && (y as usize) < WEB_VIEW_H {
        frame[y as usize * WEB_VIEW_W + x as usize] = c;
    }
}

#[derive(Clone, Copy)]
struct CreaturePalette {
    body: Rgb,
    light: Rgb,
    dark: Rgb,
    face: Rgb,
    feet: Rgb,
}

fn draw_creature(
    frame: &mut [Rgb],
    x: isize,
    y: isize,
    g: (f32, f32, f32),
    p: CreaturePalette,
    shadow: Rgb,
) {
    for dx in 0..5 {
        put_frame(frame, x + dx, y + 4, apply_grade(shadow, g));
    }
    put_frame(frame, x, y, apply_grade(p.light, g));
    put_frame(frame, x + 4, y, apply_grade(p.light, g));
    put_frame(frame, x + 1, y + 1, apply_grade(p.light, g));
    put_frame(frame, x + 3, y + 1, apply_grade(p.light, g));
    for (dx, dy, color) in [
        (2, 0, p.light),
        (1, 1, p.body),
        (2, 1, p.light),
        (3, 1, p.body),
        (1, 2, p.body),
        (2, 2, p.body),
        (3, 2, p.dark),
        (1, 3, p.dark),
        (2, 3, p.body),
        (3, 3, p.dark),
    ] {
        put_frame(frame, x + dx, y + dy, apply_grade(color, g));
    }
    put_frame(frame, x + 1, y + 2, apply_grade(p.face, g));
    put_frame(frame, x + 3, y + 2, apply_grade(p.face, g));
    put_frame(frame, x + 1, y + 4, apply_grade(p.feet, g));
    put_frame(frame, x + 3, y + 4, apply_grade(p.feet, g));
}

fn draw_egg(frame: &mut [Rgb], x: isize, y: isize, g: (f32, f32, f32), p: &Palette) {
    for (dx, dy, color) in [
        (1, 0, p.egg),
        (2, 0, p.egg),
        (0, 1, p.egg),
        (1, 1, p.egg_spot),
        (2, 1, p.egg),
        (3, 1, p.egg),
        (0, 2, p.egg),
        (1, 2, p.egg),
        (2, 2, p.egg),
        (3, 2, p.egg_spot),
        (1, 3, p.egg),
        (2, 3, p.egg),
    ] {
        put_frame(frame, x + dx, y + dy, apply_grade(color, g));
    }
}
