use replicant_game::host_led_stretch::{
    HostLedStretchState, StretchChange, StretchEnd, stretch_starting_at,
    stretch_the_story_is_inside,
};

const START_PACK: &str = "B_CENTER_VILLAGE_01";
const START_STEP: &str = "EID_3000_a0012_Start";
const END_PACK: &str = "B_CENTER_LIBRARY_01";
const END_STEP: &str = "EID_3200_a0052_End";
const PACK_ON_THE_WAY: &str = "B_SOUTH_MANSION_01";
const STORY_ON_FLAG: u16 = 556;
const STORY_OFF_FLAG: u16 = 125;
const FIRST_STEP_MS: u64 = 8_000;
const SIX_HOURS_LATER_MS: u64 = 6 * 60 * 60 * 1_000;
const LOAD_AT_THE_START_MS: u64 = 400;
const LOGGED_STEP_BETWEEN: &str = "EID_3000_a0015g_Start";

fn flags(on: &[u16]) -> impl Fn(u16) -> bool + '_ {
    move |id| on.contains(&id)
}

#[test]
fn the_first_step_the_log_shows_starts_the_watch() {
    let mut state = HostLedStretchState::new();
    let started = state.host_step(START_PACK, START_STEP, FIRST_STEP_MS);
    let Some(StretchChange::Started(stretch)) = started else {
        panic!("the first step began no stretch: {started:?}");
    };
    assert_eq!(stretch.last_step, END_STEP);
    assert_eq!(stretch.last_pack, END_PACK);
    assert_eq!(stretch_starting_at(START_PACK, START_STEP), Some(stretch));
}

#[test]
fn the_story_state_of_the_start_is_inside_in_every_pack_until_the_end_flag() {
    let inside = [STORY_ON_FLAG];
    for pack in [START_PACK, PACK_ON_THE_WAY, END_PACK] {
        assert!(stretch_the_story_is_inside(pack, flags(&inside)).is_some());
    }
    let ended = [STORY_ON_FLAG, STORY_OFF_FLAG];
    assert!(stretch_the_story_is_inside(END_PACK, flags(&ended)).is_none());
    assert!(stretch_the_story_is_inside(START_PACK, flags(&[])).is_none());
}

#[test]
fn a_guest_joining_in_the_span_starts_watching_from_the_story_state() {
    let mut state = HostLedStretchState::new();
    let started = state.start_from_the_story(
        PACK_ON_THE_WAY,
        flags(&[STORY_ON_FLAG]),
        LOAD_AT_THE_START_MS,
    );
    assert!(matches!(
        started,
        Some(StretchChange::StartedFromTheStory(_))
    ));
    assert!(state.current().is_some());
}

#[test]
fn walking_through_other_packs_and_hours_never_ends_the_watch() {
    let mut state = HostLedStretchState::new();
    state.host_step(START_PACK, START_STEP, FIRST_STEP_MS);
    assert_eq!(
        state.host_step(START_PACK, LOGGED_STEP_BETWEEN, 70_000),
        None
    );
    for pack in [PACK_ON_THE_WAY, END_PACK, START_PACK] {
        assert_eq!(state.check(Some(pack), SIX_HOURS_LATER_MS), None);
    }
    assert_eq!(state.check(None, SIX_HOURS_LATER_MS), None);
    assert!(state.current().is_some());
}

#[test]
fn the_end_step_in_its_pack_ends_the_watch_and_a_same_named_step_elsewhere_does_not() {
    let mut state = HostLedStretchState::new();
    let Some(StretchChange::Started(stretch)) =
        state.host_step(START_PACK, START_STEP, FIRST_STEP_MS)
    else {
        panic!("no stretch started");
    };
    assert_eq!(
        state.host_step(PACK_ON_THE_WAY, END_STEP, SIX_HOURS_LATER_MS),
        None
    );
    assert_eq!(
        state.host_step(END_PACK, END_STEP, SIX_HOURS_LATER_MS),
        Some(StretchChange::Ended(stretch, StretchEnd::LastStep))
    );
    assert_eq!(state.current(), None);
}

#[test]
fn a_story_state_past_the_end_ends_a_watch_whose_end_step_was_missed() {
    let mut state = HostLedStretchState::new();
    state.host_step(START_PACK, START_STEP, FIRST_STEP_MS);
    assert_eq!(
        state.end_when_the_story_moved_on(flags(&[STORY_ON_FLAG])),
        None
    );
    assert!(matches!(
        state.end_when_the_story_moved_on(flags(&[STORY_ON_FLAG, STORY_OFF_FLAG])),
        Some(StretchChange::Ended(_, StretchEnd::StoryMovedOn))
    ));
    assert_eq!(state.current(), None);
}

#[test]
fn a_save_from_before_the_start_does_not_keep_the_watch_on() {
    let mut state = HostLedStretchState::new();
    state.host_step(START_PACK, START_STEP, FIRST_STEP_MS);
    assert!(matches!(
        state.end_when_the_story_moved_on(flags(&[])),
        Some(StretchChange::Ended(_, StretchEnd::StoryMovedOn))
    ));
}

#[test]
fn the_ended_watch_is_not_started_again_by_the_story_state_after_the_end() {
    let mut state = HostLedStretchState::new();
    state.host_step(START_PACK, START_STEP, FIRST_STEP_MS);
    state.host_step(END_PACK, END_STEP, SIX_HOURS_LATER_MS);
    let after = [STORY_ON_FLAG, STORY_OFF_FLAG];
    assert_eq!(
        state.start_from_the_story(END_PACK, flags(&after), SIX_HOURS_LATER_MS + 1),
        None
    );
}
