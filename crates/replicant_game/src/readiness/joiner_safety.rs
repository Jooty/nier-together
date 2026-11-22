use crate::readiness::JOINER_WAIT_LOG_INTERVAL_MS;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SafetyBlock {
    NotFreeToAct,
    EventScene,
    ControlSlotsHeld,
    BossFight,
    ScriptedFight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SafetyReads {
    pub free_to_act: Option<bool>,
    pub in_event_scene: Option<bool>,
    pub control_slots_free: bool,
    pub boss_gauge_open: bool,
    pub fight_window_open: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct JoinerWaitLog {
    last: Option<SafetyBlock>,
    logged_at_ms: Option<u64>,
}

impl SafetyReads {
    pub fn first_block(&self) -> Option<SafetyBlock> {
        if self.boss_gauge_open {
            Some(SafetyBlock::BossFight)
        } else if self.fight_window_open {
            Some(SafetyBlock::ScriptedFight)
        } else if self.in_event_scene != Some(false) {
            Some(SafetyBlock::EventScene)
        } else if self.free_to_act != Some(true) {
            Some(SafetyBlock::NotFreeToAct)
        } else if !self.control_slots_free {
            Some(SafetyBlock::ControlSlotsHeld)
        } else {
            None
        }
    }
}

impl SafetyBlock {
    pub fn label(self) -> &'static str {
        match self {
            Self::NotFreeToAct => {
                "the game says the player is not free to act (or it is unreadable)"
            }
            Self::EventScene => "an event scene is running (or its flag is unreadable)",
            Self::ControlSlotsHeld => "the control slots are held (a menu, prompt or script)",
            Self::BossFight => "the boss life gauge is up (a boss fight)",
            Self::ScriptedFight => "a streamed fight window is open",
        }
    }
}

impl JoinerWaitLog {
    pub const fn new() -> Self {
        Self {
            last: None,
            logged_at_ms: None,
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    pub fn observe(&mut self, block: Option<SafetyBlock>, now_ms: u64) -> Option<String> {
        let Some(block) = block else {
            self.last = None;
            return None;
        };
        let changed = self.last != Some(block);
        let due = self
            .logged_at_ms
            .is_none_or(|at| now_ms.saturating_sub(at) >= JOINER_WAIT_LOG_INTERVAL_MS);
        if !changed && !due {
            return None;
        }
        self.last = Some(block);
        self.logged_at_ms = Some(now_ms);
        Some(format!("joiner waits for a safe moment: {}", block.label()))
    }
}
