//! Stone: on the host, an enemy's own message handler turns it to stone; the host tells the
//! guest, whose copy is turned by the same handler and then ticks fully, as a dying copy does.

use core::ffi::c_void;
use core::ptr;
use std::collections::HashSet;
use std::panic;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::{LazyLock, Mutex, OnceLock, PoisonError};

use replicant_game::GameMemory;
use replicant_game::effects::{
    STONE_HANDLERS, stone_handler_rva, stone_state_machine_pointer_offset, stone_state_vtable_rva,
};
use replicant_game::offsets::{
    AI_MESSAGE_RECORD_BYTES, AI_MESSAGE_RECORD_ID_OFFSET, STATE_MACHINE_NEXT_STATE_OFFSET,
    STONE_MESSAGE_ID, STONE_STATE_BYTES_OFFSET,
};
use replicant_proto::{ActorCue, ActorCueKind, Message, StoneHandler, StoneTurn};

use crate::build_gate::VerifiedBuild;
use crate::diagnostics::log_line;
use crate::enemies::{EnemySyncRole, enemy_id_text, shared_actor_id};
use crate::game_hooks::hook_address;
use crate::memory::InProcessMemory;
use crate::net::{self, NetSharedExt};

static HOOKED_BUILD: OnceLock<VerifiedBuild> = OnceLock::new();
static ORIGINAL_HANDLERS: [AtomicPtr<c_void>; 3] = [
    AtomicPtr::new(ptr::null_mut()),
    AtomicPtr::new(ptr::null_mut()),
    AtomicPtr::new(ptr::null_mut()),
];
static LOGGED_ACTORS: LazyLock<Mutex<HashSet<u64>>> = LazyLock::new(Mutex::default);

const MAX_LOGGED_STONE_ACTORS: usize = 256;

type StoneMessageHandler = unsafe extern "system" fn(*mut c_void, *mut c_void, *const u8) -> u64;

#[repr(C, align(16))]
struct MessageRecord([u8; AI_MESSAGE_RECORD_BYTES]);

pub(super) fn install(build: VerifiedBuild) {
    if HOOKED_BUILD.set(build).is_err() {
        return;
    }
    let detours: [*mut c_void; 3] = [
        leader_detour as *const () as *mut c_void,
        sword_detour as *const () as *mut c_void,
        grapple_detour as *const () as *mut c_void,
    ];
    for ((handler, detour), original) in STONE_HANDLERS.iter().zip(detours).zip(&ORIGINAL_HANDLERS)
    {
        #[expect(
            unsafe_code,
            reason = "hooks an enemy's stone message handler by address"
        )]
        // SAFETY: the RVA starts the handler (global state, actor, message record -> handled) in
        // the verified build, and every detour matches that signature.
        unsafe {
            hook_address(
                handler_name(*handler),
                (build.module_base() + u64::from(stone_handler_rva(*handler))) as usize,
                detour,
                original,
            );
        }
    }
}

/// Turns the guest's copy to stone with the handler the host's enemy ran; true when the copy's
/// state machine now holds the stone state.
pub(super) fn turn_copy(build: &VerifiedBuild, copy: u64, turn: StoneTurn) -> bool {
    let mut memory = InProcessMemory::new(build);
    for (offset, byte) in (0_u64..).zip(turn.state_bytes) {
        if memory
            .write_u8(copy + STONE_STATE_BYTES_OFFSET + offset, byte)
            .is_err()
        {
            return false;
        }
    }
    let mut record = MessageRecord([0; AI_MESSAGE_RECORD_BYTES]);
    record.0[AI_MESSAGE_RECORD_ID_OFFSET..AI_MESSAGE_RECORD_ID_OFFSET + 4]
        .copy_from_slice(&STONE_MESSAGE_ID.to_le_bytes());
    let function = (build.module_base() + u64::from(stone_handler_rva(turn.handler))) as usize;
    #[expect(unsafe_code, reason = "calls the enemy's own stone message handler")]
    // SAFETY: the handler (global state, actor, record) reads only the record's id and the actor
    // for message 0x1C, never the global state; the copy is a live copy of the same enemy
    // class, the record outlives the call, and this runs on the game thread.
    unsafe {
        core::mem::transmute::<usize, StoneMessageHandler>(function)(
            ptr::null_mut(),
            copy as usize as *mut c_void,
            record.0.as_ptr(),
        );
    }
    next_state_is_stone(&memory, copy, turn.handler)
}

