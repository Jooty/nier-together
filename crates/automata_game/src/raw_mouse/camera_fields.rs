pub const CAMERA_CUT_FRAMES_OFFSET: u64 = 0x20C;
pub const CAMERA_AUTO_FOLLOW_SUPPRESS_FRAMES_OFFSET: u64 = 0x8394;
pub const CAMERA_EASED_EYE_TO_LOOK_DISTANCE_OFFSET: u64 = 0x134;
pub const CAMERA_EASED_EYE_OFFSET: u64 = 0xF0;
pub const CAMERA_EASED_LOOK_AT_OFFSET: u64 = 0x100;
pub const AUTO_FOLLOW_SUPPRESS_VALUE: i32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CameraFieldWrites {
    pub suppress_auto_follow: bool,
}

pub fn camera_field_writes(setting_on: bool) -> CameraFieldWrites {
    CameraFieldWrites {
        suppress_auto_follow: setting_on,
    }
}

pub fn eye_to_look_distance(eye: [f32; 3], look_at: [f32; 3]) -> f32 {
    let [x, y, z] = [
        eye[0] - look_at[0],
        eye[1] - look_at[1],
        eye[2] - look_at[2],
    ];
    (x * x + y * y + z * z).sqrt()
}
