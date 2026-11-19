use replicant_game::fade_sync::{
    FADE_IN_REPEAT_WINDOW_MS, GuestFadeState, HostFadeSync, SCENE_START_FADE_WINDOW_MS,
};
use replicant_game::side_view::{
    INDOOR_ARRIVAL_SETTLE_MS, SCENE_ARRIVAL_WAITS_FOR_THE_HOST_MS, SceneArrivalReading,
    SceneArrivalWatch,
};

const VILLAGE: &str = "A_CENTER_VILLAGE_01";
const MORNING_STEP: &str = "EID_1500_a0005_010_01_End02";
const SCENE_START_FIRED_MS: u64 = 28_574;
const SCENE_FADE_OUT_MS: u64 = 28_590;
const HOST_FADE_IN_MS: u64 = 57_034;
const GUEST_OWN_FADE_IN_MS: u64 = 57_561;
const HOUSE_DOOR: u16 = 1;

fn reading(now_ms: u64) -> SceneArrivalReading {
    SceneArrivalReading {
        now_ms,
        in_play_and_free: true,
        in_side_view: false,
        host_side_view_door: HOUSE_DOOR,
    }
}

#[test]
fn the_scene_start_fade_out_is_sent_when_the_guest_is_elsewhere_and_its_fade_in_after_it() {
    let mut host = HostFadeSync::new();
    host.arm_for_scene_start(SCENE_START_FIRED_MS);
    assert!(host.should_send_fade_out(SCENE_FADE_OUT_MS));
    assert!(!host.should_send_fade_out(SCENE_FADE_OUT_MS + 1));
    assert!(host.should_send_fade_in());
}

#[test]
fn a_fade_out_long_after_the_scene_start_is_not_sent() {
    let mut host = HostFadeSync::new();
    host.arm_for_scene_start(SCENE_START_FIRED_MS);
    assert!(!host.should_send_fade_out(SCENE_START_FIRED_MS + SCENE_START_FADE_WINDOW_MS));
    assert!(!host.should_send_fade_in());
}

#[test]
fn the_guest_keeps_the_black_screen_through_the_whole_scene() {
    let mut guest = GuestFadeState::new();
    guest.started_fade_out(SCENE_FADE_OUT_MS);
    assert!(!guest.backstop_due(HOST_FADE_IN_MS));
}

#[test]
fn the_guests_own_fade_in_half_a_second_after_the_mirrored_one_is_not_run() {
    let mut guest = GuestFadeState::new();
    guest.started_fade_out(SCENE_FADE_OUT_MS);
    guest.mirrored_fade_in(HOST_FADE_IN_MS);
    assert!(!guest.holds_fade());
    assert!(guest.own_fade_in_repeats_mirror(GUEST_OWN_FADE_IN_MS));
    assert!(!guest.own_fade_in_repeats_mirror(HOST_FADE_IN_MS + FADE_IN_REPEAT_WINDOW_MS));
}

#[test]
fn the_mirrored_fade_in_is_not_called_after_the_guests_own_one_just_ran() {
    let mut guest = GuestFadeState::new();
    guest.started_fade_out(SCENE_FADE_OUT_MS);
    assert!(!guest.own_fade_in_repeats_mirror(HOST_FADE_IN_MS));
    assert!(!guest.holds_fade());
    assert!(guest.mirror_repeats_own_fade_in(GUEST_OWN_FADE_IN_MS));
    assert!(!guest.mirror_repeats_own_fade_in(HOST_FADE_IN_MS + FADE_IN_REPEAT_WINDOW_MS));
}

#[test]
fn the_morning_step_brings_the_house_view_to_a_guest_placed_inside_without_it() {
    let mut watch = SceneArrivalWatch::new();
    watch.note_callback(VILLAGE, MORNING_STEP, 429_055);
    assert!(watch.observe(&reading(429_100)).is_none());
    let door = watch
        .observe(&reading(429_100 + INDOOR_ARRIVAL_SETTLE_MS))
        .expect("the house view is due");
    assert_eq!(door.id, HOUSE_DOOR);
    assert_eq!(door.entry_steps[0], "GM_IN_Door_of_NierHouse_02");
    assert!(
        watch
            .observe(&reading(429_100 + 2 * INDOOR_ARRIVAL_SETTLE_MS))
            .is_none()
    );
}

#[test]
fn a_guest_already_in_the_view_or_a_host_outside_it_is_left_alone() {
    let mut watch = SceneArrivalWatch::new();
    watch.note_callback(VILLAGE, MORNING_STEP, 0);
    let in_view = SceneArrivalReading {
        in_side_view: true,
        ..reading(0)
    };
    assert!(watch.observe(&in_view).is_none());
    assert!(watch.observe(&reading(5_000)).is_none());

    watch.note_callback(VILLAGE, MORNING_STEP, 0);
    let host_outside = SceneArrivalReading {
        host_side_view_door: 0,
        ..reading(0)
    };
    assert!(watch.observe(&host_outside).is_none());
    assert!(watch.observe(&host_outside).is_none());
    assert!(
        watch
            .observe(&reading(SCENE_ARRIVAL_WAITS_FOR_THE_HOST_MS + 1))
            .is_none()
    );
}

#[test]
fn another_step_or_area_does_not_wait_for_the_house_view() {
    let mut watch = SceneArrivalWatch::new();
    watch.note_callback(VILLAGE, "EID_1500_a0005_010_01_End", 0);
    watch.note_callback("A_CENTER_LIBRARY_01", MORNING_STEP, 0);
    assert!(watch.observe(&reading(0)).is_none());
    assert!(watch.observe(&reading(INDOOR_ARRIVAL_SETTLE_MS)).is_none());
}

#[test]
fn a_guest_who_is_not_free_waits_for_a_full_settle_after_she_is() {
    let mut watch = SceneArrivalWatch::new();
    watch.note_callback(VILLAGE, MORNING_STEP, 0);
    let busy = SceneArrivalReading {
        in_play_and_free: false,
        ..reading(0)
    };
    assert!(watch.observe(&busy).is_none());
    assert!(watch.observe(&reading(2_000)).is_none());
    assert!(
        watch
            .observe(&reading(2_000 + INDOOR_ARRIVAL_SETTLE_MS))
            .is_some()
    );
}
