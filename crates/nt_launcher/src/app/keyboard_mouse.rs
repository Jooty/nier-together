use nt_launcher_core::{ControllerCheck, offers_keyboard_mouse_fixes};

use crate::app::Launcher;
use crate::config::LauncherConfig;

struct KnownControllers(bool);

impl ControllerCheck for KnownControllers {
    fn controller_connected(&self) -> bool {
        self.0
    }
}

impl Launcher {
    pub fn with_config(config: LauncherConfig) -> Self {
        Self::from_parts(config, None)
    }

    pub fn keyboard_mouse_offer_due(&mut self) -> bool {
        if !offers_keyboard_mouse_fixes(self.profile) || self.config.keyboard_mouse.asked {
            return false;
        }
        let connected = match self.controller_seen {
            Some(connected) => connected,
            None => {
                let connected = self.controller_check.controller_connected();
                self.controller_seen = Some(connected);
                connected
            }
        };
        self.config
            .keyboard_mouse
            .offer_due(&KnownControllers(connected))
    }

    pub fn forget_controller_check(&mut self) {
        self.controller_seen = None;
    }

    pub fn answer_keyboard_mouse_offer(&mut self, turn_on: bool) {
        self.config.keyboard_mouse.answer_offer(turn_on);
        self.apply_settings();
    }

    pub fn set_keyboard_mouse_fixes(&mut self, on: bool) {
        if self.config.keyboard_mouse.on == on && self.config.keyboard_mouse.asked {
            return;
        }
        self.config.keyboard_mouse.set_in_settings(on);
        self.apply_settings();
    }
}
