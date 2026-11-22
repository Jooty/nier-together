use core::f32::consts::{PI, TAU};

use crate::partner::{
    CORRECTION_REPORT_INTERVAL_MS, FACING_ERROR_WRITTEN_RADIANS, FIRST_CORRECTION_TICK_MS,
    MOVING_POSITION_ERROR_LEFT_ALONE, MOVING_VELOCITY_ABOVE_UNITS_PER_SECOND,
    POSITION_ERROR_LEFT_ALONE, POSITION_ERROR_TELEPORTED, POSITION_NUDGE_FRACTION,
    PREDICTION_AGE_CAP_MS, SAMPLE_INTERVAL_CEILING_MS, SAMPLE_INTERVAL_FLOOR_MS,
    TELEPORT_MIN_INTERVAL_MS, VELOCITY_BLEND_FRACTION, VELOCITY_GAP_LIMIT_MS,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PositionCorrection {
    Leave,
    Nudge([f32; 3]),
    Teleport,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PositionCorrector {
    last_teleport_ms: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PositionSample {
    pub sequence: u32,
    pub position: [f32; 3],
    pub input_speed: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CorrectionReport {
    pub left_alone: u32,
    pub nudged: u32,
    pub teleported: u32,
    pub largest_error: f32,
    pub samples_per_second: f32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct SeenSample {
    sequence: u32,
    position: [f32; 3],
    seen_ms: u64,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct CorrectionCounts {
    window_started_ms: Option<u64>,
    left_alone: u32,
    nudged: u32,
    teleported: u32,
    largest_error: f32,
    samples: u32,
}

/// Corrects a partner body toward where the other player is now: the newest sample's position
/// plus its measured velocity over the sample's age, spread over the time to the next sample.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SampleCorrector {
    sample: Option<SeenSample>,
    velocity_per_second: [f32; 3],
    window_ms: u64,
    last_tick_ms: Option<u64>,
    last_teleport_ms: Option<u64>,
    last_target: Option<[f32; 3]>,
    counts: CorrectionCounts,
}

impl PositionCorrector {
    pub fn correct(
        &mut self,
        now_ms: u64,
        local: [f32; 3],
        remote: [f32; 3],
    ) -> PositionCorrection {
        let error = [
            remote[0] - local[0],
            remote[1] - local[1],
            remote[2] - local[2],
        ];
        let distance = (error[0] * error[0] + error[1] * error[1] + error[2] * error[2]).sqrt();
        if !distance.is_finite() || distance < POSITION_ERROR_LEFT_ALONE {
            return PositionCorrection::Leave;
        }
        let teleport_allowed = self
            .last_teleport_ms
            .is_none_or(|last| now_ms.saturating_sub(last) >= TELEPORT_MIN_INTERVAL_MS);
        if distance > POSITION_ERROR_TELEPORTED && teleport_allowed {
            self.last_teleport_ms = Some(now_ms);
            return PositionCorrection::Teleport;
        }
        PositionCorrection::Nudge([
            local[0] + error[0] * POSITION_NUDGE_FRACTION,
            local[1] + error[1] * POSITION_NUDGE_FRACTION,
            local[2] + error[2] * POSITION_NUDGE_FRACTION,
        ])
    }
}

impl SampleCorrector {
    pub fn correct(
        &mut self,
        now_ms: u64,
        local: [f32; 3],
        sample: PositionSample,
    ) -> PositionCorrection {
        self.take_sample(now_ms, sample);
        let tick_ms = self
            .last_tick_ms
            .map_or(FIRST_CORRECTION_TICK_MS, |last| now_ms.saturating_sub(last))
            .max(1);
        self.last_tick_ms = Some(now_ms);
        let Some(seen) = self.sample else {
            return PositionCorrection::Leave;
        };
        let age_ms = now_ms.saturating_sub(seen.seen_ms);
        let ahead_seconds = age_ms.min(PREDICTION_AGE_CAP_MS) as f32 / 1000.0;
        let target = [
            seen.position[0] + self.velocity_per_second[0] * ahead_seconds,
            seen.position[1] + self.velocity_per_second[1] * ahead_seconds,
            seen.position[2] + self.velocity_per_second[2] * ahead_seconds,
        ];
        self.last_target = Some(target);
        let error = [
            target[0] - local[0],
            target[1] - local[1],
            target[2] - local[2],
        ];
        let distance = length(error);
        if !distance.is_finite() {
            return self.counted(PositionCorrection::Leave, 0.0);
        }
        let speed = length(self.velocity_per_second);
        let moving = sample.input_speed > 0.0 || speed > MOVING_VELOCITY_ABOVE_UNITS_PER_SECOND;
        let dead_band = if moving {
            MOVING_POSITION_ERROR_LEFT_ALONE
        } else {
            POSITION_ERROR_LEFT_ALONE
        };
        if distance < dead_band {
            return self.counted(PositionCorrection::Leave, distance);
        }
        let teleport_allowed = self
            .last_teleport_ms
            .is_none_or(|last| now_ms.saturating_sub(last) >= TELEPORT_MIN_INTERVAL_MS);
        if distance > POSITION_ERROR_TELEPORTED && teleport_allowed {
            self.last_teleport_ms = Some(now_ms);
            return self.counted(PositionCorrection::Teleport, distance);
        }
        let window_ends_ms = seen.seen_ms + self.window_ms.max(SAMPLE_INTERVAL_FLOOR_MS);
        let remaining_ms = window_ends_ms.saturating_sub(now_ms);
        let fraction = if remaining_ms > 0 {
            (tick_ms as f32 / remaining_ms as f32).min(1.0)
        } else if distance > POSITION_ERROR_TELEPORTED {
            POSITION_NUDGE_FRACTION
        } else {
            return self.counted(PositionCorrection::Leave, distance);
        };
        let mut step = [
            error[0] * fraction,
            error[1] * fraction,
            error[2] * fraction,
        ];
        if speed > MOVING_VELOCITY_ABOVE_UNITS_PER_SECOND {
            let direction = [
                self.velocity_per_second[0] / speed,
                self.velocity_per_second[1] / speed,
                self.velocity_per_second[2] / speed,
            ];
            let along = step[0] * direction[0] + step[1] * direction[1] + step[2] * direction[2];
            if along < 0.0 {
                // A nudge never pulls the body back against the way it is running.
                for (step_on_axis, direction_on_axis) in step.iter_mut().zip(direction) {
                    *step_on_axis -= along * direction_on_axis;
                }
            }
        }
        let nudge = [local[0] + step[0], local[1] + step[1], local[2] + step[2]];
        self.counted(PositionCorrection::Nudge(nudge), distance)
    }

    /// Where the other player's body is judged to be now, as of the last `correct`.
    pub fn target(&self) -> Option<[f32; 3]> {
        self.last_target
    }

    /// The last second's corrections, once a second has passed since the last report.
    pub fn take_report(&mut self, now_ms: u64) -> Option<CorrectionReport> {
        let started = self.counts.window_started_ms?;
        let elapsed_ms = now_ms.saturating_sub(started);
        if elapsed_ms < CORRECTION_REPORT_INTERVAL_MS {
            return None;
        }
        let counts = core::mem::take(&mut self.counts);
        Some(CorrectionReport {
            left_alone: counts.left_alone,
            nudged: counts.nudged,
            teleported: counts.teleported,
            largest_error: counts.largest_error,
            samples_per_second: counts.samples as f32 * 1000.0 / elapsed_ms as f32,
        })
    }

    fn counted(&mut self, correction: PositionCorrection, distance: f32) -> PositionCorrection {
        match correction {
            PositionCorrection::Leave => self.counts.left_alone += 1,
            PositionCorrection::Nudge(_) => self.counts.nudged += 1,
            PositionCorrection::Teleport => self.counts.teleported += 1,
        }
        if distance.is_finite() && distance > self.counts.largest_error {
            self.counts.largest_error = distance;
        }
        correction
    }

    fn take_sample(&mut self, now_ms: u64, sample: PositionSample) {
        if self.counts.window_started_ms.is_none() {
            self.counts.window_started_ms = Some(now_ms);
        }
        if self
            .sample
            .is_some_and(|seen| seen.sequence == sample.sequence)
        {
            return;
        }
        self.counts.samples += 1;
        let position_is_usable = sample.position.iter().all(|axis| axis.is_finite());
        if let Some(previous) = self.sample
            && position_is_usable
        {
            self.learn_velocity(now_ms, &previous, sample.position);
        }
        if position_is_usable {
            self.sample = Some(SeenSample {
                sequence: sample.sequence,
                position: sample.position,
                seen_ms: now_ms,
            });
        }
    }

    fn learn_velocity(&mut self, now_ms: u64, previous: &SeenSample, position: [f32; 3]) {
        let gap_ms = now_ms.saturating_sub(previous.seen_ms);
        let moved = [
            position[0] - previous.position[0],
            position[1] - previous.position[1],
            position[2] - previous.position[2],
        ];
        if gap_ms == 0 {
            return;
        }
        if gap_ms > VELOCITY_GAP_LIMIT_MS || length(moved) > POSITION_ERROR_TELEPORTED * 2.0 {
            self.velocity_per_second = [0.0; 3];
            self.window_ms = SAMPLE_INTERVAL_FLOOR_MS;
            return;
        }
        let seconds = gap_ms as f32 / 1000.0;
        for (velocity, moved_on_axis) in self.velocity_per_second.iter_mut().zip(moved) {
            let measured = moved_on_axis / seconds;
            *velocity =
                *velocity * (1.0 - VELOCITY_BLEND_FRACTION) + measured * VELOCITY_BLEND_FRACTION;
        }
        let blended_window = (self.window_ms.max(SAMPLE_INTERVAL_FLOOR_MS) + gap_ms) / 2;
        self.window_ms = blended_window.clamp(SAMPLE_INTERVAL_FLOOR_MS, SAMPLE_INTERVAL_CEILING_MS);
    }
}

pub fn position_correction_line(owner: &str, report: &CorrectionReport) -> String {
    format!(
        "{owner}: position corrections in the last second: {} left alone, {} nudged, {} teleported; largest error {:.2}; {:.1} samples a second",
        report.left_alone,
        report.nudged,
        report.teleported,
        report.largest_error,
        report.samples_per_second
    )
}

fn length(vector: [f32; 3]) -> f32 {
    (vector[0] * vector[0] + vector[1] * vector[1] + vector[2] * vector[2]).sqrt()
}

/// The shortest signed turn from `local` to `remote`, in radians.
pub fn facing_error(local: f32, remote: f32) -> f32 {
    let turn = (remote - local).rem_euclid(TAU);
    if turn > PI { turn - TAU } else { turn }
}

pub fn facing_needs_write(local: f32, remote: f32) -> bool {
    let error = facing_error(local, remote);
    error.is_finite() && error.abs() > FACING_ERROR_WRITTEN_RADIANS
}
