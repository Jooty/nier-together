use replicant_game::move_log::{FeetReading, format_feet_second, read_feet};

use crate::fake_memory::FakeMemory;

const ACTOR: u64 = 0x2_0000_0000;
const MOTION_PLAYER: u64 = 0x3_0000_0000;
const LAYER: u64 = 0x3_1000_0000;
const SLOTS: u64 = 0x3_2000_0000;
const SLOT_STRIDE: u64 = 0x258;

fn memory_with_slot(
    current: i32,
    index: i32,
    frame: f32,
    frame_count: i32,
    speed: f32,
) -> FakeMemory {
    let mut memory = FakeMemory::new();
    memory.put(ACTOR, &vec![0_u8; 0x16400]);
    memory.put(ACTOR + 0x163C0 + 0x10, &MOTION_PLAYER.to_le_bytes());
    memory.put(MOTION_PLAYER + 0xA8, &LAYER.to_le_bytes());
    memory.put(LAYER + 0x8, &current.to_le_bytes());
    memory.put(LAYER + 0xC, &3_i32.to_le_bytes());
    memory.put(LAYER + 0x10, &SLOTS.to_le_bytes());
    let slot = SLOTS + SLOT_STRIDE * current.max(0) as u64;
    memory.put(slot, &index.to_le_bytes());
    memory.put(slot + 4, &frame.to_le_bytes());
    memory.put(slot + 0xC, &frame_count.to_le_bytes());
    memory.put(slot + 0x10, &speed.to_le_bytes());
    memory
}

#[test]
fn the_feet_are_the_current_slot_of_the_body_layer() {
    let memory = memory_with_slot(1, 420, 12.5, 171, 1.0);
    assert_eq!(
        read_feet(&memory, ACTOR),
        Some(FeetReading {
            index: 420,
            frame: 12.5,
            frame_count: 171,
            speed: 1.0
        })
    );
}

#[test]
fn a_free_slot_or_a_slot_past_the_count_has_no_feet() {
    assert_eq!(
        read_feet(&memory_with_slot(0, -1, 0.0, 0, 1.0), ACTOR),
        None
    );
    assert_eq!(
        read_feet(&memory_with_slot(7, 5, 0.0, 10, 1.0), ACTOR),
        None
    );
}

#[test]
fn the_feet_line_names_the_motion_its_frame_and_speed_and_the_step() {
    let reading = FeetReading {
        index: 420,
        frame: 12.5,
        frame_count: 171,
        speed: 1.0,
    };
    assert_eq!(
        format_feet_second(Some("KAINE015_walk_ed_1"), Some(reading), Some(2.54)),
        "indoor feet: her body plays KAINE015_walk_ed_1 (index 420) at frame 12.5 of 171, motion speed 1.00, step 2.54/s"
    );
    assert_eq!(
        format_feet_second(None, None, None),
        "indoor feet: her body plays no motion, step ?/s"
    );
}
