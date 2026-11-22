use replicant_proto::{
    MAX_POSITION_ABS, Message, WatchedCameraCall, WatchedCameraNative, WireError, decode, encode,
};

fn fixed_camera() -> WatchedCameraCall {
    WatchedCameraCall {
        phase: 32,
        native: WatchedCameraNative::SetFixedCamera,
        numbers: [18.33, 6.72, 9.96, 2.8, 0.0, -0.44],
    }
}

#[test]
fn a_fixed_camera_call_round_trips() {
    let message = Message::WatchedCameraCall(fixed_camera());
    let bytes = encode(&message).expect("a camera call encodes");
    assert_eq!(decode(&bytes), Ok(message));
}

#[test]
fn a_camera_call_is_relayed_to_the_partner() {
    assert!(Message::WatchedCameraCall(fixed_camera()).is_relayed());
}

#[test]
fn a_camera_call_with_a_number_that_is_not_finite_is_refused() {
    let mut call = fixed_camera();
    call.numbers[3] = f32::NAN;
    assert_eq!(
        Message::WatchedCameraCall(call).validate(),
        Err(WireError::OutOfRange {
            field: "watched camera call"
        })
    );
}

#[test]
fn a_camera_call_far_outside_the_world_is_refused() {
    let mut call = fixed_camera();
    call.numbers[0] = MAX_POSITION_ABS * 2.0;
    assert!(Message::WatchedCameraCall(call).validate().is_err());
}
