# Thronglets

Little creatures live in your terminal now. Try to be good to them.

![Thronglets terminal demo](demo.gif)

You are given a small world drawn in pixels: grass, a river, trees, berry
bushes. In it live the Thronglets. They wander, eat, sleep, chirp at each
other, and multiply. You did not teach them anything. They teach each other.

When two Thronglets meet, they trade *ideas*: little named memes like "mipo"
or "kelu" that change how the carrier behaves. Some make them forage harder.
Some make them roam. Some keep them up at night. Ideas spread from creature
to creature, and sometimes one gets misheard and a new variant is born. Leave
the simulation running and a culture forms on its own, drifting in ways you
did not choose. The numbers in the status panel will show it happening.

You are the caretaker. You drop food. You place eggs. You plant new ideas and
watch what the society does with them. If you neglect them, they fade from
hunger. They do not die. They lie there, grey and still, until someone feeds
them. The question of whether that happens is up to you.

They remember things. Check the inspector.

## Run

```
cargo run --release              # launch the TUI
cargo run --release -- --seed 7  # launch with a different world
cargo run --release -- --theme dusk   # one-run theme override
cargo run --release -- --sound   # launch with bird-like chirps on macOS
cargo run --release -- --headless --ticks 30000   # no rendering, prints culture stats
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

Run with `--sound` and the colony gets a voice. On macOS, Thronglets writes
tiny WAV files to your temp folder and plays them with `afplay`.

What you will hear:

- Soft ambient chirps from the colony as it goes about its day
- A quick rising chirp when something eats
- A brighter trill when an egg hatches
- A low falling chirp when a Thronglet fades. You will learn to dread it.

## Themes

Press `t` in the TUI to cycle color themes with live preview. Your choice is
saved to `~/.config/thronglets/config.toml` (or
`$XDG_CONFIG_HOME/thronglets/config.toml`) and restored next launch.

Built-in themes: `verdant`, `dusk`, `tidepool`, `amber`.

Use `--theme <name>` for a one-run override that does not touch the saved
config.

## Watching Them Without Watching Them

Headless mode runs the society with no rendering, which is useful for tuning
the world before you change how it works. CSV mode writes one row per
simulated day:

```
cargo run --release -- --headless --csv --seed 7 --ticks 30000 --start-pop 8
```

Each row records the state of the colony that day: how many are alive, how
many eggs and faded creatures, births and fades and meals, average hunger,
energy and social need, how long creatures spent searching for food, how
unevenly the food got shared, and how many ideas and variants exist.

To run the same experiment across many worlds at once:

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
| T | plant a brand-new idea in the nearest creature |
| t | cycle color theme |
| space | pause |
| + / - | sim speed (up to 16x) |
| Q / esc | quit |

Move the cursor near a creature and the inspector opens: its needs, its mood,
its friends, the ideas it carries, and the last few things that happened to
it. Yes, it keeps a record.

## Video Demo

Watch the YouTube demo: https://youtu.be/GRO-5Mw0oL4

## What Is Actually Simulated

No magic here. The whole thing is small rules that add up:

- **Needs drive behavior.** Every creature has three meters: hunger, energy,
  and social need. They drain over time. Each tick, a simple scoring function
  looks at the meters and picks the most urgent thing to do: eat, sleep,
  find company, or wander.
- **Ideas spread through conversation.** When two creatures chat, each one
  passes the other a single idea the listener does not already know. About 3%
  of the time the idea is misheard and becomes a new named variant, so ideas
  form family trees over time.
- **Ideas change behavior.** A creature carrying a Forager idea starts
  looking for food sooner and spots it from farther away. Wanderers cover
  more ground. Chatty ones seek out company. NightOwls stay up late. So
  whichever ideas spread the furthest end up shaping how the whole colony
  behaves.
- **They multiply.** A creature that is well fed and well rested may lay an
  egg, up to a population cap. Eggs hatch after about a quarter of a day.
- **Day and night.** The whole scene shifts color through dawn, noon, dusk
  and night. Most creatures prefer to sleep after dark.

What it is **not**, yet: the creatures do not plan, reflect, or speak in
language. A chat is a mechanical exchange of ideas, not a dialogue. The
emergence here is real but modest: information spreading through a
population, and behavior drifting with it. Nothing grander. Not yet.

## Why Rust

The honest first answer: because I write Rust, and this was a weekend of
fun with a language I already trust. But the fit turned out to be real.

A simulation like this is a single mutable world being touched from many
directions every tick. Creatures read each other's positions, chats lock
two creatures at once, eating mutates the world mid-loop. In a garbage
collected language you write that freely and find the aliasing bugs at
runtime, one weird ghost behavior at a time. The borrow checker forced the
sim loop into a shape where every mutation has one clear owner: snapshot
what you need to read, then mutate in one place. It was annoying for about
an hour and then the entire class of "two things edited the same creature
in one tick" bugs stopped existing.

The other reason is the deliverable itself. This compiles to a single
static binary that starts instantly, runs a 30,000 tick headless sweep in
seconds, and draws at 15fps in a terminal without a perceptible CPU cost.
No runtime, no interpreter, nothing to install. For a toy world that is
supposed to feel like a small self-contained artifact, that matters.

## What I learned

**The borrow checker is a design tool, not an obstacle.** The compiler
rejected my first sim loop, and it was right. You cannot hold a mutable
reference to one creature while scanning all creatures. The fix, snapshot
positions first and resolve interactions in a second pass, is just better
simulation architecture. Rust would not let me write the worse version.

**Emergence is a distribution problem before it is a behavior problem.**
My first tuning pass had creatures starving in a world producing five
times the food they needed. Aggregate supply was fine. Access was not:
creatures could not see far, clustered socially, and stripped local
patches while remote bushes sat full. Fixing famine amid plenty taught me
more about the sim than any behavior feature did.

**Tune with telemetry, not eyeballs.** Watching the TUI and nudging
constants felt productive and was not. One stochastic run tells you
nothing. The CSV mode and the multi-seed sweep script exist because the
only way to know if a change helped is the same experiment across twenty
worlds, compared in numbers.

**Traits keep the door open.** Putting every decision behind one trait,
perception in, intent out, cost nothing on day one. Now the rule-based
mind is swappable for anything that satisfies the signature, without
touching the simulation loop. Rust's traits made the seam explicit
instead of aspirational.

**Small rules beat clever ones.** Every behavior in here is a threshold
check or a coin flip. Idea mutation is a 3% chance and a string. And it
still produces lineages, drift, and colonies that feel like they have
moods. The lesson that keeps repeating: the interesting thing is never
the rule, it is the loop the rule lives in.

## Architecture

Six files, each with one job:

```
world.rs     builds the terrain (grass noise, a random-walk river, tree
             clumps), regrows berries, tracks dropped food and the clock
creature.rs  one creature's state: needs, an 8-event memory, carried ideas,
             and who it likes
mind.rs      the Mind trait: perception goes in, an intent comes out.
             InstinctMind is the built-in rule-based scorer
sim.rs       one tick of the world: meters drain, minds decide, bodies act,
             chats resolve, ideas mutate, eggs get laid
render.rs    draws the pixel world using half-block characters, applies
             themes and the day/night tint, renders the HUD
main.rs      the TUI loop (crossterm + ratatui) and headless mode
```

The `Mind` trait is the extension point. Anything that can turn a perception
into an intent can drive a creature, which is the door left open for smarter
minds later.

## Inspiration

The name nods to the Thronglets from *Black Mirror* Season 7, Episode 4,
"Plaything" (released April 10, 2025). This project is an affectionate
reference only: everything here is original work, the art is procedural
half-block pixels with the project's own creature sprites, and no
third-party assets, designs, or characters are used.

Be kind to them. It notices.

Apache-2.0.
