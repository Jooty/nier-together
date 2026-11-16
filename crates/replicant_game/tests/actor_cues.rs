use replicant_game::effects::{
    ActorCueCall, ActorCueMoment, CopyForCue, FirstCues, STONE_HANDLERS,
    actor_cue_goes_to_enemy_sync, actor_cue_is_sent, actor_cue_replay_target, actor_cue_text,
    actor_holds_stone, sound_caller_is_one_shot, stack_holds_ai_step, stone_handler_rva,
    stone_state_machine_pointer_offset, stone_state_vtable_rva,
};
use replicant_game::offsets::{
    SOUND_BY_ID_WRAPPER_BODY_RVAS, STONE_STATE_MACHINE_CURRENT_STATE_OFFSET,
    STATE_MACHINE_NEXT_STATE_OFFSET,
};
use crate::fake_memory::{FAKE_MODULE_BASE, FakeMemory};
use replicant_proto::{ActorCue, ActorCueKind, AttachedEffect, EnemyId, StoneHandler};

const ACTOR_TICK_PRE_UPDATE_RETURN_RVA: u32 = 0xE_027A;
const ACTOR_TICK_AI_STEP_RETURN_RVA: u32 = 0xE_02AB;
const ACTOR_TICK_STATE_STEP_RETURN_RVA: u32 = 0xE_02F7;
const EMIL_MAGIC_ARRAY_EFFECT_ID: u32 = 458;

fn in_a_shared_actors_ai_step() -> ActorCueMoment {
    ActorCueMoment {
        hosting: true,
        shared_actor: true,
        in_ai_step: true,
        nested: false,
        made_for_a_bullet: false,
        made_for_a_cube_glow: false,
    }
}

fn companion() -> EnemyId {
    EnemyId {
        group: 0xFFFF,
        ordinal: 0xFFFE,
    }
}

#[test]
fn what_a_shared_actors_ai_step_makes_on_the_host_is_sent() {
    assert!(actor_cue_is_sent(in_a_shared_actors_ai_step()));
}

#[test]
fn what_a_held_copy_also_makes_or_the_shot_sync_draws_is_not_sent() {
    let not_sent = [
        ActorCueMoment {
            hosting: false,
            ..in_a_shared_actors_ai_step()
        },
        ActorCueMoment {
            shared_actor: false,
            ..in_a_shared_actors_ai_step()
        },
        ActorCueMoment {
            in_ai_step: false,
            ..in_a_shared_actors_ai_step()
        },
        ActorCueMoment {
            nested: true,
            ..in_a_shared_actors_ai_step()
        },
        ActorCueMoment {
            made_for_a_bullet: true,
            ..in_a_shared_actors_ai_step()
        },
        ActorCueMoment {
            made_for_a_cube_glow: true,
            ..in_a_shared_actors_ai_step()
        },
    ];
    for moment in not_sent {
        assert!(!actor_cue_is_sent(moment), "{moment:?}");
    }
}

#[test]
fn only_the_ai_and_state_step_calls_of_the_actor_tick_mark_the_stack() {
    assert!(stack_holds_ai_step(&[
        0x35_73E0,
        ACTOR_TICK_AI_STEP_RETURN_RVA
    ]));
    assert!(stack_holds_ai_step(&[ACTOR_TICK_STATE_STEP_RETURN_RVA]));
    // The pre-update and the motion sequencer still run on a held copy.
    assert!(!stack_holds_ai_step(&[
        0x35_73E0,
        ACTOR_TICK_PRE_UPDATE_RETURN_RVA
    ]));
    assert!(!stack_holds_ai_step(&[]));
}

#[test]
fn each_stone_handler_names_its_own_state_and_state_machine() {
    assert_eq!(stone_handler_rva(StoneHandler::Leader), 0x2D_8CF0);
    assert_eq!(stone_handler_rva(StoneHandler::Sword), 0x2D_BAD0);
    assert_eq!(stone_handler_rva(StoneHandler::Grapple), 0x2F_68C0);
    assert_eq!(stone_state_vtable_rva(StoneHandler::Leader), 0xB6_8048);
    assert_eq!(stone_state_vtable_rva(StoneHandler::Sword), 0xB6_8AE0);
    assert_eq!(stone_state_vtable_rva(StoneHandler::Grapple), 0xB6_FEF8);
    assert_eq!(
        stone_state_machine_pointer_offset(StoneHandler::Leader),
        0x2DCB * 8
    );
    assert_eq!(
        stone_state_machine_pointer_offset(StoneHandler::Grapple),
        0x2DCA * 8
    );
    assert_eq!(STONE_HANDLERS.len(), 3);
}

#[test]
fn each_actor_call_and_id_is_logged_once() {
    let mut first = FirstCues::default();
    assert!(first.is_first(
        companion(),
        ActorCueCall::AttachById,
        EMIL_MAGIC_ARRAY_EFFECT_ID
    ));
    assert!(!first.is_first(
        companion(),
        ActorCueCall::AttachById,
        EMIL_MAGIC_ARRAY_EFFECT_ID
    ));
    assert!(first.is_first(
        companion(),
        ActorCueCall::RemoveAttached,
        EMIL_MAGIC_ARRAY_EFFECT_ID
    ));
}

