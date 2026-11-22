pub const SECOND_FORM_AT_OR_BELOW_PERCENT: u32 = 90;
pub const DOWN_AT_OR_BELOW_PERCENT: u32 = 60;
pub const FIGHT_WON_AT_OR_BELOW_PERCENT: u32 = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BossPhase {
    FirstForm,
    SecondForm,
    Down,
    ThirdForm,
    Won,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BossPhaseFacts {
    pub life_percent: u32,
    pub event_scene: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BossPhaseChange {
    pub from: BossPhase,
    pub to: BossPhase,
    pub life_percent: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BossPhaseWatch {
    phase: BossPhase,
    down_event_seen: bool,
}

impl Default for BossPhaseWatch {
    fn default() -> Self {
        Self::new()
    }
}

impl BossPhaseWatch {
    pub const fn new() -> Self {
        Self {
            phase: BossPhase::FirstForm,
            down_event_seen: false,
        }
    }

    pub fn phase(&self) -> BossPhase {
        self.phase
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    pub fn step(&mut self, facts: BossPhaseFacts) -> Option<BossPhaseChange> {
        let from = self.phase;
        let to = match from {
            BossPhase::FirstForm if facts.life_percent <= SECOND_FORM_AT_OR_BELOW_PERCENT => {
                BossPhase::SecondForm
            }
            BossPhase::SecondForm if facts.life_percent <= DOWN_AT_OR_BELOW_PERCENT => {
                BossPhase::Down
            }
            BossPhase::Down if facts.event_scene => {
                self.down_event_seen = true;
                BossPhase::Down
            }
            BossPhase::Down if self.down_event_seen => BossPhase::ThirdForm,
            BossPhase::ThirdForm if facts.life_percent <= FIGHT_WON_AT_OR_BELOW_PERCENT => {
                BossPhase::Won
            }
            _ => from,
        };
        self.phase = to;
        (to != from).then_some(BossPhaseChange {
            from,
            to,
            life_percent: facts.life_percent,
        })
    }
}

pub fn boss_phase_text(change: BossPhaseChange) -> String {
    let what = match change.to {
        BossPhase::FirstForm => "first form",
        BossPhase::SecondForm => "second form (her life passed 90%, no scene)",
        BossPhase::Down => "went down (her life passed 60%, the scene follows)",
        BossPhase::ThirdForm => "third form (the scene is over, her life was put back to 60%)",
        BossPhase::Won => "over (her life passed 20%, the end scene follows)",
    };
    format!(
        "boss drive: phase {what}, her life {}%",
        change.life_percent
    )
}
