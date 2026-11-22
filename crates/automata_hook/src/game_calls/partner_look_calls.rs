use automata_game::address::{
    REFRESH_LOOK_PARTS_FUNCTION, SPAWN_ACCESSORY_ENTITY_FUNCTION, SPAWN_PODS_FUNCTION,
};
use automata_game::outfit_look::{LookKind, PartnerLookReading, PartnerLookStep};
use automata_game::partner::{BODY_ACCESSORY_CODE_OFFSET, BODY_OUTFIT_VARIANT_OFFSET};

use crate::game_calls::GameCalls;

impl GameCalls {
    pub fn read_partner_look(&self, body: usize, slot: usize) -> Option<PartnerLookReading> {
        Some(PartnerLookReading {
            outfit_body: self
                .read_i32_at(self.body_field(body, BODY_OUTFIT_VARIANT_OFFSET))
                .ok()?,
            accessory_body: self
                .read_i32_at(self.body_field(body, BODY_ACCESSORY_CODE_OFFSET))
                .ok()?,
            outfit_saved: self.read_look_global(LookKind::Outfit, slot)?,
            accessory_saved: self.read_look_global(LookKind::Accessory, slot)?,
            pod_saved: self.read_look_global(LookKind::PodSkin, slot)?,
        })
    }

    /// Game thread only. Writes the body's own fields; every global it borrows is put back before it returns.
    pub fn apply_partner_look_step(&self, body: usize, slot: usize, step: PartnerLookStep) -> bool {
        match step {
            PartnerLookStep::WriteOutfit(variant) => {
                let written = self
                    .write_i32_at(self.body_field(body, BODY_OUTFIT_VARIANT_OFFSET), variant)
                    .is_ok();
                self.call_on_body(REFRESH_LOOK_PARTS_FUNCTION, body);
                written
            }
            PartnerLookStep::WriteAccessory(code) => {
                let written = self
                    .write_i32_at(self.body_field(body, BODY_ACCESSORY_CODE_OFFSET), code)
                    .is_ok();
                self.call_on_body(SPAWN_ACCESSORY_ENTITY_FUNCTION, body);
                self.call_on_body(REFRESH_LOOK_PARTS_FUNCTION, body);
                written
            }
            PartnerLookStep::SwapPodSkin { value, restore } => {
                let swapped = self.write_look_global(LookKind::PodSkin, slot, value);
                self.call_on_body(SPAWN_PODS_FUNCTION, body);
                self.write_look_global(LookKind::PodSkin, slot, restore);
                swapped
            }
        }
    }

    /// Game thread only. Runs `update` with the partner's hair colour in the global, then puts the global back.
    pub fn with_hair_swapped<T>(&self, slot: usize, value: i32, update: impl FnOnce() -> T) -> T {
        let before = self.read_look_global(LookKind::Hair, slot);
        let swapped = before.is_some() && self.write_look_global(LookKind::Hair, slot, value);
        let result = update();
        if let (true, Some(before)) = (swapped, before) {
            self.write_look_global(LookKind::Hair, slot, before);
        }
        result
    }
}
