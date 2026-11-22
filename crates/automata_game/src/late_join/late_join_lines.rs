use crate::late_join::BossFight;
use crate::load_gate::StoryPlace;

pub fn take_in_waiting_line(reason: &str) -> String {
    format!(
        "late join: the other player's game has arrived, it is taken in at the first safe moment ({reason})"
    )
}

pub fn take_in_done_line() -> String {
    "late join: a safe moment, the other player's game is taken in".to_owned()
}

pub fn arrival_beside_line(host: &StoryPlace) -> String {
    format!(
        "late join: this game is in the host's phase ({}) and both players are free, walking to the host once",
        host.describe()
    )
}

pub fn landing_begins_line(own: &StoryPlace, host: &StoryPlace, start_index: usize) -> String {
    format!(
        "late join: this game is at {}, the host at {}; landing at the host's chapter start, index {start_index}",
        own.describe(),
        host.describe()
    )
}

pub fn landing_no_single_chapter_line(host: &StoryPlace) -> String {
    format!(
        "late join: no single chapter start holds the host's place {}, this game stays where it is",
        host.describe()
    )
}

pub fn boss_fight_refusal() -> String {
    "a boss gauge is up and its target is alive (a guessed reading)".to_owned()
}

pub fn boss_hold_log_line(fight: &BossFight) -> String {
    format!(
        "late join: a boss fight is guessed to be running ({}, target handle {:#x}), the joiner waits for it to end",
        fight.kind.label(),
        fight.target_handle
    )
}

pub fn boss_wait_line(name: &str) -> String {
    format!("{name} waits for the boss fight to end")
}

pub fn boss_fight_over_line() -> String {
    "late join: the boss gauge reading says the fight is over, the joiner waits no longer for it"
        .to_owned()
}

pub fn boss_unreadable_line(reason: &str) -> String {
    format!("late join: the boss gauge could not be read ({reason}), no boss fight is assumed")
}

pub fn joiner_waits_line(host_name: &str) -> String {
    format!("Waiting for {host_name}'s boss fight to end")
}
