use replicant_proto::{DefeatPick, HostDefeatScreen};

use crate::host_led_defeat::{
    DEFEAT_SCREEN_FRAMES_APART_FOR_A_NEW_SCREEN_MS, GAME_OVER_SEQUENCE_CHOOSING_STATE,
    GUEST_DEFEAT_SCREEN_WAITS_FOR_THE_HOST_MS, GameOverScreen, HOST_DEFEAT_PICK_KEPT_FOR_MS,
    USED_DEFEAT_FORGOTTEN_AFTER_MS,
    HOST_DEFEAT_SCREEN_NOTE_FRESH_MS, context_missing_line, guest_buttons_back_line,
    guest_led_line, guest_made_pick_line, guest_screen_opened_line, guest_waits_line,
    heard_pick_line, heard_screen_line, pick_dropped_line,
};

/// The guest's side: while the host shows a defeat screen of its own, this game's screen takes
/// no buttons, shows the host's highlight and makes the host's pick; with no host screen it is
/// the player's own after a short wait.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GuestDefeatFollow {
    heard: Option<HeardNote>,
    used_defeat: Option<UsedDefeat>,
    first_frame_of_screen: bool,
    own_opened_ms: u64,
    last_frame_ms: Option<u64>,
    mode: Option<FollowMode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct UsedDefeat {
    defeat: u32,
    at_ms: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct HeardNote {
    note: HostDefeatScreen,
    at_ms: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FollowMode {
    PickWaits,
    Waits,
    Led,
    Own,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GuestDefeatAct {
    OwnButtons,
    ButtonsOff { show_cursor: Option<i32> },
    MakeHostsPick(DefeatPick),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GuestDefeatFrame {
    pub act: GuestDefeatAct,
    pub lines: Vec<String>,
}

impl GuestDefeatFollow {
    pub const fn new() -> Self {
        Self {
            heard: None,
            used_defeat: None,
            first_frame_of_screen: false,
            own_opened_ms: 0,
            last_frame_ms: None,
            mode: None,
        }
    }

    pub fn hear(&mut self, note: HostDefeatScreen, now_ms: u64) -> Vec<String> {
        if self.used_defeat.is_some_and(|used| used.defeat == note.defeat) {
            return Vec::new();
        }
        let before = self.heard.map(|heard| heard.note);
        self.heard = Some(HeardNote {
            note,
            at_ms: now_ms,
        });
        if let Some(pick) = note.pick {
            return vec![heard_pick_line(note.defeat, pick)];
        }
        if before.is_none_or(|before| before.defeat != note.defeat) {
            return vec![heard_screen_line(note.defeat, note.cursor)];
        }
        Vec::new()
    }

    /// Lines for a host pick no screen here took in time; the pick is then forgotten.
    pub fn age(&mut self, now_ms: u64) -> Vec<String> {
        let Some(heard) = self.heard else {
            return Vec::new();
        };
        let waited_ms = now_ms.saturating_sub(heard.at_ms);
        match heard.note.pick {
            Some(pick) if waited_ms > HOST_DEFEAT_PICK_KEPT_FOR_MS => {
                self.heard = None;
                self.used_defeat = Some(UsedDefeat {
                    defeat: heard.note.defeat,
                    at_ms: now_ms,
                });
                vec![pick_dropped_line(heard.note.defeat, pick, waited_ms)]
            }
            _ => Vec::new(),
        }
    }

    /// One frame of this game's own defeat screen. `context_found` says whether the screen's
    /// result word can be written this frame.
    pub fn on_frame(
        &mut self,
        screen: GameOverScreen,
        context_found: bool,
        now_ms: u64,
    ) -> GuestDefeatFrame {
        let mut lines = Vec::new();
        self.first_frame_of_screen = self.last_frame_ms.is_none_or(|last| {
            now_ms.saturating_sub(last) > DEFEAT_SCREEN_FRAMES_APART_FOR_A_NEW_SCREEN_MS
        });
        if self.first_frame_of_screen {
            if self.used_defeat.is_some_and(|used| {
                now_ms.saturating_sub(used.at_ms) > USED_DEFEAT_FORGOTTEN_AFTER_MS
            }) {
                self.used_defeat = None;
            }
            self.own_opened_ms = now_ms;
            self.mode = None;
            lines.push(guest_screen_opened_line(self.heard_host_note(now_ms)));
        }
        self.last_frame_ms = Some(now_ms);
        let act = self.act(screen, context_found, now_ms, &mut lines);
        GuestDefeatFrame { act, lines }
    }

    fn act(
        &mut self,
        screen: GameOverScreen,
        context_found: bool,
        now_ms: u64,
        lines: &mut Vec<String>,
    ) -> GuestDefeatAct {
        if let Some(heard) = self.heard
            && let Some(pick) = heard.note.pick
            && now_ms.saturating_sub(heard.at_ms) <= HOST_DEFEAT_PICK_KEPT_FOR_MS
        {
            if screen.state != GAME_OVER_SEQUENCE_CHOOSING_STATE {
                return GuestDefeatAct::ButtonsOff { show_cursor: None };
            }
            if self.first_frame_of_screen || !context_found {
                self.enter(FollowMode::PickWaits, lines, || {
                    context_missing_line(heard.note.defeat, pick)
                });
                return GuestDefeatAct::ButtonsOff { show_cursor: None };
            }
            self.heard = None;
            self.used_defeat = Some(UsedDefeat {
                defeat: heard.note.defeat,
                at_ms: now_ms,
            });
            self.mode = None;
            lines.push(guest_made_pick_line(
                heard.note.defeat,
                pick,
                now_ms.saturating_sub(heard.at_ms),
                now_ms.saturating_sub(self.own_opened_ms),
            ));
            return GuestDefeatAct::MakeHostsPick(pick);
        }
        if let Some(note) = self.heard_host_note(now_ms) {
            self.enter(FollowMode::Led, lines, || guest_led_line(note.defeat));
            return GuestDefeatAct::ButtonsOff {
                show_cursor: Some(i32::from(note.cursor)),
            };
        }
        if now_ms.saturating_sub(self.own_opened_ms) < GUEST_DEFEAT_SCREEN_WAITS_FOR_THE_HOST_MS {
            self.enter(FollowMode::Waits, lines, guest_waits_line);
            return GuestDefeatAct::ButtonsOff { show_cursor: None };
        }
        self.enter(FollowMode::Own, lines, guest_buttons_back_line);
        GuestDefeatAct::OwnButtons
    }

    fn heard_host_note(&self, now_ms: u64) -> Option<HostDefeatScreen> {
        self.heard
            .filter(|heard| heard.note.pick.is_none())
            .filter(|heard| now_ms.saturating_sub(heard.at_ms) <= HOST_DEFEAT_SCREEN_NOTE_FRESH_MS)
            .map(|heard| heard.note)
    }

    fn enter(&mut self, mode: FollowMode, lines: &mut Vec<String>, line: impl FnOnce() -> String) {
        if self.mode != Some(mode) {
            self.mode = Some(mode);
            lines.push(line());
        }
    }
}
