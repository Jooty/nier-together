use crate::late_join::{TAKE_IN_WAIT_LOG_INTERVAL_MS, take_in_done_line, take_in_waiting_line};
use crate::session_jump::{
    SessionJumpOrigin, SessionJumpReadings, SessionJumpRefusal, decide_session_jump,
};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TakeInStep {
    pub take_in: bool,
    pub logs: Vec<String>,
}

/// Waits for the first safe moment to take in a game that joined while this one was playing.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct JoinerTakeIn {
    logged: Option<(String, u64)>,
}

impl JoinerTakeIn {
    pub fn step(&mut self, now_ms: u64, waiting: bool, refusal: Option<String>) -> TakeInStep {
        let mut step = TakeInStep::default();
        if !waiting {
            self.logged = None;
            return step;
        }
        let Some(reason) = refusal else {
            self.logged = None;
            step.take_in = true;
            step.logs.push(take_in_done_line());
            return step;
        };
        let due = match &self.logged {
            Some((logged_reason, at_ms)) => {
                *logged_reason != reason
                    || now_ms.saturating_sub(*at_ms) >= TAKE_IN_WAIT_LOG_INTERVAL_MS
            }
            None => true,
        };
        if due {
            step.logs.push(take_in_waiting_line(&reason));
            self.logged = Some((reason, now_ms));
        }
        step
    }
}

/// Why this moment is not a safe one to take a newcomer in or to move this game; `None` when it is.
pub fn safe_moment_refusal(readings: &SessionJumpReadings) -> Option<SessionJumpRefusal> {
    decide_session_jump(SessionJumpOrigin::LandingAtTheHost, readings).err()
}
