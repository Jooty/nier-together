use crate::free_roam_scenes::{PHASE_EVENT_ACTION_IDLE_STATE, PHASE_EVENT_ACTION_LOADED_STATE};

pub fn area_update_is_held(free_roam: bool, state: u32, play_asked: u8) -> bool {
    let waits_for_its_area =
        state == PHASE_EVENT_ACTION_IDLE_STATE || state == PHASE_EVENT_ACTION_LOADED_STATE;
    free_roam && waits_for_its_area && play_asked == 0
}
