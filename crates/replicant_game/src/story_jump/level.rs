pub const LEVEL_CHECK_SETTLE_MS: u64 = 3000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LevelCheck {
    Matches,
    ApplyAgain { expected: i32, actual: i32 },
    Unreadable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LevelWatcher {
    last: Option<i32>,
}

impl LevelWatcher {
    pub const fn new() -> Self {
        Self { last: None }
    }

    pub fn observe(&mut self, level: i32) -> Option<(Option<i32>, i32)> {
        let before = self.last.replace(level);
        (before != Some(level)).then_some((before, level))
    }
}

pub fn check_level(expected: i32, actual: Option<i32>) -> LevelCheck {
    match actual {
        None => LevelCheck::Unreadable,
        Some(actual) if actual >= expected => LevelCheck::Matches,
        Some(actual) => LevelCheck::ApplyAgain { expected, actual },
    }
}

pub fn level_check_is_due(
    phase_at_apply: &str,
    phase_now: &str,
    field_ready: bool,
    ms_since_phase_change: u64,
) -> bool {
    phase_at_apply != phase_now && field_ready && ms_since_phase_change >= LEVEL_CHECK_SETTLE_MS
}
