use std::sync::{Mutex, PoisonError};

use replicant_game::story_watch::{GuestWord, guest_is_watching_text};
use replicant_proto::Message;

use crate::diagnostics::log_line;
use crate::event_log::{clear_persistent_event, set_persistent_event};
use crate::net::{NetThread, net_clock_ms};

static WORD: Mutex<GuestWord> = Mutex::new(GuestWord::new());
static SHOWN: Mutex<bool> = Mutex::new(false);

pub(crate) fn guest_is_watched() -> bool {
    WORD.lock()
        .unwrap_or_else(PoisonError::into_inner)
        .guest_watches(net_clock_ms())
}

pub(super) fn tick_host(net: &NetThread, guest_name: Option<&str>) {
    let now = net_clock_ms();
    let mut word = WORD.lock().unwrap_or_else(PoisonError::into_inner);
    let before = word.guest_watches(now);
    for message in net.take_relayed(|message| matches!(message, Message::GuestWatches(_))) {
        if let Message::GuestWatches(heard) = message {
            word.heard(heard.watching, now);
        }
    }
    if guest_name.is_none() {
        word.forget();
    }
    let watches = word.guest_watches(now);
    drop(word);
    if watches != before {
        log_line(&format!(
            "watching: the guest {} watching this game, its puppet, name tag and party frame entry are {}",
            if watches { "is now" } else { "is no longer" },
            if watches { "hidden" } else { "shown again" }
        ));
    }
    show_the_line(watches, guest_name);
}

pub(super) fn forget_guest() {
    WORD.lock().unwrap_or_else(PoisonError::into_inner).forget();
    show_the_line(false, None);
}

fn show_the_line(watches: bool, guest_name: Option<&str>) {
    let mut shown = SHOWN.lock().unwrap_or_else(PoisonError::into_inner);
    match (watches, guest_name) {
        (true, Some(name)) => {
            set_persistent_event(&guest_is_watching_text(name));
            *shown = true;
        }
        _ if *shown => {
            clear_persistent_event();
            *shown = false;
        }
        _ => {}
    }
}
