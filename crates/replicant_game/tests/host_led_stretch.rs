use replicant_game::dialogue::{TalkOrigin, TalkOriginTracker};
use replicant_game::fade_sync::HostFadeSync;
use replicant_game::host_led_stretch::{
    HOST_LED_STRETCH_ENDS_ITSELF_AFTER_MS, HOST_LED_STRETCHES, HostLedStretchState, StretchChange,
    StretchEnd, ended_line, started_line, stretch_starting_at,
};
use replicant_game::story_watch::{GuestWatch, WatchEnd, WatchFacts, WatchStep};

const FOREST: &str = "C_FOREST_FIELD_01";
const FOREST_START: &str = "EID_1500_a0020g_0010_Start";
const FOREST_END: &str = "EID_1500_a0020g_0010_End";
const FOREST_STEPS_INSIDE: [&str; 6] = [
    "LIB_SET_CLOSE_2D_GAUGE",
    "CHANGE_WINDOW_SIZE",
    "SET_ACTOR_PAUSE_ON",
    "BG_AND_ENV_FADE_OUT",
    "BG_AND_ENV_FADE_IN",
    "BGM_C_FOREST_FIELD_01",
];
const FOREST_NEXT_TALK_STEP: &str = "EID_1500_a0107";
const VILLAGE: &str = "A_CENTER_VILLAGE_01";
const VILLAGE_START: &str = "EID_1500_a0005_010_01_Start";
const VILLAGE_END: &str = "EID_1500_a0005_010_01_End";
const VILLAGE_MORNING_STEP: &str = "EID_1500_a0005_010_01_End02";
const FOREST_START_MS: u64 = 64_897_363;
const FOREST_BG_FADE_OUT_MS: u64 = 64_963_143;
const FOREST_BG_FADE_IN_MS: u64 = 65_258_169;
const FOREST_END_MS: u64 = 65_310_015;
const FIRST_TALK: u32 = 1000;
const TALK_AFTER_WINDOW_SIZE: u32 = 1018;
const CHAINED_TALK: u32 = 1019;
const TALK_ON_THE_WAY_BACK: u32 = 1005;
const FIRST_TALK_OPEN_MS: u64 = 64_897_379;
const WINDOW_SIZE_SET_TALK_MS: u64 = 64_901_397;
const TALK_AFTER_WINDOW_SIZE_OPEN_MS: u64 = 64_901_413;
const CHAINED_TALK_OPEN_MS: u64 = 64_906_949;

fn watch_facts(host_leads_a_stretch: bool) -> WatchFacts<'static> {
    WatchFacts {
        setting_on: true,
        guest_in_session: true,
        story_has_taken_her: false,
        host_leads_a_stretch,
        pack: FOREST,
    }
}

#[test]
fn the_stretch_table_is_one_row_per_first_step() {
    for stretch in &HOST_LED_STRETCHES {
        assert_eq!(
            stretch_starting_at(stretch.pack, stretch.first_step),
            Some(stretch)
        );
        assert_eq!(stretch_starting_at(stretch.pack, stretch.last_step), None);
    }
}

#[test]
fn the_forest_text_scene_is_one_stretch_from_its_start_to_its_end() {
    let mut state = HostLedStretchState::new();
    let started = state.host_step(FOREST, FOREST_START, FOREST_START_MS);
    let Some(StretchChange::Started(stretch)) = started else {
        panic!("the forest start began no stretch: {started:?}");
    };
    assert_eq!(stretch.last_step, FOREST_END);
    for step in FOREST_STEPS_INSIDE {
        assert_eq!(state.host_step(FOREST, step, FOREST_BG_FADE_OUT_MS), None);
        assert_eq!(state.current(), Some(stretch));
    }
    assert_eq!(
        state.check(Some(FOREST), FOREST_BG_FADE_IN_MS),
        None,
        "five minutes of black screen are still inside the stretch"
    );
    assert_eq!(
        state.host_step(FOREST, FOREST_END, FOREST_END_MS),
        Some(StretchChange::Ended(stretch, StretchEnd::LastStep))
    );
    assert_eq!(state.current(), None);
    assert_eq!(
        state.host_step(FOREST, FOREST_NEXT_TALK_STEP, FOREST_END_MS + 4_159),
        None
    );
}

