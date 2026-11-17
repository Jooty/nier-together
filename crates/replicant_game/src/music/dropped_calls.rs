use replicant_proto::MUSIC_LAYER_COUNT;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DroppedCall {
    Play {
        track: i32,
        volume: f32,
        fade_seconds: f32,
    },
    Stop {
        fade_seconds: f32,
    },
    Master {
        volume: f32,
        seconds: f32,
    },
    Layer {
        layer: i32,
        volume: f32,
        seconds: f32,
    },
}

impl DroppedCall {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Play { .. } => "PlayBgm",
            Self::Stop { .. } => "StopBgm",
            Self::Master { .. } => "SetBgmMasterVolume",
            Self::Layer { .. } => "SetBgmLayerVolume",
        }
    }

    pub fn detail(&self) -> i64 {
        match self {
            Self::Play { track, .. } => i64::from(*track),
            Self::Layer { layer, .. } => i64::from(*layer),
            Self::Stop { .. } | Self::Master { .. } => 0,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct DroppedCalls {
    track_call: Option<DroppedCall>,
    master: Option<DroppedCall>,
    layers: [Option<DroppedCall>; MUSIC_LAYER_COUNT],
}

impl DroppedCalls {
    pub const fn new() -> Self {
        Self {
            track_call: None,
            master: None,
            layers: [None; MUSIC_LAYER_COUNT],
        }
    }

    pub fn record(&mut self, call: DroppedCall) {
        match call {
            DroppedCall::Play { .. } | DroppedCall::Stop { .. } => self.track_call = Some(call),
            DroppedCall::Master { .. } => self.master = Some(call),
            DroppedCall::Layer { layer, .. } => {
                if let Some(slot) = usize::try_from(layer)
                    .ok()
                    .and_then(|index| self.layers.get_mut(index))
                {
                    *slot = Some(call);
                }
            }
        }
    }

    pub fn take_replay(&mut self) -> Vec<DroppedCall> {
        let replay = self
            .track_call
            .into_iter()
            .chain(self.master)
            .chain(self.layers.iter().flatten().copied())
            .collect();
        *self = Self::new();
        replay
    }

    pub fn clear(&mut self) {
        *self = Self::new();
    }
}
