use crate::story_progress::HOST_STORY_WAITS_AFTER_A_LOAD_MS;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostStoryApply {
    Now,
    /// This game is loading a phase, or its stage just started: the host's changes wait so the
    /// stage start reads the flags as the host's stage start did, not as the host's stage left them.
    HeldForTheLoad,
}

/// Decides each tick whether the guest applies the host's story changes. The host's stage start
/// sets flags as its phase loads; a guest loading the same phase a little later must not see
/// them before its own stage start, or it takes the "already done" branch and places no actors.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HostStoryApplyGate {
    last_load_count: Option<u32>,
    loaded_at_ms: Option<u64>,
}

impl HostStoryApplyGate {
    pub const fn new() -> Self {
        Self {
            last_load_count: None,
            loaded_at_ms: None,
        }
    }

    pub fn step(&mut self, in_play: bool, phase_load_count: u32, now_ms: u64) -> HostStoryApply {
        if self.last_load_count != Some(phase_load_count) {
            if self.last_load_count.is_some() {
                self.loaded_at_ms = Some(now_ms);
            }
            self.last_load_count = Some(phase_load_count);
        }
        if !in_play {
            return HostStoryApply::HeldForTheLoad;
        }
        let settling = self.loaded_at_ms.is_some_and(|loaded_ms| {
            now_ms.saturating_sub(loaded_ms) < HOST_STORY_WAITS_AFTER_A_LOAD_MS
        });
        if settling {
            return HostStoryApply::HeldForTheLoad;
        }
        self.loaded_at_ms = None;
        HostStoryApply::Now
    }
}
