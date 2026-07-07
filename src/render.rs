//! Half-block pixel renderer: two vertical world pixels per terminal cell via '▀'.
//! Day/night is a whole-scene color grade, not a sprite swap.

use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget, Wrap};

use crate::creature::Activity;
use crate::sim::Sim;
use crate::theme::{Palette, Rgb};
use crate::world::{Tile, WORLD_H, WORLD_W};

pub struct Camera {
    pub x: usize, // top-left world pixel
    pub y: usize,
}

/// A pixel framebuffer we composite into, then blit as half-blocks.
pub struct Frame {
    pub w: usize,
    pub h: usize,
    px: Vec<(u8, u8, u8)>,
}

impl Frame {
    pub fn new(w: usize, h: usize) -> Self {
        Frame {
            w,
            h,
            px: vec![(0, 0, 0); w * h],
        }
    }

    fn put(&mut self, x: isize, y: isize, c: (u8, u8, u8)) {
        if x >= 0 && y >= 0 && (x as usize) < self.w && (y as usize) < self.h {
            self.px[y as usize * self.w + x as usize] = c;
        }
    }

    fn get(&self, x: usize, y: usize) -> (u8, u8, u8) {
        self.px[y * self.w + x]
    }
}

// -- palette ----------------------------------------------------------------

/// Scene brightness 0.45 (deep night) .. 1.0 (noon), plus a blue cast at night.
fn daylight(t: f32) -> (f32, f32, f32) {
    // t in 0..1; day is 0.2..0.8
    let lum = if (0.25..0.75).contains(&t) {
        1.0
    } else if (0.2..0.25).contains(&t) {
        0.55 + (t - 0.2) / 0.05 * 0.45 // dawn ramp
    } else if (0.75..0.8).contains(&t) {
        1.0 - (t - 0.75) / 0.05 * 0.45 // dusk ramp
    } else {
        0.55
    };
    let blue = if lum < 0.99 { 1.08 } else { 1.0 };
    (lum * 0.95, lum, lum * blue)
}

fn grade(c: (u8, u8, u8), g: (f32, f32, f32)) -> (u8, u8, u8) {
    (
        (c.0 as f32 * g.0).min(255.0) as u8,
        (c.1 as f32 * g.1).min(255.0) as u8,
        (c.2 as f32 * g.2).min(255.0) as u8,
    )
}

fn color(c: Rgb) -> Color {
    Color::Rgb(c.0, c.1, c.2)
}

// -- compositing ------------------------------------------------------------

