use crate::dialogue::SCRIPT_TALK_START_OPENS_THE_WINDOW_WITHIN_MS;

/// Whether the talk in this player's own window is this game's own copy. The game marks a talk
/// it starts in control slot one; a talk it chains in the same window can leave slot one clear,
/// and is still the own copy until the window closes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OwnTalkWindow {
    had_own_talk: bool,
}

/// When a script last ran `_SetTalk`. A text-only window it opens never marks control slot one,
/// so the window counts as this game's own copy for as long as the call is this fresh.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SetTalkCalls {
    last_call_ms: Option<u64>,
}

impl OwnTalkWindow {
    pub const fn new() -> Self {
        Self {
            had_own_talk: false,
        }
    }

    /// Once per tick: true while the open window shows a talk this game started or chained.
    pub fn observe(&mut self, own_window_open: bool, started_own_talk: bool) -> bool {
        if !own_window_open {
            self.had_own_talk = false;
            return false;
        }
        self.had_own_talk |= started_own_talk;
        self.had_own_talk
    }
}

impl SetTalkCalls {
    pub const fn new() -> Self {
        Self { last_call_ms: None }
    }

    pub fn note_call(&mut self, now_ms: u64) {
        self.last_call_ms = Some(now_ms);
    }

    pub fn opens_the_window_now(&self, now_ms: u64) -> bool {
        self.last_call_ms.is_some_and(|called| {
            now_ms.saturating_sub(called) <= SCRIPT_TALK_START_OPENS_THE_WINDOW_WITHIN_MS
        })
    }
}
