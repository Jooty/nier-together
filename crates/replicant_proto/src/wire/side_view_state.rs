use serde::{Deserialize, Serialize};

/// Whether this player's side view (the indoor camera a door's script turns on, flag at
/// `SIDE_VIEW_FLAG_RVA`) is on, sent on every change and once a second. The partner's game reads
/// it as "inside": the side view is on from a house's entry step to its exit step.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SideViewState {
    pub phase: u16,
    pub in_side_view: bool,
    pub side_view_door: u16,
}
