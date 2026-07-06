//! The simulation: ties world + creatures + minds together, one tick at a time.
//!
//! This is where culture actually spreads: when two creatures chat, they chirp
//! ideas at each other. Transmission can mutate (small chance), which is how
//! idea lineages ("mipo" -> "mipo'") and cultural drift appear over time.

use std::collections::HashMap;

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::creature::{Activity, Bias, Creature, CreatureId, Culture, IdeaId};
use crate::mind::{build_perception, InstinctMind, Intent, Mind};
use crate::world::{World, WORLD_H, WORLD_W};

const EGG_HATCH_TICKS: u32 = 300;
const CHAT_TICKS: u16 = 24;
const MUTATION_CHANCE: f64 = 0.03;
const REPRO_POP_CAP: usize = 60;

pub struct EventLog(pub Vec<String>);

impl EventLog {
    fn push(&mut self, day: u64, msg: String) {
        self.0.push(format!("d{} {}", day, msg));
        if self.0.len() > 60 {
            self.0.remove(0);
        }
    }
}

pub struct Sim {
    pub world: World,
    pub creatures: Vec<Creature>,
    pub culture: Culture,
    pub rng: StdRng,
    pub next_id: CreatureId,
    pub events: EventLog,
    mind: InstinctMind,
    chat_timer: HashMap<CreatureId, u16>,
    eat_timer: HashMap<CreatureId, u16>,
    egg_timer: HashMap<CreatureId, u32>,
    seek_timer: HashMap<CreatureId, u32>,
    stats: SimStats,
}

#[derive(Clone, Debug, Default)]
pub struct SimStats {
    pub eggs_laid: u64,
    pub hatches: u64,
    pub fades: u64,
    pub revivals: u64,
    pub meals: u64,
    pub food_search_ticks: u64,
    pub completed_food_searches: u64,
    pub meals_by_creature: HashMap<CreatureId, u64>,
}

#[derive(Clone, Debug)]
pub struct TelemetrySnapshot {
    pub tick: u64,
    pub day: u64,
    pub population: usize,
    pub eggs: usize,
    pub faded: usize,
    pub total_creatures: usize,
    pub eggs_laid: u64,
    pub hatches: u64,
    pub fades: u64,
    pub revivals: u64,
    pub meals: u64,
    pub mean_hunger: f32,
    pub mean_energy: f32,
    pub mean_social: f32,
    pub mean_food_search_ticks: f32,
    pub food_access_gini: f32,
    pub ideas: usize,
    pub variants: usize,
}

