use crate::scale_up_prompt::{PAD_ANSWER_BUTTONS, XINPUT_BUTTONS_OFFSET_BYTES, XINPUT_STATE_BYTES};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AnswerHider {
    hidden: u16,
}

impl AnswerHider {
    pub const fn new() -> Self {
        Self { hidden: 0 }
    }

    pub fn hide_in_xinput_state(&mut self, prompt_up: bool, state: &mut [u8]) -> u16 {
        if state.len() < XINPUT_STATE_BYTES {
            return 0;
        }
        let at = XINPUT_BUTTONS_OFFSET_BYTES;
        let raw = u16::from_le_bytes([state[at], state[at + 1]]);
        if prompt_up {
            self.hidden = PAD_ANSWER_BUTTONS;
        } else {
            self.hidden &= raw;
        }
        let seen_by_game = raw & !self.hidden;
        state[at..at + 2].copy_from_slice(&seen_by_game.to_le_bytes());
        raw
    }
}
