use crate::story_jump::{Landing, NOT_READY_LABEL_SUFFIX, StoryJump};

pub fn menu_label(jump: &StoryJump) -> String {
    match jump.landing {
        Landing::NotReady(_) => format!("{}{NOT_READY_LABEL_SUFFIX}", jump.name),
        Landing::NewGame | Landing::ChapterRecord(_) => jump.name.to_owned(),
    }
}

pub fn press_outcome(jump: &StoryJump, start_index: Option<usize>) -> Result<String, String> {
    match jump.landing {
        Landing::NotReady(reason) => Err(format!("not ready: {reason}")),
        Landing::NewGame => Ok(
            "works at launch only: start setting new_game; a running session cannot be restarted into it yet"
                .to_owned(),
        ),
        Landing::ChapterRecord(record) => match start_index {
            Some(index) => Ok(format!(
                "works at launch only: start setting world with start index {index} (chapter record {record}); a running session cannot be restarted into it yet"
            )),
            None => Err("the chapter table is not read yet, so the start index is not known".to_owned()),
        },
    }
}
