use crate::move_log::{
    INDOOR_FEET_LEAST_MOTION_SPEED, INDOOR_FEET_MOST_MOTION_SPEED,
    INDOOR_FEET_MOVING_STEP_PER_SECOND, INDOOR_FEET_SPEED_TOLERANCE,
    RUN_LOOP_FULL_SPEED_GROUND_UNITS_PER_SECOND, RUN_LOOP_MOTION_NAME, SPEED_CAP_LIFTED_VALUE,
    STATE_ACTION_MOVE_RVA, is_house_speed_cap_value,
};

/// One tick of the swapped body as the indoor feet drive sees it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FeetDriveReading<'a> {
    pub kaine_swapped: bool,
    pub state_rva: Option<u32>,
    pub speed_cap: f32,
    pub lifts_this_phase: u32,
    pub event_scene: bool,
    pub talk: bool,
    pub control_masked: bool,
    pub step_per_second: Option<f32>,
    pub ground_speed: f32,
    pub body_motion: Option<&'a str>,
    pub body_motion_speed: Option<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FeetDriveStep {
    Leave,
    /// Her body is moving on the house pace without her run loop under it: play the loop at `speed`.
    PlayRunLoop {
        speed: f32,
    },
    /// Her run loop plays but at a speed that does not match the pace: write `speed`.
    HoldSpeed {
        speed: f32,
    },
}

/// What the drive did since the last line, for the once-a-second feet line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FeetDriveTally {
    pub plays: u32,
    pub speed_holds: u32,
}

impl FeetDriveTally {
    pub fn count(&mut self, step: FeetDriveStep) {
        match step {
            FeetDriveStep::PlayRunLoop { .. } => self.plays += 1,
            FeetDriveStep::HoldSpeed { .. } => self.speed_holds += 1,
            FeetDriveStep::Leave => {}
        }
    }

    pub fn take(&mut self) -> Self {
        std::mem::take(self)
    }
}

/// The run loop's speed that puts its stride at `ground_speed`: the loop at 1.0 is her full run.
pub fn run_loop_speed_for(ground_speed: f32) -> f32 {
    (ground_speed / RUN_LOOP_FULL_SPEED_GROUND_UNITS_PER_SECOND).clamp(
        INDOOR_FEET_LEAST_MOTION_SPEED,
        INDOOR_FEET_MOST_MOTION_SPEED,
    )
}

pub fn house_pace_in_play(reading: &FeetDriveReading<'_>) -> bool {
    reading.kaine_swapped
        && !reading.event_scene
        && !reading.talk
        && !reading.control_masked
        && reading.state_rva == Some(STATE_ACTION_MOVE_RVA)
        && reading.lifts_this_phase > 0
        && (is_house_speed_cap_value(reading.speed_cap)
            || (reading.speed_cap - SPEED_CAP_LIFTED_VALUE).abs() < f32::EPSILON)
}

pub fn feet_drive_step(reading: &FeetDriveReading<'_>) -> FeetDriveStep {
    if !house_pace_in_play(reading) {
        return FeetDriveStep::Leave;
    }
    let moving = reading
        .step_per_second
        .is_some_and(|step| step >= INDOOR_FEET_MOVING_STEP_PER_SECOND);
    if !moving {
        return FeetDriveStep::Leave;
    }
    let speed = run_loop_speed_for(reading.ground_speed);
    let run_loop_plays = reading
        .body_motion
        .is_some_and(|name| name.eq_ignore_ascii_case(RUN_LOOP_MOTION_NAME));
    if !run_loop_plays {
        return FeetDriveStep::PlayRunLoop { speed };
    }
    match reading.body_motion_speed {
        Some(current) if (current - speed).abs() <= INDOOR_FEET_SPEED_TOLERANCE => {
            FeetDriveStep::Leave
        }
        _ => FeetDriveStep::HoldSpeed { speed },
    }
}
