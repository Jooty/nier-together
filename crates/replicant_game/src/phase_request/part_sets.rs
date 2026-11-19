const NO_PART_LETTER: char = 'X';

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartSetsRegistered {
    pub talk_set: String,
    pub radio_set: String,
}

pub fn part_sets_registered_by_request(
    old_next_phase_name: &str,
    new_phase_name: &str,
) -> Option<PartSetsRegistered> {
    let letter = new_phase_name.chars().next()?;
    if letter == NO_PART_LETTER {
        return None;
    }
    if old_next_phase_name.starts_with(letter) {
        return None;
    }
    let radio_set = match letter {
        'C' => "radio_A".to_owned(),
        'D' => "radio_B".to_owned(),
        other => format!("radio_{other}"),
    };
    Some(PartSetsRegistered {
        talk_set: format!("talk_{letter}"),
        radio_set,
    })
}
