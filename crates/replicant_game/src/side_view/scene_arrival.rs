use crate::side_view::{INDOOR_ARRIVAL_SETTLE_MS, SideViewDoor, door_by_id};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SceneArrival {
    pub area: &'static str,
    pub callback: &'static str,
    pub door: u16,
    pub source: &'static str,
}

pub static SCENE_ARRIVALS: [SceneArrival; 1] = [SceneArrival {
    area: "A_CENTER_VILLAGE_01",
    callback: "EID_1500_a0005_010_01_End02",
    door: 1,
    source: "run 1115a: the scene's own walk left both bodies on the floor of door 1's place (height 18.0) with only the host's view on",
}];

pub const SCENE_ARRIVAL_WAITS_FOR_THE_HOST_MS: u64 = 15_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SceneArrivalReading {
    pub now_ms: u64,
    pub in_play_and_free: bool,
    pub in_side_view: bool,
    pub host_side_view_door: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SceneArrivalWatch {
    pending: Option<(&'static SceneArrival, u64)>,
    free_since_ms: Option<u64>,
}

impl SceneArrivalWatch {
    pub const fn new() -> Self {
        Self {
            pending: None,
            free_since_ms: None,
        }
    }

    pub fn note_callback(&mut self, area: &str, callback: &str, now_ms: u64) {
        let Some(arrival) = SCENE_ARRIVALS
            .iter()
            .find(|arrival| arrival.area == area && arrival.callback == callback)
        else {
            return;
        };
        self.pending = Some((arrival, now_ms));
        self.free_since_ms = None;
    }

    pub fn observe(&mut self, reading: &SceneArrivalReading) -> Option<&'static SideViewDoor> {
        let (arrival, since_ms) = self.pending?;
        let waited_ms = reading.now_ms.saturating_sub(since_ms);
        if reading.in_side_view || waited_ms > SCENE_ARRIVAL_WAITS_FOR_THE_HOST_MS {
            self.pending = None;
            return None;
        }
        if reading.host_side_view_door != arrival.door || !reading.in_play_and_free {
            self.free_since_ms = None;
            return None;
        }
        let free_since_ms = *self.free_since_ms.get_or_insert(reading.now_ms);
        if reading.now_ms.saturating_sub(free_since_ms) < INDOOR_ARRIVAL_SETTLE_MS {
            return None;
        }
        self.pending = None;
        self.free_since_ms = None;
        door_by_id(arrival.area, arrival.door).filter(|door| !door.entry_steps.is_empty())
    }
}

impl Default for SceneArrivalWatch {
    fn default() -> Self {
        Self::new()
    }
}
