mod change_log;
mod local_game;
mod partner;
mod play_checks;

pub use change_log::{ReadinessChangeLog, waiting_on};
pub use local_game::LocalGameReadiness;
pub use partner::{PARTNER_GAME_CLOSED_LABEL, PartnerWatch, joining_label};
pub use play_checks::{PlayChecks, PlayReads, classify, read_play_checks, read_play_reads};

pub const JOINING_IGNORED_AFTER_MS: u64 = 60_000;
pub const SILENT_PARTNER_FALLBACK_MS: u64 = 5_000;
pub const REPORT_STALE_MS: u64 = 2_000;
pub const REPORT_MIN_INTERVAL_MS: u64 = 250;
pub const HEARTBEAT_INTERVAL_MS: u64 = 1_000;
