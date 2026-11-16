use crate::move_log::{
    BODY_SPEED_RATE_LEAST, BODY_SPEED_RATE_MOST, HOST_INDOOR_PACE_PER_SECOND,
    PACE_AVERAGE_SAMPLE_WEIGHT, PACE_BLOCKED_BELOW_SHARE_OF_THE_AIM, PACE_LARGEST_STEP_OF_THE_RATE,
    PACE_TICKS_PER_RATE_STEP, RUN_LOOP_ROOT_TRAVEL_PER_SECOND_AT_FULL_SPEED,
};

/// Keeps her body speed rate where her run loop's own root travel gives the first body's indoor
/// pace: the rate starts at the pace over the loop's full-speed travel and is corrected by the
/// steps she really takes, so the feet and the travel stay one motion whatever the game adds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PaceFeedback {
    rate: f32,
    average_step_per_second: Option<f32>,
    samples_since_last_step: u32,
}

impl Default for PaceFeedback {
    fn default() -> Self {
        Self {
            rate: starting_body_speed_rate(),
            average_step_per_second: None,
            samples_since_last_step: 0,
        }
    }
}

impl PaceFeedback {
    /// One tick of her run loop under a held stick, with the step she took this tick.
    pub fn observe(&mut self, step_per_second: f32) -> f32 {
        if !step_per_second.is_finite() {
            return self.rate;
        }
        let average = self
            .average_step_per_second
            .map_or(step_per_second, |average| {
                average + PACE_AVERAGE_SAMPLE_WEIGHT * (step_per_second - average)
            });
        self.average_step_per_second = Some(average);
        self.samples_since_last_step += 1;
        if self.samples_since_last_step < PACE_TICKS_PER_RATE_STEP {
            return self.rate;
        }
        self.samples_since_last_step = 0;
        // a body pressed against a wall travels nothing at any rate: it tells nothing about the pace
        if average < PACE_BLOCKED_BELOW_SHARE_OF_THE_AIM * HOST_INDOOR_PACE_PER_SECOND {
            return self.rate;
        }
        let wanted = (HOST_INDOOR_PACE_PER_SECOND / average).clamp(
            1.0 / PACE_LARGEST_STEP_OF_THE_RATE,
            PACE_LARGEST_STEP_OF_THE_RATE,
        );
        let rate = (self.rate * wanted).clamp(BODY_SPEED_RATE_LEAST, BODY_SPEED_RATE_MOST);
        // the travel follows the rate, so the average moves with it instead of lagging behind
        self.average_step_per_second = Some(average * rate / self.rate);
        self.rate = rate;
        rate
    }

    pub fn rate(&self) -> f32 {
        self.rate
    }

    pub fn average_step_per_second(&self) -> Option<f32> {
        self.average_step_per_second
    }
}

pub fn starting_body_speed_rate() -> f32 {
    (HOST_INDOOR_PACE_PER_SECOND / RUN_LOOP_ROOT_TRAVEL_PER_SECOND_AT_FULL_SPEED)
        .clamp(BODY_SPEED_RATE_LEAST, BODY_SPEED_RATE_MOST)
}
