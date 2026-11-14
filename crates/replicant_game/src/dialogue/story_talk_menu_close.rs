pub const STORY_TALK_MENU_CLOSE_RETRY_AFTER_MS: u64 = 1_500;
pub const STORY_TALK_MENU_CLOSE_MOST_ASKS_PER_TALK: u8 = 3;

#[derive(Debug, Default)]
pub struct StoryTalkMenuClose {
    talk_id: Option<u32>,
    asks: u8,
    last_ask_ms: u64,
}

impl StoryTalkMenuClose {
    pub const fn new() -> Self {
        Self {
            talk_id: None,
            asks: 0,
            last_ask_ms: 0,
        }
    }

    pub fn should_close_the_pause_menu(
        &mut self,
        host_story_talk: Option<u32>,
        pause_menu_open: bool,
        own_talk_window_open: bool,
        now_ms: u64,
    ) -> bool {
        let Some(talk_id) = host_story_talk else {
            *self = Self::new();
            return false;
        };
        if self.talk_id != Some(talk_id) {
            self.talk_id = Some(talk_id);
            self.asks = 0;
        }
        if !pause_menu_open || own_talk_window_open {
            return false;
        }
        let waited_enough = self.asks == 0
            || now_ms.saturating_sub(self.last_ask_ms) >= STORY_TALK_MENU_CLOSE_RETRY_AFTER_MS;
        if self.asks >= STORY_TALK_MENU_CLOSE_MOST_ASKS_PER_TALK || !waited_enough {
            return false;
        }
        self.asks += 1;
        self.last_ask_ms = now_ms;
        true
    }
}

pub fn story_talk_menu_close_text(talk_id: u32) -> String {
    format!("the host's story talk {talk_id} starts while your pause menu is open")
}
