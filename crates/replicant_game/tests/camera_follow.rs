use replicant_game::fight_sync::{
    CameraSwitch, CameraSwitchAsker, FIXED_CAMERA_EYE_LERP_END_OFFSET, FIXED_CAMERA_INDEX,
    FIXED_CAMERA_LOOK_ACTOR_GENERATION_OFFSET, FIXED_CAMERA_LOOK_LERP_END_OFFSET,
    FIXED_CAMERA_LOOK_POINT_OFFSET, FIXED_CAMERA_RVA, FixedCameraPose, GuestCameraFollow,
    HostCamera, HostCameraFollow, HostCameraReceipt, HostCameraSkip, PATH_CAMERA_INDEX,
    SCRIPT_CAMERA_INDEX, SNOW_CAMERA_INDEX, camera_switch_line, change_kind_of_camera,
    fixed_camera_text, read_fixed_camera_pose, read_fixed_camera_reading,
};

use crate::fake_memory::FakeMemory;

const POSE: FixedCameraPose = FixedCameraPose {
    eye: [-0.054, 74.72, 5.197],
    look_at: [-0.017, 49.389, -65.29],
};

fn host_camera(seq: u32, natives_before: u32) -> HostCamera {
    HostCamera {
        seq,
        natives_before,
        camera: FIXED_CAMERA_INDEX,
        blend_seconds: 3.0,
        blend_kind: 1,
        pose: Some(POSE),
    }
}

fn switch_to_fixed() -> CameraSwitch {
    CameraSwitch {
        from: SNOW_CAMERA_INDEX,
        to: FIXED_CAMERA_INDEX,
        blend_seconds: 3.0,
        blend_kind: 1,
    }
}

#[test]
fn the_guest_takes_the_hosts_camera_when_its_own_differs() {
    let mut follow = GuestCameraFollow::default();
    assert_eq!(
        follow.receive(host_camera(0, 0), true),
        HostCameraReceipt::Kept
    );
    assert_eq!(follow.tick(true, Some(SNOW_CAMERA_INDEX)), None);
    assert_eq!(
        follow.tick(true, Some(SNOW_CAMERA_INDEX)),
        Some(Ok(host_camera(0, 0)))
    );
    assert_eq!(follow.tick(true, Some(SNOW_CAMERA_INDEX)), None);
}

#[test]
fn no_camera_is_taken_outside_a_fight_window() {
    let mut follow = GuestCameraFollow::default();
    assert_eq!(
        follow.receive(host_camera(0, 0), false),
        HostCameraReceipt::OutsideAFightWindow
    );
    assert_eq!(follow.tick(true, Some(SNOW_CAMERA_INDEX)), None);
    assert_eq!(follow.tick(true, Some(SNOW_CAMERA_INDEX)), None);

    follow.receive(host_camera(1, 0), true);
    assert_eq!(follow.tick(false, Some(SNOW_CAMERA_INDEX)), None);
    assert_eq!(follow.tick(true, Some(SNOW_CAMERA_INDEX)), None);
    assert_eq!(follow.tick(true, Some(SNOW_CAMERA_INDEX)), None);

    let mut host = HostCameraFollow::default();
    assert_eq!(host.message_for(switch_to_fixed(), Some(POSE), false), None);
}

#[test]
fn a_camera_the_guest_already_has_is_not_set_again() {
    let mut follow = GuestCameraFollow::default();
    follow.receive(host_camera(0, 0), true);
    follow.tick(true, Some(FIXED_CAMERA_INDEX));
    assert_eq!(
        follow.tick(true, Some(FIXED_CAMERA_INDEX)),
        Some(Err((host_camera(0, 0), HostCameraSkip::AlreadyActive)))
    );
}

#[test]
fn an_older_message_never_undoes_a_newer_native() {
    let mut follow = GuestCameraFollow::default();
    follow.receive(host_camera(4, 2), true);
    follow.note_native(true);
    follow.note_native(true);
    follow.note_native(true);
    follow.tick(true, Some(SNOW_CAMERA_INDEX));
    assert_eq!(
        follow.tick(true, Some(SNOW_CAMERA_INDEX)),
        Some(Err((host_camera(4, 2), HostCameraSkip::OlderThanANative)))
    );

    assert_eq!(
        follow.receive(host_camera(3, 5), true),
        HostCameraReceipt::OlderThanTheLast
    );
    assert_eq!(
        follow.receive(host_camera(5, 3), true),
        HostCameraReceipt::Kept
    );
    follow.tick(true, Some(SNOW_CAMERA_INDEX));
    assert_eq!(
        follow.tick(true, Some(SNOW_CAMERA_INDEX)),
        Some(Ok(host_camera(5, 3)))
    );
}

#[test]
fn a_later_host_camera_replaces_one_not_yet_applied() {
    let mut follow = GuestCameraFollow::default();
    follow.receive(host_camera(0, 0), true);
    let back_to_snow = HostCamera {
        camera: SNOW_CAMERA_INDEX,
        pose: None,
        ..host_camera(1, 0)
    };
    follow.receive(back_to_snow, true);
    follow.tick(true, Some(FIXED_CAMERA_INDEX));
    assert_eq!(
        follow.tick(true, Some(FIXED_CAMERA_INDEX)),
        Some(Ok(back_to_snow))
    );
}

