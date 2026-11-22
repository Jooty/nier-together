use crate::callback_gate::LastPauseOn;

use super::{WARP_MENU_PAUSE_RELEASE_AFTER_MS, WARP_MENU_PAUSE_STEP_NAME_PART};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WarpMenuVeto {
    pub vetoed_at_ms: u64,
    pub changes_let_through_then: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WarpMenuPauseFacts<'a> {
    pub now_ms: u64,
    pub changes_let_through_now: u32,
    pub last_pause_on: Option<&'a LastPauseOn>,
    pub current_pause_mask: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WarpMenuPauseVerdict {
    Wait,
    NothingToRelease,
    ReleaseThePause,
}

pub fn judge_warp_menu_pause(
    veto: &WarpMenuVeto,
    facts: &WarpMenuPauseFacts<'_>,
) -> WarpMenuPauseVerdict {
    if facts.now_ms.saturating_sub(veto.vetoed_at_ms) < WARP_MENU_PAUSE_RELEASE_AFTER_MS {
        return WarpMenuPauseVerdict::Wait;
    }
    if facts.changes_let_through_now != veto.changes_let_through_then {
        return WarpMenuPauseVerdict::NothingToRelease;
    }
    let Some(pause) = facts.last_pause_on else {
        return WarpMenuPauseVerdict::NothingToRelease;
    };
    let still_held = pause.step.contains(WARP_MENU_PAUSE_STEP_NAME_PART)
        && pause.mask_after != 0
        && facts.current_pause_mask == pause.mask_after;
    if still_held {
        WarpMenuPauseVerdict::ReleaseThePause
    } else {
        WarpMenuPauseVerdict::NothingToRelease
    }
}
