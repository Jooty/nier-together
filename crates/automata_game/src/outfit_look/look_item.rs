use crate::outfit_look::LookKind;

const TWO_B_ONLY: u8 = 0b001;
const NINE_S_ONLY: u8 = 0b010;
const A2_ONLY: u8 = 0b100;
const TWO_B_AND_NINE_S: u8 = 0b011;
const TWO_B_AND_A2: u8 = 0b101;
const EVERY_CHARACTER: u8 = 0b111;

pub const CLOTH_STATE_ITEM_ID: i32 = 980;
pub const CLOTH_STATE_OFF: i32 = 0;
pub const CLOTH_STATE_ON: i32 = 1;
pub const LOOK_ITEM_COUNT: usize = 49;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LookItem {
    pub id: i32,
    pub kind: LookKind,
    pub value: i32,
    slots: u8,
}

impl LookItem {
    pub fn is_for_slot(&self, slot: usize) -> bool {
        slot < u8::BITS as usize && self.slots & (1 << slot) != 0
    }
}

const fn item(id: i32, kind: LookKind, value: i32, slots: u8) -> LookItem {
    LookItem {
        id,
        kind,
        value,
        slots,
    }
}

const fn hair(id: i32) -> LookItem {
    item(id, LookKind::Hair, id - 880, TWO_B_AND_A2)
}

const fn pod(id: i32) -> LookItem {
    item(id, LookKind::PodSkin, id - 950, EVERY_CHARACTER)
}

pub const LOOK_ITEMS: [LookItem; LOOK_ITEM_COUNT] = [
    item(981, LookKind::Outfit, 2, TWO_B_ONLY),
    item(982, LookKind::Outfit, 3, TWO_B_ONLY),
    item(983, LookKind::Outfit, 1, TWO_B_ONLY),
    item(984, LookKind::Outfit, 1, NINE_S_ONLY),
    item(985, LookKind::Outfit, 1, A2_ONLY),
    item(990, LookKind::Accessory, 4, EVERY_CHARACTER),
    item(991, LookKind::Accessory, 6, EVERY_CHARACTER),
    item(992, LookKind::Accessory, 1, EVERY_CHARACTER),
    item(993, LookKind::Accessory, 5, EVERY_CHARACTER),
    item(994, LookKind::Accessory, 2, EVERY_CHARACTER),
    item(995, LookKind::Accessory, 3, EVERY_CHARACTER),
    item(996, LookKind::Accessory, 7, EVERY_CHARACTER),
    item(997, LookKind::Accessory, 8, EVERY_CHARACTER),
    item(998, LookKind::Accessory, 9, EVERY_CHARACTER),
    item(999, LookKind::Accessory, 10, EVERY_CHARACTER),
    item(870, LookKind::Accessory, 11, EVERY_CHARACTER),
    item(871, LookKind::Accessory, 12, TWO_B_AND_NINE_S),
    item(872, LookKind::Accessory, 13, A2_ONLY),
    item(873, LookKind::Accessory, 14, EVERY_CHARACTER),
    item(874, LookKind::Accessory, 15, EVERY_CHARACTER),
    item(875, LookKind::Accessory, 16, EVERY_CHARACTER),
    item(876, LookKind::Accessory, 17, EVERY_CHARACTER),
    item(877, LookKind::Accessory, 18, EVERY_CHARACTER),
    item(878, LookKind::Accessory, 19, EVERY_CHARACTER),
    hair(880),
    hair(881),
    hair(882),
    hair(883),
    hair(884),
    hair(885),
    hair(886),
    hair(887),
    hair(888),
    hair(889),
    hair(890),
    hair(891),
    hair(892),
    hair(893),
    hair(894),
    hair(895),
    hair(896),
    hair(897),
    hair(898),
    pod(951),
    pod(952),
    pod(954),
    pod(957),
    item(960, LookKind::GunSkin, 1, EVERY_CHARACTER),
    item(961, LookKind::GunSkin, 2, EVERY_CHARACTER),
];

pub fn look_items(slot: usize, kind: LookKind) -> Vec<LookItem> {
    LOOK_ITEMS
        .into_iter()
        .filter(|item| item.kind == kind && item.is_for_slot(slot))
        .collect()
}
