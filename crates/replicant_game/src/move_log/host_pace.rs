use crate::move_log::{
    HOST_PACE_AVERAGE_SAMPLE_WEIGHT, HOST_PACE_LARGEST_PER_SECOND,
    HOST_PACE_LEAST_MOVING_PER_SECOND, HOST_PACE_LEAST_PER_SECOND,
    HOST_PACE_MEASURING_WINDOW_SECONDS,
};

/// The host's walking speed as the guest's position stream shows it: the horizontal distance over
/// windows of at least half a second, kept only while the host really walks, so a stop, a run or
/// a jump to a new place does not pull it.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct HostPaceWatch {
    window_start: Option<([f32; 3], f64)>,
    pace_per_second: Option<f32>,
}

impl HostPaceWatch {
    pub fn observe(&mut self, position: [f32; 3], seconds: f64) {
        let Some((start, start_seconds)) = self.window_start else {
            self.window_start = Some((position, seconds));
            return;
        };
        let elapsed = seconds - start_seconds;
        if elapsed < 0.0 {
            self.window_start = Some((position, seconds));
            return;
        }
        if elapsed < HOST_PACE_MEASURING_WINDOW_SECONDS {
            return;
        }
        self.window_start = Some((position, seconds));
        let travelled = (position[0] - start[0]).hypot(position[2] - start[2]);
        let speed = travelled / elapsed as f32;
        // faster than any house pace is a run outdoors, a dash or a jump to a new place
        if !speed.is_finite()
            || speed < HOST_PACE_LEAST_MOVING_PER_SECOND
            || speed > HOST_PACE_LARGEST_PER_SECOND
        {
            return;
        }
        let speed = speed.max(HOST_PACE_LEAST_PER_SECOND);
        self.pace_per_second = Some(self.pace_per_second.map_or(speed, |pace| {
            pace + HOST_PACE_AVERAGE_SAMPLE_WEIGHT * (speed - pace)
        }));
    }

    pub fn pace_per_second(&self) -> Option<f32> {
        self.pace_per_second
    }
}
