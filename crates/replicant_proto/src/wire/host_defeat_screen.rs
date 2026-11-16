use serde::{Deserialize, Serialize};

use crate::wire::WireError;

pub const DEFEAT_SCREEN_ITEM_COUNT: u8 = 2;

/// The two items of the game's defeat screen. `Continue` is the first item (the game's result 1:
/// the phase restarts), `Leave` the second (result 2, asked again by a yes or no dialog first).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DefeatPick {
    Continue,
    Leave,
}

/// The host's defeat screen: `defeat` counts the host's screens so a pick never lands on a later
/// one, `cursor` is the item the host has highlighted, `pick` the item the host chose.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostDefeatScreen {
    pub defeat: u32,
    pub cursor: u8,
    pub pick: Option<DefeatPick>,
}

impl DefeatPick {
    pub fn label(self) -> &'static str {
        match self {
            Self::Continue => "continue",
            Self::Leave => "leave",
        }
    }
}

impl HostDefeatScreen {
    pub fn validate(&self) -> Result<(), WireError> {
        if self.cursor >= DEFEAT_SCREEN_ITEM_COUNT {
            return Err(WireError::OutOfRange {
                field: "defeat screen cursor",
            });
        }
        Ok(())
    }
}
