use crate::start::{
    BASE_NAME_PART, CONDITIONAL_GROUP, ChapterRecord, OPENING_MISSION_NAME_PART,
    OPENING_MISSION_PHASE_ID,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    pub index: usize,
    pub record_index: usize,
    pub record: ChapterRecord,
}

pub fn candidates(records: &[ChapterRecord]) -> Vec<Candidate> {
    let default_starts = default_start_record_indexes(records);
    let other_records: Vec<usize> = (0..records.len())
        .filter(|record_index| !default_starts.contains(record_index))
        .collect();
    default_starts
        .iter()
        .copied()
        .chain(other_records)
        .enumerate()
        .map(|(index, record_index)| Candidate {
            index,
            record_index,
            record: records[record_index].clone(),
        })
        .collect()
}

pub fn start_index_of_record(records: &[ChapterRecord], record_index: usize) -> Option<usize> {
    candidates(records)
        .iter()
        .find(|candidate| candidate.record_index == record_index)
        .map(|candidate| candidate.index)
}

fn default_start_record_indexes(records: &[ChapterRecord]) -> Vec<usize> {
    let mut found = Vec::new();
    let mut previous_group = None;
    for (record_index, record) in records.iter().enumerate() {
        let is_group_header = previous_group != Some(record.group);
        previous_group = Some(record.group);
        if is_group_header && is_default_start(record) {
            found.push(record_index);
        }
    }
    found
}

fn is_default_start(record: &ChapterRecord) -> bool {
    let first_character_selectable = record.hero_mask & 1 != 0;
    let lowered_name = record.phase_name.to_ascii_lowercase();
    let is_opening_mission = record.phase_id == OPENING_MISSION_PHASE_ID
        && lowered_name.contains(&OPENING_MISSION_NAME_PART.to_ascii_lowercase());
    let is_base = lowered_name.contains(&BASE_NAME_PART.to_ascii_lowercase());
    first_character_selectable
        && record.group != CONDITIONAL_GROUP
        && !is_opening_mission
        && !is_base
}
