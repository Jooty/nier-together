use crate::host_led_stretch::{HostLedStretch, stretch_armed_by, stretch_starting_at};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StretchEnd {
    LastStep,
    LeftThePack,
    TookTooLong,
    SessionOver,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StretchPlace {
    NotInOne,
    InsideIt,
    ItsLastStep,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StretchChange {
    Armed(&'static HostLedStretch),
    Disarmed(&'static HostLedStretch, StretchEnd),
    Started(&'static HostLedStretch),
    Ended(&'static HostLedStretch, StretchEnd),
}

/// Which host-led stretch is on, fed with the host's steps: on the host its own pops, on the guest
/// the steps the host sends. A stretch with an arming step is armed first and starts only after.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HostLedStretchState {
    armed: Option<(&'static HostLedStretch, u64)>,
    on: Option<(&'static HostLedStretch, u64)>,
    ended_at_its_last_step: Option<&'static HostLedStretch>,
}

impl HostLedStretchState {
    pub const fn new() -> Self {
        Self {
            armed: None,
            on: None,
            ended_at_its_last_step: None,
        }
    }

    pub fn current(&self) -> Option<&'static HostLedStretch> {
        self.on.map(|(stretch, _)| stretch)
    }

    pub fn armed(&self) -> Option<&'static HostLedStretch> {
        self.armed.map(|(stretch, _)| stretch)
    }

    /// Where the host's End `step` falls: inside the stretch the guest watches now, its last
    /// step (the stretch just ended there), or neither.
    pub fn place_of_end(&self, step: &str) -> StretchPlace {
        if let Some((stretch, _)) = self.on {
            if stretch.last_step == step {
                return StretchPlace::ItsLastStep;
            }
            return StretchPlace::InsideIt;
        }
        match self.ended_at_its_last_step {
            Some(stretch) if stretch.last_step == step => StretchPlace::ItsLastStep,
            _ => StretchPlace::NotInOne,
        }
    }

    pub fn door_ban(&self) -> Option<&'static HostLedStretch> {
        self.on
            .or(self.armed)
            .map(|(stretch, _)| stretch)
            .filter(|stretch| stretch.refuses_guest_doors)
    }

    pub fn host_step(&mut self, pack: &str, step: &str, now_ms: u64) -> Option<StretchChange> {
        if let Some((stretch, _)) = self.on {
            if stretch.last_pack == pack && stretch.last_step == step {
                self.on = None;
                self.ended_at_its_last_step = Some(stretch);
                return Some(StretchChange::Ended(stretch, StretchEnd::LastStep));
            }
            return None;
        }
        if let Some(stretch) = stretch_armed_by(pack, step) {
            self.armed = Some((stretch, now_ms));
            return Some(StretchChange::Armed(stretch));
        }
        let stretch = stretch_starting_at(pack, step)?;
        if stretch.armed_by.is_some() && self.armed() != Some(stretch) {
            return None;
        }
        self.armed = None;
        self.on = Some((stretch, now_ms));
        self.ended_at_its_last_step = None;
        Some(StretchChange::Started(stretch))
    }

    /// Once per tick with the pack this game is in (`None` while it loads or is in a menu).
    pub fn check(&mut self, pack: Option<&str>, now_ms: u64) -> Option<StretchChange> {
        let Some((stretch, since_ms)) = self.on else {
            return self.check_armed(pack, now_ms);
        };
        let end = if pack.is_some_and(|pack| pack != stretch.pack && pack != stretch.last_pack) {
            StretchEnd::LeftThePack
        } else if now_ms.saturating_sub(since_ms) >= stretch.ends_itself_after_ms {
            StretchEnd::TookTooLong
        } else {
            return None;
        };
        self.on = None;
        Some(StretchChange::Ended(stretch, end))
    }

    pub fn end_with_the_session(&mut self) -> Option<StretchChange> {
        self.ended_at_its_last_step = None;
        if let Some((stretch, _)) = self.on.take() {
            self.armed = None;
            return Some(StretchChange::Ended(stretch, StretchEnd::SessionOver));
        }
        let (stretch, _) = self.armed.take()?;
        Some(StretchChange::Disarmed(stretch, StretchEnd::SessionOver))
    }

    fn check_armed(&mut self, pack: Option<&str>, now_ms: u64) -> Option<StretchChange> {
        let (stretch, since_ms) = self.armed?;
        let end = if pack.is_some_and(|pack| pack != stretch.pack) {
            StretchEnd::LeftThePack
        } else if now_ms.saturating_sub(since_ms) >= stretch.ends_itself_after_ms {
            StretchEnd::TookTooLong
        } else {
            return None;
        };
        self.armed = None;
        Some(StretchChange::Disarmed(stretch, end))
    }
}