fn handler_name(handler: StoneHandler) -> &'static str {
    match handler {
        StoneHandler::Leader => "EnemyLeader::StoneMessage",
        StoneHandler::Sword => "EnemySword::StoneMessage",
        StoneHandler::Grapple => "EnemyGrapple::StoneMessage",
    }
}

extern "system" fn leader_detour(
    global: *mut c_void,
    actor: *mut c_void,
    record: *const u8,
) -> u64 {
    run_handler(0, global, actor, record)
}

extern "system" fn sword_detour(global: *mut c_void, actor: *mut c_void, record: *const u8) -> u64 {
    run_handler(1, global, actor, record)
}

extern "system" fn grapple_detour(
    global: *mut c_void,
    actor: *mut c_void,
    record: *const u8,
) -> u64 {
    run_handler(2, global, actor, record)
}

#[expect(
    unsafe_code,
    reason = "calls the original stone message handler through the trampoline MinHook built"
)]
fn run_handler(index: usize, global: *mut c_void, actor: *mut c_void, record: *const u8) -> u64 {
    let original = ORIGINAL_HANDLERS[index].load(Ordering::Acquire);
    if original.is_null() {
        return 1;
    }
    let (actor_address, record_address) = (actor as usize as u64, record as usize as u64);
    let before = panic::catch_unwind(|| read_before(actor_address, record_address))
        .ok()
        .flatten();
    // SAFETY: the trampoline continues into the original handler, which has this signature.
    let handled = unsafe {
        core::mem::transmute::<*mut c_void, StoneMessageHandler>(original)(global, actor, record)
    };
    if let Some(state_bytes) = before {
        let handler = STONE_HANDLERS[index];
        let _ = panic::catch_unwind(|| after_stone_message(handler, actor_address, state_bytes));
    }
    handled
}

fn read_before(actor: u64, record: u64) -> Option<[u8; 2]> {
    let memory = InProcessMemory::new(HOOKED_BUILD.get()?);
    let id = memory
        .read_i32(record + AI_MESSAGE_RECORD_ID_OFFSET as u64)
        .ok()?;
    if id != STONE_MESSAGE_ID {
        return None;
    }
    memory
        .read_array::<2>(actor + STONE_STATE_BYTES_OFFSET)
        .ok()
}

fn after_stone_message(handler: StoneHandler, actor: u64, state_bytes: [u8; 2]) {
    let Some(build) = HOOKED_BUILD.get() else {
        return;
    };
    let memory = InProcessMemory::new(build);
    let turned = next_state_is_stone(&memory, actor, handler);
    let shared = shared_actor_id(actor);
    let first = {
        let mut logged = LOGGED_ACTORS.lock().unwrap_or_else(PoisonError::into_inner);
        logged.len() < MAX_LOGGED_STONE_ACTORS && logged.insert(actor)
    };
    let name = shared.map_or_else(|| format!("actor {actor:#x}"), |(_, id)| enemy_id_text(id));
    if first {
        log_line(&format!(
            "stone: {name} got the stone message ({handler:?} handler, state bytes {state_bytes:?}), turned to stone {turned}"
        ));
    }
    let Some((EnemySyncRole::Host, id)) = shared else {
        return;
    };
    let Some(net) = net::handle() else {
        return;
    };
    if !turned || !net.view().partner_in_play() {
        return;
    }
    let cue = ActorCue {
        actor: id,
        id: 0,
        kind: ActorCueKind::TurnedToStone(StoneTurn {
            handler,
            state_bytes,
        }),
    };
    if cue.validate().is_ok() {
        net.send_relayed(Message::ActorCue(cue));
        log_line(&format!("stone: {name} turned to stone, sent to the guest"));
    }
}

fn next_state_is_stone(memory: &InProcessMemory, actor: u64, handler: StoneHandler) -> bool {
    let stone_vtable = memory.module_base() + u64::from(stone_state_vtable_rva(handler));
    memory
        .read_u64(actor + stone_state_machine_pointer_offset(handler))
        .ok()
        .filter(|machine| *machine != 0)
        .and_then(|machine| {
            memory
                .read_u64(machine + STATE_MACHINE_NEXT_STATE_OFFSET)
                .ok()
        })
        .filter(|state| *state != 0)
        .and_then(|state| memory.read_u64(state).ok())
        == Some(stone_vtable)
}
