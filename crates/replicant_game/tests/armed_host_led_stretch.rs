use replicant_game::host_led_stretch::{
    GUEST_REFUSED_DOOR_CALLBACK_PREFIX, HOST_LED_LONG_STRETCH_ENDS_ITSELF_AFTER_MS,
    HOST_LED_STRETCH_ENDS_ITSELF_AFTER_MS, HostLedStretchState, StretchChange, StretchEnd,
    armed_line, door_refused_line, host_run_not_replayed_line, stretch_armed_by,
};
use replicant_game::story_calls::{PartnerPlace, Whereabouts};

const FIRST_PACK: &str = "A_SOUTH_MANSION_01";
const LAST_PACK: &str = "A_SOUTH_MANSION_02";
const ARMING_STEP: &str = "ScriptMode_to_Kaine_03_04";
const DOOR_STEP: &str = "Open_Door_01";
const LAST_STEP: &str = "EID_1100_a0080_01_End";
const DOOR_BEFORE_ARMING_MS: u64 = 1_967_294;
const ARMED_MS: u64 = 1_986_927;
const DOOR_AFTER_ARMING_MS: u64 = 1_996_927;
const FIRST_GOBACK_MS: u64 = 2_035_565;

fn started_state() -> HostLedStretchState {
    let mut state = HostLedStretchState::new();
    state.host_step(FIRST_PACK, ARMING_STEP, ARMED_MS);
    state.host_step(FIRST_PACK, DOOR_STEP, DOOR_AFTER_ARMING_MS);
    state
}

fn at(phase: u16, x: f32) -> Option<Whereabouts> {
    Some(Whereabouts {
        phase,
        position: [x, 0.0, 0.0],
    })
}

#[test]
fn a_door_used_before_the_arming_step_starts_nothing() {
    let mut state = HostLedStretchState::new();
    assert_eq!(
        state.host_step(FIRST_PACK, DOOR_STEP, DOOR_BEFORE_ARMING_MS),
        None
    );
    assert_eq!(state.current(), None);
    assert_eq!(state.door_ban(), None);
}

#[test]
fn the_arming_step_bans_the_guest_doors_before_the_stretch_starts() {
    let mut state = HostLedStretchState::new();
    let change = state.host_step(FIRST_PACK, ARMING_STEP, ARMED_MS);
    assert!(matches!(change, Some(StretchChange::Armed(_))));
    assert_eq!(state.current(), None);
    assert_eq!(state.door_ban().map(|s| s.first_step), Some(DOOR_STEP));
}

#[test]
fn the_first_door_after_arming_starts_the_stretch_and_keeps_the_ban() {
    let mut state = HostLedStretchState::new();
    state.host_step(FIRST_PACK, ARMING_STEP, ARMED_MS);
    let change = state.host_step(FIRST_PACK, DOOR_STEP, DOOR_AFTER_ARMING_MS);
    assert!(matches!(change, Some(StretchChange::Started(_))));
    assert_eq!(state.current().map(|s| s.first_step), Some(DOOR_STEP));
    assert_eq!(state.door_ban().map(|s| s.first_step), Some(DOOR_STEP));
    assert_eq!(state.armed(), None);
}

#[test]
fn later_steps_inside_the_stretch_change_nothing() {
    let mut state = started_state();
    assert_eq!(
        state.host_step(FIRST_PACK, "EID_1100_a0030_GoBack", FIRST_GOBACK_MS),
        None
    );
    assert_eq!(
        state.host_step(FIRST_PACK, DOOR_STEP, FIRST_GOBACK_MS),
        None
    );
    assert!(state.current().is_some());
}

#[test]
fn the_stretch_goes_on_into_the_next_pack_and_ends_at_its_last_step() {
    let mut state = started_state();
    assert_eq!(state.check(Some(LAST_PACK), FIRST_GOBACK_MS), None);
    assert_eq!(state.check(Some(FIRST_PACK), FIRST_GOBACK_MS), None);
    let change = state.host_step(LAST_PACK, LAST_STEP, FIRST_GOBACK_MS);
    assert!(matches!(
        change,
        Some(StretchChange::Ended(_, StretchEnd::LastStep))
    ));
    assert_eq!(state.door_ban(), None);
}

#[test]
fn the_last_step_in_the_first_pack_does_not_end_it() {
    let mut state = started_state();
    assert_eq!(
        state.host_step(FIRST_PACK, LAST_STEP, FIRST_GOBACK_MS),
        None
    );
    assert!(state.current().is_some());
}

