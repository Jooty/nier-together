use crate::watch_pause::{
    HOST_STATE_COUNTS_AS_FRESH_FOR_MS, OWN_MENU_IS_CHECKED_AFTER_THE_HOSTS_PAUSE_ENDS_MS,
    OWN_MENU_SHOULD_HAVE_OPENED_WITHIN_MS, host_pause_applied_line, host_pause_ended_line,
    own_menu_closed_by_itself_line, own_menu_has_not_opened_line, own_menu_opened_line,
    own_menu_still_open_line,
};

/// What the watching guest reads each tick for the host's pause.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WatchPauseReadings {
    pub now_ms: u64,
    /// This copy watches the host (its hero is driven as the host's body).
    pub watches: bool,
    pub host_paused: Option<(bool, u64)>,
    /// A mirrored story event plays here: the cutscene pause rule has the pause then.
    pub a_cutscene_plays_here: bool,
    /// This copy's own pause state word reads paused.
    pub own_menu_open: bool,
}

/// Why the host's pause stopped being applied here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WatchPauseRelease {
    HostResumed,
    NoFreshStateFromTheHost,
    WatchingEnded,
    ACutscenePlaysHere,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WatchPauseStep {
    /// The game's focus test is to answer "lost", so its own pause opens and holds.
    pub hold: bool,
    /// The note "<host> paused the game" is due on screen.
    pub note: bool,
    pub logs: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Hold {
    #[default]
    Off,
    On {
        since_ms: u64,
        own_menu_seen: bool,
        own_menu_missed_logged: bool,
    },
    Ended {
        at_ms: u64,
    },
}

/// While the guest watches, every pause the host opens pauses the guest's game too.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WatchPause {
    hold: Hold,
}

impl WatchPause {
    pub fn step(&mut self, readings: &WatchPauseReadings) -> WatchPauseStep {
        let mut step = WatchPauseStep::default();
        let release = self.release_of(readings);
        match self.hold {
            Hold::On { since_ms, .. } => {
                if let Some(release) = release {
                    let held_ms = readings.now_ms.saturating_sub(since_ms);
                    step.logs.push(host_pause_ended_line(release, held_ms));
                    self.hold = Hold::Ended {
                        at_ms: readings.now_ms,
                    };
                } else {
                    self.watch_the_own_menu_open(readings, &mut step);
                }
            }
            Hold::Off | Hold::Ended { .. } if release.is_none() => {
                step.logs.push(host_pause_applied_line());
                self.hold = Hold::On {
                    since_ms: readings.now_ms,
                    own_menu_seen: false,
                    own_menu_missed_logged: false,
                };
            }
            Hold::Ended { at_ms } => self.watch_the_own_menu_close(readings, at_ms, &mut step),
            Hold::Off => {}
        }
        step.hold = matches!(self.hold, Hold::On { .. });
        step.note = step.hold;
        step
    }

    fn release_of(&self, readings: &WatchPauseReadings) -> Option<WatchPauseRelease> {
        if !readings.watches {
            return Some(WatchPauseRelease::WatchingEnded);
        }
        if readings.a_cutscene_plays_here {
            return Some(WatchPauseRelease::ACutscenePlaysHere);
        }
        match readings.host_paused {
            Some((true, age_ms)) if age_ms <= HOST_STATE_COUNTS_AS_FRESH_FOR_MS => None,
            Some((false, age_ms)) if age_ms <= HOST_STATE_COUNTS_AS_FRESH_FOR_MS => {
                Some(WatchPauseRelease::HostResumed)
            }
            _ => Some(WatchPauseRelease::NoFreshStateFromTheHost),
        }
    }

    fn watch_the_own_menu_open(
        &mut self,
        readings: &WatchPauseReadings,
        step: &mut WatchPauseStep,
    ) {
        let Hold::On {
            since_ms,
            own_menu_seen,
            own_menu_missed_logged,
        } = self.hold
        else {
            return;
        };
        let after_ms = readings.now_ms.saturating_sub(since_ms);
        let mut own_menu_missed_logged = own_menu_missed_logged;
        if !own_menu_seen && readings.own_menu_open {
            step.logs.push(own_menu_opened_line(after_ms));
        } else if !own_menu_seen
            && !own_menu_missed_logged
            && after_ms >= OWN_MENU_SHOULD_HAVE_OPENED_WITHIN_MS
        {
            step.logs.push(own_menu_has_not_opened_line(after_ms));
            own_menu_missed_logged = true;
        }
        self.hold = Hold::On {
            since_ms,
            own_menu_seen: own_menu_seen || readings.own_menu_open,
            own_menu_missed_logged,
        };
    }

    fn watch_the_own_menu_close(
        &mut self,
        readings: &WatchPauseReadings,
        ended_at_ms: u64,
        step: &mut WatchPauseStep,
    ) {
        let after_ms = readings.now_ms.saturating_sub(ended_at_ms);
        if !readings.own_menu_open {
            step.logs.push(own_menu_closed_by_itself_line());
            self.hold = Hold::Off;
        } else if after_ms >= OWN_MENU_IS_CHECKED_AFTER_THE_HOSTS_PAUSE_ENDS_MS {
            step.logs.push(own_menu_still_open_line(after_ms));
            self.hold = Hold::Off;
        }
    }
}

impl WatchPauseRelease {
    pub fn label(self) -> &'static str {
        match self {
            Self::HostResumed => "the host resumed",
            Self::NoFreshStateFromTheHost => "no fresh state from the host",
            Self::WatchingEnded => "watching ended",
            Self::ACutscenePlaysHere => "a cutscene plays here, its pause rule takes over",
        }
    }
}
