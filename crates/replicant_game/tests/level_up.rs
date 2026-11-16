use replicant_game::Replicant;
use replicant_game::level_up::{LAST_LEVEL_THE_GAME_ALLOWS, LevelUpPlan, level_up_plan};
use replicant_game::offsets::PLAYER_PARAM_ARRAY_RVA;

use crate::fake_memory::{FAKE_MODULE_BASE, FakeMemory};

const TABLE_ADDRESS: u64 = 0x2_0000_0000;
const ROWS_ADDRESS: u64 = 0x2_0000_1000;
const ROW_SIZE: u64 = 0x74;
const EXPERIENCE_FIELD: u64 = 0x6C;

fn memory_with_a_table(experience_to_leave: &[i32]) -> FakeMemory {
    let mut memory = FakeMemory::new();
    let param = FAKE_MODULE_BASE + u64::from(PLAYER_PARAM_ARRAY_RVA);
    memory.put(param + 0x58, &TABLE_ADDRESS.to_le_bytes());
    let relative = i32::try_from(ROWS_ADDRESS - (TABLE_ADDRESS + 4)).unwrap();
    memory.put(TABLE_ADDRESS + 4, &relative.to_le_bytes());
    for (level, needed) in (0_u64..).zip(experience_to_leave) {
        memory.put(
            ROWS_ADDRESS + level * ROW_SIZE + EXPERIENCE_FIELD,
            &needed.to_le_bytes(),
        );
    }
    memory
}

#[test]
fn a_press_adds_exactly_the_experience_missing_for_the_next_level() {
    assert_eq!(
        level_up_plan(1, 30, Some(100)),
        LevelUpPlan::Gain { experience: 70 }
    );
}

#[test]
fn a_press_with_the_experience_already_there_adds_nothing_and_lets_the_game_redo_the_level() {
    assert_eq!(
        level_up_plan(1, 120, Some(100)),
        LevelUpPlan::Gain { experience: 0 }
    );
}

#[test]
fn a_press_stops_at_the_last_level_the_game_allows() {
    assert_eq!(
        level_up_plan(LAST_LEVEL_THE_GAME_ALLOWS, 0, Some(100)),
        LevelUpPlan::AtTheLevelCap
    );
    assert_eq!(
        level_up_plan(LAST_LEVEL_THE_GAME_ALLOWS - 1, 0, Some(100)),
        LevelUpPlan::Gain { experience: 100 }
    );
}

#[test]
fn a_press_without_a_level_table_does_nothing() {
    assert_eq!(level_up_plan(3, 0, None), LevelUpPlan::NoLevelTable);
}

#[test]
fn the_table_is_read_the_way_the_games_level_up_redo_reads_it() {
    let game = Replicant::new(memory_with_a_table(&[0, 10, 25, 60]));
    assert_eq!(game.experience_to_leave_level(0), Ok(Some(0)));
    assert_eq!(game.experience_to_leave_level(2), Ok(Some(25)));
    assert_eq!(game.experience_to_leave_level(3), Ok(Some(60)));
}

#[test]
fn a_null_table_pointer_reads_as_no_table() {
    let mut memory = FakeMemory::new();
    let param = FAKE_MODULE_BASE + u64::from(PLAYER_PARAM_ARRAY_RVA);
    memory.put(param + 0x58, &0_u64.to_le_bytes());
    assert_eq!(
        Replicant::new(memory).experience_to_leave_level(1),
        Ok(None)
    );
}

#[test]
fn a_table_with_no_rows_offset_reads_as_no_table() {
    let mut memory = memory_with_a_table(&[]);
    memory.put(TABLE_ADDRESS + 4, &0_i32.to_le_bytes());
    assert_eq!(
        Replicant::new(memory).experience_to_leave_level(1),
        Ok(None)
    );
}
