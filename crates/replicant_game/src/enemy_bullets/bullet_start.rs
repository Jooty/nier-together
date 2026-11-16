use replicant_proto::{BULLET_EFFECT_SLOTS, ShotCueKind};

use crate::memory::GameMemory;
use crate::offsets::{
    EFFECT_INVALID_HANDLE, NORMAL_BULLET_ATTACK_SETTINGS_POINTER_OFFSET,
    NORMAL_BULLET_EFFECT_HANDLES_OFFSET, NORMAL_BULLET_SETTINGS_LASTING_SOUND_ID_OFFSET,
};

/// What the game made while an ordinary bullet's create-effects step ran.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BulletStartMade {
    Effect { handle: u32 },
    Sound { id: u32 },
}

/// How a shot cue carries what an ordinary bullet made as it started. Its slot effects are drawn
/// by the flights (`None`); its start effect is made once where it starts, which no slot keeps.
pub fn bullet_start_cue_kind<M: GameMemory>(
    memory: &M,
    bullet: u64,
    made: BulletStartMade,
) -> Option<ShotCueKind> {
    match made {
        BulletStartMade::Effect { handle } => {
            if handle == EFFECT_INVALID_HANDLE {
                return None;
            }
            let kept_in_a_slot = (0..BULLET_EFFECT_SLOTS as u64).any(|slot| {
                memory
                    .read_i32(bullet + NORMAL_BULLET_EFFECT_HANDLES_OFFSET + slot * 4)
                    .is_ok_and(|kept| kept.cast_unsigned() == handle)
            });
            (!kept_in_a_slot).then_some(ShotCueKind::OneShotEffect)
        }
        BulletStartMade::Sound { id } => {
            let lasting_sound_id = memory
                .read_u64(bullet + NORMAL_BULLET_ATTACK_SETTINGS_POINTER_OFFSET)
                .ok()
                .filter(|settings| *settings != 0)
                .and_then(|settings| {
                    memory
                        .read_i32(settings + NORMAL_BULLET_SETTINGS_LASTING_SOUND_ID_OFFSET)
                        .ok()
                });
            if id != 0 && lasting_sound_id == Some(id.cast_signed()) {
                Some(ShotCueKind::LastingSound)
            } else {
                Some(ShotCueKind::OneShotSound)
            }
        }
    }
}
