use replicant_proto::{DialogueLine, TalkChoice, TalkChoiceKind};

use crate::dialogue::{
    CHOICE_FOLLOW_GIVES_THE_GUEST_ITS_BUTTONS_AFTER_MS, HOST_ANSWER_KEPT_FOR_A_LATE_PROMPT_MS,
    HOST_ANSWERS_KEPT, TALK_PROMPT_KIND_MENU_LIST, TALK_PROMPT_KIND_TALK_LIST,
    TALK_STATE_PROMPT_CURSOR, TALK_STATE_PROMPT_MENU_LIST, TALK_STATE_PROMPT_TALK_LIST,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChoiceStep {
    Take(TalkChoice),
    /// The guest's own copy of a prompt the host already answered opened late: the host's
    /// answer, kept since it came, is taken.
    TakeKept(TalkChoice),
    GiveUp(TalkChoice),
    Drop(TalkChoice),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChoiceFollowInput {
    pub follows_host: bool,
    pub own_talk: Option<DialogueLine>,
    pub prompt: Option<TalkChoiceKind>,
    pub now_ms: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PendingChoice {
    choice: TalkChoice,
    received_ms: u64,
    seen_own_talk: bool,
    taken: bool,
    kept: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct KeptAnswer {
    choice: TalkChoice,
    received_ms: u64,
}

/// The host's answer to a prompt, kept until the guest's own copy of the talk can take it. Each
/// answer is also kept for a while after that, so a copy of the prompt the guest's game opens
/// later (a step it runs after the host) takes the host's answer and never asks the guest.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ChoiceFollower {
    pending: Option<PendingChoice>,
    kept: Vec<KeptAnswer>,
}

/// The clock the wait for the host's answer runs on. It stands still while this player's menu
/// covers the talk window, because the window cannot take an answer under a menu.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FollowClock {
    last_ms: Option<u64>,
    stood_still_ms: u64,
}

impl FollowClock {
    pub const fn new() -> Self {
        Self {
            last_ms: None,
            stood_still_ms: 0,
        }
    }

    pub fn now(&mut self, now_ms: u64, menu_open: bool) -> u64 {
        if menu_open && let Some(last_ms) = self.last_ms {
            self.stood_still_ms += now_ms.saturating_sub(last_ms);
        }
        self.last_ms = Some(now_ms);
        now_ms.saturating_sub(self.stood_still_ms)
    }
}

impl ChoiceFollower {
    pub const fn new() -> Self {
        Self {
            pending: None,
            kept: Vec::new(),
        }
    }

    pub fn receive(&mut self, choice: TalkChoice, now_ms: u64) {
        self.pending = Some(PendingChoice {
            choice,
            received_ms: now_ms,
            seen_own_talk: false,
            taken: false,
            kept: false,
        });
        self.kept.retain(|kept| {
            kept.choice.talk_id != choice.talk_id || kept.choice.line != choice.line
        });
        if self.kept.len() >= HOST_ANSWERS_KEPT {
            self.kept.remove(0);
        }
        self.kept.push(KeptAnswer {
            choice,
            received_ms: now_ms,
        });
    }

    pub fn forget(&mut self) {
        self.pending = None;
    }

    /// Forgets the kept answers too (the link is gone, or the guest left the area).
    pub fn forget_all(&mut self) {
        self.pending = None;
        self.kept.clear();
    }

    pub fn is_waiting(&self) -> bool {
        self.pending.is_some()
    }

    pub fn step(&mut self, input: ChoiceFollowInput) -> Option<ChoiceStep> {
        self.kept.retain(|kept| {
            input.now_ms.saturating_sub(kept.received_ms) < HOST_ANSWER_KEPT_FOR_A_LATE_PROMPT_MS
        });
        if self.pending.is_none()
            && let Some(choice) = self.kept_answer_for(input)
        {
            self.kept.retain(|kept| kept.choice != choice);
            self.pending = Some(PendingChoice {
                choice,
                received_ms: input.now_ms,
                seen_own_talk: true,
                taken: true,
                kept: true,
            });
            return Some(ChoiceStep::TakeKept(choice));
        }
        let pending = self.pending.as_mut()?;
        let choice = pending.choice;
        if !input.follows_host && !pending.kept {
            self.pending = None;
            return Some(ChoiceStep::Drop(choice));
        }
        let on_talk = input
            .own_talk
            .is_some_and(|own| own.talk_id == choice.talk_id);
        if on_talk {
            pending.seen_own_talk = true;
        } else if pending.seen_own_talk {
            let taken = pending.taken;
            self.pending = None;
            return (!taken).then_some(ChoiceStep::Drop(choice));
        }
        if pending.taken && input.prompt.is_none() {
            self.pending = None;
            return None;
        }
        let age = input.now_ms.saturating_sub(pending.received_ms);
        if age >= CHOICE_FOLLOW_GIVES_THE_GUEST_ITS_BUTTONS_AFTER_MS {
            self.pending = None;
            return Some(if on_talk {
                ChoiceStep::GiveUp(choice)
            } else {
                ChoiceStep::Drop(choice)
            });
        }
        let at_the_prompt = on_talk
            && input.prompt == Some(choice.kind)
            && input.own_talk.is_some_and(|own| own.line == choice.line);
        if at_the_prompt && !pending.taken {
            pending.taken = true;
            self.kept.retain(|kept| kept.choice != choice);
            return Some(ChoiceStep::Take(choice));
        }
        None
    }

    fn kept_answer_for(&self, input: ChoiceFollowInput) -> Option<TalkChoice> {
        let own = input.own_talk?;
        let prompt = input.prompt?;
        self.kept
            .iter()
            .rev()
            .find(|kept| {
                kept.choice.talk_id == own.talk_id
                    && kept.choice.line == own.line
                    && kept.choice.kind == prompt
            })
            .map(|kept| kept.choice)
    }
}

/// The kind of answer prompt the talk player is showing, from its state and its prompt kind field.
pub fn open_prompt_kind(player_state: i32, prompt_kind_field: i32) -> Option<TalkChoiceKind> {
    match player_state {
        TALK_STATE_PROMPT_TALK_LIST => Some(TalkChoiceKind::TalkList),
        TALK_STATE_PROMPT_MENU_LIST => Some(TalkChoiceKind::MenuList),
        TALK_STATE_PROMPT_CURSOR => match prompt_kind_field {
            TALK_PROMPT_KIND_TALK_LIST => Some(TalkChoiceKind::TalkList),
            TALK_PROMPT_KIND_MENU_LIST => Some(TalkChoiceKind::MenuList),
            _ => None,
        },
        _ => None,
    }
}
