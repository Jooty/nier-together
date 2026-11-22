pub const BOSS_FIGHT_PACK: &str = "A_CLIFF_VILLAGE_01";
pub const BOSS_FIGHT_KAINE_MODE: i32 = 2;
pub const FIGHT_START_JOIN_WINDOW_MS: u64 = 15_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuestSeat {
    Absent,
    Watching,
    Playing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BossDriveFacts {
    pub in_boss_fight_pack: bool,
    pub her_mode: Option<i32>,
    pub her_active: bool,
    pub guest: GuestSeat,
    pub game_poses_her: bool,
    pub now_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BossDriveEvent {
    FightBegan {
        guest: GuestSeat,
    },
    Engaged,
    HeldBackToTheGame {
        why: HeldBackWhy,
    },
    GuestLeft,
    JoinWindowClosed {
        guest: GuestSeat,
    },
    FightEnded {
        was_driving: bool,
        guest_had_left: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeldBackWhy {
    GamePosesHer,
    NotActive,
    GuestWatching,
    NoGuestYet,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct BossDrive {
    fight_live: bool,
    driving: bool,
    guest_cannot_play_her: bool,
    held_back: Option<HeldBackWhy>,
    joined: bool,
    join_until_ms: u64,
}

impl BossDrive {
    pub const fn new() -> Self {
        Self {
            fight_live: false,
            driving: false,
            guest_cannot_play_her: false,
            held_back: None,
            joined: false,
            join_until_ms: 0,
        }
    }

    pub fn boss_fight_is_live(&self) -> bool {
        self.fight_live
    }

    pub fn is_driving(&self) -> bool {
        self.driving
    }

    pub fn guest_may_play_her(&self) -> bool {
        self.fight_live && !self.guest_cannot_play_her
    }

    pub fn step(&mut self, facts: &BossDriveFacts) -> Vec<BossDriveEvent> {
        let live = facts.in_boss_fight_pack && facts.her_mode == Some(BOSS_FIGHT_KAINE_MODE);
        let mut events = Vec::new();
        if !live {
            if self.fight_live {
                events.push(BossDriveEvent::FightEnded {
                    was_driving: self.driving,
                    guest_had_left: self.guest_cannot_play_her,
                });
            }
            *self = Self::new();
            return events;
        }
        if !self.fight_live {
            self.fight_live = true;
            self.joined = false;
            self.join_until_ms = facts.now_ms.saturating_add(FIGHT_START_JOIN_WINDOW_MS);
            events.push(BossDriveEvent::FightBegan { guest: facts.guest });
        }
        if self.guest_cannot_play_her {
            self.driving = false;
            return events;
        }
        if facts.guest == GuestSeat::Playing {
            self.joined = true;
        }
        let in_join_window = !self.joined && facts.now_ms < self.join_until_ms;
        if facts.guest != GuestSeat::Playing && !self.joined && !in_join_window {
            self.guest_cannot_play_her = true;
            self.driving = false;
            self.held_back = None;
            events.push(BossDriveEvent::JoinWindowClosed { guest: facts.guest });
            return events;
        }
        if facts.guest == GuestSeat::Absent && self.joined {
            self.guest_cannot_play_her = true;
            self.driving = false;
            self.held_back = None;
            events.push(BossDriveEvent::GuestLeft);
            return events;
        }
        let held_back = if facts.guest == GuestSeat::Absent {
            Some(HeldBackWhy::NoGuestYet)
        } else if facts.guest == GuestSeat::Watching {
            Some(HeldBackWhy::GuestWatching)
        } else if facts.game_poses_her {
            Some(HeldBackWhy::GamePosesHer)
        } else if !facts.her_active {
            Some(HeldBackWhy::NotActive)
        } else {
            None
        };
        match held_back {
            None => {
                if !self.driving {
                    self.driving = true;
                    self.held_back = None;
                    events.push(BossDriveEvent::Engaged);
                }
            }
            Some(why) => {
                self.driving = false;
                if self.held_back != Some(why) {
                    self.held_back = Some(why);
                    events.push(BossDriveEvent::HeldBackToTheGame { why });
                }
            }
        }
        events
    }
}

pub fn boss_drive_event_text(event: BossDriveEvent) -> String {
    match event {
        BossDriveEvent::FightBegan { guest } => match guest {
            GuestSeat::Playing => {
                "boss drive: her mode is 2 in the fight's pack, the fight is live; the guest plays her, her AI decisions go off and she follows the guest's Kaine".to_owned()
            }
            GuestSeat::Watching => {
                "boss drive: the fight is live while the guest is not playing yet; her AI fights as the game made her, and the guest may take her over within the first seconds".to_owned()
            }
            GuestSeat::Absent => {
                "boss drive: the fight is live with no guest here yet; her AI fights as the game made her, and a guest that arrives within the first seconds may take her over".to_owned()
            }
        },
        BossDriveEvent::Engaged => {
            "boss drive: she follows the guest's Kaine now (her AI decisions are off)".to_owned()
        }
        BossDriveEvent::HeldBackToTheGame { why } => format!(
            "boss drive: her AI is back for now ({}), the drive resumes when that ends",
            match why {
                HeldBackWhy::GamePosesHer => "the game poses her in a scene or a script",
                HeldBackWhy::NotActive => "she is not active",
                HeldBackWhy::GuestWatching => "the guest is watching",
                HeldBackWhy::NoGuestYet => "no guest is here yet",
            }
        ),
        BossDriveEvent::GuestLeft => {
            "boss drive: the guest left or the link dropped, her AI is on again at the life she has and the fight carries on; a guest that comes back watches until the fight ends".to_owned()
        }
        BossDriveEvent::JoinWindowClosed { guest } => format!(
            "boss drive: the first seconds of the fight are over and the guest is {}; her AI fights it out and a guest that comes later watches until the fight ends",
            match guest {
                GuestSeat::Absent => "not here",
                GuestSeat::Watching => "still watching",
                GuestSeat::Playing => "playing",
            }
        ),
        BossDriveEvent::FightEnded {
            was_driving,
            guest_had_left,
        } => format!(
            "boss drive: the fight is over (her mode is no longer 2 here){}{}",
            if was_driving {
                ", the drive ended with it"
            } else {
                ""
            },
            if guest_had_left {
                " (the guest had left it)"
            } else {
                ""
            }
        ),
    }
}
