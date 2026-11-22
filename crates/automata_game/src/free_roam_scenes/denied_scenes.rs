use crate::free_roam_scenes::{
    DENIED_SCENE_ACTION_IDS, FREE_ROAM_NEEDED_SCENE_ACTION_IDS, HAP_WALK_CAPS,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HapWalkCap {
    pub index: u32,
    pub action_id: u32,
}

pub fn scene_is_denied(action_id: u32) -> bool {
    DENIED_SCENE_ACTION_IDS.binary_search(&action_id).is_ok()
}

pub fn scene_is_free_roam_needed(action_id: u32) -> bool {
    FREE_ROAM_NEEDED_SCENE_ACTION_IDS
        .binary_search(&action_id)
        .is_ok()
}

pub fn hap_walk_cap(
    free_roam: bool,
    action_count: u32,
    cursor: u32,
    mut action_id_at: impl FnMut(u32) -> Option<u32>,
) -> Option<HapWalkCap> {
    if !free_roam {
        return None;
    }
    HAP_WALK_CAPS.iter().copied().find(|cap| {
        cap.index < action_count
            && cap.index >= cursor
            && action_id_at(cap.index) == Some(cap.action_id)
    })
}
