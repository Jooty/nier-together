use crate::story_jump::{Landing, StoryJump};

const NEEDS_STORY_STATE_BETWEEN_RECORDS: &str =
    "needs the story state between two chapter records, not understood yet";

pub const OPEN_STORY_JUMPS: [StoryJump; 14] = [
    jump(10, "Factory, flight start", Landing::NewGame),
    jump(
        20,
        "Factory, second flight",
        Landing::NotReady(NEEDS_STORY_STATE_BETWEEN_RECORDS),
    ),
    jump(
        30,
        "Factory, third flight",
        Landing::NotReady(NEEDS_STORY_STATE_BETWEEN_RECORDS),
    ),
    jump(40, "Factory, after flights", Landing::ChapterRecord(2)),
    jump(
        50,
        "Factory, combat tutorial",
        Landing::NotReady(NEEDS_STORY_STATE_BETWEEN_RECORDS),
    ),
    jump(
        60,
        "Factory, first saw",
        Landing::NotReady(NEEDS_STORY_STATE_BETWEEN_RECORDS),
    ),
    jump(70, "Factory, first area", Landing::ChapterRecord(4)),
    jump(
        80,
        "Factory, second area",
        Landing::NotReady(NEEDS_STORY_STATE_BETWEEN_RECORDS),
    ),
    jump(
        90,
        "Factory, third area",
        Landing::NotReady(NEEDS_STORY_STATE_BETWEEN_RECORDS),
    ),
    jump(100, "Factory, last area", Landing::ChapterRecord(5)),
    jump(
        110,
        "Factory, second saws",
        Landing::NotReady(NEEDS_STORY_STATE_BETWEEN_RECORDS),
    ),
    jump(120, "Factory, boss 1, ground", Landing::ChapterRecord(6)),
    jump(
        130,
        "Factory, boss 1, climb",
        Landing::NotReady(NEEDS_STORY_STATE_BETWEEN_RECORDS),
    ),
    jump(140, "Factory, boss 1, air", Landing::ChapterRecord(8)),
];

pub fn story_jump_by_id(id: u32) -> Option<&'static StoryJump> {
    OPEN_STORY_JUMPS.iter().find(|jump| jump.id == id)
}

const fn jump(id: u32, name: &'static str, landing: Landing) -> StoryJump {
    StoryJump { id, name, landing }
}
