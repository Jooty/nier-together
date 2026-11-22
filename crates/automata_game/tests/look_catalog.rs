use std::collections::HashSet;

use automata_game::address::{
    LOOK_ACCESSORY_CODE_VALUE, LOOK_GUN_SKIN_VALUE, LOOK_HAIR_INDEX_VALUE, LOOK_POD_SKIN_VALUE,
    LOOK_SANITIZE_FUNCTION, SET_ACCESSORY_CODE_FUNCTION,
};
use automata_game::character::Character;
use automata_game::exe::ExeVariant;
use automata_game::outfit_look::{
    HIGHEST_ACCESSORY_CODE, HIGHEST_GUN_SKIN, HIGHEST_HAIR_COLOUR, HIGHEST_OUTFIT_VARIANT,
    HIGHEST_POD_SKIN, LOOK_ITEM_COUNT, LOOK_ITEMS, LOOK_SLOT_COUNT, LookKind, look_items,
    look_slot_of,
};

fn highest_of(kind: LookKind) -> i32 {
    i32::from(match kind {
        LookKind::Outfit => HIGHEST_OUTFIT_VARIANT,
        LookKind::Accessory => HIGHEST_ACCESSORY_CODE,
        LookKind::Hair => HIGHEST_HAIR_COLOUR,
        LookKind::PodSkin => HIGHEST_POD_SKIN,
        LookKind::GunSkin => HIGHEST_GUN_SKIN,
        LookKind::ClothState => 1,
    })
}

#[test]
fn the_catalog_holds_forty_nine_items_with_one_id_each() {
    assert_eq!(LOOK_ITEMS.len(), LOOK_ITEM_COUNT);
    let ids: HashSet<i32> = LOOK_ITEMS.iter().map(|item| item.id).collect();
    assert_eq!(ids.len(), LOOK_ITEM_COUNT);
}

#[test]
fn every_value_is_one_the_wire_accepts() {
    for item in LOOK_ITEMS {
        assert!(
            (0..=highest_of(item.kind)).contains(&item.value),
            "item {} has value {}",
            item.id,
            item.value
        );
    }
}

#[test]
fn no_two_items_of_one_kind_give_one_value_to_one_character() {
    for slot in 0..LOOK_SLOT_COUNT {
        for kind in LookKind::ALL {
            let values: Vec<i32> = look_items(slot, kind)
                .iter()
                .map(|item| item.value)
                .collect();
            let unique: HashSet<i32> = values.iter().copied().collect();
            assert_eq!(values.len(), unique.len(), "slot {slot} {kind:?}");
        }
    }
}

#[test]
fn the_characters_map_to_the_games_slots() {
    assert_eq!(look_slot_of(Character::TwoB), 0);
    assert_eq!(look_slot_of(Character::NineS), 1);
    assert_eq!(look_slot_of(Character::A2), 2);
}

#[test]
fn each_character_is_offered_its_own_outfits() {
    let ids = |slot| -> Vec<i32> {
        look_items(slot, LookKind::Outfit)
            .iter()
            .map(|item| item.id)
            .collect()
    };
    assert_eq!(ids(0).len(), 3);
    assert_eq!(ids(1).len(), 1);
    assert_eq!(ids(2).len(), 1);
}

#[test]
fn nineteen_accessories_less_the_one_that_belongs_to_the_other_characters() {
    assert_eq!(look_items(0, LookKind::Accessory).len(), 18);
    assert_eq!(look_items(1, LookKind::Accessory).len(), 18);
    assert_eq!(look_items(2, LookKind::Accessory).len(), 18);
}

#[test]
fn the_six_cut_pod_skins_are_not_offered() {
    let offered: Vec<i32> = look_items(0, LookKind::PodSkin)
        .iter()
        .map(|item| item.id)
        .collect();
    for cut in [950, 953, 955, 956, 958, 959] {
        assert!(!offered.contains(&cut), "pod skin {cut}");
    }
    assert_eq!(offered.len(), 4);
}

#[test]
fn hair_is_offered_to_the_two_characters_where_dye_is_proven_to_show() {
    assert_eq!(look_items(0, LookKind::Hair).len(), 19);
    assert_eq!(look_items(2, LookKind::Hair).len(), 19);
    assert!(look_items(1, LookKind::Hair).is_empty());
}

#[test]
fn the_offered_looks_count_fifty_with_the_cloth_toggle() {
    assert_eq!(LOOK_ITEMS.len() + 1, 50);
}

#[test]
fn the_look_globals_have_a_compat_twin_the_same_distance_below() {
    let pairs = [
        LOOK_ACCESSORY_CODE_VALUE,
        LOOK_GUN_SKIN_VALUE,
        LOOK_HAIR_INDEX_VALUE,
        LOOK_POD_SKIN_VALUE,
    ];
    for address in pairs {
        assert_eq!(
            address.rva(ExeVariant::Main) - address.rva(ExeVariant::Compat),
            0xC180
        );
    }
}

#[test]
fn a_global_per_slot_sits_a_word_after_the_one_before() {
    let second = LOOK_POD_SKIN_VALUE.plus(4);
    assert_eq!(second.rva(ExeVariant::Main), 0x14948B8);
    assert_eq!(second.rva(ExeVariant::Compat), 0x1488738);
}

#[test]
fn the_look_functions_are_the_ledger_twins() {
    assert_eq!(SET_ACCESSORY_CODE_FUNCTION.rva(ExeVariant::Main), 0x4ECB60);
    assert_eq!(
        SET_ACCESSORY_CODE_FUNCTION.rva(ExeVariant::Compat),
        0x4E6980
    );
    assert_eq!(LOOK_SANITIZE_FUNCTION.rva(ExeVariant::Main), 0x9CA5A0);
    assert_eq!(LOOK_SANITIZE_FUNCTION.rva(ExeVariant::Compat), 0x9C1CD0);
}
