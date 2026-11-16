use crate::memory::GameMemory;
use crate::move_log::{FeetDriveTally, RUN_LOOP_MOTION_NAME};
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

/// One second of her feet indoors: the body motion, what the drive did, and what the game
/// last asked her body for and missed.
#[derive(Debug, Clone, PartialEq)]
pub struct FeetSecond<'a> {
    pub name: Option<&'a str>,
    pub reading: Option<FeetReading>,
    pub step_per_second: Option<f32>,
    pub drive: FeetDriveTally,
    pub last_missed: Option<&'a str>,
}

pub fn read_feet<M: GameMemory>(memory: &M, actor: u64) -> Option<FeetReading> {
    let slot = current_body_slot(memory, actor)?;
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

/// Writes the playback speed of the body layer's current slot: the game copies it there from the
/// speed a play was given, and reads it from there each frame.
pub fn write_feet_speed<M: GameMemory>(memory: &mut M, actor: u64, speed: f32) -> bool {
    let Some(slot) = current_body_slot(memory, actor) else {
        return false;
    };
    let playing = memory
        .read_i32(slot + MOTION_SLOT_MOTION_INDEX_OFFSET)
        .is_ok_and(|index| index >= 0);
    playing
        && memory
            .write_f32(slot + MOTION_SLOT_SPEED_OFFSET, speed)
            .is_ok()
}

pub fn format_feet_second(second: &FeetSecond<'_>) -> String {
    let step = second
        .step_per_second
        .map_or_else(|| "?".to_owned(), |step| format!("{step:.2}"));
    let body = match second.reading {
        None => "her body plays no motion".to_owned(),
        Some(reading) => format!(
            "her body plays {} (index {}) at frame {:.1} of {}, motion speed {:.2}",
            second.name.unwrap_or("?"),
            reading.index,
            reading.frame,
            reading.frame_count,
            reading.speed,
        ),
    };
    let drive = match (second.drive.plays, second.drive.speed_holds) {
        (0, 0) => "the drive did nothing".to_owned(),
        (plays, holds) => format!(
            "the drive played {RUN_LOOP_MOTION_NAME} {plays} times and held its speed {holds} times"
        ),
    };
    let missed = second.last_missed.map_or_else(
        || "no motion the game asked her body for was missing".to_owned(),
        |name| format!("the game last asked her body for {name} and missed"),
    );
    format!("indoor feet: {body}, step {step}/s; {drive}; {missed}")
}

fn current_body_slot<M: GameMemory>(memory: &M, actor: u64) -> Option<u64> {
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
    Some(slots + current as u64 * MOTION_SLOT_STRIDE)
}
