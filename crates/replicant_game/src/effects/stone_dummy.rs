use replicant_proto::StoneHandler;

use crate::GameMemory;
use crate::effects::{stone_state_machine_pointer_offset, stone_state_vtable_rva};
use crate::offsets::{
    ACTOR_CLASS_ID_OFFSET, DAMAGE_DUMMY_CLASS_ID, STATE_MACHINE_NEXT_STATE_OFFSET,
    STONE_STATE_DUMMY_HANDLE_GENERATION_OFFSET, STONE_STATE_DUMMY_HANDLE_SLOT_OFFSET,
    STONE_STATE_MACHINE_CURRENT_STATE_OFFSET, STONE_STATE_PHASE_DUMMY_SPAWNED,
    STONE_STATE_PHASE_OFFSET, STONE_STATE_SECONDS_IN_STATE_OFFSET,
};
use crate::puppet::actor_of_handle;

/// The damage dummy a stone state spawns once its wait is over; the statue breaks when it is gone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoneDummy {
    NotSpawnedYet,
    Standing(u64),
    Gone,
}

/// An actor's stone state as its step reads it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StoneStateView {
    pub state: u64,
    pub dummy: StoneDummy,
}

/// What the guest does this tick for a copy turned to stone by the host's cue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoneCopyStep {
    WaitForDummy,
    /// Keep the off-camera end away; nothing keeps the guest's own hits off the dummy.
    HoldUntilHostBreaks {
        state: u64,
        dummy: u64,
    },
    /// The host's statue broke: destroy the dummy, so this copy's own step breaks it now.
    BreakNow {
        dummy: u64,
    },
    Over,
}

/// The stone state in the actor's current or next state slot.
pub fn stone_state_of(memory: &impl GameMemory, actor: u64, handler: StoneHandler) -> Option<u64> {
    let machine = memory
        .read_u64(actor + stone_state_machine_pointer_offset(handler))
        .ok()
        .filter(|machine| *machine != 0)?;
    let stone_vtable = memory.module_base() + u64::from(stone_state_vtable_rva(handler));
    [
        STONE_STATE_MACHINE_CURRENT_STATE_OFFSET,
        STATE_MACHINE_NEXT_STATE_OFFSET,
    ]
    .into_iter()
    .filter_map(|slot| memory.read_u64(machine + slot).ok())
    .find(|state| *state != 0 && memory.read_u64(*state).ok() == Some(stone_vtable))
}

pub fn read_stone_state(
    memory: &impl GameMemory,
    actor: u64,
    handler: StoneHandler,
) -> Option<StoneStateView> {
    let state = stone_state_of(memory, actor, handler)?;
    let phase = memory.read_i32(state + STONE_STATE_PHASE_OFFSET).ok()?;
    if phase != STONE_STATE_PHASE_DUMMY_SPAWNED {
        return Some(StoneStateView {
            state,
            dummy: StoneDummy::NotSpawnedYet,
        });
    }
    let handle = stone_dummy_handle(memory, state)?;
    let dummy = actor_of_stone_dummy_handle(memory, handle);
    Some(StoneStateView {
        state,
        dummy: dummy.map_or(StoneDummy::Gone, StoneDummy::Standing),
    })
}

/// The (slot, generation) handle a stone state keeps for the damage dummy it spawned.
pub fn stone_dummy_handle(memory: &impl GameMemory, state: u64) -> Option<(u32, u32)> {
    let slot = memory
        .read_i32(state + STONE_STATE_DUMMY_HANDLE_SLOT_OFFSET)
        .ok()?;
    let generation = memory
        .read_i32(state + STONE_STATE_DUMMY_HANDLE_GENERATION_OFFSET)
        .ok()?;
    Some((slot.cast_unsigned(), generation.cast_unsigned()))
}

/// The damage dummy a handle still names, or None once it is gone or the slot holds another class.
pub fn actor_of_stone_dummy_handle(memory: &impl GameMemory, handle: (u32, u32)) -> Option<u64> {
    actor_of_handle(memory, handle.0, handle.1)
        .ok()
        .flatten()
        .filter(|dummy| {
            memory.read_i32(dummy + ACTOR_CLASS_ID_OFFSET).ok() == Some(DAMAGE_DUMMY_CLASS_ID)
        })
}

/// What the host does with a watched statue this tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostStatueWatch {
    Keep,
    Broke,
    Drop,
}

/// The statue broke when its stone state shows the dummy gone, or when the state can no longer
/// be read after its dummy was seen standing and that dummy's handle no longer resolves.
pub fn host_statue_watch(
    view: Option<StoneStateView>,
    remembered_dummy_gone: bool,
) -> HostStatueWatch {
    match view {
        Some(view) if view.dummy == StoneDummy::Gone => HostStatueWatch::Broke,
        Some(_) => HostStatueWatch::Keep,
        None if remembered_dummy_gone => HostStatueWatch::Broke,
        None => HostStatueWatch::Drop,
    }
}

pub fn stone_copy_step(view: Option<StoneStateView>, host_broke: bool) -> StoneCopyStep {
    let Some(view) = view else {
        return StoneCopyStep::Over;
    };
    match view.dummy {
        StoneDummy::NotSpawnedYet => StoneCopyStep::WaitForDummy,
        StoneDummy::Standing(dummy) if host_broke => StoneCopyStep::BreakNow { dummy },
        StoneDummy::Standing(dummy) => StoneCopyStep::HoldUntilHostBreaks {
            state: view.state,
            dummy,
        },
        StoneDummy::Gone => StoneCopyStep::Over,
    }
}

/// Keeps the stone state's time at zero, so its off-camera end (after 10 s) never comes before
/// the host's break.
pub fn hold_off_off_camera_end(memory: &mut impl GameMemory, state: u64) -> bool {
    memory
        .write_f32(state + STONE_STATE_SECONDS_IN_STATE_OFFSET, 0.0)
        .is_ok()
}

/// On the host: true once its own statue broke (the dummy it spawned is gone).
pub fn host_statue_broke(view: Option<StoneStateView>) -> bool {
    view.is_some_and(|view| view.dummy == StoneDummy::Gone)
}
