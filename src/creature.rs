//! Thronglets: the creatures. Needs decay, moods emerge, ideas are carried.
//!
//! Each creature has:
//! - needs (hunger / energy / social) that decay and drive behavior
//! - a tiny episodic memory of recent events (visible in the inspector)
//! - a set of *ideas* it knows — the unit of culture that spreads via chirping
//! - per-creature relationships (affinity toward specific others)

use std::collections::HashMap;

use rand::Rng;

pub type CreatureId = u32;
pub type IdeaId = u16;

/// Behavioral bias an idea confers on its carriers.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Bias {
    Forager,  // seeks food earlier, larger search radius
    Wanderer, // roams farther
    Chatty,   // socializes more eagerly
    NightOwl, // sleeps less at night
}

#[derive(Clone, Debug)]
pub struct Idea {
    pub name: String,
    pub bias: Bias,
    /// idea it mutated from, if any
    pub parent: Option<IdeaId>,
}

/// Global registry so ideas have stable ids and lineage.
pub struct Culture {
    pub ideas: Vec<Idea>,
}

impl Culture {
    pub fn new() -> Self {
        Culture { ideas: Vec::new() }
    }

    pub fn coin(&mut self, rng: &mut impl Rng, bias: Bias, parent: Option<IdeaId>) -> IdeaId {
        let name = match parent {
            Some(p) => format!("{}'", self.ideas[p as usize].name),
            None => babble(rng),
        };
        self.ideas.push(Idea { name, bias, parent });
        (self.ideas.len() - 1) as IdeaId
    }
}

/// Two-syllable chirp-names for ideas, e.g. "mipo", "kelu".
fn babble(rng: &mut impl Rng) -> String {
    const C: &[char] = &['m', 'k', 'p', 'l', 'n', 't', 'w', 'b'];
    const V: &[char] = &['a', 'e', 'i', 'o', 'u'];
    let mut s = String::new();
    for _ in 0..2 {
        s.push(C[rng.gen_range(0..C.len())]);
        s.push(V[rng.gen_range(0..V.len())]);
    }
    s
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Activity {
    Wander,
    SeekFood(usize, usize),
    Eating,
    Chatting(CreatureId),
    Sleeping,
    Hatching, // still an egg
}

#[derive(Clone, Debug)]
pub struct Creature {
    pub id: CreatureId,
    pub name: String,
    pub x: usize,
    pub y: usize,
    pub hunger: f32, // 0 full .. 1 starving
    pub energy: f32, // 1 rested .. 0 exhausted
    pub social: f32, // 1 content .. 0 lonely
    pub age: u32,    // ticks alive
    pub activity: Activity,
    pub ideas: Vec<IdeaId>,
    pub affinity: HashMap<CreatureId, f32>,
    pub memory: Vec<String>, // recent events, capped
    pub chat_cooldown: u16,
    pub faded: bool, // starving too long: dormant, revivable by feeding nearby
}

impl Creature {
    pub fn new(id: CreatureId, rng: &mut impl Rng, x: usize, y: usize, egg: bool) -> Self {
        Creature {
            id,
            name: creature_name(rng),
            x,
            y,
            hunger: 0.25,
            energy: 1.0,
            social: 0.8,
            age: 0,
            activity: if egg {
                Activity::Hatching
            } else {
                Activity::Wander
            },
            ideas: Vec::new(),
            affinity: HashMap::new(),
            memory: Vec::new(),
            chat_cooldown: 0,
            faded: false,
        }
    }

    pub fn remember(&mut self, event: String) {
        self.memory.push(event);
        if self.memory.len() > 8 {
            self.memory.remove(0);
        }
    }

    pub fn knows(&self, idea: IdeaId) -> bool {
        self.ideas.contains(&idea)
    }

    pub fn has_bias(&self, culture: &Culture, bias: Bias) -> bool {
        self.ideas
            .iter()
            .any(|&i| culture.ideas[i as usize].bias == bias)
    }

    pub fn mood(&self) -> &'static str {
        if self.faded {
            return "faded";
        }
        if matches!(self.activity, Activity::Hatching) {
            return "egg";
        }
        if self.hunger > 0.75 {
            "hungry"
        } else if self.energy < 0.25 {
            "sleepy"
        } else if self.social < 0.3 {
            "lonely"
        } else {
            "content"
        }
    }
}

fn creature_name(rng: &mut impl Rng) -> String {
    const A: &[&str] = &[
        "Pip", "Mo", "Kiwi", "Bram", "Sol", "Nia", "Tup", "Lume", "Fen", "Oro", "Wisp", "Juno",
        "Reed", "Isla", "Bix", "Tansy",
    ];
    const B: &[&str] = &["", "", "", "let", "bee", "kin", "by", "loo"];
    format!(
        "{}{}",
        A[rng.gen_range(0..A.len())],
        B[rng.gen_range(0..B.len())]
    )
}
