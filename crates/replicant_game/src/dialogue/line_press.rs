use crate::dialogue::{
    LINE_PRESS_GIVES_UP_AFTER_MS, LINE_PRESS_OPENS_THE_TALK_AFTER_MS, TALK_PLAYER_STATE_TYPING,
    TALK_PLAYER_STATE_WAITS_AT_A_PAGE_BREAK, TALK_PLAYER_STATE_WAITS_FOR_A_PRESS,
};

/// What this game's talk window shows, read right before the game's own update of it runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TalkWindowReading {
    pub open: bool,
    /// The talk data the window's player holds; it changes when the player goes on to another talk.
    pub talk_address: u64,
    pub line: u16,
    /// The presses made inside the line so far: pages turned plus page-break waits passed.
    pub step: u16,
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
    step: u16,
    on_this_talk: bool,
    talk_address: u64,
    since_ms: u64,
    presses: u32,
}

/// Counts the presses a talk line took on this game: the page it is on plus every page-break
/// wait (player state 5) it has left. Both games count their own window the same way.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LineSteps {
    line: Option<(u64, u16)>,
    waits_passed: u16,
    last_state: i32,
}

/// Moves this game's talk window to the partner's line the way the partner moved its own: a
/// confirm press in the window's update, one per frame, until the window shows that line.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LinePress {
    wanted: Option<Wanted>,
    /// The last wish the window was seen to show: asking for it again is nothing new.
    settled: Option<(u32, u16, u16)>,
}

impl LineSteps {
    pub const fn new() -> Self {
        Self {
            line: None,
            waits_passed: 0,
            last_state: 0,
        }
    }

    /// Once per frame with the window's talk, line, page and player state; returns the step.
    pub fn observe(&mut self, talk_address: u64, line: u16, page: u16, player_state: i32) -> u16 {
        if self.line != Some((talk_address, line)) {
            self.line = Some((talk_address, line));
            self.waits_passed = 0;
            self.last_state = player_state;
        }
        if self.last_state == TALK_PLAYER_STATE_WAITS_AT_A_PAGE_BREAK
            && player_state != TALK_PLAYER_STATE_WAITS_AT_A_PAGE_BREAK
        {
            self.waits_passed = self.waits_passed.saturating_add(1);
        }
        self.last_state = player_state;
        page.saturating_add(self.waits_passed)
    }

    pub fn forget(&mut self) {
        *self = Self::new();
    }
}

impl LinePress {
    pub const fn new() -> Self {
        Self {
            wanted: None,
            settled: None,
        }
    }

    /// The partner is at `step` of `line` of `talk_id`. `window_talk_id` names the talk the window
    /// holds (asked only for a new wish); on another talk the window must go on past its last
    /// line. True when a new wish was made.
    pub fn want(
        &mut self,
        talk_id: u32,
        line: u16,
        step: u16,
        reading: &TalkWindowReading,
        window_talk_id: impl FnOnce() -> Option<u32>,
        now_ms: u64,
    ) -> bool {
        let wish = (talk_id, line, step);
        let same_wish = self
            .wanted
            .is_some_and(|wanted| (wanted.talk_id, wanted.line, wanted.step) == wish);
        if same_wish || self.settled == Some(wish) {
            return false;
        }
        let on_this_talk = reading.open && window_talk_id() == Some(talk_id);
        let already_shown =
            on_this_talk && (reading.line > line || (reading.line == line && reading.step >= step));
        if already_shown {
            self.settled = Some(wish);
            return false;
        }
        // A switch the window has not made yet keeps its clock and count when the partner moves
        // on inside the new talk, so the fallback open is not pushed back by every move.
        let pending_switch = self
            .wanted
            .filter(|wanted| !wanted.on_this_talk && !on_this_talk && wanted.talk_id == talk_id);
        self.wanted = Some(Wanted {
            talk_id,
            line,
            step,
            on_this_talk,
            talk_address: reading.talk_address,
            since_ms: pending_switch.map_or(now_ms, |wanted| wanted.since_ms),
            presses: pending_switch.map_or(0, |wanted| wanted.presses),
        });
        true
    }

    pub fn clear(&mut self) {
        self.wanted = None;
        self.settled = None;
    }

    pub fn wanted(&self) -> Option<(u32, u16, u16)> {
        self.wanted
            .map(|wanted| (wanted.talk_id, wanted.line, wanted.step))
    }

    /// The wish is for a talk the window does not hold yet.
    pub fn wants_a_switch(&self) -> bool {
        self.wanted.is_some_and(|wanted| !wanted.on_this_talk)
    }

    /// Once per window update, before the game's own update runs.
    pub fn step(&mut self, reading: &TalkWindowReading, now_ms: u64) -> Option<PressStep> {
        let wanted = self.wanted.as_mut()?;
        let (talk_id, line, presses) = (wanted.talk_id, wanted.line, wanted.presses);
        if !reading.open {
            self.wanted = None;
            self.settled = None;
            return Some(PressStep::Closed { talk_id, line });
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
        if reading.talk_address != wanted.talk_address {
            self.wanted = None;
            return Some(PressStep::TalkChanged {
                talk_id,
                line,
                presses,
            });
        }
        let behind_in_the_line = reading.line == line && reading.step < wanted.step;
        if wanted.on_this_talk && reading.line >= line && !behind_in_the_line {
            self.settled = Some((talk_id, line, wanted.step));
            self.wanted = None;
            return Some(PressStep::Reached {
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
        // A press past the last line goes on to the next talk: only a wanted switch, or a step
        // the partner also made inside that line, may press there.
        let on_the_last_line = reading.line.saturating_add(1) >= reading.line_count;
        if wanted.on_this_talk && on_the_last_line && !behind_in_the_line {
            return None;
        }
        let takes_a_press = reading.player_state == TALK_PLAYER_STATE_TYPING
            || reading.player_state == TALK_PLAYER_STATE_WAITS_FOR_A_PRESS
            || reading.player_state == TALK_PLAYER_STATE_WAITS_AT_A_PAGE_BREAK;
        if !takes_a_press {
            return None;
        }
        wanted.presses += 1;
        Some(PressStep::Press)
    }
}
