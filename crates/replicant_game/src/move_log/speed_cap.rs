use crate::memory::GameMemory;
use crate::move_log::{
    RATE_TABLE_CHANNEL_STRIDE, RATE_TABLE_ID_STRIDE, RATE_TABLE_OFFSET, SPEED_CAP_HOUSE_VALUE_MIN,
    SPEED_CAP_LIFTED_VALUE, SPEED_CAP_RATE_CHANNEL, SPEED_CAP_RATE_ID, SPEED_CAP_THRESHOLD,
    STATE_ACTION_MOVE_RVA,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpeedCapLog {
    Lifted { from: f32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct SpeedCapStep {
    pub write: bool,
    pub log: Option<SpeedCapLog>,
}

/// One tick's view of the swapped body's rate 5:2 and of what may hold it down.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpeedCapReading {
    pub phase: u32,
    pub now_ms: u64,
    pub value: f32,
    pub kaine_swapped: bool,
    pub event_scene: bool,
    pub talk: bool,
    pub control_masked: bool,
    pub state_rva: Option<u32>,
}

/// Lifts rate 5:2 on every tick the game holds it at the house value in the plain move state; the
/// game writes the house value again each tick, so a lift that waited would leave her slow.
#[derive(Debug, Default)]
pub struct SpeedCapLift {
    phase: u32,
    lifts: u32,
}

impl SpeedCapLift {
    pub fn observe(&mut self, reading: SpeedCapReading) -> SpeedCapStep {
        if reading.phase != self.phase {
            *self = Self {
                phase: reading.phase,
                lifts: 0,
            };
        }
        if !should_lift_speed_cap(&reading) {
            return SpeedCapStep::default();
        }
        let log = (self.lifts == 0).then_some(SpeedCapLog::Lifted {
            from: reading.value,
        });
        self.lifts += 1;
        SpeedCapStep { write: true, log }
    }

    pub fn lifts_this_phase(&self) -> u32 {
        self.lifts
    }
}

pub fn is_house_speed_cap_value(value: f32) -> bool {
    (SPEED_CAP_HOUSE_VALUE_MIN..SPEED_CAP_THRESHOLD).contains(&value)
}

pub fn should_lift_speed_cap(reading: &SpeedCapReading) -> bool {
    reading.kaine_swapped
        && !reading.event_scene
        && !reading.talk
        && !reading.control_masked
        && reading.state_rva == Some(STATE_ACTION_MOVE_RVA)
        && is_house_speed_cap_value(reading.value)
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

pub fn write_lifted_speed_cap<M: GameMemory>(memory: &mut M, actor: u64) -> bool {
    memory
        .write_f32(speed_cap_address(actor), SPEED_CAP_LIFTED_VALUE)
        .is_ok()
}

pub fn format_speed_cap_log(log: SpeedCapLog, phase: u32) -> String {
    match log {
        SpeedCapLog::Lifted { from } => format!(
            "kaine walk: lifting the house speed cap {SPEED_CAP_RATE_ID}:{SPEED_CAP_RATE_CHANNEL} from {from:.2} to {SPEED_CAP_LIFTED_VALUE:.2} on every tick the game holds it there in the move state (phase {phase})"
        ),
    }
}
