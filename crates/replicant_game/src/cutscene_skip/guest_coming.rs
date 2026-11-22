use crate::cutscene_skip::GUEST_COMING_INTO_HOST_SCENE_FOR_MS;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct GuestComingIntoScene {
    sent_at_ms: Option<u64>,
}

impl GuestComingIntoScene {
    pub const fn new() -> Self {
        Self { sent_at_ms: None }
    }

    pub fn follow_sent(&mut self, now_ms: u64) {
        self.sent_at_ms = Some(now_ms);
    }

    pub fn clear(&mut self) {
        self.sent_at_ms = None;
    }

    pub fn is_coming(&self, now_ms: u64) -> bool {
        self.sent_at_ms
            .is_some_and(|sent| now_ms.saturating_sub(sent) < GUEST_COMING_INTO_HOST_SCENE_FOR_MS)
    }
}
