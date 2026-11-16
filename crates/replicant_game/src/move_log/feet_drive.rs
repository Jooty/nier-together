use crate::move_log::{
    GAMES_OWN_BODY_SPEED_RATE, HOUSE_PACE_TICKS_BEFORE_THE_FEET_DRIVE,
    INDOOR_FEET_MOVING_GROUND_SPEED, PaceFeedback, RUN_LOOP_MOTION_NAME, STATE_ACTION_MOVE_RVA,
    is_house_speed_cap_value,
};

/// One tick of the swapped body as the indoor feet drive sees it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FeetDriveReading<'a> {
    pub kaine_swapped: bool,
    pub state_rva: Option<u32>,
    pub speed_cap: f32,
    pub event_scene: bool,
    pub talk: bool,
    pub control_masked: bool,
    pub step_per_second: Option<f32>,
    pub ground_speed: f32,
    pub body_motion: Option<&'a str>,
}

/// What the drive wants this tick: her run loop played under her, and the body speed rate entry
/// it owns (the game's own 1.0 whenever the drive is not walking her).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FeetDriveStep {
    pub play_run_loop: bool,
    pub body_speed_rate: f32,
}

/// What the drive did since the last line, for the once-a-second feet line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FeetDriveTally {
    pub plays: u32,
    pub rate_writes: u32,
}

/// Her walk indoors: the game holds rate 5:2 at the house value, so, as for the first body, only
/// the motion's own root travel moves her; the drive puts her run loop under her and scales it.
#[derive(Debug, Default)]
pub struct FeetDrive {
    house_pace_ticks: u32,
    pace: PaceFeedback,
}

impl FeetDriveStep {
    pub const LEAVE: Self = Self {
        play_run_loop: false,
        body_speed_rate: GAMES_OWN_BODY_SPEED_RATE,
    };
}

impl FeetDriveTally {
    pub fn count_play(&mut self) {
        self.plays += 1;
    }

    pub fn count_rate_write(&mut self) {
        self.rate_writes += 1;
    }

    pub fn take(&mut self) -> Self {
        std::mem::take(self)
    }
}

impl FeetDrive {
    pub fn tick(&mut self, reading: &FeetDriveReading<'_>) -> FeetDriveStep {
        if !house_pace_in_play(reading) {
            self.house_pace_ticks = 0;
            return FeetDriveStep::LEAVE;
        }
        self.house_pace_ticks = self.house_pace_ticks.saturating_add(1);
        if self.house_pace_ticks < HOUSE_PACE_TICKS_BEFORE_THE_FEET_DRIVE {
            return FeetDriveStep::LEAVE;
        }
        let stick_held = reading.ground_speed >= INDOOR_FEET_MOVING_GROUND_SPEED;
        let run_loop_plays = reading
            .body_motion
            .is_some_and(|name| name.eq_ignore_ascii_case(RUN_LOOP_MOTION_NAME));
        if run_loop_plays {
            let rate = match reading.step_per_second {
                Some(step) if stick_held => self.pace.observe(step),
                _ => self.pace.rate(),
            };
            return FeetDriveStep {
                play_run_loop: false,
                body_speed_rate: rate,
            };
        }
        if stick_held {
            return FeetDriveStep {
                play_run_loop: true,
                body_speed_rate: self.pace.rate(),
            };
        }
        FeetDriveStep::LEAVE
    }

    pub fn pace(&self) -> &PaceFeedback {
        &self.pace
    }
}

pub fn house_pace_in_play(reading: &FeetDriveReading<'_>) -> bool {
    reading.kaine_swapped
        && !reading.event_scene
        && !reading.talk
        && !reading.control_masked
        && reading.state_rva == Some(STATE_ACTION_MOVE_RVA)
        && is_house_speed_cap_value(reading.speed_cap)
}
