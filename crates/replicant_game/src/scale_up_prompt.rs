mod hider;
mod press;
mod state;

pub use hider::AnswerHider;
pub use press::{Gesture, PressSample, keyboard_gesture, pad_gesture};
pub use state::{AskEffect, OpenAsk, PromptStep, ScaleUpPrompt};

pub const HOLD_TO_ANSWER_MS: u64 = 600;
pub const IGNORE_PRESSES_AFTER_SHOWN_MS: u64 = 1500;
pub const PAD_SCALE_UP_BUTTON: u16 = 0x0040;
pub const PAD_NOT_NOW_BUTTON: u16 = 0x0080;
pub const PAD_ANSWER_BUTTONS: u16 = PAD_SCALE_UP_BUTTON | PAD_NOT_NOW_BUTTON;
pub const KEYBOARD_SCALE_UP_VIRTUAL_KEY: u16 = 0x76;
pub const KEYBOARD_NOT_NOW_VIRTUAL_KEY: u16 = 0x77;
pub const PAD_READING_STALE_AFTER_MS: u64 = 250;
pub const XINPUT_STATE_BYTES: usize = 16;
pub const XINPUT_BUTTONS_OFFSET_BYTES: usize = 4;
pub const KEYBOARD_SCALE_UP_HINT: &str = "Hold F7";
pub const KEYBOARD_NOT_NOW_HINT: &str = "Hold F8";
pub const PAD_SCALE_UP_HINT: &str = "Hold L3";
pub const PAD_NOT_NOW_HINT: &str = "Hold R3";
pub const MOUSE_HINT: &str = "or hold ~ and click";
pub const SCALE_UP_ANSWERED_LINE_SUFFIX: &str = "gets a copy of your save";
pub const NOT_NOW_ANSWERED_LINE_SUFFIX: &str = "starts their own new game";
