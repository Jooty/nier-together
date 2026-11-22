use serde::{Deserialize, Serialize};

use crate::wire::WireError;

pub const SCALE_UP_YES_LABEL: &str = "Scale up";
pub const SCALE_UP_NO_LABEL: &str = "Not now";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScaleUpAsk {
    Open { slot: u8, name: String },
    Withdrawn { slot: u8 },
    NoSaveYet { slot: u8, name: String },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScaleUpAnswer {
    pub slot: u8,
    pub yes: bool,
}

impl ScaleUpAsk {
    pub fn slot(&self) -> u8 {
        match self {
            ScaleUpAsk::Open { slot, .. }
            | ScaleUpAsk::Withdrawn { slot }
            | ScaleUpAsk::NoSaveYet { slot, .. } => *slot,
        }
    }

    pub fn validate(&self) -> Result<(), WireError> {
        if self.slot() >= crate::wire::MAX_PLAYERS {
            return Err(WireError::OutOfRange { field: "slot" });
        }
        match self {
            ScaleUpAsk::Open { name, .. } | ScaleUpAsk::NoSaveYet { name, .. } => {
                if name.is_empty()
                    || name.len() > crate::wire::MAX_DISPLAY_NAME_BYTES
                    || name.chars().any(char::is_control)
                {
                    return Err(WireError::OutOfRange { field: "name" });
                }
                Ok(())
            }
            ScaleUpAsk::Withdrawn { .. } => Ok(()),
        }
    }
}

impl ScaleUpAnswer {
    pub fn validate(&self) -> Result<(), WireError> {
        if self.slot >= crate::wire::MAX_PLAYERS {
            return Err(WireError::OutOfRange { field: "slot" });
        }
        Ok(())
    }
}

pub fn scale_up_title(guest_name: &str) -> String {
    format!("No save for {guest_name}")
}

pub fn scale_up_body(guest_name: &str) -> String {
    format!(
        "{guest_name} has no save in this campaign. Give them a copy of your save under their name?"
    )
}

pub fn scale_up_no_save_yet_line(guest_name: &str) -> String {
    format!(
        "{guest_name} has no save here and you have no finished save to copy yet, so they start a new game"
    )
}
