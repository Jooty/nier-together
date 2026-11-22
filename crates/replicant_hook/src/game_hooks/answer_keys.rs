use replicant_game::scale_up_prompt::{
    Gesture, KEYBOARD_NOT_NOW_VIRTUAL_KEY, KEYBOARD_SCALE_UP_VIRTUAL_KEY, keyboard_gesture,
};

use crate::game_hooks::noclip_keys::{game_window_is_foreground, key_down};

pub fn read_answer_gesture() -> Gesture {
    if !game_window_is_foreground() {
        return Gesture::Idle;
    }
    keyboard_gesture(
        key_down(KEYBOARD_SCALE_UP_VIRTUAL_KEY),
        key_down(KEYBOARD_NOT_NOW_VIRTUAL_KEY),
    )
}
