pub const SWING_PROBE_YAW_STEP_RADIANS: f32 = 0.05;
pub const SWING_PROBE_TAIL_MS: u64 = 2000;
pub const SWING_PROBE_MIN_GAP_BETWEEN_LINES_MS: u64 = 1000;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SwingProbe {
    active_until_ms: Option<u64>,
    last_line_at_ms: Option<u64>,
}

impl SwingProbe {
    pub const fn new() -> Self {
        Self {
            active_until_ms: None,
            last_line_at_ms: None,
        }
    }

    pub fn step(&mut self, now_ms: u64, yaw_step_radians: f32) -> bool {
        if yaw_step_radians.abs() > SWING_PROBE_YAW_STEP_RADIANS {
            self.active_until_ms = Some(now_ms.saturating_add(SWING_PROBE_TAIL_MS));
        }
        if !self.active_until_ms.is_some_and(|until| now_ms <= until) {
            return false;
        }
        let gap_is_over = self
            .last_line_at_ms
            .is_none_or(|last| now_ms.saturating_sub(last) >= SWING_PROBE_MIN_GAP_BETWEEN_LINES_MS);
        if gap_is_over {
            self.last_line_at_ms = Some(now_ms);
        }
        gap_is_over
    }
}

pub fn yaw_step_radians(before: f32, after: f32) -> f32 {
    let tau = core::f32::consts::TAU;
    let difference = (after - before).rem_euclid(tau);
    if difference > core::f32::consts::PI {
        difference - tau
    } else {
        difference
    }
}
