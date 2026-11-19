use crate::watch_pause::WatchPauseRelease;

pub fn host_pause_applied_line() -> String {
    "watch: the host paused its game, this copy's game is paused with it: its focus test answers lost until the host resumes".to_owned()
}

pub fn own_menu_opened_line(after_ms: u64) -> String {
    format!("watch: this copy's pause menu opened {after_ms} ms after the host's pause was applied")
}

pub fn own_menu_has_not_opened_line(after_ms: u64) -> String {
    format!(
        "watch: this copy's pause menu has not opened {after_ms} ms after the host's pause was applied: the focus answer does not open it here"
    )
}

pub fn host_pause_ended_line(release: WatchPauseRelease, held_ms: u64) -> String {
    format!(
        "watch: the host's pause ends here after {held_ms} ms: {}; this copy's game goes on",
        release.label()
    )
}

pub fn own_menu_closed_by_itself_line() -> String {
    "watch: this copy's pause menu closed by itself after the host's pause ended".to_owned()
}

pub fn own_menu_still_open_line(after_ms: u64) -> String {
    format!(
        "watch: this copy's pause menu is still open {after_ms} ms after the host's pause ended; the pause key here closes it"
    )
}

pub fn own_menu_close_press_line(number: u32, after_ms: u64) -> String {
    format!(
        "watch: this copy's pause menu was opened by the host's pause and is still open {after_ms} ms after it ended; back press {number} is sent"
    )
}

pub fn own_menu_closed_by_presses_line(presses: u32, after_ms: u64) -> String {
    format!(
        "watch: this copy's pause menu closed {after_ms} ms after the host's pause ended, after {presses} back presses"
    )
}

pub fn own_menu_close_gave_up_line(presses: u32) -> String {
    format!(
        "watch: this copy's pause menu is still open after {presses} back presses; left to the player"
    )
}
