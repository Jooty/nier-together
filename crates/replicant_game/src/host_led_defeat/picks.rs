use replicant_proto::DefeatPick;

use crate::host_led_defeat::{
    GAME_OVER_CONTINUE_CURSOR, GAME_OVER_CONTINUE_RESULT, GAME_OVER_LEAVE_CURSOR,
    GAME_OVER_LEAVE_RESULT,
};

/// The item a cursor stands on, as the screen's result step reads it: any cursor but the first
/// is the second item (`GameOverRootSequence` update 0x4534E0, state 4).
pub fn pick_of_cursor(cursor: i32) -> DefeatPick {
    if cursor == GAME_OVER_CONTINUE_CURSOR {
        DefeatPick::Continue
    } else {
        DefeatPick::Leave
    }
}

pub fn cursor_of_pick(pick: DefeatPick) -> i32 {
    match pick {
        DefeatPick::Continue => GAME_OVER_CONTINUE_CURSOR,
        DefeatPick::Leave => GAME_OVER_LEAVE_CURSOR,
    }
}

/// The result `_StartGameOver` (0x708120) polls: 1 restarts the phase, 2 sets the map
/// manager's request word to 2.
pub fn game_over_result_of_pick(pick: DefeatPick) -> i32 {
    match pick {
        DefeatPick::Continue => GAME_OVER_CONTINUE_RESULT,
        DefeatPick::Leave => GAME_OVER_LEAVE_RESULT,
    }
}
