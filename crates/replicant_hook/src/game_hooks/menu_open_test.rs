use core::ffi::c_void;
use core::ptr;
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};

use replicant_game::offsets::UI_MANAGER_IS_FREE_TO_ACT_FUNCTION_RVA;

use crate::build_gate::VerifiedBuild;
use crate::diagnostics::log_line;
use nt_hook::detour::hook_address;

static ORIGINAL_IS_FREE_TO_ACT: AtomicPtr<c_void> = AtomicPtr::new(ptr::null_mut());
static INSIDE_FREE_TO_ACT_TEST: AtomicBool = AtomicBool::new(false);
static EXEMPTION_LOGGED: AtomicBool = AtomicBool::new(false);

type IsFreeToAct = unsafe extern "system" fn(*mut c_void) -> u8;

pub(super) fn inside_free_to_act_test() -> bool {
    INSIDE_FREE_TO_ACT_TEST.load(Ordering::Relaxed)
}

pub(super) fn note_held_bit_left_out() {
    if !EXEMPTION_LOGGED.swap(true, Ordering::Relaxed) {
        log_line(
            "watching: the game's free-to-act test (it gates the pause menu) runs without the held player-input bit",
        );
    }
}

pub(super) fn install(build: VerifiedBuild) {
    let base = build.module_base();
    #[expect(unsafe_code, reason = "hooks the game's free-to-act test by address")]
    // SAFETY: the RVA starts UIManager::IsFreeToAct (manager in, byte out) in the verified build,
    // and the replacement has the same signature.
    unsafe {
        hook_address(
            "UIManager_IsFreeToAct",
            (base + u64::from(UI_MANAGER_IS_FREE_TO_ACT_FUNCTION_RVA)) as usize,
            is_free_to_act_detour as *const () as *mut c_void,
            &ORIGINAL_IS_FREE_TO_ACT,
        );
    }
}

#[expect(
    unsafe_code,
    reason = "calls the original test through the trampoline MinHook built"
)]
extern "system" fn is_free_to_act_detour(manager: *mut c_void) -> u8 {
    let original = ORIGINAL_IS_FREE_TO_ACT.load(Ordering::Acquire);
    if original.is_null() {
        return 0;
    }
    INSIDE_FREE_TO_ACT_TEST.store(true, Ordering::Relaxed);
    // SAFETY: the trampoline continues into the original test, which has this exact signature.
    let free = unsafe { core::mem::transmute::<*mut c_void, IsFreeToAct>(original)(manager) };
    INSIDE_FREE_TO_ACT_TEST.store(false, Ordering::Relaxed);
    free
}