#[test]
fn the_village_text_scene_is_no_stretch_both_games_run_their_own_copy() {
    assert_eq!(stretch_starting_at(VILLAGE, VILLAGE_START), None);
    let mut state = HostLedStretchState::new();
    assert_eq!(state.host_step(VILLAGE, VILLAGE_START, 0), None);
    assert_eq!(state.host_step(VILLAGE, VILLAGE_END, 18_593), None);
    assert_eq!(state.host_step(VILLAGE, VILLAGE_MORNING_STEP, 40_459), None);
    assert_eq!(state.current(), None);
}

#[test]
fn a_first_step_in_another_pack_starts_nothing() {
    let mut state = HostLedStretchState::new();
    assert_eq!(state.host_step(VILLAGE, FOREST_START, 0), None);
    assert_eq!(state.current(), None);
}

#[test]
fn leaving_the_pack_ends_the_stretch_and_a_load_does_not() {
    let mut state = HostLedStretchState::new();
    state.host_step(FOREST, FOREST_START, 0);
    assert_eq!(state.check(None, 1_000), None);
    assert!(matches!(
        state.check(Some(VILLAGE), 2_000),
        Some(StretchChange::Ended(_, StretchEnd::LeftThePack))
    ));
    assert_eq!(state.current(), None);
}

#[test]
fn a_stretch_whose_end_never_comes_ends_itself() {
    let mut state = HostLedStretchState::new();
    state.host_step(FOREST, FOREST_START, 0);
    assert_eq!(
        state.check(Some(FOREST), HOST_LED_STRETCH_ENDS_ITSELF_AFTER_MS - 1),
        None
    );
    assert!(matches!(
        state.check(Some(FOREST), HOST_LED_STRETCH_ENDS_ITSELF_AFTER_MS),
        Some(StretchChange::Ended(_, StretchEnd::TookTooLong))
    ));
}

#[test]
fn the_session_ending_ends_the_stretch() {
    let mut state = HostLedStretchState::new();
    assert_eq!(state.end_with_the_session(), None);
    state.host_step(FOREST, FOREST_START, 0);
    assert!(matches!(
        state.end_with_the_session(),
        Some(StretchChange::Ended(_, StretchEnd::SessionOver))
    ));
}

#[test]
fn the_stretch_lines_name_its_steps() {
    let stretch = stretch_starting_at(FOREST, FOREST_START).expect("forest stretch");
    let host = started_line(stretch, true);
    assert!(host.contains(FOREST_START) && host.contains(FOREST_END));
    assert!(host.contains("the guest watches you"));
    assert!(started_line(stretch, false).contains("you watch the host"));
    assert!(ended_line(stretch, StretchEnd::LastStep).contains(FOREST_END));
}

#[test]
fn without_a_stretch_the_hosts_forest_talks_stay_its_own() {
    let mut origin = TalkOriginTracker::new();
    assert_eq!(
        origin.observe(Some(FIRST_TALK), true, FIRST_TALK_OPEN_MS),
        Some((FIRST_TALK, TalkOrigin::TalkCommand))
    );
    origin.note_script_talk_start("CHANGE_WINDOW_SIZE", WINDOW_SIZE_SET_TALK_MS);
    assert_eq!(
        origin.observe(
            Some(TALK_AFTER_WINDOW_SIZE),
            false,
            TALK_AFTER_WINDOW_SIZE_OPEN_MS
        ),
        Some((
            TALK_AFTER_WINDOW_SIZE,
            TalkOrigin::NotClassed(Some("CHANGE_WINDOW_SIZE".to_owned()))
        ))
    );
}

