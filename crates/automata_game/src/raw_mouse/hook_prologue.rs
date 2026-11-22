use crate::address::{
    APPLY_STICK_TO_ANGLES_FUNCTION, CONTROLLER_INPUT_BLOCKED_FUNCTION, GET_CAMERA_STICK_FUNCTION,
    GameAddress,
};
use crate::raw_mouse::{
    APPLY_STICK_TO_ANGLES_PROLOGUE, CONTROLLER_INPUT_BLOCKED_PROLOGUE, GET_CAMERA_STICK_PROLOGUE,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RawMouseHook {
    ApplyStickToAngles,
    GetCameraStick,
    ControllerInputBlocked,
}

impl RawMouseHook {
    pub const ALL: [Self; 3] = [
        Self::ApplyStickToAngles,
        Self::GetCameraStick,
        Self::ControllerInputBlocked,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::ApplyStickToAngles => "camera stick to angles",
            Self::GetCameraStick => "camera stick pick",
            Self::ControllerInputBlocked => "controller input blocked test",
        }
    }

    pub fn address(self) -> GameAddress {
        match self {
            Self::ApplyStickToAngles => APPLY_STICK_TO_ANGLES_FUNCTION,
            Self::GetCameraStick => GET_CAMERA_STICK_FUNCTION,
            Self::ControllerInputBlocked => CONTROLLER_INPUT_BLOCKED_FUNCTION,
        }
    }

    pub fn expected_prologue(self) -> &'static [u8] {
        match self {
            Self::ApplyStickToAngles => &APPLY_STICK_TO_ANGLES_PROLOGUE,
            Self::GetCameraStick => &GET_CAMERA_STICK_PROLOGUE,
            Self::ControllerInputBlocked => &CONTROLLER_INPUT_BLOCKED_PROLOGUE,
        }
    }
}

pub fn prologue_matches(hook: RawMouseHook, found: &[u8]) -> bool {
    found.starts_with(hook.expected_prologue())
}
