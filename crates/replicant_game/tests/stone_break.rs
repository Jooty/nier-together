use crate::fake_memory::{FAKE_MODULE_BASE, FakeMemory};
use replicant_game::GameMemory;
use replicant_game::effects::{
    HostStatueWatch, StoneCopyStep, StoneDummy, StoneStateView, actor_of_stone_dummy_handle,
    hold_off_off_camera_end, host_statue_broke, host_statue_watch, read_stone_state,
    stone_copy_step, stone_dummy_handle, stone_state_machine_pointer_offset, stone_state_vtable_rva,
};
use replicant_game::offsets::{
    ACTOR_CLASS_ID_OFFSET, DAMAGE_DUMMY_CLASS_ID, HANDLE_POOL_GENERATIONS_RVA,
    HANDLE_POOL_POINTERS_RVA, STONE_STATE_DUMMY_HANDLE_GENERATION_OFFSET,
    STONE_STATE_DUMMY_HANDLE_SLOT_OFFSET, STONE_STATE_MACHINE_CURRENT_STATE_OFFSET,
    STONE_STATE_PHASE_DUMMY_SPAWNED, STONE_STATE_PHASE_OFFSET, STONE_STATE_SECONDS_IN_STATE_OFFSET,
};
use replicant_proto::StoneHandler;

const STONED_COPY: u64 = 0x6000_0000;
const MACHINE: u64 = 0x6100_0000;
const STONE_STATE: u64 = 0x6200_0000;
const DUMMY: u64 = 0x6300_0000;
const DUMMY_SLOT: u32 = 0x2A;
const DUMMY_GENERATION: u32 = 0x51;
const STONE_WAIT_PHASE: i32 = 0x2BD;

fn stoned_copy(phase: i32, dummy_standing: bool) -> FakeMemory {
    let mut memory = FakeMemory::new();
    let vtable = FAKE_MODULE_BASE + u64::from(stone_state_vtable_rva(StoneHandler::Leader));
    memory.put(
        STONED_COPY + stone_state_machine_pointer_offset(StoneHandler::Leader),
        &MACHINE.to_le_bytes(),
    );
    memory.put(
        MACHINE + STONE_STATE_MACHINE_CURRENT_STATE_OFFSET,
        &STONE_STATE.to_le_bytes(),
    );
    memory.put(STONE_STATE, &vtable.to_le_bytes());
    memory.put(STONE_STATE + STONE_STATE_PHASE_OFFSET, &phase.to_le_bytes());
    memory.put(
        STONE_STATE + STONE_STATE_DUMMY_HANDLE_SLOT_OFFSET,
        &DUMMY_SLOT.to_le_bytes(),
    );
    memory.put(
        STONE_STATE + STONE_STATE_DUMMY_HANDLE_GENERATION_OFFSET,
        &DUMMY_GENERATION.to_le_bytes(),
    );
    memory.put(
        STONE_STATE + STONE_STATE_SECONDS_IN_STATE_OFFSET,
        &9.5_f32.to_le_bytes(),
    );
    let generation = if dummy_standing {
        DUMMY_GENERATION
    } else {
        DUMMY_GENERATION + 1
    };
    memory.put_at_rva(
        HANDLE_POOL_GENERATIONS_RVA + DUMMY_SLOT * 4,
        &generation.to_le_bytes(),
    );
    memory.put_at_rva(
        HANDLE_POOL_POINTERS_RVA + DUMMY_SLOT * 8,
        &DUMMY.to_le_bytes(),
    );
    memory.put(
        DUMMY + ACTOR_CLASS_ID_OFFSET,
        &DAMAGE_DUMMY_CLASS_ID.to_le_bytes(),
    );
    memory
}

fn view(memory: &FakeMemory) -> Option<StoneStateView> {
    read_stone_state(memory, STONED_COPY, StoneHandler::Leader)
}

#[test]
fn the_stone_state_names_its_dummy_only_once_it_spawned_it() {
    // 0x1402D7830: phase 0 attaches and waits (0x2BD), then spawns class 0xA5 and moves to 0x2F5.
    let waiting = stoned_copy(STONE_WAIT_PHASE, true);
    assert_eq!(view(&waiting).unwrap().dummy, StoneDummy::NotSpawnedYet);
    let spawned = stoned_copy(STONE_STATE_PHASE_DUMMY_SPAWNED, true);
    assert_eq!(view(&spawned).unwrap().dummy, StoneDummy::Standing(DUMMY));
    let broken = stoned_copy(STONE_STATE_PHASE_DUMMY_SPAWNED, false);
    assert_eq!(view(&broken).unwrap().dummy, StoneDummy::Gone);
}

