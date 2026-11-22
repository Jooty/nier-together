use crate::outfit_look::{LOOK_KIND_COUNT, LookKind};

pub const REAPPLY_COOLDOWN_TICKS: u64 = 120;
pub const MOST_TRIES_PER_VALUE: u8 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Attempt {
    value: i32,
    at_tick: u64,
    tries: u8,
}

/// Keeps a step the game does not take from being repeated every frame.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ApplyGuard {
    attempts: [Option<Attempt>; LOOK_KIND_COUNT],
}

impl ApplyGuard {
    pub fn allow(&mut self, kind: LookKind, value: i32, tick: u64) -> bool {
        let slot = &mut self.attempts[kind.index()];
        match slot {
            Some(attempt) if attempt.value == value => {
                if attempt.tries >= MOST_TRIES_PER_VALUE
                    || tick.saturating_sub(attempt.at_tick) < REAPPLY_COOLDOWN_TICKS
                {
                    return false;
                }
                attempt.tries += 1;
                attempt.at_tick = tick;
                true
            }
            _ => {
                *slot = Some(Attempt {
                    value,
                    at_tick: tick,
                    tries: 1,
                });
                true
            }
        }
    }

    pub fn gave_up(&self, kind: LookKind, value: i32) -> bool {
        self.attempts[kind.index()]
            .is_some_and(|attempt| attempt.value == value && attempt.tries >= MOST_TRIES_PER_VALUE)
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }
}
