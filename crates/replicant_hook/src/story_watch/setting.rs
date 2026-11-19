use std::sync::atomic::{AtomicBool, Ordering};

use crate::diagnostics::log_line;

static GUEST_WATCHES: AtomicBool = AtomicBool::new(true);

pub(crate) fn set_guest_watches(enabled: bool) {
    GUEST_WATCHES.store(enabled, Ordering::Relaxed);
    log_line(&format!(
        "watching: guest_watches_when_the_story_takes_her is {}",
        if enabled { "on" } else { "off" }
    ));
}

pub(crate) fn guest_watches_setting() -> bool {
    GUEST_WATCHES.load(Ordering::Relaxed)
}
