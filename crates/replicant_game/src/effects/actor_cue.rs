use std::collections::HashSet;

use replicant_proto::{ActorCue, ActorCueKind, EnemyId, SHOT_SOUND_LEVELS, StoneHandler};

use crate::GameMemory;
use crate::offsets::{
    ACTOR_TICK_AI_STEP_RETURN_RVAS, GRAPPLE_STATE_MACHINE_POINTER_OFFSET,
    GRAPPLE_STONE_HANDLER_FUNCTION_RVA, GRAPPLE_STONE_STATE_VTABLE_RVA,
    LEADER_AND_SWORD_STATE_MACHINE_POINTER_OFFSET, LEADER_STONE_HANDLER_FUNCTION_RVA,
    LEADER_STONE_STATE_VTABLE_RVA, SOUND_BY_ID_WRAPPER_BODY_RVAS, STATE_MACHINE_NEXT_STATE_OFFSET,
    STONE_STATE_MACHINE_CURRENT_STATE_OFFSET, SWORD_STONE_HANDLER_FUNCTION_RVA,
    SWORD_STONE_STATE_VTABLE_RVA,
};

pub const MAX_ACTOR_CUES_PER_SECOND: u32 = 60;
pub const MAX_FIRST_ACTOR_CUES_LOGGED: usize = 512;
pub const STONE_HANDLERS: [StoneHandler; 3] = [
    StoneHandler::Leader,
    StoneHandler::Sword,
    StoneHandler::Grapple,
];

/// The hooked game call that made an effect or a sound.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActorCueCall {
    AttachById,
    RemoveAttached,
    CreateById,
    SoundById,
    LastingSoundById,
    StopSound,
    StoneHandler,
}

/// What the host knew when the game made an effect or a sound.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ActorCueMoment {
    pub hosting: bool,
    /// The actor the call serves (its target, or the actor whose tick runs) is shared.
    pub shared_actor: bool,
    pub in_ai_step: bool,
    /// Inside an attach or a create this hook already turns into a cue.
    pub nested: bool,
    /// The shot sync already draws a bullet's own effects.
    pub made_for_a_bullet: bool,
    /// The cube glow this attach made is already relayed as a world effect.
    pub made_for_a_cube_glow: bool,
}

/// What the guest knows of the copy a host cue names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CopyForCue {
    pub actor: u64,
    pub held: bool,
    pub live: bool,
}

/// Remembers which (actor, call, id) were logged, so each shows once.
#[derive(Debug, Default)]
pub struct FirstCues {
    seen: HashSet<(EnemyId, ActorCueCall, u32)>,
}

impl ActorCueCall {
    pub fn of_kind(kind: &ActorCueKind) -> Self {
        match kind {
            ActorCueKind::AttachEffect(_) => Self::AttachById,
            ActorCueKind::RemoveEffect { .. } => Self::RemoveAttached,
            ActorCueKind::PlaceEffect { .. } => Self::CreateById,
            ActorCueKind::Sound { .. } => Self::SoundById,
            ActorCueKind::LastingSound { .. } => Self::LastingSoundById,
            ActorCueKind::StopSound { .. } => Self::StopSound,
            ActorCueKind::TurnedToStone(_) => Self::StoneHandler,
        }
    }
}

impl FirstCues {
    pub fn is_first(&mut self, actor: EnemyId, call: ActorCueCall, id: u32) -> bool {
        self.seen.len() < MAX_FIRST_ACTOR_CUES_LOGGED && self.seen.insert((actor, call, id))
    }
}

pub fn actor_cue_is_sent(moment: ActorCueMoment) -> bool {
    moment.hosting
        && moment.shared_actor
        && moment.in_ai_step
        && !moment.nested
        && !moment.made_for_a_bullet
        && !moment.made_for_a_cube_glow
}

pub fn actor_cue_replay_target(copy: Option<CopyForCue>) -> Option<u64> {
    copy.filter(|copy| copy.held && copy.live)
        .map(|copy| copy.actor)
}

pub fn sound_caller_is_one_shot(caller_rva: u32) -> bool {
    SOUND_BY_ID_WRAPPER_BODY_RVAS.contains(&caller_rva)
}

