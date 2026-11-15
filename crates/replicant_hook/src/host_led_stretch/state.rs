use std::sync::{Mutex, PoisonError};

use replicant_game::Replicant;
use replicant_game::host_led_stretch::{
    GUEST_REFUSED_DOOR_CALLBACK_PREFIX, HostLedStretch, HostLedStretchState, StretchChange,
    StretchPlace, armed_line, disarmed_line, ended_line, started_line,
    stretch_the_guest_waits_to_start,
};
use replicant_game::shared_events::phase_name;
use replicant_game::stay_together::FollowRole;
use replicant_proto::{CallbackFired, Message};

use crate::build_gate::VerifiedBuild;
use crate::diagnostics::log_line;
use crate::memory::InProcessMemory;
use crate::net::{self, NetState, net_clock_ms};

static STRETCH: Mutex<HostLedStretchState> = Mutex::new(HostLedStretchState::new());

pub(crate) fn note_own_step(name: &str, the_gate_sends_it: bool) {
    let Some(net) = net::handle() else {
        return;
    };
    let view = net.view();
    if FollowRole::from_slot(view.local_slot) != FollowRole::Host
        || view.state != NetState::Connected
    {
        return;
    }
    let Some(phase) = view.local.as_ref().map(|local| local.phase) else {
        return;
    };
    let Some(pack) = phase_name(phase) else {
        return;
    };
    let change = STRETCH
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .host_step(pack, name, net_clock_ms());
    let Some(change) = change else {
        return;
    };
    report(change, true, name);
    // The step's own fade out runs before the next tick would arm the fade sync.
    match change {
        StretchChange::Started(_) => crate::fade_sync::arm_host_led_stretch(true),
        StretchChange::Ended(..) => crate::fade_sync::arm_host_led_stretch(false),
        StretchChange::Armed(_) | StretchChange::Disarmed(..) => {}
    }
    // The callback gate never sends a talk the host started, so its start goes from here.
    if let StretchChange::Started(_) = change
        && !the_gate_sends_it
    {
        net.send_relayed(Message::CallbackFired(CallbackFired {
            phase,
            name: name.to_owned(),
        }));
        log_line(&format!(
            "host-led stretch: {name} was started by your talk, sent to the guest as the stretch's start"
        ));
    }
}

pub(crate) fn note_hosts_step(phase: u16, name: &str) {
    let Some(net) = net::handle() else {
        return;
    };
    let view = net.view();
    if FollowRole::from_slot(view.local_slot) != FollowRole::Guest {
        return;
    }
    let Some(pack) = phase_name(phase) else {
        return;
    };
    let here = view.local.as_ref().map(|local| local.phase) == Some(phase);
    let mut stretch = STRETCH.lock().unwrap_or_else(PoisonError::into_inner);
    if !here && stretch.current().is_none() {
        return;
    }
    let change = stretch.host_step(pack, name, net_clock_ms());
    drop(stretch);
    if let Some(change) = change {
        report(change, false, name);
    }
}

pub(crate) fn tick(build: &VerifiedBuild) {
    let connected = net::handle().is_some_and(|net| net.view().state == NetState::Connected);
    let change = if connected {
        let pack = Replicant::new(InProcessMemory::new(build))
            .current_phase()
            .ok();
        STRETCH
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .check(pack.as_deref(), net_clock_ms())
    } else {
        STRETCH
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .end_with_the_session()
    };
    if let Some(change) = change {
        let host = net::handle()
            .is_some_and(|net| FollowRole::from_slot(net.view().local_slot) == FollowRole::Host);
        report(change, host, "");
    }
    let host_leads = hosts_stretch_first_step().is_some();
    crate::fade_sync::arm_host_led_stretch(host_leads);
}

pub(crate) fn hosts_stretch_first_step() -> Option<&'static str> {
    let host = net::handle()
        .is_some_and(|net| FollowRole::from_slot(net.view().local_slot) == FollowRole::Host);
    if !host {
        return None;
    }
    STRETCH
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .current()
        .map(|stretch| stretch.first_step)
}

pub(crate) fn guest_watches_a_stretch() -> bool {
    let guest = net::handle()
        .is_some_and(|net| FollowRole::from_slot(net.view().local_slot) == FollowRole::Guest);
    guest
        && STRETCH
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .current()
            .is_some()
}

pub(crate) fn place_of_hosts_end(name: &str) -> StretchPlace {
    if !is_the_guest() {
        return StretchPlace::NotInOne;
    }
    STRETCH
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .place_of_end(name)
}

pub(crate) fn first_step_the_guest_leaves_to_the_host(
    callback: &str,
) -> Option<&'static HostLedStretch> {
    let net = net::handle()?;
    let view = net.view();
    if FollowRole::from_slot(view.local_slot) != FollowRole::Guest
        || view.state != NetState::Connected
    {
        return None;
    }
    let pack = phase_name(view.local.as_ref()?.phase)?;
    let stretch_on = STRETCH
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .current()
        .is_some();
    if stretch_on {
        return None;
    }
    stretch_the_guest_waits_to_start(pack, callback)
}

pub(crate) fn guest_door_ban(callback: &str) -> Option<&'static HostLedStretch> {
    if !callback.starts_with(GUEST_REFUSED_DOOR_CALLBACK_PREFIX) || !is_the_guest() {
        return None;
    }
    STRETCH
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .door_ban()
}

pub(crate) fn host_runs_are_not_replayed(phase: u16) -> Option<&'static HostLedStretch> {
    if !is_the_guest() {
        return None;
    }
    let stretch = STRETCH
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .current()?;
    (stretch.guest_is_not_taken_along && phase_name(phase) == Some(stretch.pack)).then_some(stretch)
}

pub(crate) fn guest_is_not_taken_along_now() -> Option<&'static HostLedStretch> {
    STRETCH
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .current()
        .filter(|stretch| stretch.guest_is_not_taken_along)
}

fn is_the_guest() -> bool {
    net::handle()
        .is_some_and(|net| FollowRole::from_slot(net.view().local_slot) == FollowRole::Guest)
}

fn report(change: StretchChange, host: bool, step: &str) {
    log_line(&match change {
        StretchChange::Armed(stretch) => armed_line(stretch, step),
        StretchChange::Disarmed(stretch, end) => disarmed_line(stretch, end),
        StretchChange::Started(stretch) => started_line(stretch, host),
        StretchChange::Ended(stretch, end) => ended_line(stretch, end),
    });
}
