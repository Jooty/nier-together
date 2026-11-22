use std::collections::HashMap;

use crate::bullet_flights::MAX_EXTRAPOLATION_MS;
use crate::enemy_bullets::SpriteRefusal;
use crate::memory::GameMemory;
use crate::offsets::{
    ACTOR_APPLIED_DISPLACEMENT_OFFSET, ACTOR_OWN_VELOCITY_OFFSET, ACTOR_PENDING_DISPLACEMENT_OFFSET,
};

pub const MAX_TRACED_BULLETS: usize = 3;
pub const MAX_BULLET_TRACE_LINES: u32 = 6000;
pub const BULLET_TRACE_FRAME_GAP_MS: u64 = 50;

const MILLIS_PER_SECOND: f32 = 1000.0;

#[derive(Debug, Default)]
struct TracedBullets {
    slots: Vec<u16>,
    lines: u32,
}

#[derive(Debug, Clone, Copy)]
struct HostSeen {
    first_ms: u64,
    last_ms: u64,
    last_position: [f32; 3],
}

/// What the game's own motion fields of a bullet held at one scan.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BulletMotionFields {
    pub pending_step: [f32; 3],
    pub applied_step: [f32; 3],
    pub own_velocity: [f32; 3],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HostSighting {
    pub slot: u16,
    pub hit_id: i32,
    pub position: [f32; 3],
    pub fields: Option<BulletMotionFields>,
}

/// Up to `MAX_TRACED_BULLETS` bullets the host follows scan by scan: each takes a place when
/// its first message goes out and gives it back when it ends.
#[derive(Debug, Default)]
pub struct HostBulletTrace {
    traced: TracedBullets,
    seen: HashMap<u16, HostSeen>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TracedDraw {
    Taken,
    Refused(SpriteRefusal),
}

#[derive(Debug, Clone, Copy)]
struct GuestSeen {
    last_message_ms: u64,
    messages: u32,
    last_frame_line_ms: u64,
}

/// The guest's side of the same trace: the same rule picks the bullets, since the flights
/// arrive in the order they left.
#[derive(Debug, Default)]
pub struct GuestBulletTrace {
    traced: TracedBullets,
    seen: HashMap<u16, GuestSeen>,
}

impl TracedBullets {
    fn start(&mut self, slot: u16) -> bool {
        if self.slots.len() >= MAX_TRACED_BULLETS || self.slots.contains(&slot) {
            return false;
        }
        self.slots.push(slot);
        true
    }

    fn stop(&mut self, slot: u16) -> bool {
        let before = self.slots.len();
        self.slots.retain(|known| *known != slot);
        self.slots.len() != before
    }

    fn admit_line(&mut self) -> bool {
        if self.lines >= MAX_BULLET_TRACE_LINES {
            return false;
        }
        self.lines += 1;
        true
    }
}

impl HostBulletTrace {
    pub fn start(&mut self, slot: u16, position: [f32; 3], now_ms: u64) {
        if self.traced.start(slot) {
            self.seen.insert(
                slot,
                HostSeen {
                    first_ms: now_ms,
                    last_ms: now_ms,
                    last_position: position,
                },
            );
        }
    }

    pub fn stop(&mut self, slot: u16) {
        if self.traced.stop(slot) {
            self.seen.remove(&slot);
        }
    }

    pub fn slots(&self) -> Vec<u16> {
        self.traced.slots.clone()
    }

    pub fn restart_line_budget(&mut self) {
        self.traced.lines = 0;
    }

    /// The line for one traced bullet at this scan, or `None` for a bullet not traced or once
    /// the line budget is spent.
    pub fn line(&mut self, sighting: &HostSighting, sent: bool, now_ms: u64) -> Option<String> {
        let seen = self.seen.get_mut(&sighting.slot)?;
        let age_ms = now_ms.saturating_sub(seen.first_ms);
        let gap_ms = now_ms.saturating_sub(seen.last_ms);
        let moved: [f32; 3] =
            core::array::from_fn(|axis| sighting.position[axis] - seen.last_position[axis]);
        seen.last_ms = now_ms;
        seen.last_position = sighting.position;
        if !self.traced.admit_line() {
            return None;
        }
        let fields = sighting.fields.map_or_else(
            || "game fields unreadable".to_owned(),
            |fields| {
                format!(
                    "game step {} applied {} velocity {}",
                    lanes_text(fields.pending_step),
                    lanes_text(fields.applied_step),
                    lanes_text(fields.own_velocity)
                )
            },
        );
        Some(format!(
            "bullet trace host: slot {} hit {} {age_ms} ms after its first message at {}, moved {} in {gap_ms} ms ({:.2} m/s), {fields}, {}",
            sighting.slot,
            sighting.hit_id,
            lanes_text(sighting.position),
            lanes_text(moved),
            speed_of(moved, gap_ms),
            if sent {
                "sent now"
            } else {
                "not sent this scan"
            }
        ))
    }
}

impl GuestBulletTrace {
    pub fn start(&mut self, slot: u16, now_ms: u64) {
        if self.traced.start(slot) {
            self.seen.insert(
                slot,
                GuestSeen {
                    last_message_ms: now_ms,
                    messages: 0,
                    last_frame_line_ms: now_ms,
                },
            );
        }
    }

