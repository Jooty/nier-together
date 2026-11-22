use automata_game::address::{
    GameAddress, LOOK_ACCESSORY_CODE_VALUE, LOOK_CLOTH_STATE_VALUE, LOOK_GUN_SKIN_VALUE,
    LOOK_HAIR_INDEX_VALUE, LOOK_OUTFIT_VARIANT_VALUE, LOOK_POD_SKIN_VALUE,
    REFRESH_LOOK_PARTS_FUNCTION, SET_ACCESSORY_CODE_FUNCTION, SET_CLOTH_STATE_FUNCTION,
    SET_OUTFIT_VARIANT_FUNCTION, SPAWN_ACCESSORY_ENTITY_FUNCTION, SPAWN_PODS_FUNCTION,
};
use automata_game::outfit_look::{LOOK_SLOT_COUNT, LookKind, LookReading, LookStep};
use automata_game::partner::{BODY_ACCESSORY_CODE_OFFSET, BODY_OUTFIT_VARIANT_OFFSET};

use crate::game_calls::GameCalls;

const SETTER_WITHOUT_MENU_PATH: i32 = 0;
const CLOTH_STATE_WITH_PARTS_PASS: i32 = 0;
const BYTES_PER_LOOK_INT: u32 = 4;

type BodyCall = unsafe extern "system" fn(usize);
type BodyIntCall = unsafe extern "system" fn(usize, i32);
type BodyTwoIntCall = unsafe extern "system" fn(usize, i32, i32);

impl GameCalls {
    pub fn read_look_global(&self, kind: LookKind, slot: usize) -> Option<i32> {
        if slot >= LOOK_SLOT_COUNT {
            return None;
        }
        match kind {
            LookKind::Outfit => self.read_look_int(LOOK_OUTFIT_VARIANT_VALUE, slot),
            LookKind::Accessory => self.read_look_int(LOOK_ACCESSORY_CODE_VALUE, slot),
            LookKind::PodSkin => self.read_look_int(LOOK_POD_SKIN_VALUE, slot),
            LookKind::ClothState => self.read_look_int(LOOK_CLOTH_STATE_VALUE, slot),
            LookKind::Hair => self.read_look_byte(LOOK_HAIR_INDEX_VALUE.plus(slot as u32)),
            LookKind::GunSkin => self.read_look_byte(LOOK_GUN_SKIN_VALUE),
        }
    }

    pub fn write_look_global(&self, kind: LookKind, slot: usize, value: i32) -> bool {
        if slot >= LOOK_SLOT_COUNT {
            return false;
        }
        match kind {
            LookKind::Outfit => self.write_look_int(LOOK_OUTFIT_VARIANT_VALUE, slot, value),
            LookKind::Accessory => self.write_look_int(LOOK_ACCESSORY_CODE_VALUE, slot, value),
            LookKind::PodSkin => self.write_look_int(LOOK_POD_SKIN_VALUE, slot, value),
            LookKind::ClothState => self.write_look_int(LOOK_CLOTH_STATE_VALUE, slot, value),
            LookKind::Hair => self.write_look_byte(LOOK_HAIR_INDEX_VALUE.plus(slot as u32), value),
            LookKind::GunSkin => self.write_look_byte(LOOK_GUN_SKIN_VALUE, value),
        }
    }

    pub fn read_look(&self, body: usize, slot: usize) -> Option<LookReading> {
        let global = |kind| self.read_look_global(kind, slot);
        Some(LookReading {
            outfit_global: global(LookKind::Outfit)?,
            outfit_body: self.read_body_look(body, BODY_OUTFIT_VARIANT_OFFSET)?,
            accessory_global: global(LookKind::Accessory)?,
            accessory_body: self.read_body_look(body, BODY_ACCESSORY_CODE_OFFSET)?,
            hair: global(LookKind::Hair)?,
            pod_skin: global(LookKind::PodSkin)?,
            gun_skin: global(LookKind::GunSkin)?,
            cloth_state: global(LookKind::ClothState)?,
        })
    }

    /// Game thread only, on a frame with the hero's model loaded.
    pub fn apply_look_step(&self, body: usize, slot: usize, step: LookStep) -> bool {
        match step {
            LookStep::SetOutfit(variant) => self.set_outfit(body, slot, variant),
            LookStep::SetAccessory(code) => self.set_accessory(body, slot, code),
            LookStep::SetPodSkin(skin) => {
                let written = self.write_look_global(LookKind::PodSkin, slot, skin);
                self.call_on_body(SPAWN_PODS_FUNCTION, body);
                written
            }
            LookStep::SetClothState(state) => {
                self.set_cloth_state(body, state);
                true
            }
            LookStep::SetHair(index) => self.write_look_global(LookKind::Hair, slot, index),
            LookStep::SetGunSkin(skin) => self.write_look_global(LookKind::GunSkin, slot, skin),
        }
    }

