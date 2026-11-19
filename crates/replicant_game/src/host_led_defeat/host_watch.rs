use replicant_proto::{DefeatPick, HostDefeatScreen};

use crate::host_led_defeat::{
    DEFEAT_SCREEN_FRAMES_APART_FOR_A_NEW_SCREEN_MS, GAME_OVER_CONTINUE_CURSOR,
    GAME_OVER_SEQUENCE_CHOOSING_STATE, GAME_OVER_SEQUENCE_DONE_STATE,
    GAME_OVER_SEQUENCE_WRITES_THE_RESULT_STATE, GameOverScreen, HOST_DEFEAT_SCREEN_RESEND_MS,
    host_cursor_line, host_pick_line, host_screen_opened_line, pick_of_cursor,
};

/// The host's side: each frame of its defeat screen becomes the note the guest's screen follows,
/// sent when the highlight moves, once a second while it stands, and once with the pick.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HostDefeatScreenWatch {
    defeat: u32,
    last_frame_ms: Option<u64>,
    picked: bool,
    sent_cursor: Option<u8>,
    sent_at_ms: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HostDefeatFrame {
    pub send: Option<HostDefeatScreen>,
    pub lines: Vec<String>,
}

impl HostDefeatScreenWatch {
    pub const fn new() -> Self {
        Self {
            defeat: 0,
            last_frame_ms: None,
            picked: false,
            sent_cursor: None,
            sent_at_ms: 0,
        }
    }

    /// One frame of the host's screen: `state_before` is its step before the game's own update
    /// ran, `after` the screen once it has. Step 4 turning into 5 is the pick the game made.
    pub fn on_frame(
        &mut self,
        state_before: i32,
        after: GameOverScreen,
        now_ms: u64,
    ) -> HostDefeatFrame {
        let mut frame = HostDefeatFrame::default();
        let apart = self.last_frame_ms.is_none_or(|last| {
            now_ms.saturating_sub(last) > DEFEAT_SCREEN_FRAMES_APART_FOR_A_NEW_SCREEN_MS
        });
        let reopened = self.picked && after.state == GAME_OVER_SEQUENCE_CHOOSING_STATE;
        self.last_frame_ms = Some(now_ms);
        if apart || reopened {
            self.defeat = self.defeat.wrapping_add(1);
            self.picked = false;
            self.sent_cursor = None;
            frame.lines.push(host_screen_opened_line(self.defeat));
        }
        if self.picked {
            return frame;
        }
        let cursor = u8::from(after.cursor != GAME_OVER_CONTINUE_CURSOR);
        if state_before == GAME_OVER_SEQUENCE_WRITES_THE_RESULT_STATE
            && after.state == GAME_OVER_SEQUENCE_DONE_STATE
        {
            let pick: DefeatPick = pick_of_cursor(after.cursor);
            self.picked = true;
            frame.send = Some(self.note(cursor, Some(pick), now_ms));
            frame.lines.push(host_pick_line(self.defeat, pick));
            return frame;
        }
        let moved = self.sent_cursor != Some(cursor);
        if moved || now_ms.saturating_sub(self.sent_at_ms) >= HOST_DEFEAT_SCREEN_RESEND_MS {
            if moved && self.sent_cursor.is_some() {
                frame.lines.push(host_cursor_line(self.defeat, cursor));
            }
            frame.send = Some(self.note(cursor, None, now_ms));
        }
        frame
    }

    fn note(&mut self, cursor: u8, pick: Option<DefeatPick>, now_ms: u64) -> HostDefeatScreen {
        self.sent_cursor = Some(cursor);
        self.sent_at_ms = now_ms;
        HostDefeatScreen {
            defeat: self.defeat,
            cursor,
            pick,
        }
    }
}