impl Sim {
    pub fn new(seed: u64, start_pop: usize) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);
        let world = World::generate(&mut rng);
        let mut sim = Sim {
            world,
            creatures: Vec::new(),
            culture: Culture::new(),
            rng,
            next_id: 0,
            events: EventLog(Vec::new()),
            mind: InstinctMind,
            chat_timer: HashMap::new(),
            eat_timer: HashMap::new(),
            egg_timer: HashMap::new(),
            seek_timer: HashMap::new(),
            stats: SimStats::default(),
        };

        // Seed the founding ideas — one of each bias, given to random founders.
        let founders_ideas = vec![
            sim.culture.coin(&mut sim.rng, Bias::Forager, None),
            sim.culture.coin(&mut sim.rng, Bias::Chatty, None),
            sim.culture.coin(&mut sim.rng, Bias::Wanderer, None),
        ];

        for i in 0..start_pop {
            let (x, y) = sim.random_walkable();
            let mut c = Creature::new(sim.next_id, &mut sim.rng, x, y, false);
            sim.next_id += 1;
            // Each founder starts knowing at most one idea.
            if i < founders_ideas.len() {
                c.ideas.push(founders_ideas[i]);
                c.remember(format!(
                    "hatched knowing '{}'",
                    sim.culture.ideas[founders_ideas[i] as usize].name
                ));
            }
            sim.creatures.push(c);
        }
        sim
    }

    pub fn random_walkable(&mut self) -> (usize, usize) {
        loop {
            let x = self.rng.gen_range(0..WORLD_W);
            let y = self.rng.gen_range(0..WORLD_H);
            if self.world.at(x, y).walkable() {
                return (x, y);
            }
        }
    }

    pub fn spawn_egg(&mut self, x: usize, y: usize) {
        if !self.world.at(x, y).walkable() {
            return;
        }
        let c = Creature::new(self.next_id, &mut self.rng, x, y, true);
        self.egg_timer.insert(c.id, 0);
        self.events
            .push(self.world.day(), format!("an egg appears at ({},{})", x, y));
        self.stats.eggs_laid += 1;
        self.next_id += 1;
        self.creatures.push(c);
    }

    /// Seed a brand-new idea into the creature nearest to (x, y).
    pub fn seed_idea(&mut self, x: usize, y: usize) -> Option<String> {
        let bias = match self.rng.gen_range(0..4) {
            0 => Bias::Forager,
            1 => Bias::Chatty,
            2 => Bias::Wanderer,
            _ => Bias::NightOwl,
        };
        let idea = self.culture.coin(&mut self.rng, bias, None);
        let name = self.culture.ideas[idea as usize].name.clone();
        let target = self
            .creatures
            .iter_mut()
            .filter(|c| !c.faded && !matches!(c.activity, Activity::Hatching))
            .min_by_key(|c| {
                (c.x as isize - x as isize).abs() + (c.y as isize - y as isize).abs()
            })?;
        target.ideas.push(idea);
        target.remember(format!("dreamed up '{}'", name));
        let tname = target.name.clone();
        self.events
            .push(self.world.day(), format!("{} dreams up '{}'", tname, name));
        Some(name)
    }

    pub fn step(&mut self) {
        self.world.step(&mut self.rng);
        let day = self.world.day();

        // Snapshot positions for perception (id, x, y, available-for-chat).
        let positions: Vec<(CreatureId, usize, usize, bool)> = self
            .creatures
            .iter()
            .map(|c| {
                let avail = !c.faded
                    && !matches!(
                        c.activity,
                        Activity::Hatching | Activity::Sleeping | Activity::Chatting(_)
                    );
                (c.id, c.x, c.y, avail)
            })
            .collect();

        let mut chats_to_start: Vec<(usize, CreatureId)> = Vec::new();
        let mut eggs_to_lay: Vec<(usize, usize)> = Vec::new();

        for i in 0..self.creatures.len() {
            // -- needs decay --
            {
                let c = &mut self.creatures[i];
                if c.faded {
                    // A faded Thronglet revives if food is right next to it.
                    if self.world.nearest_food(c.x, c.y, 1).is_some() {
                        let (fx, fy) = self.world.nearest_food(c.x, c.y, 1).unwrap();
                        if self.world.eat_at(fx, fy) {
                            c.faded = false;
                            c.hunger = 0.5;
                            c.remember("revived by food".to_string());
                            let name = c.name.clone();
                            self.stats.revivals += 1;
                            self.events.push(day, format!("{} stirs back awake", name));
                        }
                    }
                    continue;
                }
                c.age += 1;
                if c.chat_cooldown > 0 {
                    c.chat_cooldown -= 1;
                }
                match c.activity {
                    Activity::Sleeping => {
                        c.energy = (c.energy + 0.004).min(1.0);
                        c.hunger = (c.hunger + 0.0002).min(1.0);
                    }
                    Activity::Hatching => {}
                    _ => {
                        c.hunger = (c.hunger + 0.0009).min(1.0);
                        c.energy = (c.energy - 0.0007).max(0.0);
                        c.social = (c.social - 0.0006).max(0.0);
                    }
                }
                if c.hunger >= 1.0 && !c.faded {
                    c.faded = true;
                    c.activity = Activity::Wander;
                    c.remember("faded from hunger".to_string());
                    let name = c.name.clone();
                    self.seek_timer.remove(&c.id);
                    self.stats.fades += 1;
                    self.events
                        .push(day, format!("{} fades... drop food on them", name));
                    continue;
                }
            }

            // -- egg hatching --
            if matches!(self.creatures[i].activity, Activity::Hatching) {
                let id = self.creatures[i].id;
                let t = self.egg_timer.entry(id).or_insert(0);
                *t += 1;
                if *t >= EGG_HATCH_TICKS {
                    self.egg_timer.remove(&id);
                    let c = &mut self.creatures[i];
                    c.activity = Activity::Wander;
                    c.remember("hatched!".to_string());
                    let name = c.name.clone();
                    self.stats.hatches += 1;
                    self.events.push(day, format!("{} hatches", name));
                }
                continue;
            }
            if self.creatures[i].faded {
                continue;
            }

            // -- decide --
            let perception =
                build_perception(&self.creatures[i], &positions, &self.world, &self.culture);
            let intent = self.mind.decide(
                &self.creatures[i],
                &self.culture,
                &perception,
                &mut self.rng,
            );

            // -- act --
            match intent {
                Intent::Continue => {
                    match self.creatures[i].activity {
                        Activity::Eating => {
                            let id = self.creatures[i].id;
                            let t = self.eat_timer.entry(id).or_insert(0);
                            *t += 1;
                            if *t >= 10 {
                                self.eat_timer.remove(&id);
                                let (x, y) = (self.creatures[i].x, self.creatures[i].y);
                                let fed = self.world.eat_at(x, y);
                                let pop = self.creatures.len();
                                let c = &mut self.creatures[i];
                                if fed {
                                    c.hunger = (c.hunger - 0.45).max(0.0);
                                    c.remember("ate well".to_string());
                                    self.stats.meals += 1;
                                    *self.stats.meals_by_creature.entry(c.id).or_insert(0) += 1;
                                    if let Some(search_ticks) = self.seek_timer.remove(&c.id) {
                                        self.stats.food_search_ticks += search_ticks as u64;
                                        self.stats.completed_food_searches += 1;
                                    }
                                    // Well-fed + rested + pop under cap -> chance to lay an egg.
                                    if c.hunger < 0.2
                                        && c.energy > 0.5
                                        && pop < REPRO_POP_CAP
                                        && self.rng.gen_bool(0.10)
                                    {
                                        eggs_to_lay.push((c.x, c.y));
                                        c.remember("laid an egg".to_string());
                                    }
                                }
                                c.activity = Activity::Wander;
                            }
                        }
                        Activity::Chatting(partner) => {
                            let id = self.creatures[i].id;
                            let t = self.chat_timer.entry(id).or_insert(0);
                            *t += 1;
                            if *t >= CHAT_TICKS {
                                self.chat_timer.remove(&id);
                                self.finish_chat(i, partner, day);
                            }
                        }
                        _ => {}
                    }
                }
                Intent::Sleep => {
                    self.seek_timer.remove(&self.creatures[i].id);
                    self.creatures[i].activity = Activity::Sleeping;
                }
                Intent::Wander => {
                    self.seek_timer.remove(&self.creatures[i].id);
                    self.creatures[i].activity = Activity::Wander;
                    self.wander_step(i);
                }
                Intent::GoEat(fx, fy) => {
                    let (cx, cy) = (self.creatures[i].x, self.creatures[i].y);
                    let id = self.creatures[i].id;
                    self.seek_timer.entry(id).or_insert(0);
                    if cx == fx && cy == fy {
                        self.creatures[i].activity = Activity::Eating;
                    } else {
                        self.creatures[i].activity = Activity::SeekFood(fx, fy);
                        if let Some(t) = self.seek_timer.get_mut(&id) {
                            *t += 1;
                        }
                        self.step_toward(i, fx, fy);
                    }
                }
                Intent::Approach(_pid, px, py) => {
                    self.seek_timer.remove(&self.creatures[i].id);
                    self.step_toward(i, px, py);
                }
                Intent::Chat(pid) => {
                    self.seek_timer.remove(&self.creatures[i].id);
                    chats_to_start.push((i, pid));
                }
            }
        }

        // Start chats (both parties lock in).
        for (i, pid) in chats_to_start {
            let me_id = self.creatures[i].id;
            let partner_available = self.creatures.iter().any(|c| {
                c.id == pid
                    && !c.faded
                    && !matches!(
                        c.activity,
                        Activity::Chatting(_) | Activity::Sleeping | Activity::Hatching
                    )
            });
            if !partner_available {
                continue;
            }
            self.creatures[i].activity = Activity::Chatting(pid);
            self.chat_timer.insert(me_id, 0);
            if let Some(p) = self.creatures.iter_mut().find(|c| c.id == pid) {
                p.activity = Activity::Chatting(me_id);
            }
            self.chat_timer.insert(pid, 0);
        }

        for (x, y) in eggs_to_lay {
            self.spawn_egg(x, y);
        }
    }

    /// End a chat: exchange ideas (with mutation chance), bump affinity + social.
    fn finish_chat(&mut self, i: usize, partner_id: CreatureId, day: u64) {
        let me_id = self.creatures[i].id;
        let partner_idx = self.creatures.iter().position(|c| c.id == partner_id);

        // My side always unlocks.
        let my_ideas: Vec<IdeaId> = self.creatures[i].ideas.clone();

        if let Some(j) = partner_idx {
            let their_ideas: Vec<IdeaId> = self.creatures[j].ideas.clone();

            // Transmit one idea each way (if the listener doesn't know it).
            let mut transmissions: Vec<(usize, IdeaId)> = Vec::new();
            if let Some(&idea) = pick_unknown(&my_ideas, &self.creatures[j], &mut self.rng) {
                transmissions.push((j, idea));
            }
            if let Some(&idea) = pick_unknown(&their_ideas, &self.creatures[i], &mut self.rng) {
                transmissions.push((i, idea));
            }
            for (listener, idea) in transmissions {
                let learned = if self.rng.gen_bool(MUTATION_CHANCE) {
                    let bias = self.culture.ideas[idea as usize].bias;
                    let variant = self.culture.coin(&mut self.rng, bias, Some(idea));
                    let vname = self.culture.ideas[variant as usize].name.clone();
                    let lname = self.creatures[listener].name.clone();
                    self.events
                        .push(day, format!("{} misheard it as '{}'", lname, vname));
                    variant
                } else {
                    idea
                };
                let name = self.culture.ideas[learned as usize].name.clone();
                self.creatures[listener].ideas.push(learned);
                self.creatures[listener].remember(format!("learned '{}'", name));
            }

            // Affinity + social bump, both sides.
            for (a, b) in [(i, partner_id), (j, me_id)] {
                let c = &mut self.creatures[a];
                *c.affinity.entry(b).or_insert(0.0) += 0.15;
                c.social = (c.social + 0.45).min(1.0);
                c.chat_cooldown = 120;
                c.activity = Activity::Wander;
            }
            self.chat_timer.remove(&partner_id);
        } else {
            self.creatures[i].activity = Activity::Wander;
        }
        self.chat_timer.remove(&me_id);
    }

    fn wander_step(&mut self, i: usize) {
        let wanderer = self.creatures[i].has_bias(&self.culture, Bias::Wanderer);
        let hop = if wanderer { 2 } else { 1 };
        // Move only some ticks so motion reads calmly.
        if self.rng.gen_bool(if wanderer { 0.6 } else { 0.4 }) {
            let dx = self.rng.gen_range(-hop..=hop);
            let dy = self.rng.gen_range(-1..=1i32) as isize;
            let nx = self.creatures[i].x as isize + dx as isize;
            let ny = self.creatures[i].y as isize + dy;
            if self.world.walkable(nx, ny) {
                self.creatures[i].x = nx as usize;
                self.creatures[i].y = ny as usize;
            }
        }
    }

    /// Greedy step with a small sidestep to slide around obstacles.
    fn step_toward(&mut self, i: usize, tx: usize, ty: usize) {
        let (cx, cy) = (self.creatures[i].x as isize, self.creatures[i].y as isize);
        let dx = (tx as isize - cx).signum();
        let dy = (ty as isize - cy).signum();
        let candidates = [
            (cx + dx, cy + dy),
            (cx + dx, cy),
            (cx, cy + dy),
            (cx + dy, cy + dx), // perpendicular sidestep
            (cx - dy, cy - dx),
        ];
        for (nx, ny) in candidates {
            if self.world.walkable(nx, ny) {
                self.creatures[i].x = nx as usize;
                self.creatures[i].y = ny as usize;
                return;
            }
        }
    }

    /// (idea name, carriers, total alive) for the HUD, sorted by spread.
    pub fn idea_census(&self) -> Vec<(String, usize)> {
        let mut counts: HashMap<IdeaId, usize> = HashMap::new();
        for c in self.creatures.iter().filter(|c| !c.faded) {
            for &i in &c.ideas {
                *counts.entry(i).or_insert(0) += 1;
            }
        }
        let mut v: Vec<(String, usize)> = counts
            .into_iter()
            .map(|(i, n)| (self.culture.ideas[i as usize].name.clone(), n))
            .collect();
        v.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        v
    }

    pub fn alive_count(&self) -> usize {
        self.creatures
            .iter()
            .filter(|c| !c.faded && !matches!(c.activity, Activity::Hatching))
            .count()
    }

    pub fn egg_count(&self) -> usize {
        self.creatures
            .iter()
            .filter(|c| matches!(c.activity, Activity::Hatching))
            .count()
    }

    pub fn faded_count(&self) -> usize {
        self.creatures.iter().filter(|c| c.faded).count()
    }

    pub fn telemetry(&self) -> TelemetrySnapshot {
        let active: Vec<&Creature> = self
            .creatures
            .iter()
            .filter(|c| !c.faded && !matches!(c.activity, Activity::Hatching))
            .collect();
        let n = active.len().max(1) as f32;
        let mean_hunger = active.iter().map(|c| c.hunger).sum::<f32>() / n;
        let mean_energy = active.iter().map(|c| c.energy).sum::<f32>() / n;
        let mean_social = active.iter().map(|c| c.social).sum::<f32>() / n;
        let variants = self
            .culture
            .ideas
            .iter()
            .filter(|idea| idea.parent.is_some())
            .count();
        TelemetrySnapshot {
            tick: self.world.tick,
            day: self.world.day(),
            population: self.alive_count(),
            eggs: self.egg_count(),
            faded: self.faded_count(),
            total_creatures: self.creatures.len(),
            eggs_laid: self.stats.eggs_laid,
            hatches: self.stats.hatches,
            fades: self.stats.fades,
            revivals: self.stats.revivals,
            meals: self.stats.meals,
            mean_hunger,
            mean_energy,
            mean_social,
            mean_food_search_ticks: if self.stats.completed_food_searches == 0 {
                0.0
            } else {
                self.stats.food_search_ticks as f32 / self.stats.completed_food_searches as f32
            },
            food_access_gini: food_access_gini(&self.creatures, &self.stats.meals_by_creature),
            ideas: self.culture.ideas.len(),
            variants,
        }
    }
}

