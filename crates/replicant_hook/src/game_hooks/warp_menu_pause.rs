use core::ffi::c_void;
use std::panic;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Mutex, PoisonError};

use replicant_game::GameMemory;
use replicant_game::dialogue::GAME_STATE_RVA;
use replicant_game::marker::CONTROL_MASK_FIRST_RVA;
use replicant_game::zone_rules::{
    SET_PAUSE_MASK_RVA, WarpMenuPauseFacts, WarpMenuPauseVerdict, WarpMenuVeto,
    judge_warp_menu_pause,
};

use crate::build_gate::VerifiedBuild;
use crate::diagnostics::log_line;
use crate::memory::InProcessMemory;
use crate::net::net_clock_ms;

static CHANGES_LET_THROUGH: AtomicU32 = AtomicU32::new(0);
static VETO: Mutex<Option<WarpMenuVeto>> = Mutex::new(None);

type SetPauseMask = unsafe extern "system" fn(*mut c_void, u32);

pub(super) fn note_change_let_through() {
    CHANGES_LET_THROUGH.fetch_add(1, Ordering::AcqRel);
}

pub(super) fn note_warp_menu_vetoed() {
    *VETO.lock().unwrap_or_else(PoisonError::into_inner) = Some(WarpMenuVeto {
        vetoed_at_ms: net_clock_ms(),
        changes_let_through_then: CHANGES_LET_THROUGH.load(Ordering::Acquire),
    });
}

pub(crate) fn tick(build: &VerifiedBuild) {
    let Some(veto) = *VETO.lock().unwrap_or_else(PoisonError::into_inner) else {
        return;
    };
    let last_pause_on = super::last_pause_on();
    let verdict = judge_warp_menu_pause(
        &veto,
        &WarpMenuPauseFacts {
            now_ms: net_clock_ms(),
            changes_let_through_now: CHANGES_LET_THROUGH.load(Ordering::Acquire),
            last_pause_on: last_pause_on.as_ref(),
            current_pause_mask: InProcessMemory::new(build)
                .read_i32(build.module_base() + u64::from(CONTROL_MASK_FIRST_RVA))
                .unwrap_or(0)
                .cast_unsigned(),
        },
    );
    if verdict == WarpMenuPauseVerdict::Wait {
        return;
    }
    *VETO.lock().unwrap_or_else(PoisonError::into_inner) = None;
    if verdict == WarpMenuPauseVerdict::ReleaseThePause {
        let _ = panic::catch_unwind(|| release_the_pause(build));
    }
}

#[expect(
    unsafe_code,
    reason = "calls the game's SetPauseMask, the function _PauseOff ends in"
)]
fn release_the_pause(build: &VerifiedBuild) {
    let base = build.module_base();
    // SAFETY: RVA 0x3B1370 is SetPauseMask(gameState, mask) in the verified build and _PauseOff
    // is `SetPauseMask(&gameState, 0)`; the tick runs on the game thread.
    unsafe {
        core::mem::transmute::<u64, SetPauseMask>(base + u64::from(SET_PAUSE_MASK_RVA))(
            (base + u64::from(GAME_STATE_RVA)) as *mut c_void,
            0,
        );
    }
    log_line("zone rules: released the warp menu's pause ourselves");
}
