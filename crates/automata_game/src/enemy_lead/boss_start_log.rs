use std::collections::HashSet;

use crate::enemies::EnemyId;
use crate::enemy_lead::{BOSS_CLASS_HEALTH_FROM, BOSS_START_LINES_LOGGED_PER_MINUTE};
use crate::minute_line_budget::{LineAdmission, MinuteLineBudget};

const BOSS_START_LINES_TOPIC: &str = "enemies: boss state starts";

/// Which enemies have shown boss-class health, and the minute budget of their state start lines.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BossStartLog {
    bosses: HashSet<EnemyId>,
    budget: MinuteLineBudget,
}

impl Default for BossStartLog {
    fn default() -> Self {
        Self {
            bosses: HashSet::new(),
            budget: MinuteLineBudget::new(BOSS_START_LINES_LOGGED_PER_MINUTE),
        }
    }
}

impl BossStartLog {
    pub fn note_health(&mut self, id: EnemyId, health: i32) {
        if health >= BOSS_CLASS_HEALTH_FROM {
            self.bosses.insert(id);
        }
    }

    pub fn is_boss(&self, id: EnemyId) -> bool {
        self.bosses.contains(&id)
    }

    pub fn take_line(&mut self, now_ms: u64) -> LineAdmission {
        self.budget.admit(now_ms, BOSS_START_LINES_TOPIC)
    }

    pub fn forget(&mut self) {
        self.bosses.clear();
    }
}

pub fn sent_start_line(id: EnemyId, number: u32, state: [i32; 4], ago_ms: u16) -> String {
    format!(
        "enemies: boss start {number} of {} sent by this copy (host): state {:#X} {} {} {}, {ago_ms} ms before the send",
        id.label(),
        state[0],
        state[1],
        state[2],
        state[3]
    )
}

pub fn played_start_line(
    id: EnemyId,
    number: Option<u32>,
    state: [i32; 4],
    ago_ms: u16,
    started: bool,
) -> String {
    let which = number.map_or("the latest pose".to_owned(), |number| {
        format!("start {number}")
    });
    format!(
        "enemies: boss {which} of {} played on this copy (guest): state {:#X} {} {} {}, host made it {ago_ms} ms before its send (started: {started})",
        id.label(),
        state[0],
        state[1],
        state[2],
        state[3]
    )
}
