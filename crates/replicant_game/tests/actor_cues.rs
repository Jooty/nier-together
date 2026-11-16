use replicant_game::effects::{
    ActorCueCall, ActorCueMoment, FirstCues, STONE_HANDLERS, actor_cue_goes_to_enemy_sync,
    actor_cue_is_sent, actor_cue_text, stack_holds_ai_step, stone_handler_rva,
    stone_state_machine_pointer_offset, stone_state_vtable_rva,
};
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

#[test]
fn a_widening_ordinary_bullet_with_no_live_effect_is_read_for_the_log() {
    use replicant_game::enemy_bullets::read_not_drawn_normal_bullet;
    use replicant_game::offsets::{
        EFFECT_INVALID_HANDLE, NORMAL_BULLET_ATTACK_SETTINGS_POINTER_OFFSET,
        NORMAL_BULLET_EFFECT_HANDLES_OFFSET, NORMAL_BULLET_RECORD_EFFECT_IDS_OFFSET,
        NORMAL_BULLET_RECORD_POINTER_OFFSET, NORMAL_BULLET_SETTINGS_GROUND_EFFECT_ID_OFFSET,
    };

    use crate::fake_memory::FakeMemory;

    // L34 run0: "99:0's ordinary bullet ... width grows 21.43, ... ground effect 0".
    let (bullet, record, settings) = (0x1000_0000_u64, 0x2000_0000_u64, 0x3000_0000_u64);
    let boss = EnemyId {
        group: 99,
        ordinal: 0,
    };
    let mut memory = FakeMemory::new();
    memory.put(
        bullet + NORMAL_BULLET_RECORD_POINTER_OFFSET,
        &record.to_le_bytes(),
    );
    memory.put(
        bullet + NORMAL_BULLET_ATTACK_SETTINGS_POINTER_OFFSET,
        &settings.to_le_bytes(),
    );
    for slot in 0..4_u64 {
        let id: i32 = if slot == 0 { 5001 } else { 0 };
        memory.put(
            record + NORMAL_BULLET_RECORD_EFFECT_IDS_OFFSET + slot * 4,
            &id.to_le_bytes(),
        );
        memory.put(
            bullet + NORMAL_BULLET_EFFECT_HANDLES_OFFSET + slot * 4,
            &EFFECT_INVALID_HANDLE.to_le_bytes(),
        );
    }
    memory.put(
        settings + NORMAL_BULLET_SETTINGS_GROUND_EFFECT_ID_OFFSET,
        &0_i32.to_le_bytes(),
    );
    let read = read_not_drawn_normal_bullet(&memory, bullet, boss).expect("readable");
    assert_eq!(read.enemy, boss);
    assert_eq!(read.record_effect_ids, [5001, 0, 0, 0]);
    assert_eq!(read.effect_handles, [EFFECT_INVALID_HANDLE; 4]);
    assert_eq!(read.ground_effect_id, 0);
}
