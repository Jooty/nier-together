mod landing;
mod open_jumps;
mod outcome;

pub use landing::{Landing, StoryJump};
pub use open_jumps::{OPEN_STORY_JUMPS, story_jump_by_id};
pub use outcome::{menu_label, press_outcome};

pub const NOT_READY_LABEL_SUFFIX: &str = " (not ready)";
