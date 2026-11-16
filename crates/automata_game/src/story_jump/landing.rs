#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Landing {
    NewGame,
    ChapterRecord(usize),
    NotReady(&'static str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StoryJump {
    pub id: u32,
    pub name: &'static str,
    pub landing: Landing,
}