/// What a by-id sound becomes: a one-shot sound when the game's one-shot wrapper played it,
/// else a lasting sound named by the handle the game keeps for it (none without a handle).
pub fn by_id_sound_cue(
    caller_rva: u32,
    position: [f32; 3],
    levels: [f32; SHOT_SOUND_LEVELS],
    handle: u64,
) -> Option<ActorCueKind> {
    if sound_caller_is_one_shot(caller_rva) {
        Some(ActorCueKind::Sound { position, levels })
    } else {
        (handle != 0).then_some(ActorCueKind::LastingSound {
            position,
            levels,
            handle,
        })
    }
}

pub fn state_machine_holds_stone(
    memory: &impl GameMemory,
    machine: u64,
    stone_vtable: u64,
) -> bool {
    [
        STONE_STATE_MACHINE_CURRENT_STATE_OFFSET,
        STATE_MACHINE_NEXT_STATE_OFFSET,
    ]
    .into_iter()
    .any(|slot| {
        memory
            .read_u64(machine + slot)
            .ok()
            .filter(|state| *state != 0)
            .and_then(|state| memory.read_u64(state).ok())
            == Some(stone_vtable)
    })
}

pub fn actor_holds_stone(memory: &impl GameMemory, actor: u64, handler: StoneHandler) -> bool {
    let stone_vtable = memory.module_base() + u64::from(stone_state_vtable_rva(handler));
    memory
        .read_u64(actor + stone_state_machine_pointer_offset(handler))
        .ok()
        .filter(|machine| *machine != 0)
        .is_some_and(|machine| state_machine_holds_stone(memory, machine, stone_vtable))
}

pub fn stack_holds_ai_step(return_rvas: &[u32]) -> bool {
    return_rvas
        .iter()
        .any(|rva| ACTOR_TICK_AI_STEP_RETURN_RVAS.contains(rva))
}

pub fn stone_handler_rva(handler: StoneHandler) -> u32 {
    match handler {
        StoneHandler::Leader => LEADER_STONE_HANDLER_FUNCTION_RVA,
        StoneHandler::Sword => SWORD_STONE_HANDLER_FUNCTION_RVA,
        StoneHandler::Grapple => GRAPPLE_STONE_HANDLER_FUNCTION_RVA,
    }
}

pub fn stone_state_vtable_rva(handler: StoneHandler) -> u32 {
    match handler {
        StoneHandler::Leader => LEADER_STONE_STATE_VTABLE_RVA,
        StoneHandler::Sword => SWORD_STONE_STATE_VTABLE_RVA,
        StoneHandler::Grapple => GRAPPLE_STONE_STATE_VTABLE_RVA,
    }
}

pub fn stone_state_machine_pointer_offset(handler: StoneHandler) -> u64 {
    match handler {
        StoneHandler::Leader | StoneHandler::Sword => LEADER_AND_SWORD_STATE_MACHINE_POINTER_OFFSET,
        StoneHandler::Grapple => GRAPPLE_STATE_MACHINE_POINTER_OFFSET,
    }
}

pub fn actor_cue_text(actor: &str, cue: &ActorCue) -> String {
    match cue.kind {
        ActorCueKind::AttachEffect(attached) => format!(
            "effect {} attached to {actor} (bone {}, unique {})",
            cue.id, attached.bone, attached.unique
        ),
        ActorCueKind::RemoveEffect { bone, all_matching } => format!(
            "effect {} removed from {actor} (bone {bone}, all {all_matching})",
            cue.id
        ),
        ActorCueKind::PlaceEffect { transform } => format!(
            "effect {} made by {actor} at [{:.1}, {:.1}, {:.1}]",
            cue.id, transform[3], transform[7], transform[11]
        ),
        ActorCueKind::Sound { position, .. } => format!(
            "sound {} played by {actor} at [{:.1}, {:.1}, {:.1}]",
            cue.id, position[0], position[1], position[2]
        ),
        ActorCueKind::LastingSound {
            position, handle, ..
        } => format!(
            "lasting sound {} played by {actor} at [{:.1}, {:.1}, {:.1}] (host handle {handle:#x})",
            cue.id, position[0], position[1], position[2]
        ),
        ActorCueKind::StopSound { handle } => format!(
            "lasting sound {} of {actor} stopped (host handle {handle:#x})",
            cue.id
        ),
        ActorCueKind::TurnedToStone(turn) => format!(
            "{actor} turned to stone by its {:?} handler (state bytes {:?})",
            turn.handler, turn.state_bytes
        ),
    }
}

/// A stone turn changes the enemy's state, so the enemy sync takes it; the effects take the rest.
pub fn actor_cue_goes_to_enemy_sync(kind: &ActorCueKind) -> bool {
    matches!(kind, ActorCueKind::TurnedToStone(_))
}