    pub fn is_traced(&self, slot: u16) -> bool {
        self.traced.slots.contains(&slot)
    }

    pub fn slots(&self) -> Vec<u16> {
        self.traced.slots.clone()
    }

    pub fn restart_line_budget(&mut self) {
        self.traced.lines = 0;
    }

    pub fn message(
        &mut self,
        slot: u16,
        now_ms: u64,
        host_position: [f32; 3],
        drawn_position: [f32; 3],
        host_velocity: [f32; 3],
    ) -> Option<String> {
        let seen = self.seen.get_mut(&slot)?;
        let gap_ms = now_ms.saturating_sub(seen.last_message_ms);
        seen.last_message_ms = now_ms;
        seen.messages += 1;
        let messages = seen.messages;
        if !self.traced.admit_line() {
            return None;
        }
        let off_by: [f32; 3] =
            core::array::from_fn(|axis| drawn_position[axis] - host_position[axis]);
        Some(format!(
            "bullet trace guest: slot {slot} message {messages} came {gap_ms} ms after the last, host at {}, copy was drawn at {}, off by {:.3} m, host velocity {} ({:.2} m/s)",
            lanes_text(host_position),
            lanes_text(drawn_position),
            length_of(off_by),
            lanes_text(host_velocity),
            length_of(host_velocity)
        ))
    }

    pub fn frame(
        &mut self,
        slot: u16,
        now_ms: u64,
        position: [f32; 3],
        draw: Option<TracedDraw>,
    ) -> Option<String> {
        let seen = self.seen.get_mut(&slot)?;
        if now_ms.saturating_sub(seen.last_frame_line_ms) < BULLET_TRACE_FRAME_GAP_MS {
            return None;
        }
        seen.last_frame_line_ms = now_ms;
        let age_ms = now_ms.saturating_sub(seen.last_message_ms);
        if !self.traced.admit_line() {
            return None;
        }
        let stopped = if age_ms >= MAX_EXTRAPOLATION_MS {
            ", extrapolation stopped"
        } else {
            ""
        };
        let draw = match draw {
            None => "no draw result yet".to_owned(),
            Some(TracedDraw::Taken) => "the batch took it".to_owned(),
            Some(TracedDraw::Refused(why)) => format!("the batch refused it: {}", why.text()),
        };
        Some(format!(
            "bullet trace guest: slot {slot} shown at {} {age_ms} ms after its last message{stopped}, {draw}",
            lanes_text(position)
        ))
    }

    pub fn ended(&mut self, slot: u16, now_ms: u64, why: &str) -> Option<String> {
        let seen = self.seen.remove(&slot)?;
        self.traced.stop(slot);
        if !self.traced.admit_line() {
            return None;
        }
        let silent_ms = now_ms.saturating_sub(seen.last_message_ms);
        Some(format!(
            "bullet trace guest: slot {slot} {why} after {} messages, {silent_ms} ms after the last",
            seen.messages
        ))
    }
}

pub fn read_bullet_motion_fields<M: GameMemory>(
    memory: &M,
    actor: u64,
) -> Option<BulletMotionFields> {
    Some(BulletMotionFields {
        pending_step: read_lanes(memory, actor + ACTOR_PENDING_DISPLACEMENT_OFFSET)?,
        applied_step: read_lanes(memory, actor + ACTOR_APPLIED_DISPLACEMENT_OFFSET)?,
        own_velocity: read_lanes(memory, actor + ACTOR_OWN_VELOCITY_OFFSET)?,
    })
}

fn read_lanes<M: GameMemory>(memory: &M, address: u64) -> Option<[f32; 3]> {
    Some([
        memory.read_f32(address).ok()?,
        memory.read_f32(address + 4).ok()?,
        memory.read_f32(address + 8).ok()?,
    ])
}

fn lanes_text(lanes: [f32; 3]) -> String {
    format!("({:.3}, {:.3}, {:.3})", lanes[0], lanes[1], lanes[2])
}

fn length_of(lanes: [f32; 3]) -> f32 {
    lanes.iter().map(|lane| lane * lane).sum::<f32>().sqrt()
}

fn speed_of(moved: [f32; 3], gap_ms: u64) -> f32 {
    if gap_ms == 0 {
        return 0.0;
    }
    length_of(moved) * MILLIS_PER_SECOND / gap_ms as f32
}