#[test]
fn inside_a_stretch_every_talk_of_the_hosts_is_a_story_step_the_guest_is_shown() {
    let mut origin = TalkOriginTracker::new();
    origin.set_host_led_stretch(Some(FOREST_START));
    let story = TalkOrigin::StoryStep(FOREST_START.to_owned());
    assert_eq!(
        origin.observe(Some(FIRST_TALK), true, FIRST_TALK_OPEN_MS),
        Some((FIRST_TALK, story.clone()))
    );
    origin.observe(None, false, FIRST_TALK_OPEN_MS + 4_000);
    origin.note_script_talk_start("CHANGE_WINDOW_SIZE", WINDOW_SIZE_SET_TALK_MS);
    assert_eq!(
        origin.observe(
            Some(TALK_AFTER_WINDOW_SIZE),
            false,
            TALK_AFTER_WINDOW_SIZE_OPEN_MS
        ),
        Some((TALK_AFTER_WINDOW_SIZE, story.clone()))
    );
    assert_eq!(
        origin.observe(Some(CHAINED_TALK), false, CHAINED_TALK_OPEN_MS),
        Some((CHAINED_TALK, story.clone()))
    );
    assert_eq!(
        origin.observe(Some(TALK_ON_THE_WAY_BACK), true, FOREST_END_MS - 4_272),
        Some((TALK_ON_THE_WAY_BACK, story))
    );
    origin.set_host_led_stretch(None);
    origin.observe(None, false, FOREST_END_MS);
    assert_eq!(
        origin.observe(Some(FIRST_TALK), true, FOREST_END_MS + 60_000),
        Some((FIRST_TALK, TalkOrigin::TalkCommand))
    );
}

#[test]
fn without_a_stretch_the_hosts_background_fade_is_not_sent() {
    let mut host = HostFadeSync::new();
    assert!(!host.should_send_fade_out(FOREST_BG_FADE_OUT_MS));
    assert!(!host.should_send_fade_in());
}

#[test]
fn inside_a_stretch_the_hosts_fades_go_to_the_guest_however_late() {
    let mut host = HostFadeSync::new();
    host.set_host_led_stretch(true);
    assert!(host.should_send_fade_out(FOREST_BG_FADE_OUT_MS));
    assert!(
        host.should_send_fade_in(),
        "the fade in at {FOREST_BG_FADE_IN_MS} ms"
    );
    host.set_host_led_stretch(false);
    assert!(!host.should_send_fade_out(FOREST_END_MS));
    assert!(!host.should_send_fade_in());
}

#[test]
fn the_guest_watches_a_stretch_with_the_companion_in_play_and_stops_at_its_end() {
    let mut watch = GuestWatch::new();
    assert_eq!(watch.step(&watch_facts(false)), WatchStep::Idle);
    assert_eq!(watch.step(&watch_facts(true)), WatchStep::Started);
    assert!(watch.for_a_stretch());
    assert_eq!(watch.step(&watch_facts(true)), WatchStep::Continues);
    assert_eq!(
        watch.step(&watch_facts(false)),
        WatchStep::Ended(WatchEnd::HostLedStretchOver)
    );
    assert!(!watch.watching());
    assert!(!watch.for_a_stretch());
}

#[test]
fn a_companion_watch_that_outlives_a_stretch_ends_as_a_companion_watch() {
    let mut watch = GuestWatch::new();
    let taken = WatchFacts {
        story_has_taken_her: true,
        ..watch_facts(false)
    };
    let taken_in_a_stretch = WatchFacts {
        host_leads_a_stretch: true,
        ..taken
    };
    watch.step(&taken_in_a_stretch);
    assert_eq!(watch.step(&taken), WatchStep::Continues);
    assert_eq!(
        watch.step(&watch_facts(false)),
        WatchStep::Ended(WatchEnd::StoryBroughtHerBack)
    );
}

#[test]
fn the_setting_off_never_watches_a_stretch() {
    let mut watch = GuestWatch::new();
    let off = WatchFacts {
        setting_on: false,
        ..watch_facts(true)
    };
    assert_eq!(watch.step(&off), WatchStep::Idle);
}
