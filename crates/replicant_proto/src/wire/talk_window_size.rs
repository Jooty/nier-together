use serde::{Deserialize, Serialize};

/// The size the host's script gave its talk window (`_SetTalkWindowSize`) while it leads a
/// stretch of the story, so the watching guest's window shows the host's talks the same way.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TalkWindowSize {
    pub size: i32,
}
