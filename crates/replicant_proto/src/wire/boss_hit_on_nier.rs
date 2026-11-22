use serde::{Deserialize, Serialize};

use crate::wire::{MAX_HIT_PUSH_STRENGTH, MAX_POSITION_ABS, WireError};

/// The guest, playing the boss, connected with the host's Nier. The host applies it at the AI
/// boss's strength for her motion; `hit_id` repeats while one swing keeps overlapping.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct BossHitOnNier {
    pub phase: u16,
    pub hit_id: u32,
    pub motion_name_hash: u32,
    pub attack_kind: u32,
    pub flags: u32,
    pub push: f32,
    pub direction: [f32; 3],
}

impl BossHitOnNier {
    pub fn validate(&self) -> Result<(), WireError> {
        if !(0.0..=MAX_HIT_PUSH_STRENGTH).contains(&self.push) {
            return Err(WireError::OutOfRange { field: "hit push" });
        }
        if self
            .direction
            .iter()
            .any(|axis| !axis.is_finite() || axis.abs() > MAX_POSITION_ABS)
        {
            return Err(WireError::OutOfRange {
                field: "hit direction",
            });
        }
        Ok(())
    }
}
