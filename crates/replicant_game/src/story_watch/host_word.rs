use crate::story_watch::GUEST_WORD_STALE_AFTER_MS;

/// The host's record of the guest's last word on whether it watches.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct GuestWord {
    last: Option<(bool, u64)>,
}

impl GuestWord {
    pub const fn new() -> Self {
        Self { last: None }
    }

    pub fn heard(&mut self, watching: bool, now_ms: u64) {
        self.last = Some((watching, now_ms));
    }

    pub fn guest_watches(&self, now_ms: u64) -> bool {
        self.last.is_some_and(|(watching, at_ms)| {
            watching && now_ms.saturating_sub(at_ms) < GUEST_WORD_STALE_AFTER_MS
        })
    }

    pub fn forget(&mut self) {
        self.last = None;
    }
}
