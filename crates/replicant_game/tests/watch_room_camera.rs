use replicant_game::fight_sync::SET_FIXED_CAMERA_NATIVE_RVA;
use replicant_game::side_view::{
    CHANGE_CAMERA_NATIVE_RVA, CHANGE_CAMERA_STATUS_NATIVE_RVA, OwnSideViewDoor, ViewArg,
    door_by_id, side_view_on_calls,
};
use replicant_game::story_watch::{
    RoomViewReading, RoomViewStep, WatchedRoomView, camera_call_is_followed,
    watched_camera_applies_here, watched_camera_run,
};
use replicant_proto::{WatchedCameraCall, WatchedCameraNative};

const SECOND_STAGE_LIBRARY: &str = "B_CENTER_LIBRARY_01";
const WEST_ROOM_TWO_DOOR: u16 = 5;
const WEST_ROOM_TWO_ENTRY: &str = "GM_IN_Door_of_WestRoom2_02";
const POPOL_ROOM_TWO_ENTRY: &str = "GM_IN_Door_of_PopolRoom2_02";
const POPOL_ROOM_TWO_DOOR: u16 = 3;

fn reading(host_door: u16, in_side_view: bool, watching: bool) -> RoomViewReading<'static> {
    RoomViewReading {
        area: SECOND_STAGE_LIBRARY,
        watching,
        in_play_and_free: true,
        in_side_view,
        host_side_view_door: host_door,
    }
}

fn fixed_camera_from_the_run() -> WatchedCameraCall {
    WatchedCameraCall {
        phase: 32,
        native: WatchedCameraNative::SetFixedCamera,
        numbers: [18.33, 6.72, 9.96, 2.8, 0.0, -0.44],
    }
}

#[test]
fn the_hosts_west_room_in_the_second_stage_is_a_listed_door() {
    let mut own = OwnSideViewDoor::new();
    own.view_flipped(true, Some(SECOND_STAGE_LIBRARY), WEST_ROOM_TWO_ENTRY);
    assert_eq!(own.door(), WEST_ROOM_TWO_DOOR);
    own.view_flipped(
        false,
        Some(SECOND_STAGE_LIBRARY),
        "GM_OUT_Door_of_WestRoom2_02",
    );
    assert_eq!(own.door(), 0);
}

#[test]
fn the_hosts_popola_room_in_the_second_stage_is_a_listed_door() {
    let mut own = OwnSideViewDoor::new();
    own.view_flipped(true, Some(SECOND_STAGE_LIBRARY), POPOL_ROOM_TWO_ENTRY);
    assert_eq!(own.door(), POPOL_ROOM_TWO_DOOR);
}

#[test]
fn a_watching_guest_takes_the_room_the_host_walked_into() {
    let mut view = WatchedRoomView::new();
    let door = door_by_id(SECOND_STAGE_LIBRARY, WEST_ROOM_TWO_DOOR).expect("a listed door");
    assert_eq!(
        view.observe(&reading(WEST_ROOM_TWO_DOOR, false, true)),
        RoomViewStep::Take(door)
    );
    assert_eq!(
        view.observe(&reading(WEST_ROOM_TWO_DOOR, true, true)),
        RoomViewStep::Nothing
    );
}

#[test]
fn a_watching_guest_leaves_the_room_when_the_host_walks_out() {
    let mut view = WatchedRoomView::new();
    let door = door_by_id(SECOND_STAGE_LIBRARY, WEST_ROOM_TWO_DOOR).expect("a listed door");
    view.observe(&reading(WEST_ROOM_TWO_DOOR, false, true));
    assert_eq!(
        view.observe(&reading(0, true, true)),
        RoomViewStep::Leave(door)
    );
    assert_eq!(
        view.observe(&reading(0, false, true)),
        RoomViewStep::Nothing
    );
}

#[test]
fn a_guest_that_plays_takes_no_room_from_the_host() {
    let mut view = WatchedRoomView::new();
    assert_eq!(
        view.observe(&reading(WEST_ROOM_TWO_DOOR, false, false)),
        RoomViewStep::Nothing
    );
}

#[test]
fn the_room_view_on_calls_leave_the_camera_status_to_the_hosts_own_call() {
    let door = door_by_id(SECOND_STAGE_LIBRARY, WEST_ROOM_TWO_DOOR).expect("a listed door");
    assert!(
        side_view_on_calls(door, None)
            .iter()
            .all(|call| call.native_rva != CHANGE_CAMERA_STATUS_NATIVE_RVA)
    );
    assert!(
        side_view_on_calls(door, Some(2))
            .iter()
            .any(|call| call.native_rva == CHANGE_CAMERA_STATUS_NATIVE_RVA)
    );
}

#[test]
fn the_hosts_fixed_camera_from_the_run_is_replayed_with_its_eye_and_target() {
    let run = watched_camera_run(&fixed_camera_from_the_run()).expect("followed");
    assert_eq!(run.native_rva, SET_FIXED_CAMERA_NATIVE_RVA);
    assert_eq!(run.args.len(), 6);
    assert!(matches!(run.args[0], ViewArg::Real(eye) if (eye - 18.33).abs() < 1e-4));
}

#[test]
fn only_the_fixed_and_the_free_camera_switches_are_followed() {
    let change = |kind: f32| {
        camera_call_is_followed(
            WatchedCameraNative::ChangeCamera,
            &[kind, 1.0, 1.0, 0.0, 0.0, 0.0],
        )
    };
    assert!(change(0.0));
    assert!(change(1.0));
    assert!(!change(4.0));
    assert!(!change(5.0));
}

#[test]
fn the_hosts_change_camera_keeps_its_integer_slots() {
    let call = WatchedCameraCall {
        phase: 32,
        native: WatchedCameraNative::ChangeCamera,
        numbers: [1.0, 1.0, 1.0, 0.0, 0.0, 0.0],
    };
    let run = watched_camera_run(&call).expect("followed");
    assert_eq!(run.native_rva, CHANGE_CAMERA_NATIVE_RVA);
    assert_eq!(
        run.args,
        vec![ViewArg::Int(1), ViewArg::Real(1.0), ViewArg::Int(1)]
    );
}

#[test]
fn a_camera_call_from_another_area_is_not_replayed() {
    assert!(watched_camera_applies_here(32, Some(32)));
    assert!(!watched_camera_applies_here(32, Some(31)));
    assert!(!watched_camera_applies_here(32, None));
}
