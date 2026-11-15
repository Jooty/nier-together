use crate::memory::GameMemory;
use crate::offsets::{
    MOTION_LAYER_CURRENT_SLOT_INDEX_OFFSET, MOTION_LAYER_MAX_SLOT_COUNT,
    MOTION_LAYER_SLOT_ARRAY_POINTER_OFFSET, MOTION_LAYER_SLOT_COUNT_OFFSET,
    MOTION_PLAYER_BODY_LAYER_POINTER_OFFSET, MOTION_SLOT_FRAME_COUNT_OFFSET,
    MOTION_SLOT_FRAME_OFFSET, MOTION_SLOT_MOTION_INDEX_OFFSET, MOTION_SLOT_SPEED_OFFSET,
    MOTION_SLOT_STRIDE, MOTION_WORK_MOTION_PLAYER_POINTER_OFFSET,
    PLAYABLE_ACTOR_MOTION_WORK_OFFSET,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FeetReading {
    pub index: i32,
    pub frame: f32,
    pub frame_count: i32,
    pub speed: f32,
}

pub fn read_feet<M: GameMemory>(memory: &M, actor: u64) -> Option<FeetReading> {
    let motion_work = actor + PLAYABLE_ACTOR_MOTION_WORK_OFFSET;
    let player = memory
        .read_u64(motion_work + MOTION_WORK_MOTION_PLAYER_POINTER_OFFSET)
        .ok()?;
    let layer = memory
        .read_u64(player + MOTION_PLAYER_BODY_LAYER_POINTER_OFFSET)
        .ok()
        .filter(|layer| player != 0 && *layer != 0)?;
    let current = memory
        .read_i32(layer + MOTION_LAYER_CURRENT_SLOT_INDEX_OFFSET)
        .ok()?;
    let count = memory
        .read_i32(layer + MOTION_LAYER_SLOT_COUNT_OFFSET)
        .ok()?;
    let slots = memory
        .read_u64(layer + MOTION_LAYER_SLOT_ARRAY_POINTER_OFFSET)
        .ok()
        .filter(|slots| *slots != 0)?;
    if !(0..count.min(MOTION_LAYER_MAX_SLOT_COUNT)).contains(&current) {
        return None;
    }
    let slot = slots + current as u64 * MOTION_SLOT_STRIDE;
    let index = memory
        .read_i32(slot + MOTION_SLOT_MOTION_INDEX_OFFSET)
        .ok()
        .filter(|index| *index >= 0)?;
    Some(FeetReading {
        index,
        frame: memory.read_f32(slot + MOTION_SLOT_FRAME_OFFSET).ok()?,
        frame_count: memory
            .read_i32(slot + MOTION_SLOT_FRAME_COUNT_OFFSET)
            .ok()?,
        speed: memory.read_f32(slot + MOTION_SLOT_SPEED_OFFSET).ok()?,
    })
}

pub fn format_feet_second(
    name: Option<&str>,
    reading: Option<FeetReading>,
    step_per_second: Option<f32>,
) -> String {
    let step = step_per_second.map_or_else(|| "?".to_owned(), |step| format!("{step:.2}"));
    let Some(reading) = reading else {
        return format!("indoor feet: her body plays no motion, step {step}/s");
    };
    format!(
        "indoor feet: her body plays {} (index {}) at frame {:.1} of {}, motion speed {:.2}, step {step}/s",
        name.unwrap_or("?"),
        reading.index,
        reading.frame,
        reading.frame_count,
        reading.speed,
    )
}