#[test]
fn the_host_numbers_its_switches_and_counts_natives_before_them() {
    let mut host = HostCameraFollow::default();
    host.note_native(true);
    host.note_native(false);
    let first = host.message_for(switch_to_fixed(), Some(POSE), true);
    assert_eq!(first, Some(host_camera(0, 1)));
    let to_snow = CameraSwitch {
        from: FIXED_CAMERA_INDEX,
        to: SNOW_CAMERA_INDEX,
        ..switch_to_fixed()
    };
    let second = host
        .message_for(to_snow, Some(POSE), true)
        .expect("in window");
    assert_eq!((second.seq, second.pose), (1, None));
    host.window_closed();
    let third = host
        .message_for(switch_to_fixed(), Some(POSE), true)
        .expect("in window");
    assert_eq!((third.seq, third.natives_before), (2, 0));
}

#[test]
fn a_camera_without_a_change_kind_is_not_taken() {
    let mut follow = GuestCameraFollow::default();
    let other = HostCamera {
        camera: 9,
        pose: None,
        ..host_camera(0, 0)
    };
    follow.receive(other, true);
    follow.tick(true, Some(SNOW_CAMERA_INDEX));
    assert_eq!(
        follow.tick(true, Some(SNOW_CAMERA_INDEX)),
        Some(Err((other, HostCameraSkip::NoChangeKind)))
    );
}

#[test]
fn the_fixed_cameras_pose_is_read_from_its_lerp_ends_not_the_drawn_look_point() {
    let mut memory = FakeMemory::new();
    let floats = |values: [f32; 3]| -> Vec<u8> {
        values
            .iter()
            .flat_map(|value| value.to_le_bytes())
            .collect()
    };
    let at = |offset: u64| FIXED_CAMERA_RVA + offset as u32;
    let stale_look = [-14.165, 74.804, 6.049];
    memory.put_at_rva(at(FIXED_CAMERA_EYE_LERP_END_OFFSET), &floats(POSE.eye));
    memory.put_at_rva(at(FIXED_CAMERA_LOOK_LERP_END_OFFSET), &floats(POSE.look_at));
    memory.put_at_rva(at(FIXED_CAMERA_LOOK_POINT_OFFSET), &floats(stale_look));
    memory.put_at_rva(
        at(FIXED_CAMERA_LOOK_ACTOR_GENERATION_OFFSET),
        &0_i32.to_le_bytes(),
    );
    assert_eq!(read_fixed_camera_pose(&memory), Some(POSE));

    memory.put_at_rva(
        at(FIXED_CAMERA_LOOK_ACTOR_GENERATION_OFFSET),
        &7_i32.to_le_bytes(),
    );
    let following = read_fixed_camera_reading(&memory).expect("readable");
    assert!(following.follows_an_actor());
    assert_eq!(following.pose().look_at, stale_look);
    assert_eq!(
        fixed_camera_text(Some(following)),
        "fixed camera eye -0.054 74.720 5.197 look -0.017 49.389 -65.290 \
         (look now -14.165 74.804 6.049, look actor yes)"
    );

    memory.put_at_rva(
        at(FIXED_CAMERA_LOOK_LERP_END_OFFSET),
        &floats([f32::NAN, 0.0, 0.0]),
    );
    assert_eq!(read_fixed_camera_pose(&memory), None);
}

#[test]
fn the_switch_line_names_both_cameras_and_who_asked() {
    assert_eq!(
        camera_switch_line(switch_to_fixed(), CameraSwitchAsker::HostsMessage),
        "camera switch: camera 0 to camera 8 (blend 3.00 s, kind 1), asked by the host's message"
    );
}

#[test]
fn the_path_and_script_cameras_have_change_kinds_and_the_rail_cameras_do_not() {
    assert_eq!(change_kind_of_camera(PATH_CAMERA_INDEX), Some(3));
    assert_eq!(change_kind_of_camera(SCRIPT_CAMERA_INDEX), Some(2));
    for rail_or_actor_camera in [3, 4, 9] {
        assert_eq!(change_kind_of_camera(rail_or_actor_camera), None);
    }
}

#[test]
fn a_rail_camera_and_camera_9_are_skipped_on_the_guest() {
    let mut follow = GuestCameraFollow::default();
    let mut skip_of = |camera: i32| {
        let host = HostCamera {
            camera,
            pose: None,
            ..host_camera(u32::try_from(camera).expect("small"), 0)
        };
        follow.receive(host, true);
        follow.tick(true, Some(SNOW_CAMERA_INDEX));
        follow
            .tick(true, Some(SNOW_CAMERA_INDEX))
            .and_then(|step| step.err())
            .map(|(_, skip)| skip)
    };
    assert_eq!(skip_of(3), Some(HostCameraSkip::RailCameraIsTheGuestsOwn));
    assert_eq!(skip_of(4), Some(HostCameraSkip::RailCameraIsTheGuestsOwn));
    assert_eq!(skip_of(9), Some(HostCameraSkip::NoChangeKind));
}

#[test]
fn the_host_sends_no_message_for_its_rail_cameras() {
    let mut host = HostCameraFollow::default();
    let to = |to| CameraSwitch {
        from: SNOW_CAMERA_INDEX,
        to,
        ..switch_to_fixed()
    };
    assert!(host.message_for(to(3), None, true).is_none());
    assert!(host.message_for(to(4), None, true).is_none());
    let off_rail = host.message_for(to(2), None, true).expect("window");
    assert_eq!(off_rail.seq, 0);
}
