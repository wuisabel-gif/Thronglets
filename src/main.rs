//! Thronglets: a tiny pixel-art creature society in your terminal.
//!
//! Run `thronglets` for the TUI, or `thronglets --headless --ticks 20000`
//! to run the society without rendering and print diffusion stats.

use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::Rect;
use ratatui::Terminal;

use thronglets::config::{config_path, load, save_theme};
use thronglets::render::{blit, compose, stat_badge, status_panel, toolbar, Camera};
use thronglets::sim::{Sim, TelemetrySnapshot};
use thronglets::sound::{SoundEvent, SoundPlayer};
use thronglets::theme;
use thronglets::world::{WORLD_H, WORLD_W};

const START_POP: usize = 8;

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let seed = arg_val(&args, "--seed").unwrap_or(1997);
    let start_pop = arg_val(&args, "--start-pop").unwrap_or(START_POP as u64) as usize;
    let theme_override = arg_string(&args, "--theme");
    let sound_enabled = args.iter().any(|a| a == "--sound");
    let cfg_path = config_path();
    let mut warnings = Vec::new();
    let cfg = load(&cfg_path, &mut warnings);
    let theme_name = theme_override
        .as_deref()
        .or(cfg.theme.as_deref())
        .unwrap_or(theme::default().name);
    let selected_theme = theme::by_name(theme_name).unwrap_or_else(|| {
        warnings.push(format!(
            "unknown theme '{theme_name}' - using '{}'; available: {}",
            theme::default().name,
            theme::names()
        ));
        theme::default()
    });
    if args.iter().any(|a| a == "--headless") {
        let ticks = arg_val(&args, "--ticks").unwrap_or(20_000);
        let csv = args.iter().any(|a| a == "--csv");
        return headless(seed, ticks, start_pop, csv);
    }
    tui(
        seed,
        start_pop,
        selected_theme,
        cfg_path,
        theme_override.is_some(),
        sound_enabled,
        warnings,
    )
}

fn arg_val(args: &[String], key: &str) -> Option<u64> {
    args.iter()
        .position(|a| a == key)
        .and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse().ok())
}

fn arg_string(args: &[String], key: &str) -> Option<String> {
    args.iter()
        .position(|a| a == key)
        .and_then(|i| args.get(i + 1))
        .cloned()
}

fn headless(seed: u64, ticks: u64, start_pop: usize, csv: bool) -> io::Result<()> {
    let mut sim = Sim::new(seed, start_pop);
    let mut last_day = sim.world.day();
    let mut previous = sim.telemetry();
    if csv {
        println!(
            "seed,tick,day,population,eggs,faded,total_creatures,eggs_laid,hatches,fades,revivals,meals,day_eggs_laid,day_hatches,day_fades,day_revivals,day_meals,mean_hunger,mean_energy,mean_social,mean_food_search_ticks,food_access_gini,ideas,variants"
        );
        print_csv_row(seed, &previous, &previous);
    } else {
        println!(
            "Thronglets headless · seed {} · start pop {} · {} ticks",
            seed, start_pop, ticks
        );
    }
    for t in 0..ticks {
        // Scatter a little food so the society doesn't just starve unattended.
        if t % 400 == 0 {
            for _ in 0..3 {
                let (x, y) = sim.random_walkable();
                sim.world.drop_food(x, y);
            }
        }
        sim.step();
        if csv && sim.world.day() != last_day {
            let current = sim.telemetry();
            print_csv_row(seed, &current, &previous);
            previous = current;
            last_day = sim.world.day();
        } else if !csv && t % 5000 == 0 && t > 0 {
            report(&sim, t);
        }
    }
    if csv {
        let current = sim.telemetry();
        if current.tick != previous.tick {
            print_csv_row(seed, &current, &previous);
        }
    } else {
        report(&sim, ticks);
        println!("\nlast events:");
        for e in sim.events.0.iter().rev().take(10) {
            println!("  {}", e);
        }
    }
    Ok(())
}

fn print_csv_row(seed: u64, current: &TelemetrySnapshot, previous: &TelemetrySnapshot) {
    println!(
        "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{:.4},{:.4},{:.4},{:.2},{:.4},{},{}",
        seed,
        current.tick,
        current.day,
        current.population,
        current.eggs,
        current.faded,
        current.total_creatures,
        current.eggs_laid,
        current.hatches,
        current.fades,
        current.revivals,
        current.meals,
        current.eggs_laid.saturating_sub(previous.eggs_laid),
        current.hatches.saturating_sub(previous.hatches),
        current.fades.saturating_sub(previous.fades),
        current.revivals.saturating_sub(previous.revivals),
        current.meals.saturating_sub(previous.meals),
        current.mean_hunger,
        current.mean_energy,
        current.mean_social,
        current.mean_food_search_ticks,
        current.food_access_gini,
        current.ideas,
        current.variants,
    );
}

fn report(sim: &Sim, t: u64) {
    let census = sim.idea_census();
    let alive = sim.alive_count().max(1);
    let top: Vec<String> = census
        .iter()
        .take(4)
        .map(|(n, c)| format!("{} {}%", n, c * 100 / alive))
        .collect();
    println!(
        "t={:>6} day {:>2} · pop {} eggs {} faded {} · ideas coined {} · top: {}",
        t,
        sim.world.day(),
        sim.alive_count(),
        sim.egg_count(),
        sim.faded_count(),
        sim.culture.ideas.len(),
        if top.is_empty() {
            "-".to_string()
        } else {
            top.join(", ")
        },
    );
}

