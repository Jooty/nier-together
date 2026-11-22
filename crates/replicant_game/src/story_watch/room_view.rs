use replicant_proto::NO_SIDE_VIEW_DOOR;

use crate::side_view::{SideViewDoor, door_by_id};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoomViewReading<'a> {
    pub area: &'a str,
    pub watching: bool,
    pub in_play_and_free: bool,
    pub in_side_view: bool,
    pub host_side_view_door: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoomViewStep {
    Nothing,
    Take(&'static SideViewDoor),
    Leave(&'static SideViewDoor),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WatchedRoomView {
    taken: Option<&'static SideViewDoor>,
}

impl WatchedRoomView {
    pub const fn new() -> Self {
        Self { taken: None }
    }

    pub fn observe(&mut self, reading: &RoomViewReading<'_>) -> RoomViewStep {
        if let Some(door) = self.taken
            && (!reading.watching || reading.host_side_view_door != door.id)
        {
            self.taken = None;
            return if reading.in_side_view {
                RoomViewStep::Leave(door)
            } else {
                RoomViewStep::Nothing
            };
        }
        if !reading.watching
            || !reading.in_play_and_free
            || reading.in_side_view
            || reading.host_side_view_door == NO_SIDE_VIEW_DOOR
        {
            return RoomViewStep::Nothing;
        }
        match door_by_id(reading.area, reading.host_side_view_door) {
            Some(door) => {
                self.taken = Some(door);
                RoomViewStep::Take(door)
            }
            None => RoomViewStep::Nothing,
        }
    }
}
