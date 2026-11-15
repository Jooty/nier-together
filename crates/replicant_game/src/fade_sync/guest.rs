use crate::fade_sync::{FADE_IN_REPEAT_WINDOW_MS, GUEST_FADE_BACKSTOP_MS};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GuestFadeState {
    faded_out_at_ms: Option<u64>,
    mirrored_fade_in_at_ms: Option<u64>,
    own_fade_in_at_ms: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct FadeWatch {
    was_active: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FadeEdge {
    None,
    Started,
    Finished,
}

impl GuestFadeState {
    pub const fn new() -> Self {
        Self {
            faded_out_at_ms: None,
            mirrored_fade_in_at_ms: None,
            own_fade_in_at_ms: None,
        }
    }

    pub fn started_fade_out(&mut self, now_ms: u64) {
        self.faded_out_at_ms = Some(now_ms);
    }

    pub fn started_fade_in(&mut self) {
        self.faded_out_at_ms = None;
    }

    pub fn mirrored_fade_in(&mut self, now_ms: u64) {
        self.faded_out_at_ms = None;
        self.mirrored_fade_in_at_ms = Some(now_ms);
    }

    pub fn mirror_repeats_own_fade_in(&self, now_ms: u64) -> bool {
        within_repeat_window(self.own_fade_in_at_ms, now_ms)
    }

    pub fn own_fade_in_repeats_mirror(&mut self, now_ms: u64) -> bool {
        if within_repeat_window(self.mirrored_fade_in_at_ms, now_ms) {
            return true;
        }
        self.faded_out_at_ms = None;
        self.own_fade_in_at_ms = Some(now_ms);
        false
    }

    pub fn holds_fade(&self) -> bool {
        self.faded_out_at_ms.is_some()
    }

    pub fn backstop_due(&mut self, now_ms: u64) -> bool {
        let due = self
            .faded_out_at_ms
            .is_some_and(|at| now_ms.saturating_sub(at) >= GUEST_FADE_BACKSTOP_MS);
        if due {
            self.faded_out_at_ms = None;
        }
        due
    }
}

impl FadeWatch {
    pub const fn new() -> Self {
        Self { was_active: false }
    }

    pub fn observe(&mut self, active: bool) -> FadeEdge {
        let edge = match (self.was_active, active) {
            (false, true) => FadeEdge::Started,
            (true, false) => FadeEdge::Finished,
            _ => FadeEdge::None,
        };
        self.was_active = active;
        edge
    }
}

fn within_repeat_window(at_ms: Option<u64>, now_ms: u64) -> bool {
    at_ms.is_some_and(|at| now_ms.saturating_sub(at) < FADE_IN_REPEAT_WINDOW_MS)
}
