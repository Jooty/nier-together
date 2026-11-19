mod plan;

pub use plan::{LevelUpPlan, level_up_plan};

pub const LAST_LEVEL_THE_GAME_ALLOWS: i32 = 0x62;
pub const PLAYER_PARAM_LEVEL_TABLE_POINTER_OFFSET: u64 = 0x58;
pub const LEVEL_TABLE_ROWS_RELATIVE_OFFSET_FIELD: u64 = 4;
pub const LEVEL_TABLE_ROW_SIZE: i64 = 0x74;
pub const LEVEL_TABLE_EXPERIENCE_TO_LEAVE_LEVEL_OFFSET: i64 = 0x6C;
