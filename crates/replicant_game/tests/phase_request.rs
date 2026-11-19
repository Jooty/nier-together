use replicant_game::Replicant;
use replicant_game::phase_request::{
    LoadedPartSetsWatch, NEXT_PHASE_NAME_RVA, RADIO_MANAGER_RVA,
    TALK_WINDOW_LOADED_SET_NAME_OFFSET, TALK_WINDOW_RVA, loaded_talk_set_letter,
    part_sets_registered_by_request,
};

use crate::fake_memory::{FAKE_MODULE_BASE, FakeMemory};

const PART_ONE_TALK_SET: &str = "talk_A";
const PART_TWO_TALK_SET: &str = "talk_B";
const PART_TWO_HUB_PHASE: &str = "B_CENTER_VILLAGE_01";
const PART_TWO_LIBRARY_PHASE: &str = "B_CENTER_LIBRARY_01";

fn game_with(loaded_talk_set: &str, next_phase_name: &str) -> Replicant<FakeMemory> {
    let mut memory = FakeMemory::new();
    let mut set_name = [0_u8; 16];
    set_name[..loaded_talk_set.len()].copy_from_slice(loaded_talk_set.as_bytes());
    memory.put_at_rva(
        TALK_WINDOW_RVA + TALK_WINDOW_LOADED_SET_NAME_OFFSET as u32,
        &set_name,
    );
    let mut buffer = [0_u8; 0x20];
    buffer[..next_phase_name.len()].copy_from_slice(next_phase_name.as_bytes());
    memory.put_at_rva(NEXT_PHASE_NAME_RVA, &buffer);
    Replicant::new(memory)
}

fn old_name_the_game_compares(game: &Replicant<FakeMemory>) -> String {
    let bytes = game
        .memory()
        .get(FAKE_MODULE_BASE + u64::from(NEXT_PHASE_NAME_RVA), 0x20);
    String::from_utf8_lossy(&bytes)
        .trim_end_matches('\0')
        .to_owned()
}

#[test]
fn a_next_phase_name_set_by_the_guests_own_script_hides_the_part_change() {
    let game = game_with(PART_ONE_TALK_SET, PART_TWO_HUB_PHASE);
    assert_eq!(
        part_sets_registered_by_request(&old_name_the_game_compares(&game), PART_TWO_HUB_PHASE),
        None
    );
}

#[test]
fn the_phase_request_compares_against_the_part_whose_talk_set_is_loaded() {
    let mut game = game_with(PART_ONE_TALK_SET, PART_TWO_HUB_PHASE);
    game.put_loaded_talk_set_letter_in_next_phase_name()
        .unwrap();
    let registered =
        part_sets_registered_by_request(&old_name_the_game_compares(&game), PART_TWO_HUB_PHASE)
            .expect("the A to B change registers the part sets");
    assert_eq!(registered.talk_set, PART_TWO_TALK_SET);
    assert_eq!(registered.radio_set, "radio_B");
}

#[test]
fn a_request_inside_the_loaded_part_registers_nothing() {
    let mut game = game_with(PART_TWO_TALK_SET, PART_TWO_HUB_PHASE);
    game.put_loaded_talk_set_letter_in_next_phase_name()
        .unwrap();
    assert_eq!(
        part_sets_registered_by_request(&old_name_the_game_compares(&game), PART_TWO_LIBRARY_PHASE),
        None
    );
}

#[test]
fn an_empty_talk_window_name_leaves_the_next_phase_name_alone() {
    let mut game = game_with("", PART_TWO_HUB_PHASE);
    game.put_loaded_talk_set_letter_in_next_phase_name()
        .unwrap();
    assert_eq!(old_name_the_game_compares(&game), PART_TWO_HUB_PHASE);
}

#[test]
fn only_a_single_letter_talk_set_names_a_part() {
    assert_eq!(loaded_talk_set_letter("talk_B"), Some(b'B'));
    assert_eq!(loaded_talk_set_letter("talk_EX"), None);
    assert_eq!(loaded_talk_set_letter("talk_common"), None);
    assert_eq!(loaded_talk_set_letter("radio_B"), None);
}

#[test]
fn extra_content_phases_never_register_part_sets() {
    assert_eq!(
        part_sets_registered_by_request("A_X", "X_DLC_START_MAP"),
        None
    );
}

#[test]
fn the_third_and_fourth_parts_share_the_first_two_parts_radio_sets() {
    let third = part_sets_registered_by_request("B_X", "C_X").unwrap();
    assert_eq!(third.talk_set, "talk_C");
    assert_eq!(third.radio_set, "radio_A");
    let fourth = part_sets_registered_by_request("B_X", "D_X").unwrap();
    assert_eq!(fourth.radio_set, "radio_B");
}

#[test]
fn the_loaded_sets_line_is_logged_once_per_change() {
    let mut memory = FakeMemory::new();
    for (rva, text) in [
        (
            TALK_WINDOW_RVA + TALK_WINDOW_LOADED_SET_NAME_OFFSET as u32,
            "talk_B",
        ),
        (RADIO_MANAGER_RVA, "radio_B"),
        (NEXT_PHASE_NAME_RVA, PART_TWO_HUB_PHASE),
    ] {
        let mut buffer = [0_u8; 0x20];
        buffer[..text.len()].copy_from_slice(text.as_bytes());
        memory.put_at_rva(rva, &buffer);
    }
    let sets = Replicant::new(memory).loaded_part_sets().unwrap();
    let mut watch = LoadedPartSetsWatch::default();
    assert_eq!(
        watch.observe(&sets).as_deref(),
        Some("part sets: talk window holds talk_B, radio holds radio_B, phase B_CENTER_VILLAGE_01")
    );
    assert_eq!(watch.observe(&sets), None);
}
