use crate::story_jump::{LEVEL_CHECK_SETTLE_MS, PLACEMENT_READ_BACK_AFTER_MS};

pub const RESTORE_POINT_WAITS_AFTER_PHASE_CHANGE_MS: u64 =
    LEVEL_CHECK_SETTLE_MS + PLACEMENT_READ_BACK_AFTER_MS;

/// The jump's restore point (`_BackupSaveData`, a memory copy of the save block) is made once the
/// destination has loaded and the placement, its read-back and the level check have all run, so a
/// defeat right after the jump continues at the jump point with the jump's loadout and level.
pub fn restore_point_is_due(
    phase_at_apply: &str,
    phase_now: &str,
    field_was_left: bool,
    field_ready: bool,
    ms_since_phase_change: u64,
) -> bool {
    let phase_changed = phase_at_apply != phase_now || field_was_left;
    phase_changed
        && field_ready
        && ms_since_phase_change >= RESTORE_POINT_WAITS_AFTER_PHASE_CHANGE_MS
}