#[test]
fn a_third_pack_ends_the_stretch_and_lifts_the_ban() {
    let mut state = started_state();
    assert!(matches!(
        state.check(Some("A_SOUTH_FIELD_01"), FIRST_GOBACK_MS),
        Some(StretchChange::Ended(_, StretchEnd::LeftThePack))
    ));
    assert_eq!(state.door_ban(), None);
}

#[test]
fn an_armed_stretch_is_dropped_when_this_game_leaves_the_pack() {
    let mut state = HostLedStretchState::new();
    state.host_step(FIRST_PACK, ARMING_STEP, ARMED_MS);
    assert_eq!(state.check(Some(FIRST_PACK), ARMED_MS + 1), None);
    assert!(matches!(
        state.check(Some(LAST_PACK), ARMED_MS + 2),
        Some(StretchChange::Disarmed(_, StretchEnd::LeftThePack))
    ));
    assert_eq!(state.door_ban(), None);
}

#[test]
fn the_session_ending_drops_an_armed_stretch_and_a_running_one() {
    let mut armed = HostLedStretchState::new();
    armed.host_step(FIRST_PACK, ARMING_STEP, ARMED_MS);
    assert!(matches!(
        armed.end_with_the_session(),
        Some(StretchChange::Disarmed(_, StretchEnd::SessionOver))
    ));
    let mut running = started_state();
    assert!(matches!(
        running.end_with_the_session(),
        Some(StretchChange::Ended(_, StretchEnd::SessionOver))
    ));
    assert_eq!(running.door_ban(), None);
}

#[test]
fn this_stretch_outlasts_the_short_limit_and_ends_at_its_own() {
    let mut state = started_state();
    let past_short = DOOR_AFTER_ARMING_MS + HOST_LED_STRETCH_ENDS_ITSELF_AFTER_MS;
    assert_eq!(state.check(Some(FIRST_PACK), past_short), None);
    let past_long = DOOR_AFTER_ARMING_MS + HOST_LED_LONG_STRETCH_ENDS_ITSELF_AFTER_MS;
    assert!(matches!(
        state.check(Some(FIRST_PACK), past_long),
        Some(StretchChange::Ended(_, StretchEnd::TookTooLong))
    ));
}

#[test]
fn the_arming_step_belongs_to_one_pack_only() {
    assert!(stretch_armed_by(FIRST_PACK, ARMING_STEP).is_some());
    assert!(stretch_armed_by(LAST_PACK, ARMING_STEP).is_none());
    assert!(stretch_armed_by(FIRST_PACK, DOOR_STEP).is_none());
}

#[test]
fn the_door_script_names_start_with_the_refused_prefix() {
    assert!(DOOR_STEP.starts_with(GUEST_REFUSED_DOOR_CALLBACK_PREFIX));
    assert!("Open_Door_02".starts_with(GUEST_REFUSED_DOOR_CALLBACK_PREFIX));
    assert!(!"Not_Open_Door_01".starts_with(GUEST_REFUSED_DOOR_CALLBACK_PREFIX));
}

#[test]
fn the_lines_name_the_steps_and_say_nothing_of_the_story() {
    let mut state = HostLedStretchState::new();
    let Some(StretchChange::Armed(stretch)) = state.host_step(FIRST_PACK, ARMING_STEP, ARMED_MS)
    else {
        panic!("not armed");
    };
    let lines = [
        armed_line(stretch, ARMING_STEP),
        door_refused_line(stretch, DOOR_STEP),
        host_run_not_replayed_line(stretch, "EID_1100_a0030_GoBack"),
    ];
    assert!(lines[0].contains(ARMING_STEP));
    assert!(lines[1].contains(LAST_STEP));
    assert!(lines[2].contains(LAST_STEP));
}

#[test]
fn a_watching_partner_counts_as_in_the_same_place_while_far_in_the_same_phase() {
    assert_eq!(
        PartnerPlace::judge_counting_a_watcher(at(20, 0.0), at(20, 500.0), true),
        PartnerPlace::Same
    );
    assert!(matches!(
        PartnerPlace::judge_counting_a_watcher(at(20, 0.0), at(20, 500.0), false),
        PartnerPlace::TooFar { .. }
    ));
}

#[test]
fn a_watching_partner_in_another_phase_does_not_count_as_here() {
    assert_eq!(
        PartnerPlace::judge_counting_a_watcher(at(20, 0.0), at(21, 0.0), true),
        PartnerPlace::OtherPhase
    );
    assert_eq!(
        PartnerPlace::judge_counting_a_watcher(at(20, 0.0), None, true),
        PartnerPlace::Unknown
    );
}
