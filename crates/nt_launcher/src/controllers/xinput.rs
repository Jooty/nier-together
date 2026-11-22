use windows_sys::Win32::Foundation::ERROR_SUCCESS;
use windows_sys::Win32::UI::Input::XboxController::{XINPUT_STATE, XInputGetState};

const XINPUT_USER_COUNT: u32 = 4;

pub fn xinput_controller_connected() -> bool {
    (0..XINPUT_USER_COUNT).any(|user| {
        let mut state = XINPUT_STATE {
            dwPacketNumber: 0,
            Gamepad: Default::default(),
        };
        // SAFETY: `state` is a valid XINPUT_STATE for the call; XInputGetState only reads the pad.
        #[expect(unsafe_code, reason = "XInputGetState is a Win32 call")]
        let result = unsafe { XInputGetState(user, &mut state) };
        result == ERROR_SUCCESS
    })
}
