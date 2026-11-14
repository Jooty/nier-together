use crate::story_watch::PACKS_WHERE_THE_GUEST_PLAYS_THOUGH_THE_STORY_TAKES_HER;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WatchFacts<'a> {
    pub setting_on: bool,
    pub guest_in_session: bool,
    pub story_has_taken_her: bool,
    pub pack: &'a str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WatchEnd {
    StoryBroughtHerBack,
    LeftTheSession,
    SettingOff,
    PackIsOneWhereTheGuestPlays,
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
}

impl GuestWatch {
    pub const fn new() -> Self {
        Self { watching: false }
    }

    pub const fn watching(&self) -> bool {
        self.watching
    }

    pub fn step(&mut self, facts: &WatchFacts<'_>) -> WatchStep {
        let reason = end_reason(facts);
        match (self.watching, reason) {
            (false, None) => {
                self.watching = true;
                WatchStep::Started
            }
            (false, Some(_)) => WatchStep::Idle,
            (true, None) => WatchStep::Continues,
            (true, Some(end)) => {
                self.watching = false;
                WatchStep::Ended(end)
            }
        }
    }
}

fn end_reason(facts: &WatchFacts<'_>) -> Option<WatchEnd> {
    if !facts.setting_on {
        return Some(WatchEnd::SettingOff);
    }
    if !facts.guest_in_session {
        return Some(WatchEnd::LeftTheSession);
    }
    if PACKS_WHERE_THE_GUEST_PLAYS_THOUGH_THE_STORY_TAKES_HER.contains(&facts.pack) {
        return Some(WatchEnd::PackIsOneWhereTheGuestPlays);
    }
    if !facts.story_has_taken_her {
        return Some(WatchEnd::StoryBroughtHerBack);
    }
    None
}
