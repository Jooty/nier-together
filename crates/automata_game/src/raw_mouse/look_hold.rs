pub const LOOK_HOLD_AFTER_LAST_COUNTS_MS: u64 = 250;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LookHold {
    last_counts_at_ms: Option<u64>,
}

impl LookHold {
    pub const fn new() -> Self {
        Self {
            last_counts_at_ms: None,
        }
    }

    pub fn step(&mut self, now_ms: u64, counts_moved_camera: bool) -> bool {
        if counts_moved_camera {
            self.last_counts_at_ms = Some(now_ms);
        }
        self.last_counts_at_ms
            .is_some_and(|last| now_ms.saturating_sub(last) <= LOOK_HOLD_AFTER_LAST_COUNTS_MS)
    }

    pub fn clear(&mut self) {
        self.last_counts_at_ms = None;
    }
}