#[test]
fn a_handle_slot_that_holds_another_class_is_not_the_dummy() {
    let mut memory = stoned_copy(STONE_STATE_PHASE_DUMMY_SPAWNED, true);
    memory.put(DUMMY + ACTOR_CLASS_ID_OFFSET, &0xC_i32.to_le_bytes());
    assert_eq!(view(&memory).unwrap().dummy, StoneDummy::Gone);
}

#[test]
fn before_the_fix_the_guests_copy_broke_only_by_its_own_hits_now_the_hosts_break_breaks_it() {
    // L34 run1: 9:0 (class 0xc) "died (by host or world)" at 576/576 at 08:34:13.340 and its
    // body was gone 3207 ms later; the guest's copy must break when the host's statue does.
    let memory = stoned_copy(STONE_STATE_PHASE_DUMMY_SPAWNED, true);
    assert_eq!(
        stone_copy_step(view(&memory), false),
        StoneCopyStep::HoldUntilHostBreaks {
            state: STONE_STATE,
            dummy: DUMMY,
        }
    );
    assert_eq!(
        stone_copy_step(view(&memory), true),
        StoneCopyStep::BreakNow { dummy: DUMMY }
    );
}

#[test]
fn a_host_break_before_the_guests_dummy_spawned_waits_for_it() {
    let memory = stoned_copy(STONE_WAIT_PHASE, true);
    assert_eq!(
        stone_copy_step(view(&memory), true),
        StoneCopyStep::WaitForDummy
    );
}

#[test]
fn a_copy_out_of_its_stone_state_is_over() {
    let mut memory = stoned_copy(STONE_STATE_PHASE_DUMMY_SPAWNED, true);
    memory.put(
        MACHINE + STONE_STATE_MACHINE_CURRENT_STATE_OFFSET,
        &0_u64.to_le_bytes(),
    );
    assert_eq!(stone_copy_step(view(&memory), true), StoneCopyStep::Over);
}

#[test]
fn the_host_sees_its_own_break_when_its_dummy_is_gone() {
    assert!(!host_statue_broke(view(&stoned_copy(
        STONE_WAIT_PHASE,
        true
    ))));
    assert!(!host_statue_broke(view(&stoned_copy(
        STONE_STATE_PHASE_DUMMY_SPAWNED,
        true
    ))));
    assert!(host_statue_broke(view(&stoned_copy(
        STONE_STATE_PHASE_DUMMY_SPAWNED,
        false
    ))));
    assert!(!host_statue_broke(None));
}

#[test]
fn the_guests_off_camera_end_is_held_off_while_it_waits() {
    // The step ends the state silently once 10.0 s (0x140AC06B0) <= state+0xC and the actor is
    // outside the camera (0x1403F1000).
    let mut memory = stoned_copy(STONE_STATE_PHASE_DUMMY_SPAWNED, true);
    assert!(hold_off_off_camera_end(&mut memory, STONE_STATE));
    assert_eq!(
        memory
            .read_f32(STONE_STATE + STONE_STATE_SECONDS_IN_STATE_OFFSET)
            .unwrap(),
        0.0
    );
}

#[test]
fn the_guests_stone_step_never_writes_a_survive_flag_through_the_dummy() {
    // Class 0xA5 is 0x1600 bytes (factory 0x14018E750); the parameter-object pointer sits at
    // actor +0x14350, so a survive-at-one-hp write on the dummy lands in another heap block.
    let source = include_str!("../../replicant_hook/src/enemies/guest_combat.rs");
    let step = &source[source.find("fn step_stoned_copies").unwrap()
        ..source.find("fn kill_copy").unwrap()];
    assert!(!step.contains("set_survive_at_one_hp"));
    assert!(!step.contains("ACTOR_PARAMETER_OBJECT_OFFSET"));
}

#[test]
fn the_hosts_break_is_sent_when_the_statue_left_before_the_next_step() {
    let spawned = stoned_copy(STONE_STATE_PHASE_DUMMY_SPAWNED, true);
    let handle = stone_dummy_handle(&spawned, STONE_STATE).unwrap();
    assert_eq!(handle, (DUMMY_SLOT, DUMMY_GENERATION));
    assert_eq!(actor_of_stone_dummy_handle(&spawned, handle), Some(DUMMY));
    let after = stoned_copy(STONE_STATE_PHASE_DUMMY_SPAWNED, false);
    assert!(actor_of_stone_dummy_handle(&after, handle).is_none());
    // The statue is gone from the pool, so its state cannot be read: no view.
    assert_eq!(host_statue_watch(None, true), HostStatueWatch::Broke);
    assert_eq!(host_statue_watch(None, false), HostStatueWatch::Drop);
    assert_eq!(
        host_statue_watch(view(&spawned), false),
        HostStatueWatch::Keep
    );
    assert_eq!(
        host_statue_watch(view(&after), true),
        HostStatueWatch::Broke
    );
}
