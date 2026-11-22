pub const CAMERA_EASE_WINDOW_DIRECTION_TIMER_OFFSET: u64 = 0x85F4;
pub const CAMERA_EASE_PITCH_LIMIT_OFFSET: u64 = 0x87F0;
pub const CAMERA_EASE_SAVED_PITCH_SOURCE_OFFSET: u64 = 0x87F8;
pub const CAMERA_EASE_SAVED_YAW_SOURCE_OFFSET: u64 = 0x87FC;
pub const CAMERA_EASE_PITCH_RATE_SOURCE_OFFSET: u64 = 0x8800;
pub const CAMERA_EASE_YAW_RATE_SOURCE_OFFSET: u64 = 0x8804;
pub const CAMERA_TARGET_OFFSET: u64 = 0x370;
pub const CAMERA_TARGET_OWNER_OFFSET: u64 = 0x610;
pub const BODY_RECENTRED_BY_HAND_INPUT_BIT: u32 = 0x100;
pub const HINT_RECORD_CAMERA_MOVED_BY_HAND: i32 = 1;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EaseArmSource {
    pub pitch_to_ease_from: i32,
    pub yaw_to_ease_from: i32,
    pub pitch_rate: i32,
    pub yaw_rate: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EaseArmWrite {
    pub offset: u64,
    pub value: i32,
}

pub fn ease_arm_writes(source: EaseArmSource) -> [EaseArmWrite; 8] {
    let write = |offset, value| EaseArmWrite { offset, value };
    [
        write(0x8810, source.pitch_to_ease_from),
        write(0x8814, source.yaw_to_ease_from),
        write(0x87E8, 1),
        write(0x87EC, 1),
        write(0x881C, source.pitch_rate),
        write(0x8818, source.pitch_rate),
        write(0x8824, source.yaw_rate),
        write(0x8820, source.yaw_rate),
    ]
}

pub fn pitch_inside_ease_limit(pitch: f32, limit: f32, direction_timer: f32) -> Option<f32> {
    if direction_timer <= 0.0 || limit.is_nan() || limit <= 0.0 || pitch.abs() <= limit {
        return None;
    }
    Some(pitch.clamp(-limit, limit))
}
