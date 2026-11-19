use crate::prologue_levels::{CATCH_UP_SETTLE_MS, LocalLevel, PartnerLevel};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SwappedLevelChange {
    Raise { gain: i32 },
    BringDown,
    SetToHost,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct SwappedLevelStep {
    pub changed_by_the_game: Option<(LocalLevel, LocalLevel)>,
    pub change: Option<SwappedLevelChange>,
}

/// The swapped body's one level: raised to the host's when below it, brought down to it when
/// above, otherwise left to the experience the guest's own game earns.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct SwappedBodyLevel {
    host: Option<PartnerLevel>,
    trainer_floor: Option<PartnerLevel>,
    last_seen: Option<LocalLevel>,
    last_change_ms: Option<u64>,
    waits_for_a_host_report: bool,
}

impl SwappedBodyLevel {
    pub fn host_reported(&mut self, level: PartnerLevel) {
        self.host = Some(level);
        self.waits_for_a_host_report = false;
        if self
            .trainer_floor
            .is_some_and(|floor| level.experience >= floor.experience)
        {
            self.trainer_floor = None;
        }
    }

    pub fn host(&self) -> Option<PartnerLevel> {
        match (self.host, self.trainer_floor) {
            (Some(host), Some(floor)) if floor.experience > host.experience => Some(floor),
            (Some(host), _) => Some(host),
            (None, floor) => floor,
        }
    }

    pub fn raised_by_the_trainer(&mut self, after: LocalLevel) {
        self.trainer_floor = Some(PartnerLevel {
            level: after.level,
            experience: after.experience,
        });
        self.last_seen = Some(after);
    }

    pub fn step(&mut self, now_ms: u64, her: LocalLevel) -> SwappedLevelStep {
        let mut step = SwappedLevelStep::default();
        if let Some(seen) = self.last_seen {
            if seen.level != her.level {
                step.changed_by_the_game = Some((seen, her));
            }
            // A script that takes levels away runs here before the host's report of it arrives.
            if her.experience < seen.experience || her.level < seen.level {
                self.waits_for_a_host_report = true;
                self.trainer_floor = None;
            }
        }
        self.last_seen = Some(her);
        let Some(host) = self.host() else {
            return step;
        };
        let settled = self
            .last_change_ms
            .is_none_or(|at| now_ms.saturating_sub(at) >= CATCH_UP_SETTLE_MS);
        if !settled {
            return step;
        }
        step.change = if her.level > host.level || her.experience > host.experience {
            Some(SwappedLevelChange::BringDown)
        } else if self.waits_for_a_host_report {
            None
        } else if her.experience < host.experience {
            Some(SwappedLevelChange::Raise {
                gain: host.experience - her.experience,
            })
        } else if her.level < host.level {
            Some(SwappedLevelChange::SetToHost)
        } else {
            None
        };
        if step.change.is_some() {
            self.last_change_ms = Some(now_ms);
        }
        step
    }

    pub fn change_applied(&mut self, after: LocalLevel) {
        self.last_seen = Some(after);
    }
}

pub fn swapped_level_line(
    change: SwappedLevelChange,
    from: LocalLevel,
    to: LocalLevel,
    host: PartnerLevel,
) -> String {
    let why = match change {
        SwappedLevelChange::Raise { gain } => {
            format!("below the host's, +{gain} experience through the game")
        }
        SwappedLevelChange::BringDown => "above the host's, brought down to it".to_owned(),
        SwappedLevelChange::SetToHost => {
            "below the host's level at its experience, set to it".to_owned()
        }
    };
    format!(
        "swapped body level: {} (exp {}) -> {} (exp {}), host {} (exp {}): {why}",
        from.level, from.experience, to.level, to.experience, host.level, host.experience
    )
}
