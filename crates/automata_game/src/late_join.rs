mod boss_gauge;
mod boss_wait;
mod joiner_take_in;
mod landing;
mod late_join_lines;

pub use boss_gauge::{
    BossFight, BossFightConditions, BossGaugeKind, BossGaugeObject, BossReading, TargetReading,
    boss_reading_of, conditions_of, entity_body_pointer, handle_slot_index, slot_holds_handle,
};
pub use boss_wait::{BossWait, BossWaitStep, JoinerNotice, WaitingJoiner};
pub use joiner_take_in::{JoinerTakeIn, TakeInStep, safe_moment_refusal};
pub use landing::{LandingReadings, LandingStep, LateLanding, start_index_of_place};
pub use late_join_lines::{
    arrival_beside_line, boss_fight_over_line, boss_fight_refusal, boss_hold_log_line,
    boss_unreadable_line, boss_wait_line, joiner_waits_line, landing_begins_line,
    landing_no_single_chapter_line, take_in_done_line, take_in_waiting_line,
};

pub const LANDING_AFTER_THE_PLACES_DIFFER_MS: u64 = 5_000;
pub const ARRIVAL_BESIDE_AFTER_BOTH_FREE_MS: u64 = 1_000;
pub const TAKE_IN_WAIT_LOG_INTERVAL_MS: u64 = 10_000;
pub const BOSS_SCAN_EVERY_MS: u64 = 250;
pub const MOST_UI_OBJECTS_SCANNED: usize = 4_096;
pub const UI_REGISTRY_HEADER_LEN: usize = 0x18;
pub const UI_REGISTRY_COUNT_OFFSET: usize = 0x10;
pub const UI_OBJECT_STATE_OFFSET: usize = 0x20;
pub const UI_OBJECT_FLAGS_OFFSET: usize = 0x58;
pub const UI_OBJECT_SHOWN_FLAG: u32 = 0x4000_0000;
pub const UI_OBJECT_LOWEST_ALIVE_STATE: u32 = 1;
pub const UI_OBJECT_HIGHEST_ALIVE_STATE: u32 = 3;
pub const BOSS_GAUGE_FIRST_HANDLE_OFFSET: usize = 0x90;
pub const BOSS_GAUGE_SECOND_HANDLE_OFFSET: usize = 0x94;
pub const BOSS_GAUGE_SECOND_HANDLE_USED_OFFSET: usize = 0xAC;
pub const BOSS_GAUGE_READ_LEN: usize = 0xB0;
pub const ENTITY_LIVENESS_WORD_OFFSET: usize = 0x2C;
pub const ENTITY_DEAD_OR_DYING_BITS: u32 = 3;
pub const ENTITY_HEADER_READ_LEN: usize = 0x50;
pub const ENTITY_SLOT_HANDLE_INDEX_SHIFT: u32 = 8;
pub const ENTITY_SLOT_HANDLE_INDEX_MASK: u32 = 0xFFFF;
pub const ENTITY_SLOT_HANDLE_GENERATION_MASK: u32 = 0xFFFF_FF00;
