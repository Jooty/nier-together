use core::ffi::c_void;
use core::ptr;
use std::panic;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicPtr, Ordering};

use replicant_game::kaine::{
    OpenDoorStateClock, OpenDoorStateVerdict, body_has_a_capped_open_door_state,
    door_carry_is_sane, door_carry_text, open_door_state_ended_text, other_side_of_a_door,
};
use replicant_game::offsets::{
    ACTOR_FROM_HANDLE_RVA, DOOR_ACTOR_CLASS_ID, DOOR_ACTOR_CLASS_OFFSET, DOOR_STAND_POINT_RVA,
    OPEN_DOOR_STATE_DOOR_HANDLE_OFFSET, OPEN_DOOR_STATE_DOOR_INDEX_OFFSET,
    OPEN_DOOR_STATE_DOOR_SIDE_OFFSET, OPEN_DOOR_STATE_FINISHED_OFFSET,
    OPEN_DOOR_STATE_OPEN_VSLOT_OFFSET, OPEN_DOOR_STATE_OPENED_OFFSET,
    OPEN_DOOR_STATE_RUNNING_OFFSET, OPEN_DOOR_STATE_STEP_OFFSET, OPEN_DOOR_STATE_UPDATE_RVA,
};

use crate::build_gate::VerifiedBuild;
use replicant_game::{Position, Replicant};

use crate::diagnostics::log_line;
use crate::game_hooks::hook_address;
use crate::memory::InProcessMemory;
use crate::net::net_clock_ms;
use crate::player_body::real_body;

static ORIGINAL_UPDATE: AtomicPtr<c_void> = AtomicPtr::new(ptr::null_mut());
static BUILD: OnceLock<VerifiedBuild> = OnceLock::new();
static CLOCK: Mutex<OpenDoorStateClock> = Mutex::new(OpenDoorStateClock::new());

type OpenDoorUpdate = unsafe extern "system" fn(*mut c_void, *mut c_void) -> u64;
type ActorFromHandle = unsafe extern "system" fn(*const u32) -> *mut u8;
type DoorStandPoint = unsafe extern "system" fn(*mut u8, *mut f32, i32, i32) -> *mut f32;
type OpenDoorOpenSlot = unsafe extern "system" fn(*mut c_void, *mut c_void);

pub fn note_door_closed() {
    if let Ok(mut clock) = CLOCK.try_lock() {
        clock.note_door_closed(net_clock_ms());
    }
}

pub fn install(build: VerifiedBuild) {
    let _ = BUILD.set(build);
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
    let _ = panic::catch_unwind(|| carry_her_through_the_door(state));
    // SAFETY: the trampoline continues into the original update, which has this exact signature.
    unsafe { core::mem::transmute::<*mut c_void, OpenDoorUpdate>(original)(state, context) }
}

fn has_a_capped_body() -> bool {
    real_body().is_some_and(|real| body_has_a_capped_open_door_state(real.body, real.by_swap))
}

fn state_runs_too_long(state: *mut c_void) -> Option<u64> {
    let capped_body = has_a_capped_body();
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

#[repr(align(16))]
struct StandPointMatrix([f32; 16]);

#[expect(
    unsafe_code,
    reason = "reads the open-door state's own fields and asks the game for the door's stand point on the other side"
)]
fn carry_her_through_the_door(state: *mut c_void) {
    let Some(build) = BUILD.get() else {
        return;
    };
    let base = state.cast::<u8>();
    // SAFETY: `state` is the live StateActionOpenDoor being updated on this thread; the offsets are
    // the step, door index, door side and door handle fields its own update reads.
    let (step, index, side) = unsafe {
        (
            base.add(OPEN_DOOR_STATE_STEP_OFFSET)
                .cast::<i32>()
                .read_unaligned(),
            base.add(OPEN_DOOR_STATE_DOOR_INDEX_OFFSET)
                .cast::<i32>()
                .read_unaligned(),
            base.add(OPEN_DOOR_STATE_DOOR_SIDE_OFFSET)
                .cast::<i32>()
                .read_unaligned(),
        )
    };
    let capped_body = has_a_capped_body();
    let due = CLOCK
        .try_lock()
        .is_ok_and(|mut clock| clock.carry_is_due(capped_body, step));
    if !due {
        return;
    }
    let Some(other_side) = other_side_of_a_door(side) else {
        return;
    };
    let module_base = build.module_base();
    // SAFETY: the RVAs are the game's actor-from-handle lookup (takes the 8-byte handle at the state's
    // +0xE8, returns the actor or 0) and the door's stand point (door actor, 16-float output, door
    // index, side; returns the output), both in the verified build, both plain win64 calls.
    let spot = unsafe {
        let handle = base.add(OPEN_DOOR_STATE_DOOR_HANDLE_OFFSET).cast::<u32>();
        let lookup = core::mem::transmute::<u64, ActorFromHandle>(
            module_base + u64::from(ACTOR_FROM_HANDLE_RVA),
        );
        let door = lookup(handle);
        if door.is_null()
            || door
                .add(DOOR_ACTOR_CLASS_OFFSET)
                .cast::<i32>()
                .read_unaligned()
                != DOOR_ACTOR_CLASS_ID
        {
            return;
        }
        let stand_point = core::mem::transmute::<u64, DoorStandPoint>(
            module_base + u64::from(DOOR_STAND_POINT_RVA),
        );
        let mut matrix = StandPointMatrix([0.0; 16]);
        stand_point(door, matrix.0.as_mut_ptr(), index, other_side);
        [matrix.0[3], matrix.0[7], matrix.0[11]]
    };
    let game = Replicant::new(InProcessMemory::new(build));
    let from = game.position().ok().map(|at| [at.x, at.y, at.z]);
    let heading_degrees = game.yaw_radians().map_or(0.0, f32::to_degrees);
    let carried = door_carry_is_sane(from, spot);
    log_line(&door_carry_text(side, from, spot, heading_degrees, carried));
    if carried {
        crate::teleport::move_player_facing(
            build,
            Position {
                x: spot[0],
                y: spot[1],
                z: spot[2],
            },
            heading_degrees,
        );
    }
}
