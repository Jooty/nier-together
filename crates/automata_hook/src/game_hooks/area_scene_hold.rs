use core::ffi::c_void;
use core::ptr;
use std::panic;
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};
use std::sync::{LazyLock, Mutex, OnceLock, PoisonError};

use automata_game::address::PHASE_EVENT_ACTION_UPDATE_FUNCTION;
use automata_game::free_roam_scenes::{
    HAP_ACTION_COUNT_OFFSET, HAP_ACTION_LIST_OFFSET, HAP_CURSOR_OFFSET, HAP_READ_LEN,
    HAP_STEP_FUNCTION, HAP_STEP_RESULT_OFFSET, HAP_STEP_START, HapCapCache, HapWalkCap,
    HeldActions, PHASE_EVENT_ACTION_FLAGS_OFFSET, PHASE_EVENT_ACTION_ID_OFFSET,
    PHASE_EVENT_ACTION_PLAY_ASKED_OFFSET, PHASE_EVENT_ACTION_STATE_OFFSET,
    PHASE_EVENT_ACTION_UPDATE_START, POINTER_SIZE, UpdateHold, capped_hap_line,
    hap_step_result_for_held_update, held_action_line, update_hold,
};
use automata_game::partner::code_starts_with;
use nt_hook::diagnostics::log_line;

use crate::game_calls::GameCalls;

static ORIGINAL_AREA_UPDATE: AtomicPtr<c_void> = AtomicPtr::new(ptr::null_mut());
static HOLD_CALLS: OnceLock<GameCalls> = OnceLock::new();
static AREA_STARTS_HELD: AtomicBool = AtomicBool::new(false);
static AREA_UPDATE_HOOKED: AtomicBool = AtomicBool::new(false);
static HELD_ACTIONS: Mutex<HeldActions> = Mutex::new(HeldActions::new());
static ORIGINAL_HAP_STEP: AtomicPtr<c_void> = AtomicPtr::new(ptr::null_mut());
static HAP_CAP_CACHE: LazyLock<Mutex<HapCapCache>> =
    LazyLock::new(|| Mutex::new(HapCapCache::new()));
static CAPPED_HAPS: Mutex<HeldActions> = Mutex::new(HeldActions::new());

type AreaUpdate = unsafe extern "system" fn(usize, usize);
type HapStep = unsafe extern "system" fn(usize, u8);

/// Hooks the event action's per-frame update so a free roam copy never starts a scene from an area
/// and never starts a scene of the deny table at all. False, with nothing hooked, when the update is not the noted one.
pub fn install_area_scene_hold(calls: GameCalls) -> bool {
    let start_matches = calls
        .read_bytes(
            PHASE_EVENT_ACTION_UPDATE_FUNCTION,
            PHASE_EVENT_ACTION_UPDATE_START.len(),
        )
        .is_ok_and(|found| code_starts_with(&found, &PHASE_EVENT_ACTION_UPDATE_START));
    if !start_matches {
        log_line(
            "free roam: the event action's update does not start with the noted bytes on this exe, area starts of scenes are never held",
        );
        return false;
    }
    if HOLD_CALLS.set(calls).is_err() {
        return false;
    }
    #[expect(
        unsafe_code,
        reason = "hooks the game's event action update; the replacement has the signature of the original"
    )]
    // SAFETY: the address is the update of the verified build and its first bytes were just
    // checked; the replacement takes the same two pointers and gives nothing back, like the original.
    let hooked = unsafe {
        nt_hook::detour::hook_address(
            "event action update (free roam keeps area starts of scenes off)",
            calls.address_value(PHASE_EVENT_ACTION_UPDATE_FUNCTION) as usize,
            area_update_detour as *const () as *mut c_void,
            &ORIGINAL_AREA_UPDATE,
        )
    };
    AREA_UPDATE_HOOKED.store(hooked, Ordering::Release);
    if hooked {
        install_hap_walk_cap(&calls);
    }
    hooked
}

fn install_hap_walk_cap(calls: &GameCalls) {
    let start_matches = calls
        .read_bytes(HAP_STEP_FUNCTION, HAP_STEP_START.len())
        .is_ok_and(|found| code_starts_with(&found, &HAP_STEP_START));
    if !start_matches {
        log_line(
            "free roam: the hap step does not start with the noted bytes on this exe, haps are never stopped before their nearness gates",
        );
        return;
    }
    #[expect(
        unsafe_code,
        reason = "hooks the game's hap step; the replacement has the signature of the original"
    )]
    // SAFETY: the address is the hap step of the verified build and its first bytes were just
    // checked; the replacement takes the same hap pointer and flag byte and gives nothing back.
    let _ = unsafe {
        nt_hook::detour::hook_address(
            "hap step (free roam stops haps before their nearness gates)",
            calls.address_value(HAP_STEP_FUNCTION) as usize,
            hap_step_detour as *const () as *mut c_void,
            &ORIGINAL_HAP_STEP,
        )
    };
}

pub fn area_scene_hold_is_hooked() -> bool {
    AREA_UPDATE_HOOKED.load(Ordering::Acquire)
}

/// While on, an event action that waits for its area is not updated; game thread or any thread.
pub fn set_area_starts_held(held: bool) {
    if AREA_STARTS_HELD.swap(held, Ordering::AcqRel) == held {
        return;
    }
    log_line(if held {
        "free roam: area starts of scenes are held from now on (event actions that wait for an area are not updated)"
    } else {
        "free roam: area starts of scenes are no longer held"
    });
}

