use core::ffi::c_void;
use core::ptr;
use std::panic;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::{Mutex, OnceLock, PoisonError};

use replicant_game::dialogue::SetTalkCalls;
use replicant_game::offsets::SET_TALK_NATIVE_RVA;

use crate::build_gate::VerifiedBuild;
use crate::diagnostics::log_line;
use crate::game_hooks::{hook_address, last_pop};
use crate::net::net_clock_ms;

static HOOKED_BUILD: OnceLock<VerifiedBuild> = OnceLock::new();
static ORIGINAL_SET_TALK: AtomicPtr<c_void> = AtomicPtr::new(ptr::null_mut());
static SET_TALK_CALLS: Mutex<SetTalkCalls> = Mutex::new(SetTalkCalls::new());

type LuaNative = unsafe extern "system" fn(*mut c_void) -> i32;

pub(super) fn install(build: VerifiedBuild) {
    let _ = HOOKED_BUILD.set(build);
    let base = build.module_base();
    #[expect(unsafe_code, reason = "hooks the _SetTalk native by address")]
    // SAFETY: the RVA starts the `_SetTalk` Lua native (args block in, result code out) in the
    // verified build (see docs off-hand/story-choices.md), and the replacement has the same
    // signature and passes the original's result through.
    unsafe {
        hook_address(
            "_SetTalk native",
            (base + u64::from(SET_TALK_NATIVE_RVA)) as usize,
            set_talk_detour as *const () as *mut c_void,
            &ORIGINAL_SET_TALK,
        );
    }
}

pub(super) fn set_talk_opens_the_window_now() -> bool {
    SET_TALK_CALLS
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .opens_the_window_now(net_clock_ms())
}

#[expect(
    unsafe_code,
    reason = "calls the original _SetTalk through the trampoline MinHook built"
)]
extern "system" fn set_talk_detour(args: *mut c_void) -> i32 {
    let _ = panic::catch_unwind(note_set_talk);
    let original = ORIGINAL_SET_TALK.load(Ordering::Acquire);
    if original.is_null() {
        return 0;
    }
    // SAFETY: the trampoline continues into the original native with this exact signature.
    unsafe { core::mem::transmute::<*mut c_void, LuaNative>(original)(args) }
}

fn note_set_talk() {
    SET_TALK_CALLS
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .note_call(net_clock_ms());
    let Some((call, _)) = last_pop() else {
        return;
    };
    let shared_step = HOOKED_BUILD
        .get()
        .is_some_and(|build| crate::cutscene_skip::is_led_step_here(build, &call));
    crate::dialogue::note_script_talk_start(&call, shared_step);
    log_line(&format!(
        "talk start: _SetTalk ran in script call {call}, the text window it opens is this game's own copy"
    ));
}
