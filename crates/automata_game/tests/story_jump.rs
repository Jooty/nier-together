use automata_game::story_jump::{
    Landing, OPEN_STORY_JUMPS, menu_label, press_outcome, story_jump_by_id,
};

#[test]
fn the_open_jumps_are_listed_in_id_order_from_10_to_140_in_steps_of_10() {
    let ids: Vec<u32> = OPEN_STORY_JUMPS.iter().map(|jump| jump.id).collect();
    assert_eq!(ids, (1..=14).map(|n| n * 10).collect::<Vec<_>>());
}

#[test]
fn every_name_has_a_place_first_and_is_unique() {
    let mut names: Vec<&str> = OPEN_STORY_JUMPS.iter().map(|jump| jump.name).collect();
    assert!(names.iter().all(|name| name.contains(", ")));
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), OPEN_STORY_JUMPS.len());
}

#[test]
fn the_landing_of_each_row_is_the_one_the_list_names() {
    let landings: Vec<(u32, Option<usize>)> = OPEN_STORY_JUMPS
        .iter()
        .filter_map(|jump| match jump.landing {
            Landing::NewGame => Some((jump.id, None)),
            Landing::ChapterRecord(record) => Some((jump.id, Some(record))),
            Landing::NotReady(_) => None,
        })
        .collect();
    assert_eq!(
        landings,
        [
            (10, None),
            (40, Some(2)),
            (70, Some(4)),
            (100, Some(5)),
            (120, Some(6)),
            (140, Some(8)),
        ]
    );
}

#[test]
fn the_rows_that_need_reverse_engineering_show_as_not_ready() {
    let not_ready: Vec<u32> = OPEN_STORY_JUMPS
        .iter()
        .filter(|jump| menu_label(jump).ends_with("(not ready)"))
        .map(|jump| jump.id)
        .collect();
    assert_eq!(not_ready, [20, 30, 50, 60, 80, 90, 110, 130]);
}

#[test]
fn a_not_ready_row_does_nothing_and_says_why() {
    let jump = story_jump_by_id(20).unwrap();
    let outcome = press_outcome(jump, Some(3));
    assert!(outcome.is_err_and(|text| text.starts_with("not ready: ")));
}

#[test]
fn a_chapter_row_names_the_start_index_to_set_and_says_it_works_at_launch_only() {
    let text = press_outcome(story_jump_by_id(120).unwrap(), Some(41)).unwrap();
    assert!(text.contains("launch only"));
    assert!(text.contains("start index 41"));
    assert!(text.contains("chapter record 6"));
}

#[test]
fn a_chapter_row_before_the_table_is_read_is_refused() {
    assert!(press_outcome(story_jump_by_id(120).unwrap(), None).is_err());
}

#[test]
fn the_first_row_is_the_new_game_and_needs_no_index() {
    let text = press_outcome(story_jump_by_id(10).unwrap(), None).unwrap();
    assert!(text.contains("new_game"));
}

#[test]
fn an_unknown_id_is_not_a_jump() {
    assert!(story_jump_by_id(150).is_none());
}
