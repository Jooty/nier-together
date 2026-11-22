use crate::raw_mouse::{GAME_PITCH_LIMIT_RADIANS, radians_per_count};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraAngles {
    pub pitch: f32,
    pub yaw: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModeLimit {
    pub value: f32,
    pub enable: f32,
}

impl ModeLimit {
    pub const OFF: Self = Self {
        value: 0.0,
        enable: 0.0,
    };

    fn is_on(self) -> bool {
        self.enable > 0.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RawLook {
    pub yaw_multiplier: f32,
    pub pitch_multiplier: f32,
    pub invert_pitch: bool,
    pub invert_yaw: bool,
    pub pitch_enabled: bool,
    pub yaw_enabled: bool,
    pub lower_limit: ModeLimit,
    pub upper_limit: ModeLimit,
}

pub fn apply_raw_look(
    angles: CameraAngles,
    counts: (i32, i32),
    look: &RawLook,
) -> Option<CameraAngles> {
    let yaw_counts = if look.yaw_enabled { counts.0 } else { 0 };
    let pitch_counts = if look.pitch_enabled { counts.1 } else { 0 };
    if yaw_counts == 0 && pitch_counts == 0 {
        return None;
    }
    let yaw =
        wrap_yaw(angles.yaw + signed_radians(yaw_counts, look.invert_yaw, look.yaw_multiplier));
    let pitch = limit_pitch(
        angles.pitch + signed_radians(pitch_counts, look.invert_pitch, look.pitch_multiplier),
        look,
    );
    Some(CameraAngles { pitch, yaw })
}

fn signed_radians(counts: i32, inverted: bool, multiplier: f32) -> f32 {
    let radians = counts as f32 * radians_per_count(multiplier);
    if inverted { radians } else { -radians }
}

fn wrap_yaw(yaw: f32) -> f32 {
    let half = core::f32::consts::PI;
    (yaw + half).rem_euclid(core::f32::consts::TAU) - half
}

fn limit_pitch(pitch: f32, look: &RawLook) -> f32 {
    let mut limited = pitch.clamp(-GAME_PITCH_LIMIT_RADIANS, GAME_PITCH_LIMIT_RADIANS);
    if look.lower_limit.is_on() && limited < look.lower_limit.value {
        limited = look.lower_limit.value;
    }
    if look.upper_limit.is_on() && limited > look.upper_limit.value {
        limited = look.upper_limit.value;
    }
    limited
}
