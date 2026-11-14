use crate::event_follow::{
    EventCall, EventId, EventSend, EventStep, PauseDebounce, PauseView,
    own_pause_outside_a_cutscene_line, own_pause_outside_the_rule_line, own_pause_told_line,
    partner_pause_ended_line, partner_pause_event_ended_here_line, partner_pause_heard_line,
    partner_pause_rule_ended_line, pause_clock_line, resume_clock_line,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PartnerPause {
    id: EventId,
    seen_playing_here: bool,
}

/// One cutscene pause: either player's event pause pauses the event clock on both copies.
#[derive(Debug, Clone, Default)]
pub struct EventPause {
    told_paused: Option<EventId>,
    partner: Option<PartnerPause>,
    holding: bool,
    was_open: bool,
    debounce: PauseDebounce,
}

impl EventPause {
    /// Runs one tick; true while the note "the partner paused it" is due.
    pub fn step(&mut self, view: &PauseView, step: &mut EventStep) -> bool {
        let view = &PauseView {
            local_open: self.debounce.observe(view.now_ms, view.local_open),
            ..*view
        };
        if !view.rule_active {
            if let Some(pause) = self.partner.take() {
                step.logs.push(partner_pause_rule_ended_line(pause.id));
            }
            if let Some(id) = view.event
                && self.opened_just_now(view)
            {
                step.logs.push(own_pause_outside_the_rule_line(id));
            }
            self.told_paused = None;
            self.settle(false, view, step);
            self.was_open = view.local_open;
            return false;
        }
        self.tell_own_pause(view, step);
        self.hear_partner(view, step);
        let wanted = self.partner_pause_applies(view);
        self.settle(wanted, view, step);
        self.was_open = view.local_open;
        wanted
    }

    fn tell_own_pause(&mut self, view: &PauseView, step: &mut EventStep) {
        match (view.local_open, view.event, self.told_paused) {
            (true, Some(id), None) => {
                self.told_paused = Some(id);
                step.sends.push(EventSend::Pause { id, paused: true });
                step.logs.push(own_pause_told_line(id, true));
            }
            (true, None, None) if self.opened_just_now(view) => {
                step.logs.push(own_pause_outside_a_cutscene_line());
            }
            (false, _, Some(id)) => {
                self.told_paused = None;
                step.sends.push(EventSend::Pause { id, paused: false });
                step.logs.push(own_pause_told_line(id, false));
            }
            _ => {}
        }
    }

    fn hear_partner(&mut self, view: &PauseView, step: &mut EventStep) {
        for &(id, paused) in view.heard {
            step.logs.push(partner_pause_heard_line(id, paused));
            if paused {
                self.partner = Some(PartnerPause {
                    id,
                    seen_playing_here: false,
                });
            } else if self.partner.is_some_and(|pause| pause.id == id) {
                self.partner = None;
            }
        }
        let Some(pause) = self.partner.as_mut() else {
            return;
        };
        if view.event == Some(pause.id) {
            pause.seen_playing_here = true;
        } else if view.event.is_some() {
            step.logs.push(partner_pause_ended_line(pause.id));
            self.partner = None;
        } else if pause.seen_playing_here {
            step.logs
                .push(partner_pause_event_ended_here_line(pause.id));
            self.partner = None;
        }
    }

    fn opened_just_now(&self, view: &PauseView) -> bool {
        view.local_open && !self.was_open
    }

    fn partner_pause_applies(&self, view: &PauseView) -> bool {
        self.partner
            .is_some_and(|pause| view.event == Some(pause.id))
    }

    fn settle(&mut self, wanted: bool, view: &PauseView, step: &mut EventStep) {
        let id = self.partner.map(|pause| pause.id).or(view.event);
        let closed_just_now = self.was_open && !view.local_open;
        if wanted && (!self.holding || closed_just_now) {
            self.holding = true;
            if !view.local_open {
                step.calls.push(EventCall::PauseClock);
                if let Some(id) = id {
                    step.logs.push(pause_clock_line(id));
                }
            }
        } else if !wanted && self.holding {
            self.holding = false;
            if !view.local_open {
                step.calls.push(EventCall::ResumeClock);
                step.logs.push(resume_clock_line());
            }
        }
    }
}