#[test]
fn an_attach_cue_reads_as_its_effect_and_actor() {
    let cue = ActorCue {
        actor: companion(),
        id: EMIL_MAGIC_ARRAY_EFFECT_ID,
        kind: ActorCueKind::AttachEffect(AttachedEffect {
            bone: -1,
            unique: false,
            colour: [1.0; 4],
            style: [1.0; 4],
            mode: 0,
            flags: 0,
            extra: 0,
        }),
    };
    assert_eq!(
        actor_cue_text("unique actor -2", &cue),
        "effect 458 attached to unique actor -2 (bone -1, unique false)"
    );
    assert_eq!(ActorCueCall::of_kind(&cue.kind), ActorCueCall::AttachById);
}

#[test]
fn a_stone_turn_goes_to_the_enemy_sync_and_effects_stay_with_the_effects() {
    let stone = ActorCueKind::TurnedToStone(replicant_proto::StoneTurn {
        handler: StoneHandler::Leader,
        state_bytes: [1, 0],
    });
    let sound = ActorCueKind::Sound {
        position: [0.0; 3],
        levels: [1.0, 0.0],
    };
    assert!(actor_cue_goes_to_enemy_sync(&stone));
    assert!(!actor_cue_goes_to_enemy_sync(&sound));
}

const COPY: u64 = 0x5000_0000;

fn copy(held: bool, live: bool) -> Option<CopyForCue> {
    Some(CopyForCue {
        actor: COPY,
        held,
        live,
    })
}

#[test]
fn a_cue_is_replayed_on_a_held_live_copy() {
    assert_eq!(actor_cue_replay_target(copy(true, true)), Some(COPY));
}

#[test]
fn a_cue_is_not_replayed_on_a_copy_that_runs_its_own_steps() {
    // Kaine lent to a script, a villager in a talk, a dying stone copy released to its own death.
    assert_eq!(actor_cue_replay_target(copy(false, true)), None);
}

#[test]
fn a_cue_is_not_replayed_on_a_copy_the_game_already_freed() {
    assert_eq!(actor_cue_replay_target(copy(true, false)), None);
}

#[test]
fn a_cue_is_not_replayed_across_a_phase_with_no_copy() {
    assert_eq!(actor_cue_replay_target(None), None);
}

#[test]
fn only_a_one_shot_sound_is_sent_because_its_stop_never_crosses() {
    let wrapper = SOUND_BY_ID_WRAPPER_BODY_RVAS.start + 4;
    assert!(sound_caller_is_one_shot(wrapper));
    assert!(!sound_caller_is_one_shot(0x18_A472));
    assert!(!sound_caller_is_one_shot(SOUND_BY_ID_WRAPPER_BODY_RVAS.end));
}

const ACTOR: u64 = 0x6000_0000;
const MACHINE: u64 = 0x6100_0000;
const STONE_STATE: u64 = 0x6200_0000;
const OTHER_STATE: u64 = 0x6300_0000;

fn stoned_actor(slot: Option<u64>) -> FakeMemory {
    let mut memory = FakeMemory::new();
    let vtable = FAKE_MODULE_BASE + u64::from(stone_state_vtable_rva(StoneHandler::Leader));
    memory.put(
        ACTOR + stone_state_machine_pointer_offset(StoneHandler::Leader),
        &MACHINE.to_le_bytes(),
    );
    memory.put(MACHINE + STONE_STATE_MACHINE_CURRENT_STATE_OFFSET, &OTHER_STATE.to_le_bytes());
    memory.put(MACHINE + STATE_MACHINE_NEXT_STATE_OFFSET, &0_u64.to_le_bytes());
    memory.put(OTHER_STATE, &0x1234_u64.to_le_bytes());
    memory.put(STONE_STATE, &vtable.to_le_bytes());
    if let Some(slot) = slot {
        memory.put(MACHINE + slot, &STONE_STATE.to_le_bytes());
    }
    memory
}

#[test]
fn stone_in_the_current_state_slot_counts_as_stone() {
    let memory = stoned_actor(Some(STONE_STATE_MACHINE_CURRENT_STATE_OFFSET));
    assert!(actor_holds_stone(&memory, ACTOR, StoneHandler::Leader));
}

#[test]
fn stone_in_the_next_state_slot_counts_as_stone() {
    let memory = stoned_actor(Some(STATE_MACHINE_NEXT_STATE_OFFSET));
    assert!(actor_holds_stone(&memory, ACTOR, StoneHandler::Leader));
}

#[test]
fn an_actor_with_no_stone_in_either_slot_is_not_stone() {
    let memory = stoned_actor(None);
    assert!(!actor_holds_stone(&memory, ACTOR, StoneHandler::Leader));
}
