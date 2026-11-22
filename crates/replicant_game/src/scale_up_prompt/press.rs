use crate::scale_up_prompt::{PAD_NOT_NOW_BUTTON, PAD_SCALE_UP_BUTTON};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Gesture {
    #[default]
    Idle,
    Yes,
    No,
    Mixed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PressSample {
    pub keyboard: Gesture,
    pub pad: Gesture,
    pub click: Option<bool>,
}

impl Gesture {
    pub fn is_answer(self) -> bool {
        matches!(self, Gesture::Yes | Gesture::No)
    }
}

impl PressSample {
    pub fn combined(&self) -> Gesture {
        match (self.keyboard, self.pad) {
            (Gesture::Yes, Gesture::No) | (Gesture::No, Gesture::Yes) => Gesture::Mixed,
            (Gesture::Mixed, _) | (_, Gesture::Mixed) => Gesture::Mixed,
            (Gesture::Idle, other) | (other, Gesture::Idle) => other,
            (same, _) => same,
        }
    }
}

pub fn keyboard_gesture(scale_up_key: bool, not_now_key: bool) -> Gesture {
    match (scale_up_key, not_now_key) {
        (true, false) => Gesture::Yes,
        (false, true) => Gesture::No,
        (true, true) => Gesture::Mixed,
        (false, false) => Gesture::Idle,
    }
}

pub fn pad_gesture(buttons: u16) -> Gesture {
    keyboard_gesture(
        buttons & PAD_SCALE_UP_BUTTON != 0,
        buttons & PAD_NOT_NOW_BUTTON != 0,
    )
}
