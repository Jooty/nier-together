#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpriteRefusal {
    BatchFull,
    OffScreen,
    NearCamera,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpriteAddView {
    pub batch_full: bool,
    pub in_view: bool,
    pub camera_distance_squared: f32,
    pub nearest_distance_squared: f32,
}

#[derive(Debug, Default)]
pub struct SpriteRefusalTally {
    batch_full: u32,
    off_screen: u32,
    near_camera: u32,
    other: u32,
}

impl SpriteRefusal {
    pub fn text(self) -> &'static str {
        match self {
            Self::BatchFull => "batch full",
            Self::OffScreen => "off screen",
            Self::NearCamera => "near camera",
            Self::Other => "other",
        }
    }
}

impl SpriteRefusalTally {
    pub fn note(&mut self, refusal: SpriteRefusal) {
        match refusal {
            SpriteRefusal::BatchFull => self.batch_full += 1,
            SpriteRefusal::OffScreen => self.off_screen += 1,
            SpriteRefusal::NearCamera => self.near_camera += 1,
            SpriteRefusal::Other => self.other += 1,
        }
    }

    /// The line for the time since the last one, or `None` when the batch refused nothing.
    pub fn take_line(&mut self) -> Option<String> {
        let tally = core::mem::take(self);
        let total = tally.batch_full + tally.off_screen + tally.near_camera + tally.other;
        if total == 0 {
            return None;
        }
        Some(format!(
            "enemy bullets: last 5 s the sprite batch refused {total} host sprites: {} off screen, {} near the camera, {} batch full, {} other",
            tally.off_screen, tally.near_camera, tally.batch_full, tally.other
        ))
    }
}

/// Why the game's sprite batch refused a sprite, in the order its add function checks: no free
/// entry, outside the view frustum, closer to the camera than the nearest distance, then the
/// depth range.
pub fn sprite_refusal(view: SpriteAddView) -> SpriteRefusal {
    if view.batch_full {
        SpriteRefusal::BatchFull
    } else if !view.in_view {
        SpriteRefusal::OffScreen
    } else if view.camera_distance_squared < view.nearest_distance_squared {
        SpriteRefusal::NearCamera
    } else {
        SpriteRefusal::Other
    }
}
