use crate::enemies::boss_bar_probe::boss_bar_state_is_open;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BossGaugeReading {
    pub state: Option<i32>,
    pub bound_enemy: Option<String>,
    pub boss: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct BossGaugeWatch {
    last: Option<(Option<i32>, Option<String>)>,
}

impl BossGaugeWatch {
    pub const fn new() -> Self {
        Self { last: None }
    }

    pub fn observe(&mut self, side: &str, reading: &BossGaugeReading) -> Option<String> {
        let now = (reading.state, reading.bound_enemy.clone());
        if self.last.as_ref() == Some(&now) {
            return None;
        }
        self.last = Some(now);
        let state = reading.state.map_or_else(
            || "unreadable".to_owned(),
            |state| {
                let word = if boss_bar_state_is_open(state) {
                    "open"
                } else {
                    "not open"
                };
                format!("{word} (state {state})")
            },
        );
        Some(format!(
            "boss gauge ({side}): {state}, bound to {}, the fight's boss is {}",
            reading
                .bound_enemy
                .as_deref()
                .unwrap_or("no enemy of the roster"),
            reading.boss.as_deref().unwrap_or("none"),
        ))
    }

    pub fn forget(&mut self) {
        self.last = None;
    }
}
