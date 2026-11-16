//! The defeat screen led by the host: the host's highlight and pick decide the guest's screen.

mod game_over_memory;
mod guest_follow;
mod host_watch;
mod lines;
mod picks;

pub use game_over_memory::{
    GameOverScreen, find_game_over_context, make_pick_on_screen, read_game_over_screen,
};
pub use guest_follow::{GuestDefeatAct, GuestDefeatFollow, GuestDefeatFrame};
pub use host_watch::{HostDefeatFrame, HostDefeatScreenWatch};
pub use lines::{
    context_missing_line, guest_buttons_back_line, guest_led_line, guest_made_pick_line,
    guest_screen_opened_line, guest_waits_line, heard_pick_line, heard_screen_line,
    host_cursor_line, host_pick_line, host_screen_opened_line, pick_dropped_line,
};
pub use picks::{cursor_of_pick, game_over_result_of_pick, pick_of_cursor};

pub const GAME_OVER_SCENE_ID: u8 = 14;
pub const GAME_OVER_ROOT_SEQUENCE_VFTABLE_RVA: u32 = 0xBC_77B8;
pub const GAME_OVER_ROOT_SEQUENCE_UPDATE_RVA: u32 = 0x45_34E0;
pub const GAME_OVER_CONTEXT_VFTABLE_RVA: u32 = 0xBC_7780;
pub const GAME_OVER_SEQUENCE_CURSOR_OFFSET: u64 = 0x68;
pub const GAME_OVER_SEQUENCE_STATE_OFFSET: u64 = 0x88;
pub const GAME_OVER_SEQUENCE_CHOOSING_STATE: i32 = 0;
pub const GAME_OVER_SEQUENCE_WRITES_THE_RESULT_STATE: i32 = 4;
pub const GAME_OVER_SEQUENCE_DONE_STATE: i32 = 5;
pub const GAME_OVER_CONTEXT_RESULT_OFFSET: u64 = 0x24;
pub const GAME_OVER_CONTINUE_CURSOR: i32 = 0;
pub const GAME_OVER_LEAVE_CURSOR: i32 = 1;
pub const GAME_OVER_CONTINUE_RESULT: i32 = 1;
pub const GAME_OVER_LEAVE_RESULT: i32 = 2;
pub const UI_SCENE_CONTEXT_LIST_OFFSET: u64 = 0x78;
pub const UI_SCENE_FIRST_CONTEXT_LINK_OFFSET: u64 = 0x80;
pub const UI_SCENE_CONTEXT_WALK_LIMIT: usize = 16;
pub const DEFEAT_SCREEN_FRAMES_APART_FOR_A_NEW_SCREEN_MS: u64 = 1_000;
pub const HOST_DEFEAT_SCREEN_RESEND_MS: u64 = 1_000;
pub const HOST_DEFEAT_SCREEN_NOTE_FRESH_MS: u64 = 3_000;
pub const GUEST_DEFEAT_SCREEN_WAITS_FOR_THE_HOST_MS: u64 = 2_000;
pub const HOST_DEFEAT_PICK_KEPT_FOR_MS: u64 = 10_000;
