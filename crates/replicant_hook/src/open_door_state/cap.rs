use core::ffi::c_void;
use core::ptr;
use std::panic;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::Mutex;

use replicant_game::kaine::{
    OpenDoorStateClock, OpenDoorStateVerdict, body_has_a_capped_open_door_state,
    open_door_state_ended_text,
};
use replicant_game::offsets::{
    OPEN_DOOR_STATE_FINISHED_OFFSET, OPEN_DOOR_STATE_OPEN_VSLOT_OFFSET,
    OPEN_DOOR_STATE_OPENED_OFFSET, OPEN_DOOR_STATE_RUNNING_OFFSET, OPEN_DOOR_STATE_UPDATE_RVA,
};

use crate::build_gate::VerifiedBuild;
use crate::diagnostics::log_line;
use crate::game_hooks::hook_address;
use crate::net::net_clock_ms;
use crate::player_body::real_body;

static ORIGINAL_UPDATE: AtomicPtr<c_void> = AtomicPtr::new(ptr::null_mut());
static CLOCK: Mutex<OpenDoorStateClock> = Mutex::new(OpenDoorStateClock::new());

type OpenDoorUpdate = unsafe extern "system" fn(*mut c_void, *mut c_void) -> u64;
type OpenDoorOpenSlot = unsafe extern "system" fn(*mut c_void, *mut c_void);

pub fn install(build: VerifiedBuild) {
    #[expect(
        unsafe_code,
        reason = "hooks the player's open-door state update by address"
    )]
    // SAFETY: the RVA starts StateActionOpenDoor's update (vtable slot 0x38: rcx state, rdx context,
    // returns 1 once the state is done) in the verified build, and it reads no stack argument; the
    // replacement has the same signature.
    unsafe {
        hook_address(
            "StateActionOpenDoor_Update",
            (build.module_base() + u64::from(OPEN_DOOR_STATE_UPDATE_RVA)) as usize,
            open_door_update_detour as *const () as *mut c_void,
            &ORIGINAL_UPDATE,
        );
    }
}

#[expect(
    unsafe_code,
    reason = "calls the original update through the trampoline MinHook built"
)]
extern "system" fn open_door_update_detour(state: *mut c_void, context: *mut c_void) -> u64 {
    let original = ORIGINAL_UPDATE.load(Ordering::Acquire);
    if original.is_null() {
        return 0;
    }
    let ended = panic::catch_unwind(|| state_runs_too_long(state)).unwrap_or(None);
    if let Some(ran_ms) = ended {
        log_line(&open_door_state_ended_text(ran_ms));
        end_state(state, context);
        return 1;
    }
    // SAFETY: the trampoline continues into the original update, which has this exact signature.
    unsafe { core::mem::transmute::<*mut c_void, OpenDoorUpdate>(original)(state, context) }
}

fn state_runs_too_long(state: *mut c_void) -> Option<u64> {
    let capped_body =
        real_body().is_some_and(|real| body_has_a_capped_open_door_state(real.body, real.by_swap));
    let mut clock = CLOCK.try_lock().ok()?;
    match clock.observe(state as u64, capped_body, net_clock_ms()) {
        OpenDoorStateVerdict::Run => None,
        OpenDoorStateVerdict::EndNow { ran_ms } => Some(ran_ms),
    }
}

#[expect(
    unsafe_code,
    reason = "does what the update does at its own end: the open call if it has not run, then the done flags"
)]
fn end_state(state: *mut c_void, context: *mut c_void) {
    let base = state.cast::<u8>();
    // SAFETY: `state` is the live StateActionOpenDoor the game is updating on this thread; the flag
    // offsets are the bytes its own update reads and writes at its end, and vtable slot 0x88 is
    // the open call the update makes with the same two arguments.
    unsafe {
        if base.add(OPEN_DOOR_STATE_OPENED_OFFSET).read() == 0 {
            let vtable = state.cast::<*const usize>().read();
            let open_slot = vtable.byte_add(OPEN_DOOR_STATE_OPEN_VSLOT_OFFSET).read();
            core::mem::transmute::<usize, OpenDoorOpenSlot>(open_slot)(state, context);
            base.add(OPEN_DOOR_STATE_OPENED_OFFSET).write(1);
        }
        base.add(OPEN_DOOR_STATE_RUNNING_OFFSET).write(0);
        base.add(OPEN_DOOR_STATE_FINISHED_OFFSET).write(1);
    }
}
