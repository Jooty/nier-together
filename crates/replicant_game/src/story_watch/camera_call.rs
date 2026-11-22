use replicant_proto::{WatchedCameraCall, WatchedCameraNative};

use crate::fight_sync::{CHANGE_CAMERA_TO_FREE_CAMERA_KIND, SET_FIXED_CAMERA_NATIVE_RVA};
use crate::side_view::{CHANGE_CAMERA_NATIVE_RVA, CHANGE_CAMERA_STATUS_NATIVE_RVA, ViewArg};

pub const CHANGE_CAMERA_TO_FIXED_CAMERA_KIND: i32 = 1;

#[derive(Debug, Clone, PartialEq)]
pub struct WatchedCameraRun {
    pub native_rva: u32,
    pub args: Vec<ViewArg>,
}

pub fn camera_call_is_followed(native: WatchedCameraNative, numbers: &[f32]) -> bool {
    match native {
        WatchedCameraNative::SetFixedCamera | WatchedCameraNative::ChangeCameraStatus => true,
        WatchedCameraNative::ChangeCamera => numbers.first().is_some_and(|kind| {
            [
                CHANGE_CAMERA_TO_FREE_CAMERA_KIND,
                CHANGE_CAMERA_TO_FIXED_CAMERA_KIND,
            ]
            .iter()
            .any(|followed| (*kind - *followed as f32).abs() < 0.5)
        }),
    }
}

pub fn watched_camera_run(call: &WatchedCameraCall) -> Option<WatchedCameraRun> {
    if !camera_call_is_followed(call.native, &call.numbers) {
        return None;
    }
    let number = |index: usize| f64::from(call.numbers[index]);
    let integer = |index: usize| ViewArg::Int(call.numbers[index].round() as i32);
    let real = |index: usize| ViewArg::Real(number(index));
    Some(match call.native {
        WatchedCameraNative::SetFixedCamera => WatchedCameraRun {
            native_rva: SET_FIXED_CAMERA_NATIVE_RVA,
            args: (0..6).map(real).collect(),
        },
        WatchedCameraNative::ChangeCamera => WatchedCameraRun {
            native_rva: CHANGE_CAMERA_NATIVE_RVA,
            args: vec![integer(0), real(1), integer(2)],
        },
        WatchedCameraNative::ChangeCameraStatus => WatchedCameraRun {
            native_rva: CHANGE_CAMERA_STATUS_NATIVE_RVA,
            args: vec![integer(0), real(1), integer(2)],
        },
    })
}

pub fn watched_camera_applies_here(call_phase: u16, local_phase: Option<u16>) -> bool {
    local_phase == Some(call_phase)
}
