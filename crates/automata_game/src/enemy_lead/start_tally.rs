use std::collections::HashMap;

use crate::enemies::EnemyId;
use crate::enemy_lead::{MOST_STARTS_QUEUED_PER_ENEMY, QueuedStart};

/// The host's count of state starts per shared enemy. A start the state start hook saw counts
/// with the arguments it was made with; without one, a change of the state id read from the
/// enemy counts as a start of what was read (a class whose own state start the hook misses).
#[derive(Debug, Clone, Default)]
pub struct StartTally {
    enemies: HashMap<EnemyId, Tallied>,
}

/// A state start as the state start hook saw it: how many it has seen for that body, and the
/// arguments of the last.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HookedStarts {
    pub count: u32,
    pub last: [i32; 4],
    pub recent: [HookedStart; MOST_STARTS_QUEUED_PER_ENEMY],
}

/// One start the state start hook saw: its arguments and the wall clock when it saw it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HookedStart {
    pub state: [i32; 4],
    pub at_ms: u64,
}

#[derive(Debug, Clone, Copy)]
struct PendingStart {
    number: u32,
    state: [i32; 4],
    at_ms: u64,
}

#[derive(Debug, Clone)]
struct Tallied {
    pending: Vec<PendingStart>,
    state_id_read: i32,
    hooked_count: Option<u32>,
    starts: u32,
    sent: [i32; 4],
}

impl HookedStarts {
    /// Counts one more start; the newest of `recent` comes first.
    pub fn with_start(self, state: [i32; 4], at_ms: u64) -> Self {
        let mut recent = [HookedStart::default(); MOST_STARTS_QUEUED_PER_ENEMY];
        recent[0] = HookedStart { state, at_ms };
        recent[1..].copy_from_slice(&self.recent[..MOST_STARTS_QUEUED_PER_ENEMY - 1]);
        Self {
            count: self.count.wrapping_add(1),
            last: state,
            recent,
        }
    }

    pub fn first(state: [i32; 4], at_ms: u64) -> Self {
        Self {
            count: 0,
            last: state,
            recent: [HookedStart::default(); MOST_STARTS_QUEUED_PER_ENEMY],
        }
        .with_start(state, at_ms)
    }
}

impl StartTally {
    /// The start count and the state to send for one enemy this round.
    pub fn tally(
        &mut self,
        id: EnemyId,
        read: [i32; 4],
        hooked: Option<HookedStarts>,
    ) -> (u32, [i32; 4]) {
        let Some(tallied) = self.enemies.get_mut(&id) else {
            let sent = hooked.map_or(read, |hooked| hooked.last);
            self.enemies.insert(
                id,
                Tallied {
                    pending: Vec::new(),
                    state_id_read: read[0],
                    hooked_count: hooked.map(|hooked| hooked.count),
                    starts: 0,
                    sent,
                },
            );
            return (0, sent);
        };
        let hooked_now = hooked.filter(|hooked| Some(hooked.count) != tallied.hooked_count);
        if let Some(hooked) = hooked_now {
            let seen = tallied
                .hooked_count
                .map_or(1, |before| hooked.count.wrapping_sub(before));
            tallied.starts = tallied.starts.wrapping_add(seen);
            tallied.sent = hooked.last;
            let kept = (seen as usize).min(MOST_STARTS_QUEUED_PER_ENEMY);
            for (back, start) in hooked.recent[..kept].iter().enumerate().rev() {
                tallied.pending.push(PendingStart {
                    number: tallied.starts.wrapping_sub(back as u32),
                    state: start.state,
                    at_ms: start.at_ms,
                });
            }
        } else if read[0] != tallied.state_id_read {
            tallied.starts = tallied.starts.wrapping_add(1);
            tallied.sent = read;
            tallied.pending.push(PendingStart {
                number: tallied.starts,
                state: read,
                at_ms: 0,
            });
        }
        let overflow = tallied
            .pending
            .len()
            .saturating_sub(MOST_STARTS_QUEUED_PER_ENEMY);
        tallied.pending.drain(..overflow);
        tallied.state_id_read = read[0];
        tallied.hooked_count = hooked.map(|hooked| hooked.count).or(tallied.hooked_count);
        (tallied.starts, tallied.sent)
    }

    /// The starts counted for an enemy since the last call, oldest first, each with how long
    /// before `now_ms` it was made (zero when the hook did not stamp it).
    pub fn take_queued(&mut self, id: EnemyId, now_ms: u64) -> Vec<QueuedStart> {
        let Some(tallied) = self.enemies.get_mut(&id) else {
            return Vec::new();
        };
        tallied
            .pending
            .drain(..)
            .map(|start| QueuedStart {
                id,
                number: start.number,
                state: start.state,
                ago_ms: if start.at_ms == 0 {
                    0
                } else {
                    now_ms.saturating_sub(start.at_ms).min(u64::from(u16::MAX)) as u16
                },
            })
            .collect()
    }

    pub fn keep_only(&mut self, ids: &[EnemyId]) {
        self.enemies.retain(|id, _| ids.contains(id));
    }

    pub fn forget(&mut self) {
        self.enemies.clear();
    }
}
