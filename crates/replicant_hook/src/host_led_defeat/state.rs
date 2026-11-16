use std::sync::{Mutex, PoisonError};

use replicant_game::host_led_defeat::{GuestDefeatFollow, HostDefeatScreenWatch};
use replicant_proto::Message;

use crate::build_gate::VerifiedBuild;
use crate::diagnostics::log_line;
use crate::net::{self, net_clock_ms};

pub(super) static HOST_WATCH: Mutex<HostDefeatScreenWatch> =
    Mutex::new(HostDefeatScreenWatch::new());
pub(super) static GUEST_FOLLOW: Mutex<GuestDefeatFollow> = Mutex::new(GuestDefeatFollow::new());

pub fn tick(_build: &VerifiedBuild) {
    let Some(net) = net::handle() else {
        return;
    };
    let notes = net.take_relayed(|message| matches!(message, Message::HostDefeatScreen(_)));
    let now_ms = net_clock_ms();
    let mut follow = GUEST_FOLLOW.lock().unwrap_or_else(PoisonError::into_inner);
    for message in notes {
        if let Message::HostDefeatScreen(note) = message {
            for line in follow.hear(note, now_ms) {
                log_line(&line);
            }
        }
    }
    for line in follow.age(now_ms) {
        log_line(&line);
    }
}
