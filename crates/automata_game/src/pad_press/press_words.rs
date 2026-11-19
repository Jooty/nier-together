use crate::pad_press::PAD_UPDATES_A_PRESS_TAKES;

pub fn pad_words_after_press(updates_left: u32, mask: u32, words: [u32; 3]) -> Option<[u32; 3]> {
    let [held, pressed, released] = words;
    match PAD_UPDATES_A_PRESS_TAKES.checked_sub(updates_left)? {
        0 => Some([held | mask, pressed | mask, released]),
        1 => Some([held | mask, pressed, released]),
        2 => Some([held, pressed, released | mask]),
        _ => None,
    }
}
