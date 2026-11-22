#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RawMouseAccumulator {
    pending: (i32, i32),
    frame: (i32, i32),
}

impl RawMouseAccumulator {
    pub const fn new() -> Self {
        Self {
            pending: (0, 0),
            frame: (0, 0),
        }
    }

    pub fn add(&mut self, dx: i32, dy: i32) {
        self.pending.0 = self.pending.0.saturating_add(dx);
        self.pending.1 = self.pending.1.saturating_add(dy);
    }

    pub fn begin_frame(&mut self) {
        self.frame = std::mem::take(&mut self.pending);
    }

    pub fn take_frame_counts(&mut self) -> (i32, i32) {
        std::mem::take(&mut self.frame)
    }

    pub fn discard_all(&mut self) {
        *self = Self::default();
    }
}

pub fn counts_of_device_move(dx: f32, dy: f32) -> (i32, i32) {
    (dx.round() as i32, dy.round() as i32)
}
