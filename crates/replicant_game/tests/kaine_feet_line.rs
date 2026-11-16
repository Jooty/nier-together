use replicant_game::move_log::{
    FeetDriveTally, FeetReading, FeetSecond, format_feet_second, read_feet, write_feet_speed,
};

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
fn the_speed_write_lands_in_the_current_slot_of_the_body_layer() {
    let mut memory = memory_with_slot(1, 226, 3.0, 39, 1.0);
    assert!(write_feet_speed(&mut memory, ACTOR, 0.5));
    assert_eq!(read_feet(&memory, ACTOR).map(|feet| feet.speed), Some(0.5));
    let mut no_slot = memory_with_slot(0, -1, 0.0, 0, 1.0);
    assert!(!write_feet_speed(&mut no_slot, ACTOR, 0.5));
}

#[test]
fn the_feet_line_names_the_motion_the_drive_and_the_last_miss() {
    let reading = FeetReading {
        index: 420,
        frame: 12.5,
        frame_count: 171,
        speed: 1.0,
    };
    assert_eq!(
        format_feet_second(&FeetSecond {
            name: Some("KAINE015_walk_ed_1"),
            reading: Some(reading),
            step_per_second: Some(2.54),
            drive: FeetDriveTally {
                plays: 1,
                speed_holds: 3
            },
            last_missed: Some("NIER025_jog_lp"),
        }),
        "indoor feet: her body plays KAINE015_walk_ed_1 (index 420) at frame 12.5 of 171, motion speed 1.00, step 2.54/s; \
         the drive played KAINE015_run_lp 1 times and held its speed 3 times; \
         the game last asked her body for NIER025_jog_lp and missed"
    );
    assert_eq!(
        format_feet_second(&FeetSecond {
            name: None,
            reading: None,
            step_per_second: None,
            drive: FeetDriveTally::default(),
            last_missed: None,
        }),
        "indoor feet: her body plays no motion, step ?/s; the drive did nothing; \
         no motion the game asked her body for was missing"
    );
}
