use crate::story_watch::{
    AERIE_PACK_PREFIX, PACKS_WHERE_THE_GUEST_PLAYS_THOUGH_THE_STORY_TAKES_HER,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WatchFacts<'a> {
    pub setting_on: bool,
    pub guest_in_session: bool,
    pub story_has_taken_her: bool,
    pub taken_at_the_aerie_exit: bool,
    pub host_leads_a_stretch: bool,
    pub pack: &'a str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WatchEnd {
    StoryBroughtHerBack,
    LeftTheSession,
    SettingOff,
    PackIsOneWhereTheGuestPlays,
    LeftTheAerieAfterTheExitRemoval,
    HostLedStretchOver,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WatchStep {
    Idle,
    Started,
    Continues,
    Ended(WatchEnd),
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct GuestWatch {
    watching: bool,
    for_a_stretch: bool,
}

impl GuestWatch {
    pub const fn new() -> Self {
        Self {
            watching: false,
            for_a_stretch: false,
        }
    }

    pub const fn watching(&self) -> bool {
        self.watching
    }

    pub const fn for_a_stretch(&self) -> bool {
        self.watching && self.for_a_stretch
    }

    pub fn step(&mut self, facts: &WatchFacts<'_>) -> WatchStep {
        let reason = end_reason(facts);
        let step = match (self.watching, reason) {
            (false, None) => {
                self.watching = true;
                WatchStep::Started
            }
            (false, Some(_)) => WatchStep::Idle,
            (true, None) => WatchStep::Continues,
            (true, Some(WatchEnd::StoryBroughtHerBack)) if self.for_a_stretch => {
                self.watching = false;
                WatchStep::Ended(WatchEnd::HostLedStretchOver)
            }
            (true, Some(end)) => {
                self.watching = false;
                WatchStep::Ended(end)
            }
        };
        self.for_a_stretch = self.watching && facts.host_leads_a_stretch;
        step
    }
}

fn end_reason(facts: &WatchFacts<'_>) -> Option<WatchEnd> {
    if !facts.setting_on {
        return Some(WatchEnd::SettingOff);
    }
    if !facts.guest_in_session {
        return Some(WatchEnd::LeftTheSession);
    }
    if facts.host_leads_a_stretch {
        return None;
    }
    if PACKS_WHERE_THE_GUEST_PLAYS_THOUGH_THE_STORY_TAKES_HER.contains(&facts.pack) {
        return Some(WatchEnd::PackIsOneWhereTheGuestPlays);
    }
    if facts.story_has_taken_her
        && facts.taken_at_the_aerie_exit
        && !facts.pack.starts_with(AERIE_PACK_PREFIX)
    {
        return Some(WatchEnd::LeftTheAerieAfterTheExitRemoval);
    }
    if !facts.story_has_taken_her {
        return Some(WatchEnd::StoryBroughtHerBack);
    }
    None
}
