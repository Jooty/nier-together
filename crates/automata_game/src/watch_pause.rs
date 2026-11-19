mod pause_lines;
mod pause_machine;

pub use pause_lines::{
    host_pause_applied_line, host_pause_ended_line, own_menu_close_gave_up_line,
    own_menu_close_press_line, own_menu_closed_by_itself_line, own_menu_closed_by_presses_line,
    own_menu_has_not_opened_line, own_menu_opened_line, own_menu_still_open_line,
};
pub use pause_machine::{WatchPause, WatchPauseReadings, WatchPauseRelease, WatchPauseStep};

pub const HOST_STATE_COUNTS_AS_FRESH_FOR_MS: u64 = 3_000;
pub const OWN_MENU_SHOULD_HAVE_OPENED_WITHIN_MS: u64 = 1_000;
pub const OWN_MENU_IS_CHECKED_AFTER_THE_HOSTS_PAUSE_ENDS_MS: u64 = 500;
pub const FIRST_CLOSE_PRESS_AFTER_THE_HOSTS_PAUSE_ENDS_MS: u64 = 200;
pub const CLOSE_PRESSES_ARE_SPACED_MS: u64 = 1_500;
pub const MOST_CLOSE_PRESSES: u32 = 4;