fn hold_this_update(calls: &GameCalls, action: usize, hap_step_result: usize) -> bool {
    if action == 0 {
        return false;
    }
    let action = action as u64;
    let Ok(state) = calls.read_u32_at(action + PHASE_EVENT_ACTION_STATE_OFFSET) else {
        return false;
    };
    let Ok(play_asked) = calls.read_bytes_at(action + PHASE_EVENT_ACTION_PLAY_ASKED_OFFSET, 1)
    else {
        return false;
    };
    let play_asked = play_asked.first().copied().unwrap_or(1);
    let id = calls
        .read_u32_at(action + PHASE_EVENT_ACTION_ID_OFFSET)
        .unwrap_or(0);
    let Some(hold) = update_hold(true, id, state, play_asked) else {
        return false;
    };
    let flags = calls
        .read_u32_at(action + PHASE_EVENT_ACTION_FLAGS_OFFSET)
        .unwrap_or(0);
    if let Some(result) = hap_step_result_for_held_update(flags)
        && hap_step_result != 0
    {
        // The skipped update tail reports "still running" to the hap step; without it the hap walks on.
        let _ = calls.write_u32_at(hap_step_result as u64 + HAP_STEP_RESULT_OFFSET, result);
    }
    log_first_hold(hold, id, state, play_asked);
    true
}

fn log_first_hold(hold: UpdateHold, id: u32, state: u32, play_asked: u8) {
    let mut held = HELD_ACTIONS.lock().unwrap_or_else(PoisonError::into_inner);
    if held.is_first_hold_of(id) {
        log_line(&held_action_line(
            hold,
            id,
            state,
            play_asked,
            held.actions_held(),
        ));
    }
}

#[expect(
    unsafe_code,
    reason = "calls the original event action update through the trampoline MinHook built"
)]
extern "system" fn area_update_detour(action: usize, second: usize) {
    if AREA_STARTS_HELD.load(Ordering::Acquire)
        && let Some(calls) = HOLD_CALLS.get()
        && panic::catch_unwind(|| hold_this_update(calls, action, second)).unwrap_or(false)
    {
        return;
    }
    let original = ORIGINAL_AREA_UPDATE.load(Ordering::Acquire);
    if original.is_null() {
        return;
    }
    // SAFETY: the trampoline continues into the original update, which takes these two pointers.
    unsafe { core::mem::transmute::<*mut c_void, AreaUpdate>(original)(action, second) }
}

fn cap_this_step(calls: &GameCalls, hap: usize) -> Option<(u64, u32)> {
    if hap == 0 {
        return None;
    }
    let hap = hap as u64;
    let head = calls
        .read_bytes_at(hap + HAP_ACTION_LIST_OFFSET, HAP_READ_LEN)
        .ok()?;
    let list = u64::from_le_bytes(head.get(..8)?.try_into().ok()?);
    let action_count = u32::from_le_bytes(
        head.get((HAP_ACTION_COUNT_OFFSET - HAP_ACTION_LIST_OFFSET) as usize..)?
            .get(..4)?
            .try_into()
            .ok()?,
    );
    let cursor = u32::from_le_bytes(
        head.get((HAP_CURSOR_OFFSET - HAP_ACTION_LIST_OFFSET) as usize..)?
            .get(..4)?
            .try_into()
            .ok()?,
    );
    if list == 0 {
        return None;
    }
    let cap = HAP_CAP_CACHE
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .cap_for(hap, list, action_count, cursor, |index| {
            let action = calls
                .read_u64_at(list + u64::from(index) * POINTER_SIZE)
                .ok()?;
            if action == 0 {
                return None;
            }
            calls
                .read_u32_at(action + PHASE_EVENT_ACTION_ID_OFFSET)
                .ok()
        })?;
    calls
        .write_u32_at(hap + HAP_ACTION_COUNT_OFFSET, cap.index)
        .ok()?;
    log_first_cap(cap, action_count);
    Some((hap + HAP_ACTION_COUNT_OFFSET, action_count))
}

fn log_first_cap(cap: HapWalkCap, action_count: u32) {
    let mut capped = CAPPED_HAPS.lock().unwrap_or_else(PoisonError::into_inner);
    if capped.is_first_hold_of(cap.action_id) {
        log_line(&capped_hap_line(
            cap.action_id,
            cap.index,
            action_count,
            capped.actions_held(),
        ));
    }
}

#[expect(
    unsafe_code,
    reason = "calls the original hap step through the trampoline MinHook built"
)]
extern "system" fn hap_step_detour(hap: usize, flag: u8) {
    let restore = if AREA_STARTS_HELD.load(Ordering::Acquire)
        && let Some(calls) = HOLD_CALLS.get()
    {
        panic::catch_unwind(|| cap_this_step(calls, hap)).unwrap_or(None)
    } else {
        None
    };
    let original = ORIGINAL_HAP_STEP.load(Ordering::Acquire);
    if !original.is_null() {
        // SAFETY: the trampoline continues into the original hap step, which takes this pointer and byte.
        unsafe { core::mem::transmute::<*mut c_void, HapStep>(original)(hap, flag) }
    }
    if let (Some((count_address, action_count)), Some(calls)) = (restore, HOLD_CALLS.get()) {
        let _ = calls.write_u32_at(count_address, action_count);
    }
}
