use core::ffi::c_void;
use core::ptr;
use std::panic;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::{Mutex, PoisonError};

use replicant_game::quiet_end::{
    LUA_ARG_PAYLOAD_OFFSET, LUA_ARG_TAG_OFFSET, lua_arg_as_int, lua_arg_offset,
};
use replicant_game::story_watch::{
    ACTIVATE_UNIQUE_ACTOR_NATIVE_RVA, AERIE_EXIT_SCRIPT_FUNCTION, COMPANION_SCRIPT_ID,
    DISACTIVATE_UNIQUE_ACTOR_NATIVE_RVA, RemovalChange, StoryRemoval,
};

use crate::build_gate::VerifiedBuild;
use crate::diagnostics::log_line;
use crate::game_hooks::last_pop;
use crate::net::net_clock_ms;
use crate::story_jump::mod_native_call_running;
use nt_hook::detour::hook_address;

static REMOVAL: Mutex<StoryRemoval> = Mutex::new(StoryRemoval::new());
static ORIGINAL_ACTIVATE: AtomicPtr<c_void> = AtomicPtr::new(ptr::null_mut());
static ORIGINAL_DISACTIVATE: AtomicPtr<c_void> = AtomicPtr::new(ptr::null_mut());

type LuaNative = unsafe extern "system" fn(*mut c_void);

pub(crate) fn install(build: VerifiedBuild) {
    let base = build.module_base();
    #[expect(unsafe_code, reason = "hooks the two companion natives by address")]
    // SAFETY: each RVA starts the Lua native of that name in the verified build (argument block in,
    // nothing out), and each replacement has the same signature and calls the original.
    unsafe {
        hook_address(
            "ActivateUniqueActor_native",
            (base + u64::from(ACTIVATE_UNIQUE_ACTOR_NATIVE_RVA)) as usize,
            activate_detour as *const () as *mut c_void,
            &ORIGINAL_ACTIVATE,
        );
        hook_address(
            "DisactivateUniqueActor_native",
            (base + u64::from(DISACTIVATE_UNIQUE_ACTOR_NATIVE_RVA)) as usize,
            disactivate_detour as *const () as *mut c_void,
            &ORIGINAL_DISACTIVATE,
        );
    }
}

pub(super) fn step_removal() -> Option<RemovalChange> {
    REMOVAL
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .step(net_clock_ms())
}

pub(super) fn restore_aerie_exit_removal(
    aerie_exit_flag_on: bool,
    companion_inactive: bool,
) -> bool {
    REMOVAL
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .restore_aerie_exit_from_game_state(aerie_exit_flag_on, companion_inactive)
}

pub(super) fn taken_at_the_aerie_exit() -> bool {
    REMOVAL
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .taken_at_the_aerie_exit()
}

pub(super) fn story_has_taken_her() -> bool {
    REMOVAL
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .settled_out()
}

pub(crate) fn forget_removal() {
    REMOVAL
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .forget();
}

extern "system" fn activate_detour(args: *mut c_void) {
    run_native(&ORIGINAL_ACTIVATE, false, args);
}

extern "system" fn disactivate_detour(args: *mut c_void) {
    run_native(&ORIGINAL_DISACTIVATE, true, args);
}

#[expect(
    unsafe_code,
    reason = "calls the original Lua native through the trampoline MinHook built"
)]
fn run_native(original: &AtomicPtr<c_void>, taken_out: bool, args: *mut c_void) {
    let original = original.load(Ordering::Acquire);
    if original.is_null() {
        return;
    }
    let _ = panic::catch_unwind(|| note_call(taken_out, args));
    // SAFETY: the trampoline continues into the original native with this exact signature.
    unsafe { core::mem::transmute::<*mut c_void, LuaNative>(original)(args) };
}

fn note_call(taken_out: bool, args: *mut c_void) {
    if mod_native_call_running() || first_argument(args) != COMPANION_SCRIPT_ID {
        return;
    }
    let running = last_pop().map_or_else(|| "none".to_owned(), |(name, _)| name);
    let mut removal = REMOVAL.lock().unwrap_or_else(PoisonError::into_inner);
    if taken_out && running.contains(AERIE_EXIT_SCRIPT_FUNCTION) {
        removal.aerie_exit_removed_her(net_clock_ms());
    } else {
        removal.story_called(taken_out, net_clock_ms());
    }
    drop(removal);
    log_line(&format!(
        "watching: the story called {} on script id {COMPANION_SCRIPT_ID} (running {running})",
        if taken_out {
            "_DisactivateUniqueActor"
        } else {
            "_ActivateUniqueActor"
        }
    ));
}

#[expect(
    unsafe_code,
    reason = "reads the first argument slot of a Lua native's argument block"
)]
fn first_argument(args: *mut c_void) -> i32 {
    let slot = lua_arg_offset(0) as usize;
    // SAFETY: both natives read their first argument from this slot, so the block holds it.
    unsafe {
        let tag = *args
            .cast::<u8>()
            .add(slot + LUA_ARG_TAG_OFFSET as usize)
            .cast::<i32>();
        let payload = *args
            .cast::<u8>()
            .add(slot + LUA_ARG_PAYLOAD_OFFSET as usize)
            .cast::<[u8; 8]>();
        lua_arg_as_int(tag, payload)
    }
}
