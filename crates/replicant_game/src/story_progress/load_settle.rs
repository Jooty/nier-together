use crate::story_progress::{
    HOST_STORY_WAITS_FOR_THE_STAGE_START_AT_MOST_MS, STAGE_START_SCRIPT_FUNCTION,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostStoryApply {
    Now,
    /// This game is changing or loading its phase, or its stage start has not run yet: the host's
    /// changes wait so the stage start reads the flags as the host's stage start did, not as the
    /// host's stage left them.
    HeldForTheLoad,
}

/// What ended the last hold.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostStoryRelease {
    StageStart,
    Cap,
}

/// Decides each tick whether the guest applies the host's story changes. The host's stage start
/// sets flags as its phase loads; a guest changing to the same phase a little later must not see
/// them before its own stage start, or it takes the "already done" branch and places no actors.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HostStoryApplyGate {
    last_load_count: Option<u32>,
    last_phase: Option<u16>,
    held_since_ms: Option<u64>,
    stage_started: bool,
    release: Option<HostStoryRelease>,
}

impl HostStoryApplyGate {
    pub const fn new() -> Self {
        Self {
            last_load_count: None,
            last_phase: None,
            held_since_ms: None,
            stage_started: true,
            release: None,
        }
    }

    /// A script function ran on this game; the stage start ends the wait.
    pub fn note_function_ran(&mut self, name: &str) {
        if name == STAGE_START_SCRIPT_FUNCTION {
            self.stage_started = true;
        }
    }

    pub fn last_release(&self) -> Option<HostStoryRelease> {
        self.release
    }

    pub fn step(
        &mut self,
        in_play: bool,
        phase_load_count: u32,
        local_phase: Option<u16>,
        now_ms: u64,
    ) -> HostStoryApply {
        if local_phase.is_some() && self.last_phase != local_phase {
            if self.last_phase.is_some() {
                self.held_since_ms.get_or_insert(now_ms);
                self.stage_started = false;
            }
            self.last_phase = local_phase;
        }
        if self.last_load_count != Some(phase_load_count) {
            if self.last_load_count.is_some() {
                // The cap counts from the load itself, however long the change took to get there.
                self.held_since_ms = Some(now_ms);
                self.stage_started = false;
            }
            self.last_load_count = Some(phase_load_count);
        }
        if !in_play {
            return HostStoryApply::HeldForTheLoad;
        }
        let Some(held_since_ms) = self.held_since_ms else {
            return HostStoryApply::Now;
        };
        let capped =
            now_ms.saturating_sub(held_since_ms) >= HOST_STORY_WAITS_FOR_THE_STAGE_START_AT_MOST_MS;
        if !self.stage_started && !capped {
            return HostStoryApply::HeldForTheLoad;
        }
        self.release = Some(if self.stage_started {
            HostStoryRelease::StageStart
        } else {
            HostStoryRelease::Cap
        });
        self.held_since_ms = None;
        self.stage_started = true;
        HostStoryApply::Now
    }
}
