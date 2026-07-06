# Thronglets

A tiny pixel-art creature society in your terminal, written in pure Rust.

![Thronglets terminal demo](demo.gif)

You watch a top-down half-block-pixel world — grass, a river, trees, berry
bushes — where small bright Thronglets wander, eat, sleep,
chirp at each other, and multiply. When two of them chat, they exchange
*ideas*: named memes ("mipo", "kelu") that bias behavior (foraging, wandering,
chattiness, night-owl sleep). Transmission occasionally mutates an idea into a
variant, so over simulated days you get idea lineages and measurable cultural
drift — visible live in the bottom status panel.

Your role is the caretaker: drop food, place eggs, seed new ideas, and watch
what the society does with them. Neglect them and they fade from hunger
(they don't die — drop food on a faded Thronglet and it stirs back awake).

## Run

```
cargo run --release              # launch the TUI
cargo run --release -- --seed 7  # launch with a different world
cargo run --release -- --theme dusk   # one-run theme override
cargo run --release -- --sound   # launch with bird-like chirps on macOS
cargo run --release -- --headless --ticks 30000   # no rendering, prints diffusion stats
cargo run --release -- --headless --csv --ticks 30000   # daily telemetry CSV
cargo test                       # diffusion / fading / reproduction tests
```

If you install the binary with `cargo install --path .`, launch it later with:

```
thronglets
thronglets --seed 7
thronglets --theme tidepool
thronglets --sound
thronglets --headless --ticks 30000
thronglets --headless --csv --ticks 30000
```

## Sound

Use `--sound` to enable small bird-like chirps. On macOS, Thronglets generates
tiny WAV files in your temp folder and plays them with `afplay`.

Sound cues:

- Population ambience: occasional soft chirps from the colony
- Eating: quick upward chirp when food is eaten
- Hatching: brighter trill when an egg hatches
- Fading: lower falling chirp when a Thronglet fades

## Themes

Press `t` in the TUI to cycle themes with live preview. The selected theme is
saved to `~/.config/thronglets/config.toml` (or
`$XDG_CONFIG_HOME/thronglets/config.toml`) and reused next launch.

Built-in themes: `verdant`, `dusk`, `tidepool`, `amber`.

Use `--theme <name>` for a one-run override without changing the saved theme.

## Headless Telemetry

Use CSV mode to tune the closed-loop sim before changing mechanics:

```
cargo run --release -- --headless --csv --seed 7 --ticks 30000 --start-pop 8
```

It emits one row per simulated day with population, eggs, faded creatures,
births/hatches, fades, meals, mean hunger/energy/social, mean food-search time,
food-access Gini, and idea/variant counts.

For a multi-seed sweep:

```
scripts/sweep.sh > telemetry.csv
SEEDS="0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19" TICKS=30000 scripts/sweep.sh > telemetry.csv
```

## Controls

| key | action |
|---|---|
| arrows | move cursor (camera follows) |
| F | drop food at cursor |
| E | place an egg at cursor |
| T | seed a brand-new idea into the nearest creature |
| t | cycle color theme |
| space | pause |
| + / - | sim speed (up to 16x) |
| Q / esc | quit |

Move the cursor near a creature to open its inspector: needs, mood, friends,
known ideas, and its last few memories.

## What's actually simulated (honest scope)

- **Needs-driven behavior**: hunger / energy / social decay; a utility scorer
  (`InstinctMind`) picks among eat / sleep / socialize / wander each tick.
- **Idea diffusion**: chats transmit one unknown idea each way. ~3% of
  transmissions mutate into a named variant, producing lineages. The status
  panel gives you the diffusion numbers directly.
- **Ideas change behavior**: carriers of a Forager idea seek food earlier and
  see farther; Wanderers roam; Chatty creatures socialize more; NightOwls
  sleep less. So which ideas win *changes how the society behaves*.
- **Reproduction**: well-fed, rested creatures lay eggs (population-capped).
  Eggs hatch after ~a quarter of a day.
- **Day/night**: a whole-scene color grade with dawn/dusk ramps; most
  creatures prefer to sleep at night.

What it is **not** (yet): the creatures don't plan, reflect, or talk in
language — chats are mechanical idea exchange, not dialogue. Emergence here
means measurable information diffusion + behavioral drift, nothing grander.

## Architecture

```
world.rs     terrain gen (hash-noise grass, random-walk river, tree clumps),
             berry regrowth, food pellets, day/night clock
creature.rs  needs, memory (8-event episodic log), ideas, affinity map
mind.rs      the Mind trait: Perception in -> Intent out. InstinctMind = utility scorer
sim.rs       the tick: decay, decide, act, chat resolution, mutation, eggs
render.rs    pixel framebuffer -> '▀' half-block blit, themes, HUD overlays
main.rs      TUI loop (crossterm + ratatui) and --headless mode
```

## Inspiration

The name nods to the Thronglets from *Black Mirror* Season 7, Episode 4,
"Plaything" (released April 10, 2025). This project is only an affectionate
reference: all art here is procedural half-block pixels, the creatures use
Thronglets-specific terminal sprites, and no third-party assets or characters
are used.

MIT.
