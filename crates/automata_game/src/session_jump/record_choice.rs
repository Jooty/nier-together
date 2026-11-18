use crate::start::{ChapterRecord, candidates};

pub fn record_of_start_index(
    records: &[ChapterRecord],
    start_index: usize,
) -> Result<ChapterRecord, String> {
    let found = candidates(records);
    found
        .get(start_index)
        .map(|candidate| candidate.record.clone())
        .ok_or_else(|| {
            format!(
                "start index {start_index} is out of range ({} candidates)",
                found.len()
            )
        })
}
