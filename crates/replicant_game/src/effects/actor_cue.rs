use std::collections::HashSet;

use replicant_proto::{ActorCue, ActorCueKind, EnemyId, StoneHandler};

use crate::offsets::{
    ACTOR_TICK_AI_STEP_RETURN_RVAS, GRAPPLE_STATE_MACHINE_POINTER_OFFSET,
    GRAPPLE_STONE_HANDLER_FUNCTION_RVA, GRAPPLE_STONE_STATE_VTABLE_RVA,
    LEADER_AND_SWORD_STATE_MACHINE_POINTER_OFFSET, LEADER_STONE_HANDLER_FUNCTION_RVA,
    LEADER_STONE_STATE_VTABLE_RVA, SWORD_STONE_HANDLER_FUNCTION_RVA, SWORD_STONE_STATE_VTABLE_RVA,
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
    SpawnByName,
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
        ActorCueKind::TurnedToStone(turn) => format!(
            "{actor} turned to stone by its {:?} handler (state bytes {:?})",
            turn.handler, turn.state_bytes
        ),
    }
}
