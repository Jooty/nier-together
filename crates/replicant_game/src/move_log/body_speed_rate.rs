use crate::memory::GameMemory;
use crate::move_log::{
    BODY_LAYER_MOTION_SPEED_RATE_ID, BODY_SPEED_RATE_CHANNEL_THE_FEET_DRIVE_OWNS,
    RATE_TABLE_CHANNEL_STRIDE, RATE_TABLE_ID_STRIDE, RATE_TABLE_OFFSET,
};

/// The rate-table entry the feet drive owns: the game plays her body layer at the product of
/// every entry of rate 1 (0x670A40), so this one scales whatever speed the game itself asks for.
pub fn body_speed_rate_address(actor: u64) -> u64 {
    actor
        + RATE_TABLE_OFFSET
        + BODY_LAYER_MOTION_SPEED_RATE_ID as u64 * RATE_TABLE_ID_STRIDE as u64
        + BODY_SPEED_RATE_CHANNEL_THE_FEET_DRIVE_OWNS as u64 * RATE_TABLE_CHANNEL_STRIDE as u64
}

pub fn read_body_speed_rate<M: GameMemory>(memory: &M, actor: u64) -> Option<f32> {
    memory.read_f32(body_speed_rate_address(actor)).ok()
}

/// Writes the entry; the game reads it only when its rates are applied (0x6745A0), so the caller
/// applies them right after.
pub fn write_body_speed_rate<M: GameMemory>(memory: &mut M, actor: u64, rate: f32) -> bool {
    memory
        .write_f32(body_speed_rate_address(actor), rate)
        .is_ok()
}
