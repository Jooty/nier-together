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
    last_removal_was_the_aerie_exit: bool,
}

impl StoryRemoval {
    pub const fn new() -> Self {
        Self {
            settled_out: false,
            last_call: None,
            last_removal_was_the_aerie_exit: false,
        }
    }

    pub fn story_called(&mut self, taken_out: bool, now_ms: u64) {
        self.last_call = Some((taken_out, now_ms));
        if !taken_out {
            self.last_removal_was_the_aerie_exit = false;
        }
    }

    pub fn restore_aerie_exit_from_game_state(
        &mut self,
        aerie_exit_flag_on: bool,
        companion_inactive: bool,
    ) -> bool {
        if self.last_call.is_some()
            || self.settled_out
            || !aerie_exit_flag_on
            || !companion_inactive
        {
            return false;
        }
        self.settled_out = true;
        self.last_removal_was_the_aerie_exit = true;
        true
    }

    pub fn aerie_exit_removed_her(&mut self, now_ms: u64) {
        self.last_call = Some((true, now_ms));
        self.last_removal_was_the_aerie_exit = true;
    }

    pub const fn taken_at_the_aerie_exit(&self) -> bool {
        self.settled_out && self.last_removal_was_the_aerie_exit
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
