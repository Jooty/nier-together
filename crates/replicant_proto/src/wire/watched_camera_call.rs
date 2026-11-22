use serde::{Deserialize, Serialize};

use crate::wire::{MAX_POSITION_ABS, WireError};

pub const WATCHED_CAMERA_CALL_NUMBERS: usize = 6;

/// A camera native the host's game ran, for a guest that watches the host to run too, so the
/// watched room or scripted angle shows on its screen. `numbers` is the native's arguments in
/// order; the slots past the native's own count are 0.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct WatchedCameraCall {
    pub phase: u16,
    pub native: WatchedCameraNative,
    pub numbers: [f32; WATCHED_CAMERA_CALL_NUMBERS],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WatchedCameraNative {
    SetFixedCamera,
    ChangeCamera,
    ChangeCameraStatus,
}

impl WatchedCameraCall {
    pub fn validate(&self) -> Result<(), WireError> {
        let in_range = self
            .numbers
            .iter()
            .all(|number| number.is_finite() && number.abs() <= MAX_POSITION_ABS);
        if in_range {
            Ok(())
        } else {
            Err(WireError::OutOfRange {
                field: "watched camera call",
            })
        }
    }
}