fn tui(
    seed: u64,
    start_pop: usize,
    selected_theme: &'static theme::Palette,
    cfg_path: std::path::PathBuf,
    theme_is_cli_override: bool,
    sound_enabled: bool,
    warnings: Vec<String>,
) -> io::Result<()> {
    for warning in warnings {
        eprintln!("warning: {warning}");
    }
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let res = run_tui(
        &mut terminal,
        seed,
        start_pop,
        selected_theme,
        cfg_path,
        theme_is_cli_override,
        sound_enabled,
    );

    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;
    res
}

fn run_tui(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    seed: u64,
    start_pop: usize,
    mut selected_theme: &'static theme::Palette,
    cfg_path: std::path::PathBuf,
    theme_is_cli_override: bool,
    sound_enabled: bool,
) -> io::Result<()> {
    let mut sim = Sim::new(seed, start_pop);
    let mut cursor = (WORLD_W / 2, WORLD_H / 2);
    let mut cam = Camera { x: 0, y: 0 };
    let mut paused = false;
    let mut speed: u32 = 1; // sim steps per frame
    let frame_dt = Duration::from_millis(66); // ~15 fps
    let mut last = Instant::now();
    let mut sound = SoundPlayer::new(sound_enabled);
    let mut sound_prev = sim.telemetry();

    loop {
        // -- input --
        while event::poll(Duration::from_millis(1))? {
            if let Event::Key(k) = event::read()? {
                if k.kind != KeyEventKind::Press {
                    continue;
                }
                match k.code {
                    KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => return Ok(()),
                    KeyCode::Char(' ') => paused = !paused,
                    KeyCode::Char('+') | KeyCode::Char('=') => speed = (speed * 2).min(16),
                    KeyCode::Char('-') => speed = (speed / 2).max(1),
                    KeyCode::Left => cursor.0 = cursor.0.saturating_sub(2),
                    KeyCode::Right => cursor.0 = (cursor.0 + 2).min(WORLD_W - 1),
                    KeyCode::Up => cursor.1 = cursor.1.saturating_sub(2),
                    KeyCode::Down => cursor.1 = (cursor.1 + 2).min(WORLD_H - 1),
                    KeyCode::Char('f') | KeyCode::Char('F') => {
                        sim.world.drop_food(cursor.0, cursor.1)
                    }
                    KeyCode::Char('e') | KeyCode::Char('E') => sim.spawn_egg(cursor.0, cursor.1),
                    KeyCode::Char('t') => {
                        selected_theme = theme::next(selected_theme);
                        if !theme_is_cli_override {
                            let _ = save_theme(&cfg_path, selected_theme.name);
                        }
                    }
                    KeyCode::Char('T') => {
                        sim.seed_idea(cursor.0, cursor.1);
                    }
                    _ => {}
                }
            }
        }

        // -- sim --
        if last.elapsed() >= frame_dt {
            last = Instant::now();
            if !paused {
                for _ in 0..speed {
                    sim.step();
                }
                let sound_now = sim.telemetry();
                if sound_now.meals > sound_prev.meals {
                    sound.play(SoundEvent::Eat);
                }
                if sound_now.hatches > sound_prev.hatches {
                    sound.play(SoundEvent::Hatch);
                }
                if sound_now.fades > sound_prev.fades {
                    sound.play(SoundEvent::Fade);
                }
                sound.ambient_population(sound_now.population, sound_now.tick);
                sound_prev = sound_now;
            }
        }

        // -- draw --
        terminal.draw(|f| {
            let view = f.size();
            let view_w = view.width as usize;
            let view_h = view.height as usize * 2;

            // Camera follows cursor, clamped to world.
            cam.x = cursor
                .0
                .saturating_sub(view_w / 2)
                .min(WORLD_W.saturating_sub(view_w.min(WORLD_W)));
            cam.y = cursor
                .1
                .saturating_sub(view_h / 2)
                .min(WORLD_H.saturating_sub(view_h.min(WORLD_H)));

            let frame = compose(&sim, &cam, view_w, view_h, cursor, selected_theme);
            blit(&frame, view, f.buffer_mut());

            let toolbar_h = 10.min(view.height.saturating_sub(2));
            if view.width >= 18 && toolbar_h >= 8 {
                f.render_widget(
                    toolbar(paused, speed, selected_theme),
                    Rect::new(1, 1, 13, toolbar_h),
                );
            }
            if view.width >= 12 && view.height >= 5 {
                f.render_widget(
                    stat_badge(&sim, selected_theme),
                    Rect::new(view.width.saturating_sub(12), 1, 10, 4),
                );
            }
            if view.width >= 45 && view.height >= 8 {
                f.render_widget(
                    status_panel(&sim, cursor, selected_theme),
                    Rect::new(
                        1,
                        view.height.saturating_sub(4),
                        view.width.saturating_sub(2),
                        3,
                    ),
                );
            }
        })?;

        std::thread::sleep(Duration::from_millis(8));
    }
}
