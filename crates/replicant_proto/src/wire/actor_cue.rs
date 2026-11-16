use serde::{Deserialize, Serialize};

use crate::wire::{
    BULLET_TRANSFORM_FLOATS, EnemyId, MAX_BULLET_AXIS_ABS, MAX_BULLET_STYLE_ABS, MAX_POSITION_ABS,
    MAX_SHOT_SOUND_LEVEL_ABS, SHOT_SOUND_LEVELS, WireError,
};

pub const MAX_ACTOR_CUE_ID: u32 = 0x00FF_FFFF;
pub const ATTACHED_EFFECT_COLOUR_FLOATS: usize = 4;
pub const ATTACHED_EFFECT_STYLE_FLOATS: usize = 4;

/// What a shared actor's own AI or state step made on the host, for the guest's copy of that
/// actor (which never runs those steps) to make too. `id` is the game's effect or sound id.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ActorCue {
    pub actor: EnemyId,
    pub id: u32,
    pub kind: ActorCueKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum ActorCueKind {
    AttachEffect(AttachedEffect),
    /// The actor's attached effects of this id and bone were removed; a bone of -99 or less
    /// means every bone, and `all_matching` is the call's last argument.
    RemoveEffect {
        bone: i32,
        all_matching: bool,
    },
    /// An effect made by id at a world transform (3x4 rows, translation in the last column).
    PlaceEffect {
        transform: [f32; BULLET_TRANSFORM_FLOATS],
    },
    Sound {
        position: [f32; 3],
        levels: [f32; SHOT_SOUND_LEVELS],
    },
    /// The actor's message handler turned it to stone; `id` is unused (0).
    TurnedToStone(StoneTurn),
    /// A sound the game keeps playing until it stops it; `handle` is the host's own handle for
    /// it, which names it in the later stop.
    LastingSound {
        position: [f32; 3],
        levels: [f32; SHOT_SOUND_LEVELS],
        handle: u64,
    },
    /// The host's game stopped the lasting sound it knows by `handle`.
    StopSound {
        handle: u64,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct AttachedEffect {
    pub bone: i16,
    pub unique: bool,
    pub colour: [f32; ATTACHED_EFFECT_COLOUR_FLOATS],
    pub style: [f32; ATTACHED_EFFECT_STYLE_FLOATS],
    pub mode: i32,
    pub flags: u8,
    pub extra: u32,
}

/// Which of the game's stone handlers ran, and the two state bytes it reads, as the host had them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoneTurn {
    pub handler: StoneHandler,
    pub state_bytes: [u8; 2],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StoneHandler {
    Leader,
    Sword,
    Grapple,
}

impl ActorCue {
    pub fn validate(&self) -> Result<(), WireError> {
        if self.id > MAX_ACTOR_CUE_ID {
            return Err(WireError::OutOfRange {
                field: "actor cue id",
            });
        }
        let in_range = match self.kind {
            ActorCueKind::AttachEffect(attached) => attached
                .colour
                .iter()
                .chain(&attached.style)
                .all(|value| value.is_finite() && value.abs() <= MAX_BULLET_STYLE_ABS),
            ActorCueKind::RemoveEffect { .. }
            | ActorCueKind::TurnedToStone(_)
            | ActorCueKind::StopSound { .. } => true,
            ActorCueKind::PlaceEffect { transform } => {
                transform.iter().enumerate().all(|(index, value)| {
                    let limit = if index % 4 == 3 {
                        MAX_POSITION_ABS
                    } else {
                        MAX_BULLET_AXIS_ABS
                    };
                    value.is_finite() && value.abs() <= limit
                })
            }
            ActorCueKind::Sound { position, levels }
            | ActorCueKind::LastingSound {
                position, levels, ..
            } => {
                position
                    .iter()
                    .all(|axis| axis.is_finite() && axis.abs() <= MAX_POSITION_ABS)
                    && levels
                        .iter()
                        .all(|level| level.is_finite() && level.abs() <= MAX_SHOT_SOUND_LEVEL_ABS)
            }
        };
        if in_range {
            Ok(())
        } else {
            Err(WireError::OutOfRange { field: "actor cue" })
        }
    }
}
