use replicant_game::move_log::{
    FeetDriveTally, FeetReading, FeetSecond, body_speed_rate_address, format_feet_second,
    read_body_speed_rate, read_feet, write_body_speed_rate,
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
fn the_body_speed_rate_the_drive_owns_is_rate_one_channel_zero_of_the_table() {
    // 0x66DAE0 writes layer 0's rate into id 1; 0x6745A0 gives layer 0 the product of id 1
    // (0x670A40); channel 0 of id 1 shows in no rates= dump of any run, so nothing else writes it
    assert_eq!(body_speed_rate_address(ACTOR), ACTOR + 0x6_01E0 + 0x60);
    let mut memory = memory_with_slot(1, 226, 3.0, 39, 1.0);
    memory.put(body_speed_rate_address(ACTOR), &1.0_f32.to_le_bytes());
    assert!(write_body_speed_rate(&mut memory, ACTOR, 0.48));
    assert_eq!(read_body_speed_rate(&memory, ACTOR), Some(0.48));
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
            body_speed_rate: Some(0.48),
            pace_per_second: Some(2.87),
            drive: FeetDriveTally {
                plays: 1,
                rate_writes: 3
            },
            last_missed: Some("NIER025_jog_lp"),
        }),
        "indoor feet: her body plays KAINE015_walk_ed_1 (index 420) at frame 12.5 of 171, motion speed 1.00, step 2.54/s; \
         her body speed rate 1:0 0.48 (her run loop's pace 2.87/s against the first body's 2.90/s), 5:2 left as the game holds it; \
         the drive played KAINE015_run_lp 1 times and wrote the rate 3 times; \
         the game last asked her body for NIER025_jog_lp and missed"
    );
    assert_eq!(
        format_feet_second(&FeetSecond {
            name: None,
            reading: None,
            step_per_second: None,
            body_speed_rate: None,
            pace_per_second: None,
            drive: FeetDriveTally::default(),
            last_missed: None,
        }),
        "indoor feet: her body plays no motion, step ?/s; \
         her body speed rate 1:0 ? (her run loop's pace ?/s against the first body's 2.90/s), 5:2 left as the game holds it; \
         the drive did nothing; no motion the game asked her body for was missing"
    );
}
