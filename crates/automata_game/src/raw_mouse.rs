mod accumulator;
mod camera_angles;
mod camera_fields;
mod camera_plan;
mod cap_probe;
mod ease_window;
mod follow_probe;
mod follow_rotation;
mod hook_prologue;
mod look_hold;
mod sensitivity;
mod swing_probe;

pub use accumulator::{RawMouseAccumulator, counts_of_device_move};
pub use camera_angles::{CameraAngles, ModeLimit, RawLook, apply_raw_look};
pub use camera_fields::{
    AUTO_FOLLOW_SUPPRESS_VALUE, CAMERA_AUTO_FOLLOW_SUPPRESS_FRAMES_OFFSET,
    CAMERA_CUT_FRAMES_OFFSET, CAMERA_EASED_EYE_OFFSET, CAMERA_EASED_EYE_TO_LOOK_DISTANCE_OFFSET,
    CAMERA_EASED_LOOK_AT_OFFSET, CameraFieldWrites, camera_field_writes, eye_to_look_distance,
};
pub use camera_plan::{RawCameraPlan, raw_camera_plan};
pub use cap_probe::{
    CAP_PROBE_MIN_GAP_BETWEEN_LINES_MS, CAP_PROBE_WINDOW_REPORT_GAP_MS, CapProbe, CapProbeLine,
    ScreenRect, clip_cursor_is_set,
};
pub use ease_window::{
    BODY_RECENTRED_BY_HAND_INPUT_BIT, CAMERA_EASE_PITCH_LIMIT_OFFSET,
    CAMERA_EASE_PITCH_RATE_SOURCE_OFFSET, CAMERA_EASE_SAVED_PITCH_SOURCE_OFFSET,
    CAMERA_EASE_SAVED_YAW_SOURCE_OFFSET, CAMERA_EASE_WINDOW_DIRECTION_TIMER_OFFSET,
    CAMERA_EASE_YAW_RATE_SOURCE_OFFSET, CAMERA_TARGET_OFFSET, CAMERA_TARGET_OWNER_OFFSET,
    EaseArmSource, EaseArmWrite, HINT_RECORD_CAMERA_MOVED_BY_HAND, ease_arm_writes,
    pitch_inside_ease_limit,
};
pub use follow_probe::{
    FOLLOW_PROBE_HERO_MOVED_METRES, FOLLOW_PROBE_MIN_GAP_BETWEEN_LINES_MICROS, FollowProbe,
    FollowProbeLine, FollowSample, body_to_camera_change,
};
pub use follow_rotation::{
    BASIS_ORTHONORMAL_TOLERANCE, Basis, CAMERA_BASIS_MATRIX_OFFSET,
    CAMERA_TARGET_BODY_POSITION_OFFSET, IDENTITY_BASIS, rotate_eased_offset, rotation_basis,
};
pub use hook_prologue::{RawMouseHook, prologue_matches};
pub use look_hold::{LOOK_HOLD_AFTER_LAST_COUNTS_MS, LookHold};
pub use sensitivity::{radians_per_count, sensitivity_multiplier, x_sensitivity_value};
pub use swing_probe::{
    SWING_PROBE_MIN_GAP_BETWEEN_LINES_MS, SWING_PROBE_TAIL_MS, SWING_PROBE_YAW_STEP_RADIANS,
    SwingProbe, yaw_step_radians,
};

pub const CAMERA_PITCH_OFFSET: u64 = 0x3F0;
pub const CAMERA_YAW_OFFSET: u64 = 0x3F4;
pub const CAMERA_MOVED_BY_HAND_OFFSET: u64 = 0x400;
pub const CAMERA_INVERT_PITCH_OFFSET: u64 = 0x5D4;
pub const CAMERA_INVERT_YAW_OFFSET: u64 = 0x5D8;
pub const CAMERA_LOWER_PITCH_LIMIT_OFFSET: u64 = 0x851C;
pub const CAMERA_LOWER_PITCH_LIMIT_ENABLE_OFFSET: u64 = 0x8520;
pub const CAMERA_UPPER_PITCH_LIMIT_OFFSET: u64 = 0x8524;
pub const CAMERA_UPPER_PITCH_LIMIT_ENABLE_OFFSET: u64 = 0x8528;
pub const CONTROLLER_MOUSE_STICK_LENGTH_OFFSET: u64 = 0x2E8;
pub const GAME_PITCH_LIMIT_RADIANS: f32 = 1.256_637_1;
pub const MOUSE_SOURCE_GOOD_READ_OFFSET: u64 = 0;
pub const MOUSE_SOURCE_DX_OFFSET: u64 = 0x14;
pub const MOUSE_SOURCE_DY_OFFSET: u64 = 0x18;
pub const SENSITIVITY_LAST_LINEAR_VALUE: i32 = 10;
pub const SENSITIVITY_LINEAR_STEP: f32 = 0.09;
pub const SENSITIVITY_LINEAR_START: f32 = 0.1;
pub const SENSITIVITY_STEP_ABOVE_LAST_LINEAR_VALUE: f32 = 0.2;
pub const SENSITIVITY_MULTIPLIER_AT_LAST_LINEAR_VALUE: f32 = 1.0;
pub const X_SENSITIVITY_FLOOR_FLAG_BIT: u32 = 0x4000;
pub const X_SENSITIVITY_FLOOR_VALUE: i32 = 4;
pub const DEFAULT_X_SENSITIVITY_VALUE: i32 = 8;
pub const DEGREES_PER_COUNT_AT_DEFAULT_X_SENSITIVITY: f32 = 0.07;
pub const HOOK_PROLOGUE_LEN: usize = 19;
pub const APPLY_STICK_TO_ANGLES_PROLOGUE: [u8; 17] = [
    0x48, 0x8B, 0xC4, 0x53, 0x41, 0x55, 0x41, 0x57, 0x48, 0x81, 0xEC, 0x20, 0x01, 0x00, 0x00, 0xF7,
    0x05,
];
pub const GET_CAMERA_STICK_PROLOGUE: [u8; 19] = [
    0x83, 0xB9, 0x24, 0x07, 0x00, 0x00, 0x00, 0x0F, 0x57, 0xC0, 0x74, 0x17, 0x0F, 0x2F, 0x81, 0xE8,
    0x02, 0x00, 0x00,
];
pub const CONTROLLER_INPUT_BLOCKED_PROLOGUE: [u8; 11] = [
    0x48, 0x83, 0xEC, 0x28, 0x83, 0xB9, 0x24, 0x07, 0x00, 0x00, 0x00,
];
