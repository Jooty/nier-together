use automata_game::pad_press::{
    BACK_BUTTON_PAD_MASK, PAD_UPDATES_A_PRESS_TAKES, pad_words_after_press,
};

#[test]
fn a_press_is_held_and_pressed_first_then_held_then_released() {
    let mask = BACK_BUTTON_PAD_MASK;
    let words = [0x1, 0x2, 0x4];
    assert_eq!(
        pad_words_after_press(PAD_UPDATES_A_PRESS_TAKES, mask, words),
        Some([0x1 | mask, 0x2 | mask, 0x4])
    );
    assert_eq!(
        pad_words_after_press(PAD_UPDATES_A_PRESS_TAKES - 1, mask, words),
        Some([0x1 | mask, 0x2, 0x4])
    );
    assert_eq!(
        pad_words_after_press(PAD_UPDATES_A_PRESS_TAKES - 2, mask, words),
        Some([0x1, 0x2, 0x4 | mask])
    );
    assert_eq!(pad_words_after_press(0, mask, words), None);
}
