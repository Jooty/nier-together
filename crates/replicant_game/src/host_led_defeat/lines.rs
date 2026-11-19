use replicant_proto::{DefeatPick, HostDefeatScreen};

use crate::host_led_defeat::{
    GUEST_DEFEAT_SCREEN_WAITS_FOR_THE_HOST_MS, HOST_DEFEAT_PICK_KEPT_FOR_MS,
    game_over_result_of_pick,
};

pub fn host_screen_opened_line(defeat: u32) -> String {
    format!(
        "defeat screen: your defeat screen opened (defeat {defeat}), the guest's screen follows your highlight and your pick"
    )
}

pub fn host_cursor_line(defeat: u32, cursor: u8) -> String {
    format!("defeat screen: you moved to item {cursor} (defeat {defeat}), sent to the guest")
}

pub fn host_pick_line(defeat: u32, pick: DefeatPick) -> String {
    format!(
        "defeat screen: you picked {} (defeat {defeat}, the game wrote result {}), sent to the guest",
        pick.label(),
        game_over_result_of_pick(pick)
    )
}

pub fn heard_screen_line(defeat: u32, cursor: u8) -> String {
    format!("defeat screen: the host's defeat screen is open (defeat {defeat}, item {cursor})")
}

pub fn heard_pick_line(defeat: u32, pick: DefeatPick) -> String {
    format!(
        "defeat screen: the host picked {} (defeat {defeat}); your screen makes it when it takes a pick, for up to {HOST_DEFEAT_PICK_KEPT_FOR_MS} ms",
        pick.label()
    )
}

pub fn guest_screen_opened_line(host: Option<HostDefeatScreen>) -> String {
    match host {
        Some(note) => format!(
            "defeat screen: your defeat screen opened while the host's is open (defeat {}), the host leads it",
            note.defeat
        ),
        None => format!(
            "defeat screen: your defeat screen opened, the host shows none yet; it waits up to {GUEST_DEFEAT_SCREEN_WAITS_FOR_THE_HOST_MS} ms for the host's"
        ),
    }
}

pub fn guest_led_line(defeat: u32) -> String {
    format!(
        "defeat screen: the host leads your screen (defeat {defeat}): your buttons are off, it shows the host's highlight"
    )
}

pub fn guest_waits_line() -> String {
    "defeat screen: your buttons are off while your screen waits for the host's".to_owned()
}

pub fn guest_buttons_back_line() -> String {
    "defeat screen: the host shows no defeat screen, yours takes your own buttons".to_owned()
}

pub fn guest_made_pick_line(
    defeat: u32,
    pick: DefeatPick,
    after_heard_ms: u64,
    after_open_ms: u64,
) -> String {
    format!(
        "defeat screen: made the host's pick {} here (defeat {defeat}, result {} written as the screen's own step does), {after_heard_ms} ms after the host's pick, {after_open_ms} ms after your screen opened",
        pick.label(),
        game_over_result_of_pick(pick)
    )
}

pub fn context_missing_line(defeat: u32, pick: DefeatPick) -> String {
    format!(
        "defeat screen: the host's pick {} (defeat {defeat}) waits: your screen's result word was not found this frame",
        pick.label()
    )
}

pub fn pick_dropped_line(defeat: u32, pick: DefeatPick, waited_ms: u64) -> String {
    format!(
        "defeat screen: the host's pick {} (defeat {defeat}) was dropped after {waited_ms} ms: no defeat screen here took it",
        pick.label()
    )
}