    fn set_outfit(&self, body: usize, slot: usize, variant: i32) -> bool {
        self.call_outfit_setter(body, variant);
        if self.read_body_look(body, BODY_OUTFIT_VARIANT_OFFSET) == Some(variant) {
            return self.read_look_global(LookKind::Outfit, slot) == Some(variant);
        }
        let written = self
            .write_i32_at(self.body_field(body, BODY_OUTFIT_VARIANT_OFFSET), variant)
            .is_ok()
            && self.write_look_global(LookKind::Outfit, slot, variant);
        self.call_on_body(REFRESH_LOOK_PARTS_FUNCTION, body);
        written
    }

    fn set_accessory(&self, body: usize, slot: usize, code: i32) -> bool {
        let mut written = true;
        if self.read_body_look(body, BODY_ACCESSORY_CODE_OFFSET) != Some(code) {
            self.call_accessory_setter(body, code);
            if self.read_body_look(body, BODY_ACCESSORY_CODE_OFFSET) != Some(code) {
                written = self
                    .write_i32_at(self.body_field(body, BODY_ACCESSORY_CODE_OFFSET), code)
                    .is_ok();
            }
            self.call_on_body(SPAWN_ACCESSORY_ENTITY_FUNCTION, body);
            self.call_on_body(REFRESH_LOOK_PARTS_FUNCTION, body);
        }
        written && self.write_look_global(LookKind::Accessory, slot, code)
    }

    fn read_look_int(&self, address: GameAddress, slot: usize) -> Option<i32> {
        let at = address.plus(BYTES_PER_LOOK_INT * slot as u32);
        self.read_i32(at).ok()
    }

    fn write_look_int(&self, address: GameAddress, slot: usize, value: i32) -> bool {
        let at = address.plus(BYTES_PER_LOOK_INT * slot as u32);
        self.write_i32(at, value).is_ok()
    }

    fn read_look_byte(&self, address: GameAddress) -> Option<i32> {
        self.read_u8_at(self.address_of(address) as u64)
            .ok()
            .map(i32::from)
    }

    fn write_look_byte(&self, address: GameAddress, value: i32) -> bool {
        u8::try_from(value).is_ok_and(|byte| {
            self.write_u8_at(self.address_of(address) as u64, byte)
                .is_ok()
        })
    }

    fn read_body_look(&self, body: usize, offset: u32) -> Option<i32> {
        self.read_i32_at(self.body_field(body, offset)).ok()
    }

    #[expect(
        unsafe_code,
        reason = "calls one of the game's look functions on a body"
    )]
    pub(super) fn call_on_body(&self, function: GameAddress, body: usize) {
        // SAFETY: the address is a look function of the verified build that takes only the body
        // (the accessory entity spawn, the parts pass or the pod spawn); body is the possessed
        // hero read this frame on the game thread.
        unsafe { core::mem::transmute::<usize, BodyCall>(self.address_of(function))(body) }
    }

    #[expect(unsafe_code, reason = "calls the game's outfit setter on a body")]
    fn call_outfit_setter(&self, body: usize, variant: i32) {
        // SAFETY: the address is Pl0000::setOutfitVariant of the verified build: the body, the
        // variant and a menu path flag that is 0 here, called on the possessed hero on the game
        // thread.
        unsafe {
            core::mem::transmute::<usize, BodyTwoIntCall>(
                self.address_of(SET_OUTFIT_VARIANT_FUNCTION),
            )(body, variant, SETTER_WITHOUT_MENU_PATH);
        }
    }

    #[expect(unsafe_code, reason = "calls the game's accessory setter on a body")]
    fn call_accessory_setter(&self, body: usize, code: i32) {
        // SAFETY: the address is Pl0000::setAccessoryCode of the verified build: the body and the
        // code, called on the possessed hero on the game thread.
        unsafe {
            core::mem::transmute::<usize, BodyIntCall>(
                self.address_of(SET_ACCESSORY_CODE_FUNCTION),
            )(body, code);
        }
    }

    #[expect(unsafe_code, reason = "calls the game's cloth state setter on a body")]
    fn set_cloth_state(&self, body: usize, state: i32) {
        // SAFETY: the address is Pl0000::setClothState of the verified build: the body, the state
        // and 0 to run the parts pass, called on the possessed hero on the game thread.
        unsafe {
            core::mem::transmute::<usize, BodyTwoIntCall>(
                self.address_of(SET_CLOTH_STATE_FUNCTION),
            )(body, state, CLOTH_STATE_WITH_PARTS_PASS);
        }
    }
}
