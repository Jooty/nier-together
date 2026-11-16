use crate::callback_gate::{
    END_FOLLOW_EVENT_SUFFIX, END_RUNS_ANYWAY_AFTER_WAITING_MS,
    END_RUNS_UNDER_ITS_OWN_STARTS_PAUSE_AFTER_MS, ENDS_THAT_RUN_ANYWAY_AFTER_WAITING,
    ENDS_THAT_RUN_UNDER_THEIR_OWN_STARTS_PAUSE, GATED_EVENT_SUFFIX,
};

/// The last `_PauseOn` this game's scripts made, by the step that made it and the pause mask it
/// left; none once a `_PauseOff` ran after it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LastPauseOn {
    pub step: String,
    pub mask_after: u32,
}

/// What the guest knows, once per tick, about an End of the host's that waits for its controls.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OwnStartPauseFacts<'a> {
    pub end_name: &'a str,
    pub waited_ms: u64,
    pub last_pause_on: Option<&'a LastPauseOn>,
    /// The four control slots, with the talk bits the mod's own watcher hold wrote taken out.
    pub slots: [u32; 4],
    pub menu_open: bool,
    pub script_slot_idle: bool,
    pub talk_window_open: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OwnStartPauseRelease {
    OnlyItsOwnStartsPauseHolds,
    RunsAnywayAfterWaiting { waited_ms: u64 },
}

impl OwnStartPauseRelease {
    pub fn describe(self, end_name: &str) -> String {
        match self {
            Self::OnlyItsOwnStartsPauseHolds => format!(
                "the only pause held is the one your own {} set, and your script slot is idle",
                own_start_of_end(end_name).unwrap_or_default()
            ),
            Self::RunsAnywayAfterWaiting { waited_ms } => format!(
                "it waited {}s for your controls with your script slot idle, so it runs anyway (bounded release, the guest is never left held)",
                waited_ms / 1000
            ),
        }
    }
}

/// The Start step of the same event, whose pause an End (or a numbered End such as `_End02`)
/// lifts: `EID_x_010_01_End02` -> `EID_x_010_01_Start`.
pub fn own_start_of_end(end_name: &str) -> Option<String> {
    let base = end_name
        .trim_end_matches(|character: char| character.is_ascii_digit())
        .strip_suffix(END_FOLLOW_EVENT_SUFFIX)?;
    Some(format!("{base}{GATED_EVENT_SUFFIX}"))
}

/// Whether a listed End of the host's runs here although this game's control mask is not free.
/// The scripts of such an End lift the pause its own Start set, so waiting for a free mask never
/// ends (run 1116a guest 08:30:22 to 08:31:22).
pub fn own_start_pause_release(facts: &OwnStartPauseFacts<'_>) -> Option<OwnStartPauseRelease> {
    if !facts.script_slot_idle || facts.talk_window_open || facts.menu_open {
        return None;
    }
    if ENDS_THAT_RUN_UNDER_THEIR_OWN_STARTS_PAUSE.contains(&facts.end_name)
        && facts.waited_ms >= END_RUNS_UNDER_ITS_OWN_STARTS_PAUSE_AFTER_MS
        && only_its_own_starts_pause_holds(facts)
    {
        return Some(OwnStartPauseRelease::OnlyItsOwnStartsPauseHolds);
    }
    (ENDS_THAT_RUN_ANYWAY_AFTER_WAITING.contains(&facts.end_name)
        && facts.waited_ms >= END_RUNS_ANYWAY_AFTER_WAITING_MS)
        .then_some(OwnStartPauseRelease::RunsAnywayAfterWaiting {
            waited_ms: facts.waited_ms,
        })
}

fn only_its_own_starts_pause_holds(facts: &OwnStartPauseFacts<'_>) -> bool {
    let Some(start) = own_start_of_end(facts.end_name) else {
        return false;
    };
    let Some(pause) = facts.last_pause_on else {
        return false;
    };
    let [pause_slot, talk_slot, save_slot, message_slot] = facts.slots;
    pause.step == start
        && pause.mask_after != 0
        && pause_slot == pause.mask_after
        && talk_slot | save_slot | message_slot == 0
}
