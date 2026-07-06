//! The Mind trait: perception in, intent out.
//!
//! The default `InstinctMind` is a utility scorer for lightweight creature
//! behavior. The trait keeps decision-making separate from the simulation loop.

use rand::Rng;

use crate::creature::{Activity, Bias, Creature, CreatureId, Culture};
use crate::world::World;

/// What a creature can perceive in one tick.
pub struct Perception {
    pub nearest_food: Option<(usize, usize)>,
    pub nearest_gift: Option<(usize, usize)>,
    pub nearest_peer: Option<(CreatureId, usize, usize, f32)>, // id, x, y, affinity
    pub is_night: bool,
}

/// What a creature wants to do this tick.
#[derive(Clone, Copy, Debug)]
pub enum Intent {
    Wander,
    GoEat(usize, usize),
    Approach(CreatureId, usize, usize),
    Chat(CreatureId),
    Sleep,
    Continue,
}

pub trait Mind {
    fn decide(
        &mut self,
        me: &Creature,
        culture: &Culture,
        p: &Perception,
        rng: &mut dyn rand::RngCore,
    ) -> Intent;
}

/// Utility-based instincts. Scores a handful of drives and picks the max.
pub struct InstinctMind;

impl Mind for InstinctMind {
    fn decide(
        &mut self,
        me: &Creature,
        culture: &Culture,
        p: &Perception,
        rng: &mut dyn rand::RngCore,
    ) -> Intent {
        // Committed activities run to completion elsewhere; only re-decide when free.
        match me.activity {
            Activity::Eating | Activity::Chatting(_) | Activity::Hatching => {
                return Intent::Continue
            }
            Activity::Sleeping => {
                // Wake when rested (night owls wake earlier).
                let wake_at = if me.has_bias(culture, Bias::NightOwl) {
                    0.55
                } else {
                    0.85
                };
                if me.energy >= wake_at {
                    return Intent::Wander;
                }
                return Intent::Continue;
            }
            _ => {}
        }

        let forager = me.has_bias(culture, Bias::Forager);
        let chatty = me.has_bias(culture, Bias::Chatty);
        let night_owl = me.has_bias(culture, Bias::NightOwl);

        // Drive scores.
        let hunger_threshold = if forager { 0.35 } else { 0.55 };
        let food_target = p.nearest_gift.or(p.nearest_food);
        let gift_score = if p.nearest_gift.is_some() && me.hunger > 0.12 {
            1.15 + me.hunger
        } else {
            0.0
        };
        let food_score = if food_target.is_some() {
            if gift_score > 0.0 {
                gift_score
            } else if me.hunger > hunger_threshold {
                me.hunger * 2.0
            } else {
                0.0
            }
        } else {
            0.0
        };

        let sleep_score = {
            let tired = 1.0 - me.energy;
            let night_bonus = if p.is_night && !night_owl { 0.5 } else { 0.0 };
            if me.energy < 0.3 || (p.is_night && me.energy < 0.6 && !night_owl) {
                tired * 1.5 + night_bonus
            } else {
                0.0
            }
        };

        let social_score = if me.chat_cooldown == 0 {
            if let Some((_, _, _, aff)) = p.nearest_peer {
                let loneliness = 1.0 - me.social;
                let base = if chatty { 0.9 } else { 0.55 };
                loneliness * (base + aff * 0.4)
            } else {
                0.0
            }
        } else {
            0.0
        };

        let wander_score = 0.2 + (rng.gen_range(0u8..10) as f32) / 50.0;

        let mut best = ("wander", wander_score);
        if food_score > best.1 {
            best = ("food", food_score);
        }
        if sleep_score > best.1 {
            best = ("sleep", sleep_score);
        }
        if social_score > best.1 {
            best = ("social", social_score);
        }

        match best.0 {
            "food" => {
                let (fx, fy) = food_target.unwrap();
                Intent::GoEat(fx, fy)
            }
            "sleep" => Intent::Sleep,
            "social" => {
                let (pid, px, py, _) = p.nearest_peer.unwrap();
                let dist =
                    (px as isize - me.x as isize).abs() + (py as isize - me.y as isize).abs();
                if dist <= 2 {
                    Intent::Chat(pid)
                } else {
                    Intent::Approach(pid, px, py)
                }
            }
            _ => Intent::Wander,
        }
    }
}

/// Perception radius depends on carried ideas.
pub fn perception_radius(me: &Creature, culture: &Culture) -> isize {
    let mut r = 12;
    if me.has_bias(culture, Bias::Forager) {
        r += 6;
    }
    if me.has_bias(culture, Bias::Wanderer) {
        r += 3;
    }
    r
}

pub fn build_perception(
    me: &Creature,
    others: &[(CreatureId, usize, usize, bool)], // id, x, y, available
    world: &World,
    culture: &Culture,
) -> Perception {
    let r = perception_radius(me, culture);
    let nearest_gift = world.nearest_pellet(me.x, me.y, r);
    let nearest_food = world.nearest_food(me.x, me.y, r);

    let mut nearest_peer: Option<(CreatureId, usize, usize, f32)> = None;
    let mut best_d = isize::MAX;
    for &(oid, ox, oy, available) in others {
        if oid == me.id || !available {
            continue;
        }
        let d = (ox as isize - me.x as isize).abs() + (oy as isize - me.y as isize).abs();
        if d <= r && d < best_d {
            best_d = d;
            let aff = *me.affinity.get(&oid).unwrap_or(&0.0);
            nearest_peer = Some((oid, ox, oy, aff));
        }
    }

    let t = world.time_of_day();
    let is_night = !(0.2..0.8).contains(&t);

    Perception {
        nearest_food,
        nearest_gift,
        nearest_peer,
        is_night,
    }
}
