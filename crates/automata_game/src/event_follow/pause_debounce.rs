use crate::event_follow::EVENT_PAUSE_READING_MUST_HOLD_MS;

#[derive(Debug, Clone, Copy, Default)]
pub struct PauseDebounce {
    stable: bool,
    reading: bool,
    reading_since_ms: u64,
}

impl PauseDebounce {
    pub fn observe(&mut self, now_ms: u64, open: bool) -> bool {
        if open != self.reading {
            self.reading = open;
            self.reading_since_ms = now_ms;
        }
        if self.reading != self.stable
            && now_ms.saturating_sub(self.reading_since_ms) >= EVENT_PAUSE_READING_MUST_HOLD_MS
        {
            self.stable = self.reading;
        }
        self.stable
    }
}
