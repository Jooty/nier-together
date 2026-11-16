use std::sync::{Mutex, PoisonError};

use replicant_game::talk_guard::{PartnerSideView, SideViewReporter};
use replicant_proto::{Message, SideViewState};

use crate::build_gate::VerifiedBuild;
use crate::diagnostics::log_line;
use crate::net::{NetState, NetThread, net_clock_ms};

static REPORTER: Mutex<SideViewReporter> = Mutex::new(SideViewReporter::new());
static PARTNER_REPORT: Mutex<Option<(SideViewState, u64)>> = Mutex::new(None);

pub(super) fn tick(build: &VerifiedBuild, net: &NetThread) {
    let received = net.take_relayed(|message| matches!(message, Message::SideViewState(_)));
    if let Some(Message::SideViewState(state)) = received.into_iter().last() {
        let mut partner = PARTNER_REPORT
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        if partner.is_none_or(|(last, _)| last.in_side_view != state.in_side_view) {
            log_line(&format!(
                "side view report: the partner's side view is {} (phase {}, door {})",
                if state.in_side_view { "on" } else { "off" },
                state.phase,
                state.side_view_door
            ));
        }
        *partner = Some((state, net_clock_ms()));
    }
    let view = net.view();
    let mut reporter = REPORTER.lock().unwrap_or_else(PoisonError::into_inner);
    let (true, Some(local)) = (view.state == NetState::Connected, view.local.as_ref()) else {
        reporter.forget();
        return;
    };
    let state = SideViewState {
        phase: local.phase,
        in_side_view: crate::stay_together::side_view_now(build),
        side_view_door: crate::game_hooks::own_side_view_door(),
    };
    if reporter.report_due(state, net_clock_ms()) {
        net.send_relayed(Message::SideViewState(state));
    }
}

pub(crate) fn partner_side_view() -> Option<PartnerSideView> {
    let (state, received_ms) = (*PARTNER_REPORT
        .lock()
        .unwrap_or_else(PoisonError::into_inner))?;
    Some(PartnerSideView {
        phase: state.phase,
        in_side_view: state.in_side_view,
        age_ms: net_clock_ms().saturating_sub(received_ms),
    })
}

pub(crate) fn partner_side_view_text() -> String {
    partner_side_view().map_or_else(
        || "no side view report from the partner yet".to_owned(),
        |view| {
            format!(
                "the partner's side view is {} in phase {}, reported {} ms ago",
                if view.in_side_view { "on" } else { "off" },
                view.phase,
                view.age_ms
            )
        },
    )
}
