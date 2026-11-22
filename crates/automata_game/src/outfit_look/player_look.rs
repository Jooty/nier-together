use serde::{Deserialize, Serialize};

use crate::outfit_look::{
    HIGHEST_ACCESSORY_CODE, HIGHEST_GUN_SKIN, HIGHEST_HAIR_COLOUR, HIGHEST_OUTFIT_VARIANT,
    HIGHEST_POD_SKIN, LOOK_UNPICKED,
};

/// What one player wears, as the values the game's own setters take. A field holding
/// `LOOK_UNPICKED` leaves that kind as the receiving game has it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerLook {
    pub outfit_variant: u8,
    pub accessory_code: u8,
    pub hair_colour: u8,
    pub pod_skin: u8,
    pub gun_skin: u8,
}

impl PlayerLook {
    pub const UNPICKED: Self = Self {
        outfit_variant: LOOK_UNPICKED,
        accessory_code: LOOK_UNPICKED,
        hair_colour: LOOK_UNPICKED,
        pod_skin: LOOK_UNPICKED,
        gun_skin: LOOK_UNPICKED,
    };

    pub fn is_valid(&self) -> bool {
        field_is_valid(self.outfit_variant, HIGHEST_OUTFIT_VARIANT)
            && field_is_valid(self.accessory_code, HIGHEST_ACCESSORY_CODE)
            && field_is_valid(self.hair_colour, HIGHEST_HAIR_COLOUR)
            && field_is_valid(self.pod_skin, HIGHEST_POD_SKIN)
            && field_is_valid(self.gun_skin, HIGHEST_GUN_SKIN)
    }

    pub fn is_unpicked(&self) -> bool {
        *self == Self::UNPICKED
    }
}

fn field_is_valid(value: u8, highest: u8) -> bool {
    value == LOOK_UNPICKED || value <= highest
}
