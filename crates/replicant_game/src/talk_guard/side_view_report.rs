use replicant_proto::SideViewState;

use crate::talk_guard::SIDE_VIEW_STATE_RESENT_EVERY_MS;

/// When this game sends its [`SideViewState`] to the partner: on every change, and again once
/// the last one is [`SIDE_VIEW_STATE_RESENT_EVERY_MS`] old, so the partner's copy stays fresh.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SideViewReporter {
    last_sent: Option<(SideViewState, u64)>,
}

impl SideViewReporter {
    pub const fn new() -> Self {
        Self { last_sent: None }
    }

    pub fn report_due(&mut self, state: SideViewState, now_ms: u64) -> bool {
        let due = self.last_sent.is_none_or(|(sent, at_ms)| {
            sent != state || now_ms.saturating_sub(at_ms) >= SIDE_VIEW_STATE_RESENT_EVERY_MS
        });
        if due {
            self.last_sent = Some((state, now_ms));
        }
        due
    }

    pub fn forget(&mut self) {
        self.last_sent = None;
    }
}
