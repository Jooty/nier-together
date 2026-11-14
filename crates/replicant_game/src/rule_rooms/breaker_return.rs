use crate::rule_rooms::{RETURN_FADE_IN_MS, RETURN_FADE_OUT_MS, RulePlayer, RuleRoom};

/// The one return in progress: fade out, put the breaker back at the start spot, fade in, then arm
/// the breaks again. The breaker's input stays blocked until the fade in ends, and no break starts
/// a second return before that.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BreakerReturn {
    waiting: Option<(&'static RuleRoom, RulePlayer, u64)>,
    arm_waiting: Option<(&'static RuleRoom, u64)>,
    input_blocked_until_ms: u64,
}

impl BreakerReturn {
    pub const fn new() -> Self {
        Self {
            waiting: None,
            arm_waiting: None,
            input_blocked_until_ms: 0,
        }
    }

    pub fn is_returning(&self) -> bool {
        self.waiting.is_some()
    }

    pub fn input_blocked(&self, now_ms: u64) -> bool {
        now_ms < self.input_blocked_until_ms
    }

    /// False while a return, its fade in or its input block is running: that break changes nothing.
    pub fn begin(&mut self, room: &'static RuleRoom, player: RulePlayer, now_ms: u64) -> bool {
        if self.waiting.is_some() || self.input_blocked(now_ms) {
            return false;
        }
        self.waiting = Some((room, player, now_ms.saturating_add(RETURN_FADE_OUT_MS)));
        self.input_blocked_until_ms = now_ms.saturating_add(RETURN_FADE_OUT_MS + RETURN_FADE_IN_MS);
        true
    }

    pub fn take_if_due(&mut self, now_ms: u64) -> Option<(&'static RuleRoom, RulePlayer)> {
        let (room, player, due_ms) = self.waiting?;
        if now_ms < due_ms {
            return None;
        }
        self.waiting = None;
        self.arm_waiting = Some((room, self.input_blocked_until_ms));
        Some((room, player))
    }

    /// The room whose breaks are armed again: only once the fade in and the input block are over.
    pub fn take_arm_if_due(&mut self, now_ms: u64) -> Option<&'static RuleRoom> {
        let (room, due_ms) = self.arm_waiting?;
        if now_ms < due_ms {
            return None;
        }
        self.arm_waiting = None;
        Some(room)
    }
}
