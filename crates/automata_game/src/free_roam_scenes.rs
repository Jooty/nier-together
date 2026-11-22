use crate::address::GameAddress;

mod area_hold;
mod denied_scenes;
mod hap_cap_cache;
mod held_actions;
mod held_lines;
mod scene_action_ids;
mod update_hold;

pub use area_hold::area_update_is_held;
pub use denied_scenes::{HapWalkCap, hap_walk_cap, scene_is_denied, scene_is_free_roam_needed};
pub use hap_cap_cache::HapCapCache;
pub use held_actions::HeldActions;
pub use held_lines::{capped_hap_line, held_action_line};
pub use scene_action_ids::{
    DENIED_SCENE_ACTION_IDS, FREE_ROAM_NEEDED_SCENE_ACTION_IDS, HAP_WALK_CAPS,
};
pub use update_hold::{UpdateHold, hap_step_result_for_held_update, update_hold};

pub const HAP_STEP_FUNCTION: GameAddress = GameAddress::new(0x3E_F570, 0x3E_98B0);
pub const HAP_STEP_START: [u8; 20] = [
    0x48, 0x89, 0x5C, 0x24, 0x08, 0x48, 0x89, 0x6C, 0x24, 0x10, 0x48, 0x89, 0x74, 0x24, 0x18, 0x57,
    0x48, 0x83, 0xEC, 0x40,
];
pub const HAP_ACTION_LIST_OFFSET: u64 = 0x30;
pub const HAP_ACTION_COUNT_OFFSET: u64 = 0x38;
pub const HAP_CURSOR_OFFSET: u64 = 0x50;
pub const POINTER_SIZE: u64 = 8;
pub const HAP_READ_LEN: usize = 0x24;
pub const MOST_HAPS_REMEMBERED: usize = 4096;
pub const PHASE_EVENT_ACTION_ID_OFFSET: u64 = 0x18;
pub const PHASE_EVENT_ACTION_FLAGS_OFFSET: u64 = 0x1C;
pub const HAP_STEP_RESULT_OFFSET: u64 = 0xC;
pub const HAP_STEP_RESULT_THE_ACTION_BLOCKS_THE_HAP: u32 = 1;
pub const PHASE_EVENT_ACTION_FLAG_BLOCKS_THE_HAP_BIT: u32 = 3;
pub const PHASE_EVENT_ACTION_FLAG_LEAVES_THE_RESULT_ALONE_BIT: u32 = 0x17;
pub const PHASE_EVENT_ACTION_STATE_OFFSET: u64 = 0x20;
pub const PHASE_EVENT_ACTION_PLAY_ASKED_OFFSET: u64 = 0x48;
pub const PHASE_EVENT_ACTION_IDLE_STATE: u32 = 0;
pub const PHASE_EVENT_ACTION_LOADED_STATE: u32 = 0xBB7F_80A1;
pub const PHASE_EVENT_ACTION_LOADING_STATE: u32 = 0x6825_C6AE;
pub const PHASE_EVENT_ACTION_LOAD_REFUSED_STATE: u32 = 0x1EDE_FDDB;
pub const PHASE_EVENT_ACTION_PRE_START_STATES: [u32; 4] = [
    PHASE_EVENT_ACTION_IDLE_STATE,
    PHASE_EVENT_ACTION_LOAD_REFUSED_STATE,
    PHASE_EVENT_ACTION_LOADING_STATE,
    PHASE_EVENT_ACTION_LOADED_STATE,
];
pub const MOST_HELD_ACTION_IDS_REMEMBERED: usize = 512;
pub const PHASE_EVENT_ACTION_UPDATE_START: [u8; 29] = [
    0x48, 0x89, 0x54, 0x24, 0x10, 0x55, 0x53, 0x56, 0x57, 0x41, 0x54, 0x41, 0x55, 0x41, 0x56, 0x41,
    0x57, 0x48, 0x8D, 0x6C, 0x24, 0xE1, 0x48, 0x81, 0xEC, 0x88, 0x00, 0x00, 0x00,
];