pub fn compose(
    sim: &Sim,
    cam: &Camera,
    view_w: usize,
    view_h: usize,
    cursor: (usize, usize),
    palette: &Palette,
) -> Frame {
    let mut f = Frame::new(view_w, view_h);
    let g = daylight(sim.world.time_of_day());
    let tick = sim.world.tick;

    // Terrain.
    for vy in 0..view_h {
        for vx in 0..view_w {
            let wx = cam.x + vx;
            let wy = cam.y + vy;
            let c = if wx >= WORLD_W || wy >= WORLD_H {
                let shade = ((wx / 6 + wy / 5) % 4) as u8;
                palette.grass[shade as usize]
            } else {
                match sim.world.at(wx, wy) {
                    Tile::Grass(s) => {
                        if diagonal_path(wx, wy) {
                            palette.grass_path[(wx / 5 + wy / 4) % 2]
                        } else {
                            let stipple =
                                ((wx * 13 + wy * 7 + tick as usize / 5) % 17 == 0) as usize;
                            palette.grass[(s as usize + stipple) % 4]
                        }
                    }
                    Tile::Water => {
                        // gentle shimmer
                        if (wx + wy + (tick / 8) as usize) % 7 == 0 {
                            palette.water_light
                        } else {
                            palette.water
                        }
                    }
                    Tile::Rock => {
                        if (wx + wy) % 3 == 0 {
                            palette.rock_dark
                        } else {
                            palette.rock
                        }
                    }
                    Tile::Tree => {
                        if (wx + wy * 2) % 11 == 0 {
                            palette.tree_trunk
                        } else if (wx * 3 + wy) % 4 == 0 {
                            palette.tree_light
                        } else {
                            palette.tree_canopy
                        }
                    }
                    Tile::Bush { berries } => {
                        if berries > 0 && (wx + wy * 2) % 3 == 0 {
                            palette.berry
                        } else {
                            palette.bush
                        }
                    }
                }
            };
            f.put(vx as isize, vy as isize, grade(c, g));
        }
    }

    // Pellets.
    for &(px, py, amt) in &sim.world.pellets {
        if amt > 0 {
            let vx = px as isize - cam.x as isize;
            let vy = py as isize - cam.y as isize;
            f.put(vx, vy, grade(palette.pellet, g));
            f.put(vx + 1, vy, grade(palette.pellet, g));
            f.put(vx, vy + 1, grade(palette.pellet, g));
        }
    }

    // Creatures: 5x5 sprites, anchored near their center.
    for c in &sim.creatures {
        let vx = c.x as isize - cam.x as isize;
        let vy = c.y as isize - cam.y as isize;
        if matches!(c.activity, Activity::Hatching) {
            draw_egg(&mut f, vx, vy, g, palette);
            continue;
        }
        let creature_palette = if c.faded {
            CreaturePalette {
                body: palette.faded,
                light: palette.faded,
                dark: (83, 87, 92),
                face: (55, 58, 62),
                feet: (82, 92, 98),
            }
        } else {
            CreaturePalette {
                body: palette.body,
                light: palette.body_light,
                dark: palette.body_dark,
                face: palette.face,
                feet: palette.feet,
            }
        };
        let bob = if ((tick / 10) as u32 + c.id) % 2 == 0 {
            0
        } else {
            -1
        };
        draw_creature(&mut f, vx, vy + bob, g, creature_palette, palette.shadow);
        // Status pixel above.
        match c.activity {
            Activity::Sleeping => {
                f.put(vx + 3, vy - 3, grade(palette.zzz, g));
                f.put(vx + 4, vy - 4, grade(palette.zzz, g));
            }
            Activity::Chatting(_) => {
                if (tick / 6) % 2 == 0 {
                    f.put(vx + 2, vy - 3, palette.chirp); // ungraded: chirps glow at night
                }
            }
            _ => {}
        }
    }

    // Cursor: corners of a 3x3 box.
    let (cx, cy) = cursor;
    let vx = cx as isize - cam.x as isize;
    let vy = cy as isize - cam.y as isize;
    for (dx, dy) in [(-1, -1), (2, -1), (-1, 2), (2, 2)] {
        f.put(vx + dx, vy + dy, palette.cursor);
    }

    f
}

fn diagonal_path(x: usize, y: usize) -> bool {
    let band_a = ((x as isize - y as isize * 2 + 28).abs() % 46) < 13;
    let band_b = ((x as isize + y as isize * 2 - 132).abs() % 58) < 10;
    band_a || band_b
}

#[derive(Clone, Copy)]
struct CreaturePalette {
    body: (u8, u8, u8),
    light: (u8, u8, u8),
    dark: (u8, u8, u8),
    face: (u8, u8, u8),
    feet: (u8, u8, u8),
}

fn draw_creature(
    f: &mut Frame,
    x: isize,
    y: isize,
    g: (f32, f32, f32),
    p: CreaturePalette,
    shadow: Rgb,
) {
    for dx in 0..5 {
        f.put(x + dx, y + 4, grade(shadow, g));
    }
    // Ears.
    f.put(x, y, grade(p.light, g));
    f.put(x + 4, y, grade(p.light, g));
    f.put(x + 1, y + 1, grade(p.light, g));
    f.put(x + 3, y + 1, grade(p.light, g));
    // Head and body.
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
        f.put(x + dx, y + dy, grade(color, g));
    }
    // Face and little blue feet.
    f.put(x + 1, y + 2, grade(p.face, g));
    f.put(x + 3, y + 2, grade(p.face, g));
    f.put(x + 1, y + 4, grade(p.feet, g));
    f.put(x + 3, y + 4, grade(p.feet, g));
}

fn draw_egg(f: &mut Frame, x: isize, y: isize, g: (f32, f32, f32), p: &Palette) {
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
        f.put(x + dx, y + dy, grade(color, g));
    }
}

/// Blit a Frame into the ratatui buffer as half-blocks.
pub fn blit(f: &Frame, area: Rect, buf: &mut Buffer) {
    for row in 0..(f.h / 2).min(area.height as usize) {
        for col in 0..f.w.min(area.width as usize) {
            let top = f.get(col, row * 2);
            let bot = f.get(col, row * 2 + 1);
            let cell = buf.get_mut(area.x + col as u16, area.y + row as u16);
            cell.set_symbol("▀");
            cell.set_fg(Color::Rgb(top.0, top.1, top.2));
            cell.set_bg(Color::Rgb(bot.0, bot.1, bot.2));
        }
    }
}

