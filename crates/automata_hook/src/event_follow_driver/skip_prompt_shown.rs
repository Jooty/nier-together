use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use automata_game::event_follow::SkipPrompt;

static SHOWN_SKIP_PROMPT: Mutex<Option<SkipPrompt>> = Mutex::new(None);
static HOST_PAUSE_NOTE_DUE: AtomicBool = AtomicBool::new(false);

pub(super) fn set_skip_prompt(prompt: Option<SkipPrompt>) {
    if let Ok(mut shown) = SHOWN_SKIP_PROMPT.lock() {
        *shown = prompt;
    }
}

/// Game thread: whether "<host> paused the game" is due on the watching guest's screen.
pub fn set_host_pause_note_due(due: bool) {
    HOST_PAUSE_NOTE_DUE.store(due, Ordering::Relaxed);
}

/// The line to draw this frame, if any: a skip vote's line wins over the host's pause note.
pub fn shown_skip_prompt() -> Option<SkipPrompt> {
    let vote = SHOWN_SKIP_PROMPT.lock().ok().and_then(|shown| *shown);
    vote.or_else(|| {
        HOST_PAUSE_NOTE_DUE
            .load(Ordering::Relaxed)
            .then_some(SkipPrompt::PartnerPausedTheGame)
    })
}
