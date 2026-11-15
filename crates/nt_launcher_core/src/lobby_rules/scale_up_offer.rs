#[derive(Debug, Clone, Copy)]
pub struct ScaleUpInputs {
    pub viewer_is_guest: bool,
    pub continue_mode: bool,
    pub already_started: bool,
    pub picked_slot: Option<u8>,
    pub guest_connected: bool,
    pub picked_slot_has_seat: Option<bool>,
    pub declined_slot: Option<u8>,
}

pub fn scale_up_offered(inputs: ScaleUpInputs) -> bool {
    let Some(picked) = inputs.picked_slot else {
        return false;
    };
    !inputs.viewer_is_guest
        && inputs.continue_mode
        && !inputs.already_started
        && inputs.guest_connected
        && inputs.picked_slot_has_seat == Some(false)
        && inputs.declined_slot != Some(picked)
}

pub fn declined_scale_up_kept(
    declined_slot: Option<u8>,
    picked_slot: Option<u8>,
    guest_connected: bool,
) -> Option<u8> {
    declined_slot.filter(|declined| guest_connected && picked_slot == Some(*declined))
}
