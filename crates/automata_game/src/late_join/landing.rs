use crate::late_join::{ARRIVAL_BESIDE_AFTER_BOTH_FREE_MS, LANDING_AFTER_THE_PLACES_DIFFER_MS};
use crate::load_gate::{FIRST_SYSTEM_PHASE_ID, StoryPlace};
use crate::start::{ChapterRecord, candidates};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LandingReadings {
    pub now_ms: u64,
    pub applies: bool,
    pub can_jump: bool,
    pub own_place: StoryPlace,
    pub host_place: StoryPlace,
    pub host_in_play: bool,
    pub safe_moment: bool,
    pub host_free: bool,
    pub host_body_present: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LandingStep {
    pub land_at: Option<StoryPlace>,
    pub go_to_partner: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Stage {
    #[default]
    NotLate,
    Landing {
        differing_since_ms: Option<u64>,
    },
    Beside {
        both_free_since_ms: Option<u64>,
    },
    Done,
}

/// A game that arrives after the host lands in the host's phase, then walks to the host once.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LateLanding {
    stage: Stage,
}

impl LateLanding {
    pub fn arm(&mut self) {
        if self.stage == Stage::NotLate {
            self.stage = Stage::Landing {
                differing_since_ms: None,
            };
        }
    }

    pub fn is_armed(&self) -> bool {
        matches!(self.stage, Stage::Landing { .. } | Stage::Beside { .. })
    }

    pub fn give_up(&mut self) {
        self.stage = Stage::Done;
    }

    pub fn step(&mut self, readings: &LandingReadings) -> LandingStep {
        let mut step = LandingStep::default();
        let places_are_story = readings.own_place.has_a_name()
            && readings.host_place.has_a_name()
            && readings.own_place.phase_id < FIRST_SYSTEM_PHASE_ID
            && readings.host_place.phase_id < FIRST_SYSTEM_PHASE_ID;
        let usable = readings.applies && places_are_story && readings.host_in_play;
        let same_phase = readings.own_place.phase_id == readings.host_place.phase_id;
        match self.stage {
            Stage::NotLate | Stage::Done => {}
            Stage::Landing { .. } if !usable => {
                self.stage = Stage::Landing {
                    differing_since_ms: None,
                };
            }
            Stage::Landing { .. } if same_phase => {
                self.stage = Stage::Beside {
                    both_free_since_ms: None,
                };
            }
            Stage::Landing { differing_since_ms } => {
                let since_ms = differing_since_ms.unwrap_or(readings.now_ms);
                let long_enough =
                    readings.now_ms.saturating_sub(since_ms) >= LANDING_AFTER_THE_PLACES_DIFFER_MS;
                if long_enough && readings.safe_moment && readings.can_jump {
                    self.stage = Stage::Landing {
                        differing_since_ms: None,
                    };
                    step.land_at = Some(readings.host_place);
                } else {
                    self.stage = Stage::Landing {
                        differing_since_ms: Some(since_ms),
                    };
                }
            }
            Stage::Beside { .. } if !usable || !same_phase => {
                self.stage = Stage::Beside {
                    both_free_since_ms: None,
                };
            }
            Stage::Beside { both_free_since_ms } => {
                let both_free =
                    readings.safe_moment && readings.host_free && readings.host_body_present;
                let since_ms = both_free.then(|| both_free_since_ms.unwrap_or(readings.now_ms));
                match since_ms {
                    Some(since_ms)
                        if readings.now_ms.saturating_sub(since_ms)
                            >= ARRIVAL_BESIDE_AFTER_BOTH_FREE_MS =>
                    {
                        self.stage = Stage::Done;
                        step.go_to_partner = true;
                    }
                    _ => {
                        self.stage = Stage::Beside {
                            both_free_since_ms: since_ms,
                        };
                    }
                }
            }
        }
        step
    }
}

/// The start index of the one chapter start that holds `place`, or `None` when none or several do.
pub fn start_index_of_place(records: &[ChapterRecord], place: &StoryPlace) -> Option<usize> {
    let in_phase: Vec<_> = candidates(records)
        .into_iter()
        .filter(|candidate| i32::from(candidate.record.phase_id) == place.phase_id)
        .collect();
    match in_phase.as_slice() {
        [only] => Some(only.index),
        [] => None,
        several => {
            let named: Vec<_> = several
                .iter()
                .filter(|candidate| candidate.record.phase_name == place.sub_phase())
                .collect();
            match named.as_slice() {
                [only] => Some(only.index),
                _ => None,
            }
        }
    }
}
