use crate::story_watch::STORY_REMOVAL_SETTLES_MS;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemovalChange {
    TakenOut,
    BroughtBack,
}

/// What the story's own calls of the two companion natives say: she is out of play once the last
/// call was the removal and no call has followed it for the settle time.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct StoryRemoval {
    settled_out: bool,
    last_call: Option<(bool, u64)>,
}

impl StoryRemoval {
    pub const fn new() -> Self {
        Self {
            settled_out: false,
            last_call: None,
        }
    }

    pub fn story_called(&mut self, taken_out: bool, now_ms: u64) {
        self.last_call = Some((taken_out, now_ms));
    }

    pub fn step(&mut self, now_ms: u64) -> Option<RemovalChange> {
        let (taken_out, at_ms) = self.last_call?;
        if now_ms.saturating_sub(at_ms) < STORY_REMOVAL_SETTLES_MS || taken_out == self.settled_out
        {
            return None;
        }
        self.settled_out = taken_out;
        Some(if taken_out {
            RemovalChange::TakenOut
        } else {
            RemovalChange::BroughtBack
        })
    }

    pub const fn settled_out(&self) -> bool {
        self.settled_out
    }

    pub fn forget(&mut self) {
        *self = Self::new();
    }
}
