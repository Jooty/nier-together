use crate::kaine::FIGHT_WON_AT_OR_BELOW_PERCENT;

pub const OWN_LIFE_RESTORE_WAITS_FOR_PLAY_MS: u64 = 60_000;
pub const FIGHT_WON_SHARE_MARGIN_PERCENT: u32 = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OwnLifeStep {
    Leave,
    Restore { to: i32 },
}

#[derive(Debug, Default)]
pub struct OwnLifeKeeper {
    remembered: Option<i32>,
    last_boss_percent: Option<u32>,
    restore_until_ms: Option<u64>,
}

impl OwnLifeKeeper {
    pub const fn new() -> Self {
        Self {
            remembered: None,
            last_boss_percent: None,
            restore_until_ms: None,
        }
    }

    pub fn fight_began(&mut self, own_hp: i32) {
        if self.remembered.is_none() {
            self.remembered = Some(own_hp);
        }
        self.restore_until_ms = None;
        self.last_boss_percent = None;
    }

    pub fn boss_life_seen(&mut self, hp: u32, max_hp: u32) {
        let percent = (u64::from(hp.min(max_hp)) * 100).checked_div(u64::from(max_hp));
        if let Some(percent) = percent {
            self.last_boss_percent = u32::try_from(percent).ok();
        }
    }

    pub fn restore_is_pending(&self) -> bool {
        self.restore_until_ms.is_some()
    }

    pub fn remembered(&self) -> Option<i32> {
        self.remembered
    }

    pub fn fight_over(&mut self, now_ms: u64, link_lost: bool) -> bool {
        let won = self.last_boss_percent.is_some_and(|percent| {
            percent <= FIGHT_WON_AT_OR_BELOW_PERCENT + FIGHT_WON_SHARE_MARGIN_PERCENT
        });
        self.last_boss_percent = None;
        if self.remembered.is_some() && (won || link_lost) {
            self.restore_until_ms = Some(now_ms.saturating_add(OWN_LIFE_RESTORE_WAITS_FOR_PLAY_MS));
            return true;
        }
        self.remembered = None;
        self.restore_until_ms = None;
        false
    }

    pub fn step(
        &mut self,
        now_ms: u64,
        in_play: bool,
        own_hp: i32,
        own_max_hp: i32,
    ) -> OwnLifeStep {
        let Some(until) = self.restore_until_ms else {
            return OwnLifeStep::Leave;
        };
        if now_ms >= until {
            self.remembered = None;
            self.restore_until_ms = None;
            return OwnLifeStep::Leave;
        }
        if !in_play || own_hp <= 0 || own_max_hp <= 0 {
            return OwnLifeStep::Leave;
        }
        let remembered = self.remembered.take();
        self.restore_until_ms = None;
        match remembered {
            Some(hp) if hp.clamp(1, own_max_hp) != own_hp => OwnLifeStep::Restore {
                to: hp.clamp(1, own_max_hp),
            },
            _ => OwnLifeStep::Leave,
        }
    }
}
