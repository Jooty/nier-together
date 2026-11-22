use crate::profile::{AUTOMATA, GameProfile};
use serde::{Deserialize, Serialize};

pub trait ControllerCheck {
    fn controller_connected(&self) -> bool;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct KeyboardMouseFixes {
    #[serde(default)]
    pub on: bool,
    #[serde(default)]
    pub asked: bool,
}

impl KeyboardMouseFixes {
    pub fn offer_due(self, controllers: &dyn ControllerCheck) -> bool {
        !self.asked && !controllers.controller_connected()
    }

    pub fn answer_offer(&mut self, turn_on: bool) {
        self.on = turn_on;
        self.asked = true;
    }

    pub fn set_in_settings(&mut self, on: bool) {
        self.on = on;
        self.asked = true;
    }
}

pub fn offers_keyboard_mouse_fixes(profile: &GameProfile) -> bool {
    profile.game_id == AUTOMATA.game_id
}
