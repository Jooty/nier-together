use crate::dialogue::{
    LINE_PRESS_GIVES_UP_AFTER_MS, LINE_PRESS_OPENS_THE_TALK_AFTER_MS, TALK_PLAYER_STATE_TYPING,
    TALK_PLAYER_STATE_WAITS_FOR_A_PRESS,
};

/// What this game's talk window shows, read right before the game's own update of it runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TalkWindowReading {
    pub open: bool,
    /// The talk data the window's player holds; it changes when the player goes on to another talk.
    pub talk_address: u64,
    pub line: u16,
    pub line_count: u16,
    pub player_state: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PressStep {
    /// Put a confirm press into this update: the window is behind the partner's line.
    Press,
    Reached {
        talk_id: u32,
        line: u16,
        presses: u32,
    },
    /// The presses took the window on to another talk; the caller checks it is the wanted one.
    TalkChanged {
        talk_id: u32,
        line: u16,
        presses: u32,
    },
    /// The window did not leave its talk by presses: the caller opens the wanted talk itself.
    OpenInstead {
        talk_id: u32,
        line: u16,
        presses: u32,
    },
    GaveUp {
        talk_id: u32,
        line: u16,
        presses: u32,
    },
    Closed {
        talk_id: u32,
        line: u16,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Wanted {
    talk_id: u32,
    line: u16,
    on_this_talk: bool,
    talk_address: u64,
    since_ms: u64,
    presses: u32,
}

/// Moves this game's talk window to the partner's line the way the partner moved its own: a
/// confirm press in the window's update, one per frame, until the window shows that line.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LinePress {
    wanted: Option<Wanted>,
}

impl LinePress {
    pub const fn new() -> Self {
        Self { wanted: None }
    }

    /// The partner is on `line` of `talk_id`. `on_this_talk` when that is the talk the window
    /// holds now (at `talk_address`); otherwise the window must go on to it past its last line.
    pub fn want(
        &mut self,
        talk_id: u32,
        line: u16,
        on_this_talk: bool,
        talk_address: u64,
        now_ms: u64,
    ) {
        let same_wish = self
            .wanted
            .is_some_and(|wanted| wanted.talk_id == talk_id && wanted.line == line);
        if same_wish {
            return;
        }
        self.wanted = Some(Wanted {
            talk_id,
            line,
            on_this_talk,
            talk_address,
            since_ms: now_ms,
            presses: 0,
        });
    }

    pub fn clear(&mut self) {
        self.wanted = None;
    }

    pub fn wanted(&self) -> Option<(u32, u16)> {
        self.wanted.map(|wanted| (wanted.talk_id, wanted.line))
    }

    /// Once per window update, before the game's own update runs.
    pub fn step(&mut self, reading: &TalkWindowReading, now_ms: u64) -> Option<PressStep> {
        let wanted = self.wanted.as_mut()?;
        let (talk_id, line, presses) = (wanted.talk_id, wanted.line, wanted.presses);
        if !reading.open {
            self.wanted = None;
            return Some(PressStep::Closed { talk_id, line });
        }
        if reading.talk_address != wanted.talk_address {
            self.wanted = None;
            return Some(PressStep::TalkChanged {
                talk_id,
                line,
                presses,
            });
        }
        if wanted.on_this_talk && reading.line >= line {
            self.wanted = None;
            return Some(PressStep::Reached {
                talk_id,
                line,
                presses,
            });
        }
        let waited_ms = now_ms.saturating_sub(wanted.since_ms);
        if waited_ms >= LINE_PRESS_GIVES_UP_AFTER_MS {
            self.wanted = None;
            return Some(PressStep::GaveUp {
                talk_id,
                line,
                presses,
            });
        }
        if !wanted.on_this_talk && waited_ms >= LINE_PRESS_OPENS_THE_TALK_AFTER_MS {
            self.wanted = None;
            return Some(PressStep::OpenInstead {
                talk_id,
                line,
                presses,
            });
        }
        // A press on the last line goes on to the next talk: only a wanted switch may do that.
        let on_the_last_line = reading.line.saturating_add(1) >= reading.line_count;
        if wanted.on_this_talk && on_the_last_line {
            return None;
        }
        let takes_a_press = reading.player_state == TALK_PLAYER_STATE_TYPING
            || reading.player_state == TALK_PLAYER_STATE_WAITS_FOR_A_PRESS;
        if !takes_a_press {
            return None;
        }
        wanted.presses += 1;
        Some(PressStep::Press)
    }
}
