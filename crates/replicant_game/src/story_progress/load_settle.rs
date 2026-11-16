use crate::story_progress::{
    HOST_STORY_WAITS_FOR_THE_STAGE_START_AT_MOST_MS, STAGE_START_SCRIPT_FUNCTION,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostStoryApply {
    Now,
    /// This game is loading a phase, or its stage start has not run yet: the host's changes wait so
    /// the stage start reads the flags as the host's stage start did, not as the host's stage left them.
    HeldForTheLoad,
}

/// Decides each tick whether the guest applies the host's story changes. The host's stage start
/// sets flags as its phase loads; a guest loading the same phase a little later must not see
/// them before its own stage start, or it takes the "already done" branch and places no actors.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HostStoryApplyGate {
    last_load_count: Option<u32>,
    loaded_at_ms: Option<u64>,
    stage_started: bool,
}

impl HostStoryApplyGate {
    pub const fn new() -> Self {
        Self {
            last_load_count: None,
            loaded_at_ms: None,
            stage_started: true,
        }
    }

    /// A script function ran on this game; the stage start ends the wait.
    pub fn note_function_ran(&mut self, name: &str) {
        if name == STAGE_START_SCRIPT_FUNCTION {
            self.stage_started = true;
        }
    }

    pub fn step(&mut self, in_play: bool, phase_load_count: u32, now_ms: u64) -> HostStoryApply {
        if self.last_load_count != Some(phase_load_count) {
            if self.last_load_count.is_some() {
                self.loaded_at_ms = Some(now_ms);
                self.stage_started = false;
            }
            self.last_load_count = Some(phase_load_count);
        }
        if !in_play {
            return HostStoryApply::HeldForTheLoad;
        }
        let waiting_for_the_stage_start = !self.stage_started
            && self.loaded_at_ms.is_some_and(|loaded_ms| {
                now_ms.saturating_sub(loaded_ms) < HOST_STORY_WAITS_FOR_THE_STAGE_START_AT_MOST_MS
            });
        if waiting_for_the_stage_start {
            return HostStoryApply::HeldForTheLoad;
        }
        self.loaded_at_ms = None;
        self.stage_started = true;
        HostStoryApply::Now
    }
}
