use crate::fade_sync::{SCENE_START_FADE_WINDOW_MS, ZONE_CHANGE_FADE_WINDOW_MS};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HostFadeSync {
    armed_until_ms: Option<u64>,
    scene_start_until_ms: Option<u64>,
    sent_fade_out: bool,
}

impl HostFadeSync {
    pub const fn new() -> Self {
        Self {
            armed_until_ms: None,
            scene_start_until_ms: None,
            sent_fade_out: false,
        }
    }

    pub fn arm(&mut self, now_ms: u64) {
        self.armed_until_ms = Some(now_ms + ZONE_CHANGE_FADE_WINDOW_MS);
    }

    pub fn arm_for_scene_start(&mut self, now_ms: u64) {
        self.scene_start_until_ms = Some(now_ms + SCENE_START_FADE_WINDOW_MS);
    }

    pub fn should_send_fade_out(&mut self, now_ms: u64) -> bool {
        let live = |until: Option<u64>| until.is_some_and(|until| now_ms < until);
        let armed = live(self.armed_until_ms) || live(self.scene_start_until_ms);
        if armed {
            self.armed_until_ms = None;
            self.scene_start_until_ms = None;
            self.sent_fade_out = true;
        }
        armed
    }

    pub fn should_send_fade_in(&mut self) -> bool {
        std::mem::take(&mut self.sent_fade_out)
    }
}
