use crate::late_join::{
    BOSS_SCAN_EVERY_MS, BossReading, boss_fight_over_line, boss_hold_log_line,
    boss_unreadable_line, boss_wait_line,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WaitingJoiner {
    pub slot: u8,
    pub name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JoinerNotice {
    pub slot: u8,
    pub waiting: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BossWaitStep {
    pub logs: Vec<String>,
    pub in_game_lines: Vec<String>,
    pub notices: Vec<JoinerNotice>,
}

/// Holds a late joiner out while the guessed boss reading says a fight runs; reads only on a scan.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BossWait {
    last_scan_ms: Option<u64>,
    holding: bool,
    unreadable_logged: bool,
    told: Vec<u8>,
}

impl BossWait {
    pub fn scan_due(&self, now_ms: u64) -> bool {
        self.last_scan_ms
            .is_none_or(|last| now_ms.saturating_sub(last) >= BOSS_SCAN_EVERY_MS)
    }

    pub fn holding(&self) -> bool {
        self.holding
    }

    pub fn nobody_waits(&mut self) {
        *self = Self::default();
    }

    pub fn step(
        &mut self,
        now_ms: u64,
        waiting: &[WaitingJoiner],
        reading: BossReading,
    ) -> BossWaitStep {
        let mut step = BossWaitStep::default();
        if waiting.is_empty() {
            self.nobody_waits();
            return step;
        }
        self.last_scan_ms = Some(now_ms);
        match reading {
            BossReading::Fight(fight) => {
                self.unreadable_logged = false;
                if !self.holding {
                    step.logs.push(boss_hold_log_line(&fight));
                }
                self.holding = true;
                for joiner in waiting {
                    if !self.told.contains(&joiner.slot) {
                        self.told.push(joiner.slot);
                        step.in_game_lines.push(boss_wait_line(&joiner.name));
                        step.notices.push(JoinerNotice {
                            slot: joiner.slot,
                            waiting: true,
                        });
                    }
                }
            }
            BossReading::NoBoss => {
                self.unreadable_logged = false;
                self.release_with_a_line(&mut step);
            }
            BossReading::Unreadable(reason) => {
                if !self.unreadable_logged {
                    step.logs.push(boss_unreadable_line(&reason));
                    self.unreadable_logged = true;
                }
                self.release_with_a_line(&mut step);
            }
        }
        step
    }

    fn release_with_a_line(&mut self, step: &mut BossWaitStep) {
        if self.holding {
            step.logs.push(boss_fight_over_line());
        }
        self.holding = false;
        step.notices
            .extend(self.told.drain(..).map(|slot| JoinerNotice {
                slot,
                waiting: false,
            }));
    }
}
