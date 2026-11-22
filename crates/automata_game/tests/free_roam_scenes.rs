use automata_game::event_follow::{EventId, EventMachine, EventReadings, free_roam_scene_line};
use automata_game::free_roam_scenes::{
    HeldActions, MOST_HELD_ACTION_IDS_REMEMBERED, PHASE_EVENT_ACTION_IDLE_STATE,
    PHASE_EVENT_ACTION_LOADED_STATE, UpdateHold, area_update_is_held, held_action_line,
};
use automata_game::load_gate::StoryPlace;
use automata_game::travel::JumpRole;

const RUNNING_STATE: u32 = 0xA09A_3421;
const FADE_STATE: u32 = 0xB572_274C;
const NOT_ASKED: u8 = 0;
const ASKED: u8 = 1;

fn scene() -> EventId {
    EventId {
        kind: 0,
        id: 0x210,
        sub: -1,
    }
}

fn readings(now_ms: u64) -> EventReadings {
    EventReadings {
        now_ms,
        role: JumpRole::Host,
        partner_present: true,
        local_in_play: true,
        partner_in_play: true,
        current_phase_id: 0x100,
        current_place: StoryPlace::new(0x100, "A"),
        partner_place: StoryPlace::new(0x100, "A"),
        partner_playing: None,
        slots: Vec::new(),
        event_flags_say_playing: false,
        started_heard: Vec::new(),
        ended_heard: Vec::new(),
        asks: Vec::new(),
        partner_report: None,
        local_unattended: None,
        skip_asks_heard: Vec::new(),
        local_event_pause_open: false,
        pauses_heard: Vec::new(),
        progress: None,
    }
}

fn logs_of_a_start(area_starts_held: Option<bool>, asked_by_the_script: bool) -> Vec<String> {
    let mut machine = EventMachine::new(true);
    machine.set_free_roam_area_starts_held(area_starts_held);
    machine.step(&readings(0));
    if asked_by_the_script {
        machine.on_play_request(scene(), 90);
    }
    machine.on_start(scene(), 100);
    machine.step(&readings(110)).logs
}

#[test]
fn an_idle_action_that_nobody_asked_for_is_held_in_free_roam() {
    assert!(area_update_is_held(
        true,
        PHASE_EVENT_ACTION_IDLE_STATE,
        NOT_ASKED
    ));
}

#[test]
fn a_loaded_action_that_nobody_asked_for_is_held_in_free_roam() {
    assert!(area_update_is_held(
        true,
        PHASE_EVENT_ACTION_LOADED_STATE,
        NOT_ASKED
    ));
}

#[test]
fn the_campaign_never_holds_an_area_update() {
    assert!(!area_update_is_held(
        false,
        PHASE_EVENT_ACTION_IDLE_STATE,
        NOT_ASKED
    ));
    assert!(!area_update_is_held(
        false,
        PHASE_EVENT_ACTION_LOADED_STATE,
        NOT_ASKED
    ));
}

#[test]
fn a_loaded_action_the_script_asked_to_play_is_never_held() {
    assert!(!area_update_is_held(
        true,
        PHASE_EVENT_ACTION_LOADED_STATE,
        ASKED
    ));
}

#[test]
fn an_idle_action_the_script_asked_to_play_is_never_held() {
    assert!(!area_update_is_held(
        true,
        PHASE_EVENT_ACTION_IDLE_STATE,
        ASKED
    ));
}

#[test]
fn an_action_that_is_already_running_is_never_held() {
    assert!(!area_update_is_held(true, RUNNING_STATE, NOT_ASKED));
    assert!(!area_update_is_held(true, FADE_STATE, NOT_ASKED));
    assert!(!area_update_is_held(true, FADE_STATE, ASKED));
}

#[test]
fn the_first_hold_of_an_action_is_logged_and_the_next_ones_are_not() {
    let mut held = HeldActions::new();
    assert!(held.is_first_hold_of(0x1234));
    assert!(!held.is_first_hold_of(0x1234));
    assert!(held.is_first_hold_of(0x1235));
    assert_eq!(held.actions_held(), 2);
}

#[test]
fn the_held_actions_are_remembered_up_to_a_limit() {
    let mut held = HeldActions::new();
    for id in 0..MOST_HELD_ACTION_IDS_REMEMBERED as u32 {
        assert!(held.is_first_hold_of(id));
    }
    assert!(!held.is_first_hold_of(u32::MAX));
    assert_eq!(held.actions_held(), MOST_HELD_ACTION_IDS_REMEMBERED);
}

#[test]
fn the_held_line_names_the_action_its_state_and_the_play_flag() {
    let line = held_action_line(
        UpdateHold::AreaStart,
        0xABCD,
        PHASE_EVENT_ACTION_LOADED_STATE,
        NOT_ASKED,
        3,
    );
    assert!(line.contains("0xABCD"));
    assert!(line.contains("0xBB7F80A1"));
    assert!(line.contains("play asked 0"));
}

#[test]
fn a_free_roam_scene_start_says_where_it_came_from() {
    let logs = logs_of_a_start(Some(true), true);
    let script_line = logs
        .iter()
        .find(|line| line.starts_with("free roam: scene"));
    assert!(
        script_line
            .is_some_and(|line| line.contains("story script") && line.contains("held: true"))
    );
    let logs = logs_of_a_start(Some(false), false);
    let other_line = logs
        .iter()
        .find(|line| line.starts_with("free roam: scene"));
    assert!(other_line.is_some_and(
        |line| line.contains("something other than the story script")
            && line.contains("held: false")
    ));
}

#[test]
fn a_campaign_scene_start_has_no_free_roam_line() {
    let logs = logs_of_a_start(None, true);
    assert!(logs.iter().all(|line| !line.starts_with("free roam:")));
}

#[test]
fn the_free_roam_scene_line_is_the_one_the_machine_logs() {
    let logs = logs_of_a_start(Some(true), true);
    let expected = free_roam_scene_line(
        scene(),
        automata_game::event_follow::StartOrigin::Script,
        true,
    );
    assert!(logs.contains(&expected));
}
