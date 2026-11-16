use automata_game::start::{ChapterRecord, candidates, start_index_of_record};

fn record(chapter: u8, group: u8, mask: u8, phase_id: i16, name: &str) -> ChapterRecord {
    ChapterRecord {
        chapter,
        group,
        route: 0,
        hero_mask: mask,
        phase_id,
        phase_name: name.to_owned(),
    }
}

fn sample_table() -> Vec<ChapterRecord> {
    vec![
        record(0, 1, 1, 0x100, "00_30_Prologue_STG1"),
        record(1, 1, 1, 0x101, "00_31_Prologue_STG2"),
        record(2, 2, 1, 0x200, "01_10_City"),
        record(3, 2, 1, 0x201, "01_11_City_B"),
        record(4, 3, 1, 0x300, "02_10_Bunker_Hub"),
        record(5, 4, 0b10, 0x400, "03_10_Second_Only"),
        record(6, 0x11, 1, 0x500, "99_10_Conditional"),
        record(7, 5, 0b101, 0x600, "04_10_Desert"),
    ]
}

#[test]
fn the_default_starts_come_first_in_table_order() {
    let found = candidates(&sample_table());
    let phases: Vec<i16> = found.iter().map(|c| c.record.phase_id).collect();
    assert_eq!(phases[..2], [0x200, 0x600]);
}

#[test]
fn every_record_of_the_table_has_exactly_one_start_index() {
    let table = sample_table();
    let found = candidates(&table);
    assert_eq!(found.len(), table.len());
    let mut record_indexes: Vec<usize> = found.iter().map(|c| c.record_index).collect();
    record_indexes.sort_unstable();
    assert_eq!(record_indexes, (0..table.len()).collect::<Vec<_>>());
}

#[test]
fn the_records_the_old_rule_left_out_follow_the_default_starts_in_table_order() {
    let found = candidates(&sample_table());
    let left_out: Vec<usize> = found[2..].iter().map(|c| c.record_index).collect();
    assert_eq!(left_out, [0, 1, 3, 4, 5, 6]);
}

#[test]
fn a_table_of_93_records_is_reached_in_full() {
    let table: Vec<ChapterRecord> = (0..93)
        .map(|n| record(n as u8, 1 + (n / 5) as u8, 1, 0x100 + n, "name"))
        .collect();
    let found = candidates(&table);
    assert_eq!(found.len(), 93);
    for record_index in 0..93 {
        let index = start_index_of_record(&table, record_index).unwrap();
        assert_eq!(found[index].record_index, record_index);
    }
}

#[test]
fn a_record_past_the_table_has_no_start_index() {
    assert_eq!(start_index_of_record(&sample_table(), 8), None);
}

#[test]
fn candidates_are_numbered_in_table_order_and_remember_their_record() {
    let found = candidates(&sample_table());
    assert_eq!(found[0].index, 0);
    assert_eq!(found[0].record_index, 2);
    assert_eq!(found[1].index, 1);
    assert_eq!(found[1].record_index, 7);
}

#[test]
fn a_group_that_comes_back_later_starts_a_new_header() {
    let table = vec![
        record(0, 2, 1, 0x20, "a"),
        record(1, 3, 1, 0x30, "b"),
        record(2, 2, 1, 0x21, "c"),
    ];
    let found = candidates(&table);
    let record_indexes: Vec<usize> = found.iter().map(|c| c.record_index).collect();
    assert_eq!(record_indexes, [0, 1, 2]);
}

#[test]
fn an_empty_table_has_no_candidates() {
    assert!(candidates(&[]).is_empty());
}

#[test]
fn the_name_tests_ignore_case() {
    let table = vec![
        record(0, 2, 1, 0x100, "00_PROLOGUE_stg"),
        record(1, 3, 1, 0x300, "02_10_BUNKER_hub"),
        record(2, 4, 1, 0x400, "03_10_bunker_hub"),
        record(3, 5, 1, 0x600, "04_10_Desert"),
    ];
    assert_eq!(candidates(&table)[0].record.phase_id, 0x600);
}

#[test]
fn candidates_that_share_a_phase_id_stay_distinct_by_chapter_and_name() {
    let table = vec![
        record(12, 2, 3, 0x100, "name_one"),
        record(14, 3, 3, 0x100, "name_two"),
    ];
    let found = candidates(&table);
    assert_eq!(found.len(), 2);
    assert_ne!(found[0].record.chapter, found[1].record.chapter);
}
