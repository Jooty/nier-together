use crate::memory::GameMemory;
use crate::move_log::{
    RATE_TABLE_CHANNEL_STRIDE, RATE_TABLE_ID_STRIDE, RATE_TABLE_OFFSET, SPEED_CAP_HOUSE_VALUE_MIN,
    SPEED_CAP_RATE_CHANNEL, SPEED_CAP_RATE_ID, SPEED_CAP_THRESHOLD,
};

/// Rate 5:2 at the value the game holds it at indoors (0.01: her ground speed is then 0.06 with the
/// stick held, and only a motion's own root travel moves the body, as it moves the first body).
pub fn is_house_speed_cap_value(value: f32) -> bool {
    (SPEED_CAP_HOUSE_VALUE_MIN..SPEED_CAP_THRESHOLD).contains(&value)
}

pub fn speed_cap_address(actor: u64) -> u64 {
    actor
        + RATE_TABLE_OFFSET
        + SPEED_CAP_RATE_ID as u64 * RATE_TABLE_ID_STRIDE as u64
        + SPEED_CAP_RATE_CHANNEL as u64 * RATE_TABLE_CHANNEL_STRIDE as u64
}

pub fn read_speed_cap<M: GameMemory>(memory: &M, actor: u64) -> Option<f32> {
    memory.read_f32(speed_cap_address(actor)).ok()
}
