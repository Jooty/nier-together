use replicant_proto::{ScaleUpAnswer, ScaleUpAsk, scale_up_no_save_yet_line};

use crate::scale_up_prompt::{
    Gesture, HOLD_TO_ANSWER_MS, IGNORE_PRESSES_AFTER_SHOWN_MS, PressSample,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenAsk {
    pub slot: u8,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AskEffect {
    Nothing,
    Line(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PromptStep {
    pub answer: Option<ScaleUpAnswer>,
    pub hold: Option<(bool, f32)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ScaleUpPrompt {
    open: Option<OpenAsk>,
    answered_slot: Option<u8>,
    shown_since_ms: Option<u64>,
    holding: Option<(bool, u64)>,
    must_release: bool,
}

impl ScaleUpPrompt {
    pub const fn new() -> Self {
        Self {
            open: None,
            answered_slot: None,
            shown_since_ms: None,
            holding: None,
            must_release: false,
        }
    }

    pub fn on_ask(&mut self, ask: ScaleUpAsk) -> AskEffect {
        match ask {
            ScaleUpAsk::Open { slot, name } => {
                let same_ask = self.open.as_ref().is_some_and(|open| open.slot == slot)
                    || self.answered_slot == Some(slot);
                if !same_ask {
                    self.open = Some(OpenAsk { slot, name });
                    self.answered_slot = None;
                    self.reset_timers();
                }
                AskEffect::Nothing
            }
            ScaleUpAsk::Withdrawn { slot } => {
                if self.open.as_ref().is_some_and(|open| open.slot == slot) {
                    self.open = None;
                    self.reset_timers();
                }
                if self.answered_slot == Some(slot) {
                    self.answered_slot = None;
                }
                AskEffect::Nothing
            }
            ScaleUpAsk::NoSaveYet { name, .. } => AskEffect::Line(scale_up_no_save_yet_line(&name)),
        }
    }

    pub fn clear(&mut self) {
        self.open = None;
        self.answered_slot = None;
        self.reset_timers();
    }

    pub fn is_open(&self) -> bool {
        self.open.is_some()
    }

    pub fn shown(&self) -> Option<&OpenAsk> {
        self.shown_since_ms.and(self.open.as_ref())
    }

    pub fn wants_input_masked(&self) -> bool {
        self.shown_since_ms.is_some()
    }

    pub fn step(&mut self, safe: bool, now_ms: u64, input: PressSample) -> PromptStep {
        let Some(open) = &self.open else {
            self.reset_timers();
            return PromptStep::default();
        };
        if !safe {
            self.reset_timers();
            return PromptStep::default();
        }
        let slot = open.slot;
        let since = *self.shown_since_ms.get_or_insert(now_ms);
        let gesture = input.combined();
        if now_ms.saturating_sub(since) < IGNORE_PRESSES_AFTER_SHOWN_MS {
            self.must_release = gesture != Gesture::Idle;
            self.holding = None;
            return PromptStep::default();
        }
        if self.must_release {
            self.must_release = gesture != Gesture::Idle;
            self.holding = None;
            return PromptStep::default();
        }
        if let Some(yes) = input.click {
            return self.answer(slot, yes);
        }
        let Some(yes) = answer_of(gesture) else {
            self.holding = None;
            return PromptStep::default();
        };
        let started = match self.holding {
            Some((held_yes, started)) if held_yes == yes => started,
            _ => now_ms,
        };
        self.holding = Some((yes, started));
        let held = now_ms.saturating_sub(started);
        if held >= HOLD_TO_ANSWER_MS {
            return self.answer(slot, yes);
        }
        PromptStep {
            answer: None,
            hold: Some((yes, held as f32 / HOLD_TO_ANSWER_MS as f32)),
        }
    }

    fn answer(&mut self, slot: u8, yes: bool) -> PromptStep {
        self.open = None;
        self.answered_slot = Some(slot);
        self.reset_timers();
        PromptStep {
            answer: Some(ScaleUpAnswer { slot, yes }),
            hold: None,
        }
    }

    fn reset_timers(&mut self) {
        self.shown_since_ms = None;
        self.holding = None;
        self.must_release = false;
    }
}

fn answer_of(gesture: Gesture) -> Option<bool> {
    match gesture {
        Gesture::Yes => Some(true),
        Gesture::No => Some(false),
        Gesture::Idle | Gesture::Mixed => None,
    }
}
