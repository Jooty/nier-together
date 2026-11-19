mod loaded_talk_set;
mod part_sets;

pub use loaded_talk_set::loaded_talk_set_letter;
pub use part_sets::{PartSetsRegistered, part_sets_registered_by_request};

pub const NEXT_PHASE_NAME_RVA: u32 = 0x437_4A24;
pub const TALK_WINDOW_RVA: u32 = 0x4C5_6970;
pub const TALK_WINDOW_LOADED_SET_NAME_OFFSET: u64 = 4;
pub const LOADED_TALK_SET_NAME_READ_LEN: usize = 16;
