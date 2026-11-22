pub const CAP_PROBE_MIN_GAP_BETWEEN_LINES_MS: u64 = 1000;
pub const CAP_PROBE_WINDOW_REPORT_GAP_MS: u64 = 1000;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CapProbeLine {
    pub largest_lx: i32,
    pub largest_ly: i32,
    pub largest_yaw_step: f32,
    pub frames: u32,
    pub with_window_report: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct CapProbe {
    largest_lx: i32,
    largest_ly: i32,
    largest_yaw_step: f32,
    frames: u32,
    last_line_at_ms: Option<u64>,
    last_window_report_at_ms: Option<u64>,
}

impl CapProbe {
    pub const fn new() -> Self {
        Self {
            largest_lx: 0,
            largest_ly: 0,
            largest_yaw_step: 0.0,
            frames: 0,
            last_line_at_ms: None,
            last_window_report_at_ms: None,
        }
    }

    pub fn step(&mut self, now_ms: u64, counts: (i32, i32), yaw_step: f32) -> Option<CapProbeLine> {
        if counts == (0, 0) {
            return None;
        }
        if counts.0.unsigned_abs() > self.largest_lx.unsigned_abs() {
            self.largest_lx = counts.0;
        }
        if counts.1.unsigned_abs() > self.largest_ly.unsigned_abs() {
            self.largest_ly = counts.1;
        }
        if yaw_step.abs() > self.largest_yaw_step.abs() {
            self.largest_yaw_step = yaw_step;
        }
        self.frames = self.frames.saturating_add(1);
        let gap_is_over = self
            .last_line_at_ms
            .is_none_or(|last| now_ms.saturating_sub(last) >= CAP_PROBE_MIN_GAP_BETWEEN_LINES_MS);
        if !gap_is_over {
            return None;
        }
        let with_window_report = self
            .last_window_report_at_ms
            .is_none_or(|last| now_ms.saturating_sub(last) >= CAP_PROBE_WINDOW_REPORT_GAP_MS);
        if with_window_report {
            self.last_window_report_at_ms = Some(now_ms);
        }
        let line = CapProbeLine {
            largest_lx: self.largest_lx,
            largest_ly: self.largest_ly,
            largest_yaw_step: self.largest_yaw_step,
            frames: self.frames,
            with_window_report,
        };
        *self = Self {
            last_line_at_ms: Some(now_ms),
            last_window_report_at_ms: self.last_window_report_at_ms,
            ..Self::new()
        };
        Some(line)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenRect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

pub fn clip_cursor_is_set(clip: ScreenRect, whole_screen: ScreenRect) -> bool {
    clip != whole_screen
}
