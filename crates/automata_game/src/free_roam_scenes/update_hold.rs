use crate::free_roam_scenes::{
    HAP_STEP_RESULT_THE_ACTION_BLOCKS_THE_HAP, PHASE_EVENT_ACTION_FLAG_BLOCKS_THE_HAP_BIT,
    PHASE_EVENT_ACTION_FLAG_LEAVES_THE_RESULT_ALONE_BIT, PHASE_EVENT_ACTION_PRE_START_STATES,
    area_update_is_held, scene_is_denied,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateHold {
    AreaStart,
    DeniedScene,
}

pub fn update_hold(
    free_roam: bool,
    action_id: u32,
    state: u32,
    play_asked: u8,
) -> Option<UpdateHold> {
    if !free_roam {
        return None;
    }
    if scene_is_denied(action_id) && PHASE_EVENT_ACTION_PRE_START_STATES.contains(&state) {
        return Some(UpdateHold::DeniedScene);
    }
    area_update_is_held(true, state, play_asked).then_some(UpdateHold::AreaStart)
}

pub fn hap_step_result_for_held_update(action_flags: u32) -> Option<u32> {
    let blocks_the_hap = action_flags >> PHASE_EVENT_ACTION_FLAG_BLOCKS_THE_HAP_BIT & 1 != 0;
    let leaves_the_result_alone =
        action_flags >> PHASE_EVENT_ACTION_FLAG_LEAVES_THE_RESULT_ALONE_BIT & 1 != 0;
    (blocks_the_hap && !leaves_the_result_alone)
        .then_some(HAP_STEP_RESULT_THE_ACTION_BLOCKS_THE_HAP)
}
