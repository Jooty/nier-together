use core::ffi::{CStr, c_void};
use core::ptr;
use std::ffi::CString;
use std::panic::{self, AssertUnwindSafe};
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::{Mutex, OnceLock, PoisonError};

use replicant_game::kaine::{MissedMotionNames, swapped_body_motion_twin};
use replicant_game::offsets::MOTION_WORK_INDEX_BY_NAME_FUNCTION_RVA;

use crate::build_gate::VerifiedBuild;
use crate::diagnostics::log_line;
use crate::game_hooks::hook_address;

static ORIGINAL_LOOKUP: AtomicPtr<c_void> = AtomicPtr::new(ptr::null_mut());
static HOOKED: OnceLock<VerifiedBuild> = OnceLock::new();
static MISSED_NAMES: Mutex<MissedMotionNames> = Mutex::new(MissedMotionNames::new());

type MotionIndexByName = unsafe extern "system" fn(*mut c_void, *const u8) -> i64;

pub fn install(build: VerifiedBuild) {
    if HOOKED.set(build).is_err() {
        return;
    }
    let address =
        (build.module_base() + u64::from(MOTION_WORK_INDEX_BY_NAME_FUNCTION_RVA)) as usize;
    #[expect(
        unsafe_code,
        reason = "hooks the motion index by name lookup by address"
    )]
    // SAFETY: the RVA starts the lookup (motion work, NUL-terminated name, returns the motion index
    // or -1) in the verified build, and the replacement has the same signature.
    unsafe {
        hook_address(
            "MotionWorkIndexByName",
            address,
            motion_index_detour as *const () as *mut c_void,
            &ORIGINAL_LOOKUP,
        )
    };
}

#[expect(
    unsafe_code,
    reason = "calls the original lookup through the trampoline MinHook built"
)]
extern "system" fn motion_index_detour(this: *mut c_void, name: *const u8) -> i64 {
    let original = ORIGINAL_LOOKUP.load(Ordering::Acquire);
    if original.is_null() {
        return -1;
    }
    // SAFETY: the trampoline continues into the original lookup, which has this exact signature.
    let lookup = unsafe { core::mem::transmute::<*mut c_void, MotionIndexByName>(original) };
    // SAFETY: same call the game made, with the same arguments.
    let index = unsafe { lookup(this, name) };
    if index as i32 >= 0 || name.is_null() {
        return index;
    }
    panic::catch_unwind(AssertUnwindSafe(|| twin_index(lookup, this, name)))
        .ok()
        .flatten()
        .unwrap_or(index)
}

#[expect(
    unsafe_code,
    reason = "reads the game's NUL-terminated motion name and calls the lookup again with the twin"
)]
fn twin_index(lookup: MotionIndexByName, this: *mut c_void, name: *const u8) -> Option<i64> {
    // SAFETY: the game passes a NUL-terminated name that outlives this call.
    let asked = unsafe { CStr::from_ptr(name.cast()) }.to_str().ok()?;
    let twin = swapped_body_motion_twin(asked);
    let found = twin.as_deref().and_then(|twin| {
        let twin = CString::new(twin).ok()?;
        // SAFETY: the twin is NUL-terminated and lives past the call.
        let index = unsafe { lookup(this, twin.as_ptr().cast()) };
        (index as i32 >= 0).then_some(index)
    });
    let line = MISSED_NAMES
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .observe_on(
            this as u64,
            asked,
            twin.as_deref().filter(|_| found.is_some()),
        );
    if let Some(line) = line {
        log_line(&line);
    }
    found
}

/// The motion name the game last asked the motion work at `motion_work` for and did not find
/// (nor a twin for), if any miss was seen there.
pub fn last_missed_motion_on(motion_work: u64) -> Option<String> {
    MISSED_NAMES
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .last_missed_on(motion_work)
        .map(str::to_owned)
}
