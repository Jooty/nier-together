use automata_game::outfit_look::{LOOK_UNPICKED, LookKind, LookPicks, PlayerLook, SavedLook};

const TWO_B: usize = 0;
const A2: usize = 2;
const SAVED_ACCESSORY: i32 = 5;
const SAVED_HAIR: i32 = 7;

#[test]
fn a_pick_is_remembered_per_character() {
    let mut picks = LookPicks::default();
    picks.pick(TWO_B, LookKind::Outfit, Some(2), 0);
    picks.pick(A2, LookKind::Outfit, Some(1), 0);
    assert_eq!(picks.picked(TWO_B, LookKind::Outfit), Some(2));
    assert_eq!(picks.picked(A2, LookKind::Outfit), Some(1));
    assert_eq!(picks.picked(1, LookKind::Outfit), None);
}

#[test]
fn the_first_pick_of_a_kind_keeps_the_value_the_save_had() {
    let mut picks = LookPicks::default();
    picks.pick(TWO_B, LookKind::Accessory, Some(9), SAVED_ACCESSORY);
    picks.pick(TWO_B, LookKind::Accessory, Some(10), 9);
    assert_eq!(
        picks.saved_looks(),
        vec![SavedLook {
            slot: TWO_B,
            kind: LookKind::Accessory,
            value: SAVED_ACCESSORY
        }]
    );
}

#[test]
fn a_pick_survives_a_load_by_being_written_again() {
    let mut picks = LookPicks::default();
    picks.pick(TWO_B, LookKind::Hair, Some(3), SAVED_HAIR);
    picks.pick(A2, LookKind::PodSkin, Some(4), -1);
    let writes = picks.globals_after_load(true);
    assert_eq!(
        writes,
        vec![(TWO_B, LookKind::Hair, 3), (A2, LookKind::PodSkin, 4)]
    );
}

#[test]
fn a_rule_that_is_off_writes_nothing_after_a_load() {
    let mut picks = LookPicks::default();
    picks.pick(TWO_B, LookKind::Hair, Some(3), SAVED_HAIR);
    assert!(picks.globals_after_load(false).is_empty());
}

#[test]
fn a_rule_that_is_off_hands_back_the_saved_look_once() {
    let mut picks = LookPicks::default();
    picks.pick(TWO_B, LookKind::Hair, Some(3), SAVED_HAIR);
    assert_eq!(
        picks.take_targets(TWO_B, true)[LookKind::Hair.index()],
        Some(3)
    );
    assert_eq!(
        picks.take_targets(TWO_B, false)[LookKind::Hair.index()],
        Some(SAVED_HAIR)
    );
    assert_eq!(
        picks.take_targets(TWO_B, false)[LookKind::Hair.index()],
        None
    );
}

#[test]
fn the_pick_comes_back_when_the_rule_turns_on_again() {
    let mut picks = LookPicks::default();
    picks.pick(TWO_B, LookKind::Hair, Some(3), SAVED_HAIR);
    let _ = picks.take_targets(TWO_B, false);
    assert_eq!(
        picks.take_targets(TWO_B, true)[LookKind::Hair.index()],
        Some(3)
    );
    assert!(!picks.is_saved(TWO_B, LookKind::Hair));
}

#[test]
fn choosing_the_game_default_restores_what_the_save_had() {
    let mut picks = LookPicks::default();
    picks.pick(TWO_B, LookKind::Accessory, Some(9), SAVED_ACCESSORY);
    picks.pick(TWO_B, LookKind::Accessory, None, 9);
    assert_eq!(
        picks.take_targets(TWO_B, true)[LookKind::Accessory.index()],
        Some(SAVED_ACCESSORY)
    );
    assert!(!picks.is_saved(TWO_B, LookKind::Accessory));
}

#[test]
fn the_look_sent_holds_only_what_is_picked_and_allowed() {
    let mut picks = LookPicks::default();
    picks.pick(TWO_B, LookKind::Outfit, Some(2), 0);
    picks.pick(TWO_B, LookKind::PodSkin, Some(7), -1);
    assert_eq!(
        picks.wanted_look(TWO_B, true),
        PlayerLook {
            outfit_variant: 2,
            pod_skin: 7,
            ..PlayerLook::UNPICKED
        }
    );
    assert_eq!(picks.wanted_look(TWO_B, false), PlayerLook::UNPICKED);
    assert_eq!(picks.wanted_look(A2, true).outfit_variant, LOOK_UNPICKED);
}

#[test]
fn ending_the_session_forgets_the_picks_and_keeps_every_saved_value_to_restore() {
    let mut picks = LookPicks::default();
    picks.pick(TWO_B, LookKind::Hair, Some(3), SAVED_HAIR);
    picks.pick(A2, LookKind::Accessory, Some(9), SAVED_ACCESSORY);
    picks.end_session();
    assert_eq!(picks.picked(TWO_B, LookKind::Hair), None);
    assert_eq!(picks.wanted_look(TWO_B, true), PlayerLook::UNPICKED);
    assert_eq!(
        picks.take_targets(TWO_B, true)[LookKind::Hair.index()],
        Some(SAVED_HAIR)
    );
    assert!(picks.is_saved(A2, LookKind::Accessory));
}

#[test]
fn the_saved_looks_can_be_taken_once_for_a_restore_without_a_body() {
    let mut picks = LookPicks::default();
    picks.pick(TWO_B, LookKind::Hair, Some(3), SAVED_HAIR);
    picks.pick(A2, LookKind::Accessory, Some(9), SAVED_ACCESSORY);
    picks.end_session();
    assert_eq!(picks.take_saved_looks().len(), 2);
    assert!(picks.take_saved_looks().is_empty());
}

#[test]
fn a_slot_past_the_third_is_ignored() {
    let mut picks = LookPicks::default();
    picks.pick(3, LookKind::Outfit, Some(1), 0);
    assert_eq!(picks.picked(3, LookKind::Outfit), None);
    assert!(picks.saved_looks().is_empty());
}