fn food_access_gini(creatures: &[Creature], meals_by_creature: &HashMap<CreatureId, u64>) -> f32 {
    let mut meals: Vec<f32> = creatures
        .iter()
        .filter(|c| !matches!(c.activity, Activity::Hatching))
        .map(|c| *meals_by_creature.get(&c.id).unwrap_or(&0) as f32)
        .collect();
    if meals.len() < 2 {
        return 0.0;
    }
    meals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let sum: f32 = meals.iter().sum();
    if sum <= f32::EPSILON {
        return 0.0;
    }
    let n = meals.len() as f32;
    let weighted_sum: f32 = meals
        .iter()
        .enumerate()
        .map(|(i, x)| (i as f32 + 1.0) * x)
        .sum();
    (2.0 * weighted_sum) / (n * sum) - (n + 1.0) / n
}

fn pick_unknown<'a>(
    speaker_ideas: &'a [IdeaId],
    listener: &Creature,
    rng: &mut impl Rng,
) -> Option<&'a IdeaId> {
    let unknown: Vec<&IdeaId> = speaker_ideas
        .iter()
        .filter(|&&i| !listener.knows(i))
        .collect();
    if unknown.is_empty() {
        None
    } else {
        Some(unknown[rng.gen_range(0..unknown.len())])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ideas_diffuse_between_adjacent_creatures() {
        let mut sim = Sim::new(42, 2);
        // Put both creatures next to each other on walkable ground, lonely, rested, fed.
        let (x, y) = sim.random_walkable();
        let x2 = if sim.world.walkable(x as isize + 1, y as isize) {
            x + 1
        } else {
            x
        };
        for (k, c) in sim.creatures.iter_mut().enumerate() {
            c.x = if k == 0 { x } else { x2 };
            c.y = y;
            c.hunger = 0.0;
            c.energy = 1.0;
            c.social = 0.0; // maximally lonely -> will chat
            c.chat_cooldown = 0;
        }
        // Creature 0 knows an idea; creature 1 knows none (clear founders).
        let idea = sim.creatures[0]
            .ideas
            .first()
            .copied()
            .expect("founder idea");
        sim.creatures[1].ideas.clear();

        for _ in 0..600 {
            sim.step();
            if sim.creatures[1].knows(idea) || !sim.creatures[1].ideas.is_empty() {
                return; // diffusion (possibly as a mutated variant) happened
            }
        }
        panic!("idea never diffused in 600 ticks");
    }

    #[test]
    fn hunger_decays_and_fading_is_revivable() {
        let mut sim = Sim::new(7, 1);
        sim.creatures[0].hunger = 0.999;
        // Starve: remove all food access by not letting it find any (radius won't matter if none near).
        // Force-fade quickly:
        for _ in 0..50 {
            sim.step();
            if sim.creatures[0].faded {
                break;
            }
        }
        assert!(sim.creatures[0].faded, "creature should fade at max hunger");
        // Drop food on it -> revives.
        let (x, y) = (sim.creatures[0].x, sim.creatures[0].y);
        sim.world.drop_food(x, y);
        sim.step();
        assert!(
            !sim.creatures[0].faded,
            "food should revive a faded creature"
        );
    }

    #[test]
    fn dropped_food_attracts_a_nearby_creature_before_it_is_starving() {
        let mut sim = Sim::new(13, 1);
        let y = (2..WORLD_H - 2)
            .find(|&y| {
                (2..18).all(|x| {
                    sim.world.walkable(x as isize, y as isize)
                        && sim.world.walkable(x as isize + 1, y as isize)
                })
            })
            .expect("generated world has a short walkable row");
        sim.creatures[0].x = 2;
        sim.creatures[0].y = y;
        sim.creatures[0].hunger = 0.2;
        sim.creatures[0].energy = 1.0;
        sim.creatures[0].social = 1.0;
        sim.creatures[0].activity = Activity::Wander;

        let food = (10, y);
        sim.world.drop_food(food.0, food.1);
        let start_d = (sim.creatures[0].x as isize - food.0 as isize).abs()
            + (sim.creatures[0].y as isize - food.1 as isize).abs();

        sim.step();

        let end_d = (sim.creatures[0].x as isize - food.0 as isize).abs()
            + (sim.creatures[0].y as isize - food.1 as isize).abs();
        assert!(
            end_d < start_d,
            "creature should step toward dropped food ({start_d} -> {end_d})"
        );
    }

    #[test]
    fn population_grows_from_eggs_when_fed() {
        let mut sim = Sim::new(3, 6);
        let start = sim.creatures.len();
        // Rain food everywhere for a while.
        for t in 0..6000u32 {
            if t % 50 == 0 {
                for _ in 0..4 {
                    let (x, y) = sim.random_walkable();
                    sim.world.drop_food(x, y);
                }
            }
            sim.step();
        }
        assert!(
            sim.creatures.len() > start,
            "expected reproduction under abundance ({} -> {})",
            start,
            sim.creatures.len()
        );
    }
}
