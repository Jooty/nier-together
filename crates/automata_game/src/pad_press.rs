mod press_words;

pub use press_words::pad_words_after_press;

pub const BACK_BUTTON_PAD_MASK: u32 = 0x20;
pub const PAD_UPDATES_A_PRESS_TAKES: u32 = 3;
pub const INPUT_UPDATE_TAIL_FIRST_BYTES: [u8; 2] = [0x89, 0x0D];
pub const INPUT_UPDATE_TAIL_BYTES_AT_OFFSET_SIX: [u8; 5] = [0xB8, 0x01, 0x00, 0x00, 0x00];
pub const INPUT_UPDATE_TAIL_RETURN_OFFSET: usize = 29;
pub const INPUT_UPDATE_TAIL_LEN: usize = 30;
