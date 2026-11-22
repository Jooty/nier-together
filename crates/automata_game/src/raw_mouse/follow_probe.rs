pub const FOLLOW_PROBE_MIN_GAP_BETWEEN_LINES_MICROS: u64 = 1_000_000;
pub const FOLLOW_PROBE_HERO_MOVED_METRES: f32 = 0.0005;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FollowSample {
    pub body: [f32; 3],
    pub look_at_from_body: [f32; 3],
    pub eye_from_look_at_turn_removed: [f32; 3],
    pub raw_turned: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FollowProbeLine {
    pub largest_jump: f32,
    pub largest_frame_ms: f32,
    pub raw_turned: bool,
    pub frames: u32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct FollowProbe {
    previous: Option<FollowSample>,
    previous_at_micros: Option<u64>,
    window_started_at_micros: Option<u64>,
    largest_jump: f32,
    largest_frame_ms: f32,
    raw_turned: bool,
    hero_moved: bool,
    frames: u32,
}

impl FollowProbe {
    pub const fn new() -> Self {
        Self {
            previous: None,
            previous_at_micros: None,
            window_started_at_micros: None,
            largest_jump: 0.0,
            largest_frame_ms: 0.0,
            raw_turned: false,
            hero_moved: false,
            frames: 0,
        }
    }

    pub fn step(&mut self, now_micros: u64, sample: FollowSample) -> Option<FollowProbeLine> {
        let window_start = *self.window_started_at_micros.get_or_insert(now_micros);
        if let Some(previous) = self.previous {
            let jump = body_to_camera_change(&previous, &sample);
            self.largest_jump = self.largest_jump.max(jump);
            self.hero_moved |=
                distance(previous.body, sample.body) > FOLLOW_PROBE_HERO_MOVED_METRES;
        }
        if let Some(previous_at) = self.previous_at_micros {
            let frame_ms = now_micros.saturating_sub(previous_at) as f32 / 1000.0;
            self.largest_frame_ms = self.largest_frame_ms.max(frame_ms);
        }
        self.raw_turned |= sample.raw_turned;
        self.frames = self.frames.saturating_add(1);
        self.previous = Some(sample);
        self.previous_at_micros = Some(now_micros);
        if now_micros.saturating_sub(window_start) < FOLLOW_PROBE_MIN_GAP_BETWEEN_LINES_MICROS {
            return None;
        }
        let line = self.hero_moved.then_some(FollowProbeLine {
            largest_jump: self.largest_jump,
            largest_frame_ms: self.largest_frame_ms,
            raw_turned: self.raw_turned,
            frames: self.frames,
        });
        *self = Self {
            previous: self.previous,
            previous_at_micros: self.previous_at_micros,
            window_started_at_micros: Some(now_micros),
            ..Self::new()
        };
        line
    }
}

pub fn body_to_camera_change(before: &FollowSample, after: &FollowSample) -> f32 {
    let change = |axis: usize| {
        (after.look_at_from_body[axis] - before.look_at_from_body[axis])
            + (after.eye_from_look_at_turn_removed[axis]
                - before.eye_from_look_at_turn_removed[axis])
    };
    (change(0).powi(2) + change(1).powi(2) + change(2).powi(2)).sqrt()
}

fn distance(a: [f32; 3], b: [f32; 3]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}
