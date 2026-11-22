use core::ffi::c_void;
use core::ptr;
use std::panic;
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};

use automata_game::address::{LOOK_CLEAR_OTHER_SLOTS_FUNCTION, LOOK_SANITIZE_FUNCTION};
use nt_hook::build_gate::VerifiedBuild;

use crate::game_calls::GameCalls;
use crate::look_driver::net_reading::net_reading;
use crate::look_driver::shared::try_state;

static ORIGINAL_SANITIZE: AtomicPtr<c_void> = AtomicPtr::new(ptr::null_mut());
static ORIGINAL_CLEAR_OTHER_SLOTS: AtomicPtr<c_void> = AtomicPtr::new(ptr::null_mut());
static LOAD_PASS_SEEN: AtomicBool = AtomicBool::new(false);

type SanitizeCall = unsafe extern "system" fn() -> u64;
type ClearOtherSlotsCall = unsafe extern "system" fn();

pub(super) fn install(build: VerifiedBuild, calls: &GameCalls) {
    let hook = |label: &str, rva: u32, replacement: *mut c_void, original: &AtomicPtr<c_void>| {
        let address = build.module_base() as usize + rva as usize;
        #[expect(
            unsafe_code,
            reason = "hooks a look function the load path calls; the replacement forwards to the original"
        )]
        // SAFETY: the address starts the look function of the verified build (it takes no
        // arguments); the replacement has that signature and returns the original's result.
        unsafe {
            nt_hook::detour::hook_address(label, address, replacement, original)
        }
    };
    let variant = calls.variant();
    hook(
        "look sanitize (keeps a picked look)",
        LOOK_SANITIZE_FUNCTION.rva(variant),
        sanitize_detour as *const () as *mut c_void,
        &ORIGINAL_SANITIZE,
    );
    hook(
        "look clear other slots (keeps a picked look)",
        LOOK_CLEAR_OTHER_SLOTS_FUNCTION.rva(variant),
        clear_other_slots_detour as *const () as *mut c_void,
        &ORIGINAL_CLEAR_OTHER_SLOTS,
    );
}

pub(super) fn take_load_pass_seen() -> bool {
    LOAD_PASS_SEEN.swap(false, Ordering::AcqRel)
}

fn write_picks_again() {
    LOAD_PASS_SEEN.store(true, Ordering::Release);
    let _ = panic::catch_unwind(|| {
        let Some(state) = try_state() else {
            return;
        };
        let Some(calls) = state.calls else {
            return;
        };
        let allowed = net_reading().allowed();
        for (slot, kind, value) in state.picks.globals_after_load(allowed) {
            calls.write_look_global(kind, slot, value);
        }
    });
}

#[expect(
    unsafe_code,
    reason = "calls the original look sanitizer through the trampoline MinHook built"
)]
extern "system" fn sanitize_detour() -> u64 {
    let original = ORIGINAL_SANITIZE.load(Ordering::Acquire);
    if original.is_null() {
        return 0;
    }
    // SAFETY: the trampoline continues into the original function, which takes no arguments.
    let result = unsafe { core::mem::transmute::<*mut c_void, SanitizeCall>(original)() };
    write_picks_again();
    result
}

#[expect(
    unsafe_code,
    reason = "calls the original other-slots clear through the trampoline MinHook built"
)]
extern "system" fn clear_other_slots_detour() {
    let original = ORIGINAL_CLEAR_OTHER_SLOTS.load(Ordering::Acquire);
    if original.is_null() {
        return;
    }
    // SAFETY: the trampoline continues into the original function, which takes no arguments.
    unsafe { core::mem::transmute::<*mut c_void, ClearOtherSlotsCall>(original)() };
    write_picks_again();
}
