use crate::host_led_stretch::{
    HostLedStretch, stretch_armed_by, stretch_starting_at, stretch_the_story_is_inside,
};

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
    InsideItTheGuestIsNotTakenAlong,
    ItsLastStep,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StretchChange {
    Armed(&'static HostLedStretch),
    Disarmed(&'static HostLedStretch, StretchEnd),
    Started(&'static HostLedStretch),
    StartedFromTheStory(&'static HostLedStretch),
    Ended(&'static HostLedStretch, StretchEnd),
}

/// Which host-led stretch is on, fed with the host's steps: on the host its own pops, on the guest
/// the steps the host sends. A stretch with an arming step is armed first and starts only after.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HostLedStretchState {
    armed: Option<(&'static HostLedStretch, u64)>,
    on: Option<(&'static HostLedStretch, u64)>,
    ended_at_its_last_step: Option<&'static HostLedStretch>,
    over_in_these_packs: Option<&'static HostLedStretch>,
}

impl HostLedStretchState {
    pub const fn new() -> Self {
        Self {
            armed: None,
            on: None,
            ended_at_its_last_step: None,
            over_in_these_packs: None,
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
            if stretch.guest_is_not_taken_along {
                return StretchPlace::InsideItTheGuestIsNotTakenAlong;
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
                self.over_in_these_packs = Some(stretch);
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
        self.over_in_these_packs = None;
        Some(StretchChange::Started(stretch))
    }

    /// Starts the stretch the story state of `pack` is inside when none is on: after a save is
    /// loaded or a story jump lands there, no live step starts it. A stretch that ended at its
    /// last step or ran past its limit is not started again until this game leaves its packs.
    pub fn start_from_the_story(
        &mut self,
        pack: &str,
        flag_on: impl Fn(u16) -> bool,
        now_ms: u64,
    ) -> Option<StretchChange> {
        if self.on.is_some() {
            return None;
        }
        let stretch = stretch_the_story_is_inside(pack, flag_on)?;
        if self.over_in_these_packs == Some(stretch) {
            return None;
        }
        self.armed = None;
        self.on = Some((stretch, now_ms));
        self.ended_at_its_last_step = None;
        Some(StretchChange::StartedFromTheStory(stretch))
    }

    /// Once per tick with the pack this game is in (`None` while it loads or is in a menu).
    pub fn check(&mut self, pack: Option<&str>, now_ms: u64) -> Option<StretchChange> {
        if let (Some(over), Some(pack)) = (self.over_in_these_packs, pack)
            && pack != over.pack
            && pack != over.last_pack
        {
            self.over_in_these_packs = None;
        }
        let Some((stretch, since_ms)) = self.on else {
            return self.check_armed(pack, now_ms);
        };
        let end = if pack.is_some_and(|pack| pack != stretch.pack && pack != stretch.last_pack) {
            StretchEnd::LeftThePack
        } else if now_ms.saturating_sub(since_ms) >= stretch.ends_itself_after_ms {
            self.over_in_these_packs = Some(stretch);
            StretchEnd::TookTooLong
        } else {
            return None;
        };
        self.on = None;
        Some(StretchChange::Ended(stretch, end))
    }

    pub fn end_with_the_session(&mut self) -> Option<StretchChange> {
        self.ended_at_its_last_step = None;
        self.over_in_these_packs = None;
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
