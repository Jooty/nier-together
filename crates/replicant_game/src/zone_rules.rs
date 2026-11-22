mod attempt;
mod door_kind;
mod exit_doors;
mod exit_names;
mod prompt;
mod radius;
mod verdict;
mod warp_cancel;
mod warp_menu_pause;

pub use attempt::AttemptCooldown;
pub use door_kind::{DoorKind, ExitDoor};
pub use exit_doors::EXIT_DOORS;
pub use exit_names::is_zone_exit_callback;
pub use prompt::{VetoNotice, VetoRecord};
pub use radius::ZoneRadius;
pub use verdict::{
    Companion, Placement, ZoneChangeAsk, ZoneChangeSource, ZoneVerdict, judge_zone_change,
};
pub use warp_cancel::warp_pick_is_a_cancel;
pub use warp_menu_pause::{
    WarpMenuPauseFacts, WarpMenuPauseVerdict, WarpMenuVeto, judge_warp_menu_pause,
};

pub const DEFAULT_ZONE_RADIUS_UNITS: f32 = 15.0;
pub const MIN_ZONE_RADIUS_UNITS: f32 = 3.0;
pub const MAX_ZONE_RADIUS_UNITS: f32 = 200.0;
pub const EXIT_DOOR_RADIUS_FRACTION: f32 = 0.7;
pub const VETO_LOG_INTERVAL_MS: u64 = 1000;
pub const GUEST_VETO_TEXT: &str = "Only the host can change zones";
pub const ZONE_EXIT_CALLBACK_NAME_PARTS: [&str; 4] = ["jumpmap", "jump_map", "mapjump", "map_jump"];
pub const HOST_WANTS_TO_GO_COOLDOWN_MS: u64 = 5000;
pub const WARP_MENU_CANCEL_POINT: i32 = -1;
pub const WARP_MENU_KEEP_ORIGIN_POINT: i32 = -1;
pub const WARP_MENU_PAUSE_RELEASE_AFTER_MS: u64 = 3000;
pub const WARP_MENU_PAUSE_STEP_NAME_PART: &str = "Open_Port_Window";
pub const SET_PAUSE_MASK_RVA: u32 = 0x3B_1370;
