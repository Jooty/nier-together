use crate::character::Character;
use crate::session_jump::SessionJumpOrigin;
use crate::start::{
    ARRIVAL_LIMIT_MS, ArrivalReadings, ChapterRecord, PHASE_START_RETRY_LIMIT_MS,
    POLL_LOG_INTERVAL_MS, PhaseStartCall, RUNNING_SCENE_STATE, StartPlan, build_plan, has_arrived,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionJumpCharacters {
    pub hero: Character,
    pub partner: Character,
    pub route_lead: Character,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
enum SessionJumpState {
    #[default]
    Idle,
    PlanReady(Box<StartPlan>),
    PhaseStarting {
        call: PhaseStartCall,
        since_ms: u64,
    },
    Arriving {
        phase_id: i32,
        phase_name: String,
        since_ms: u64,
        scene_left: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionJumpAction {
    Nothing,
    RunPlan(Box<StartPlan>),
    TryPhaseStart(PhaseStartCall),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionJumpStep {
    pub action: SessionJumpAction,
    pub logs: Vec<String>,
    pub landed: bool,
}

#[derive(Debug, Clone, Default)]
pub struct SessionJumpMachine {
    state: SessionJumpState,
    number: u32,
    last_poll_log_ms: Option<u64>,
}

impl SessionJumpMachine {
    pub fn is_active(&self) -> bool {
        self.state != SessionJumpState::Idle
    }

    pub fn wants_arrival_readings(&self) -> bool {
        matches!(self.state, SessionJumpState::Arriving { .. })
    }

    pub fn number(&self) -> u32 {
        self.number
    }

    pub fn begin(
        &mut self,
        record: &ChapterRecord,
        characters: SessionJumpCharacters,
        origin: SessionJumpOrigin,
    ) -> Vec<String> {
        if self.is_active() {
            return vec![format!(
                "session jump: not begun, jump #{} is still under way",
                self.number
            )];
        }
        self.number += 1;
        let plan = build_plan(
            record,
            characters.hero,
            characters.partner,
            characters.route_lead,
        );
        let line = format!(
            "session jump #{}: begins ({}), phase {:#X} {}, chapter filter {}; no restore point is made, a defeat restores the earlier one",
            self.number,
            origin.label(),
            plan.phase_start.phase_id,
            plan.phase_start.phase_name,
            plan.phase_start.chapter_filter
        );
        self.set_state(SessionJumpState::PlanReady(Box::new(plan)));
        vec![line]
    }

    pub fn step(&mut self, now_ms: u64, arrival: Option<&ArrivalReadings>) -> SessionJumpStep {
        let mut logs = Vec::new();
        let mut landed = false;
        let action = match self.state.clone() {
            SessionJumpState::Idle => SessionJumpAction::Nothing,
            SessionJumpState::PlanReady(plan) => {
                self.set_state(SessionJumpState::PhaseStarting {
                    call: plan.phase_start.clone(),
                    since_ms: now_ms,
                });
                SessionJumpAction::RunPlan(plan)
            }
            SessionJumpState::PhaseStarting { call, since_ms } => {
                if now_ms.saturating_sub(since_ms) > PHASE_START_RETRY_LIMIT_MS {
                    self.abandon(
                        "the phase start kept returning 0 for 20 s (the story host never became ready)",
                        &mut logs,
                    );
                    SessionJumpAction::Nothing
                } else {
                    SessionJumpAction::TryPhaseStart(call)
                }
            }
            SessionJumpState::Arriving {
                phase_id,
                phase_name,
                since_ms,
                scene_left,
            } => {
                if let Some(arrival) = arrival {
                    landed = self.step_arriving(
                        now_ms,
                        (phase_id, &phase_name, since_ms, scene_left),
                        arrival,
                        &mut logs,
                    );
                }
                SessionJumpAction::Nothing
            }
        };
        SessionJumpStep {
            action,
            logs,
            landed,
        }
    }

    pub fn phase_start_returned(&mut self, now_ms: u64, returned_nonzero: bool) -> Vec<String> {
        let SessionJumpState::PhaseStarting { call, .. } = self.state.clone() else {
            return Vec::new();
        };
        if !returned_nonzero {
            return Vec::new();
        }
        let line = format!(
            "session jump #{}: phase start accepted for phase {:#X} {}",
            self.number, call.phase_id, call.phase_name
        );
        self.set_state(SessionJumpState::Arriving {
            phase_id: call.phase_id,
            phase_name: call.phase_name,
            since_ms: now_ms,
            scene_left: false,
        });
        vec![line]
    }

    pub fn call_failed(&mut self, reason: &str) -> Vec<String> {
        let mut logs = Vec::new();
        self.abandon(reason, &mut logs);
        logs
    }

    fn step_arriving(
        &mut self,
        now_ms: u64,
        target: (i32, &str, u64, bool),
        arrival: &ArrivalReadings,
        logs: &mut Vec<String>,
    ) -> bool {
        let (phase_id, phase_name, since_ms, was_left) = target;
        // A jump into the phase already running shows only as the scene leaving its running state.
        let scene_left =
            was_left || arrival.scene_state != RUNNING_SCENE_STATE || arrival.phase_id != phase_id;
        if scene_left != was_left {
            logs.push(format!(
                "session jump #{}: the scene is changing ({})",
                self.number,
                arrival.describe()
            ));
            self.set_state(SessionJumpState::Arriving {
                phase_id,
                phase_name: phase_name.to_owned(),
                since_ms,
                scene_left,
            });
        }
        if scene_left && has_arrived(arrival, phase_id) {
            if arrival.player_held_by.is_some() {
                self.poll_log(
                    now_ms,
                    format!(
                        "session jump #{}: reached phase {phase_id:#X}, waiting until the player is free: {}",
                        self.number,
                        arrival.describe()
                    ),
                    logs,
                );
                return false;
            }
            logs.push(format!(
                "session jump #{}: landed in phase {phase_id:#X} {phase_name}, the player is free",
                self.number
            ));
            self.set_state(SessionJumpState::Idle);
            return true;
        }
        if now_ms.saturating_sub(since_ms) > ARRIVAL_LIMIT_MS {
            self.abandon(
                &format!(
                    "phase {phase_id:#X} was not reached after 300 s ({})",
                    arrival.describe()
                ),
                logs,
            );
            return false;
        }
        self.poll_log(
            now_ms,
            format!(
                "session jump #{}: waiting to land in phase {phase_id:#X}: {}",
                self.number,
                arrival.describe()
            ),
            logs,
        );
        false
    }

    fn abandon(&mut self, reason: &str, logs: &mut Vec<String>) {
        logs.push(format!(
            "session jump #{}: abandoned, {reason}",
            self.number
        ));
        self.set_state(SessionJumpState::Idle);
    }

    fn set_state(&mut self, state: SessionJumpState) {
        self.state = state;
        self.last_poll_log_ms = None;
    }

    fn poll_log(&mut self, now_ms: u64, line: String, logs: &mut Vec<String>) {
        let due = self
            .last_poll_log_ms
            .is_none_or(|last| now_ms.saturating_sub(last) >= POLL_LOG_INTERVAL_MS);
        if due {
            self.last_poll_log_ms = Some(now_ms);
            logs.push(line);
        }
    }
}
