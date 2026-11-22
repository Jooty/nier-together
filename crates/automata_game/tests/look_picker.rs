use automata_game::mode::SessionMode;
use automata_game::outfit_look::{
    CAMPAIGN_LOOK_ALLOWED_TEXT, CAMPAIGN_LOOK_NOT_ALLOWED_TEXT, CLOTH_STATE_ITEM_ID,
    FREE_ROAM_LOOK_TEXT, GAME_DEFAULT_LABEL, GUN_SKIN_ONLY_YOU_TEXT, HAIR_NOT_OFFERED_TEXT,
    LookKind, LookNames, LookPicks, picker_is_usable, picker_note, picker_rows,
};

#[test]
fn free_roam_is_always_usable_and_says_so() {
    assert!(picker_is_usable(SessionMode::FreeRoam, false));
    assert_eq!(
        picker_note(SessionMode::FreeRoam, false),
        FREE_ROAM_LOOK_TEXT
    );
}

#[test]
fn the_campaign_picker_is_greyed_until_the_host_allows_it() {
    assert!(!picker_is_usable(SessionMode::Campaign, false));
    assert!(picker_is_usable(SessionMode::Campaign, true));
}

#[test]
fn the_campaign_text_says_the_items_are_meant_to_be_unlocked_by_playing() {
    for text in [CAMPAIGN_LOOK_ALLOWED_TEXT, CAMPAIGN_LOOK_NOT_ALLOWED_TEXT] {
        assert!(text.contains("meant to be unlocked by playing"));
    }
    assert_eq!(
        picker_note(SessionMode::Campaign, true),
        CAMPAIGN_LOOK_ALLOWED_TEXT
    );
    assert_eq!(
        picker_note(SessionMode::Campaign, false),
        CAMPAIGN_LOOK_NOT_ALLOWED_TEXT
    );
}

#[test]
fn there_is_a_row_for_each_kind_and_each_starts_with_the_game_default() {
    let rows = picker_rows(0, &LookPicks::default(), &LookNames::default());
    assert_eq!(rows.len(), LookKind::ALL.len());
    for row in rows {
        assert_eq!(row.entries[0].label, GAME_DEFAULT_LABEL);
        assert_eq!(row.entries[0].value, None);
        assert!(row.entries[0].chosen);
    }
}

#[test]
fn a_pick_marks_exactly_its_own_entry() {
    let mut picks = LookPicks::default();
    picks.pick(0, LookKind::Hair, Some(4), 0);
    let rows = picker_rows(0, &picks, &LookNames::default());
    let hair = rows.iter().find(|row| row.kind == LookKind::Hair).unwrap();
    let chosen: Vec<_> = hair.entries.iter().filter(|entry| entry.chosen).collect();
    assert_eq!(chosen.len(), 1);
    assert_eq!(chosen[0].value, Some(4));
}

#[test]
fn a_name_that_cannot_be_read_shows_the_id() {
    let rows = picker_rows(0, &LookPicks::default(), &LookNames::default());
    let outfit = &rows[LookKind::Outfit.index()];
    assert_eq!(outfit.entries[1].label, "981");
}

#[test]
fn a_name_that_was_read_is_shown_instead_of_the_id() {
    let mut names = LookNames::default();
    names.record(981, Some("  read text ".to_owned()));
    let rows = picker_rows(0, &LookPicks::default(), &names);
    assert_eq!(rows[LookKind::Outfit.index()].entries[1].label, "read text");
}

#[test]
fn the_cloth_row_is_headed_by_the_game_name_of_its_item_once_read() {
    let mut names = LookNames::default();
    names.record(CLOTH_STATE_ITEM_ID, Some("read heading".to_owned()));
    let rows = picker_rows(0, &LookPicks::default(), &names);
    assert_eq!(rows[LookKind::ClothState.index()].heading, "read heading");
    assert_eq!(rows[LookKind::ClothState.index()].entries.len(), 3);
}

#[test]
fn the_gun_row_says_only_the_player_sees_the_gun_skin() {
    let rows = picker_rows(0, &LookPicks::default(), &LookNames::default());
    assert_eq!(
        rows[LookKind::GunSkin.index()].note,
        Some(GUN_SKIN_ONLY_YOU_TEXT)
    );
}

#[test]
fn a_character_without_hair_dye_gets_the_default_entry_and_a_note() {
    let rows = picker_rows(1, &LookPicks::default(), &LookNames::default());
    let hair = &rows[LookKind::Hair.index()];
    assert_eq!(hair.entries.len(), 1);
    assert_eq!(hair.note, Some(HAIR_NOT_OFFERED_TEXT));
}

#[test]
fn names_are_read_a_few_at_a_time_and_given_up_after_three_misses() {
    let mut names = LookNames::default();
    assert_eq!(names.unread_ids(4).len(), 4);
    for _ in 0..3 {
        names.record(981, None);
    }
    assert!(!names.unread_ids(100).contains(&981));
    names.record(982, Some("name".to_owned()));
    assert!(!names.unread_ids(100).contains(&982));
    assert!(names.unread_ids(100).contains(&CLOTH_STATE_ITEM_ID));
}
