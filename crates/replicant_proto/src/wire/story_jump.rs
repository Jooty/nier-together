use serde::{Deserialize, Serialize};

use crate::wire::story_jump_point::StoryJumpPoint;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoryJump {
    pub point: StoryJumpPoint,
}