pub fn toolbar(paused: bool, speed: u32, palette: &Palette) -> impl Widget {
    let panel_bg = color(palette.panel_bg);
    let key = Style::default().fg(Color::LightYellow).bg(panel_bg);
    let cyan = Style::default().fg(Color::Cyan).bg(panel_bg);
    let magenta = Style::default().fg(Color::Magenta).bg(panel_bg);
    let white = Style::default().fg(Color::White).bg(panel_bg);
    let green = Style::default().fg(Color::Green).bg(panel_bg);
    let lines = vec![
        Line::from(Span::styled("F food", key)),
        Line::from(Span::styled("E egg", key)),
        Line::from(Span::styled("T idea", magenta)),
        Line::from(Span::styled("t theme", white)),
        Line::from(Span::styled("+ fast", cyan)),
        Line::from(Span::styled("- slow", cyan)),
        Line::from(Span::styled(
            if paused { "space play" } else { "space pause" },
            white,
        )),
        Line::from(Span::styled(format!("x{}", speed), green)),
    ];
    Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(" tools "))
        .style(Style::default().bg(panel_bg))
}

pub fn stat_badge<'a>(sim: &'a Sim, palette: &'a Palette) -> impl Widget + 'a {
    let badge_bg = color(palette.badge_bg);
    let lines = vec![
        Line::from(Span::styled(
            "THRONG",
            Style::default().fg(Color::White).bg(badge_bg),
        )),
        Line::from(Span::styled(
            format!("{}", sim.alive_count()),
            Style::default().fg(Color::LightYellow).bg(badge_bg),
        )),
    ];
    Paragraph::new(lines)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title(" pop "))
        .style(Style::default().bg(badge_bg).fg(Color::Black))
}

pub fn status_panel<'a>(
    sim: &'a Sim,
    cursor: (usize, usize),
    palette: &'a Palette,
) -> impl Widget + 'a {
    let panel_bg = color(palette.panel_bg_2);
    let white = Style::default().fg(Color::White).bg(panel_bg);
    let cyan = Style::default().fg(Color::Cyan).bg(panel_bg);
    let yellow = Style::default().fg(Color::LightYellow).bg(panel_bg);
    let magenta = Style::default().fg(Color::Magenta).bg(panel_bg);
    let near = sim.creatures.iter().min_by_key(|c| {
        (c.x as isize - cursor.0 as isize).abs() + (c.y as isize - cursor.1 as isize).abs()
    });
    let variants = sim
        .culture
        .ideas
        .iter()
        .filter(|idea| idea.parent.is_some())
        .count();
    let mut lines = Vec::new();
    lines.push(Line::from(vec![
        Span::styled(format!("day {}", sim.world.day()), white),
        Span::styled(
            format!(
                "  theme {}  eggs {}  faded {}  food {}  scarcity {:.0}%  ideas {}+{}",
                palette.name,
                sim.egg_count(),
                sim.faded_count(),
                sim.food_units(),
                sim.scarcity_pressure() * 100.0,
                sim.culture.ideas.len().saturating_sub(variants),
                variants
            ),
            cyan,
        ),
    ]));
    if let Some(c) = near {
        let d = (c.x as isize - cursor.0 as isize).abs() + (c.y as isize - cursor.1 as isize).abs();
        if d <= 6 {
            let known: Vec<String> = c
                .ideas
                .iter()
                .take(3)
                .map(|&i| sim.culture.ideas[i as usize].name.clone())
                .collect();
            lines.push(Line::from(vec![
                Span::styled(format!("{} {}", c.name, c.mood()), yellow),
                Span::styled(
                    format!(
                        "  hunger {:.0}% energy {:.0}% social {:.0}%",
                        c.hunger * 100.0,
                        c.energy * 100.0,
                        c.social * 100.0
                    ),
                    white,
                ),
                Span::styled(
                    format!(
                        "  knows {}",
                        if known.is_empty() {
                            "-".to_string()
                        } else {
                            known.join(", ")
                        }
                    ),
                    magenta,
                ),
            ]));
        } else {
            lines.push(Line::from(Span::styled(
                "move the cursor near a Thronglet for details",
                white,
            )));
        }
    }
    Paragraph::new(lines)
        .wrap(Wrap { trim: true })
        .block(Block::default().borders(Borders::ALL).title(" Thronglets "))
        .style(Style::default().bg(panel_bg))
}
