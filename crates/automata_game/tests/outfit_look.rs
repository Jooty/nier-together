use automata_game::mode::SessionMode;
use automata_game::outfit_look::{
    HIGHEST_ACCESSORY_CODE, HIGHEST_GUN_SKIN, HIGHEST_HAIR_COLOUR, HIGHEST_OUTFIT_VARIANT,
    HIGHEST_POD_SKIN, LOOK_UNPICKED, PlayerLook, look_send_due, looks_allowed,
};

const PICKED: PlayerLook = PlayerLook {
    outfit_variant: 2,
    accessory_code: 11,
    hair_colour: 5,
    pod_skin: 1,
    gun_skin: 0,
};

#[test]
fn free_roam_always_allows_a_look() {
    assert!(looks_allowed(SessionMode::FreeRoam, false));
    assert!(looks_allowed(SessionMode::FreeRoam, true));
}

#[test]
fn the_campaign_allows_a_look_only_when_the_host_turned_it_on() {
    assert!(!looks_allowed(SessionMode::Campaign, false));
    assert!(looks_allowed(SessionMode::Campaign, true));
}

#[test]
fn every_value_the_picker_offers_is_valid() {
    assert!(PICKED.is_valid());
    assert!(PlayerLook::UNPICKED.is_valid());
    let highest = PlayerLook {
        outfit_variant: HIGHEST_OUTFIT_VARIANT,
        accessory_code: HIGHEST_ACCESSORY_CODE,
        hair_colour: HIGHEST_HAIR_COLOUR,
        pod_skin: HIGHEST_POD_SKIN,
        gun_skin: HIGHEST_GUN_SKIN,
    };
    assert!(highest.is_valid());
}

#[test]
fn a_value_past_the_highest_of_its_kind_is_invalid() {
    let past = [
        PlayerLook {
            outfit_variant: HIGHEST_OUTFIT_VARIANT + 1,
            ..PICKED
        },
        PlayerLook {
            accessory_code: HIGHEST_ACCESSORY_CODE + 1,
            ..PICKED
        },
        PlayerLook {
            hair_colour: HIGHEST_HAIR_COLOUR + 1,
            ..PICKED
        },
        PlayerLook {
            pod_skin: HIGHEST_POD_SKIN + 1,
            ..PICKED
        },
        PlayerLook {
            gun_skin: HIGHEST_GUN_SKIN + 1,
            ..PICKED
        },
    ];
    for look in past {
        assert!(!look.is_valid(), "{look:?}");
    }
}

#[test]
fn only_a_look_with_every_kind_unpicked_is_unpicked() {
    assert!(PlayerLook::UNPICKED.is_unpicked());
    let one_picked = PlayerLook {
        hair_colour: 0,
        ..PlayerLook::UNPICKED
    };
    assert!(!one_picked.is_unpicked());
    assert_eq!(PlayerLook::UNPICKED.gun_skin, LOOK_UNPICKED);
}

#[test]
fn a_look_is_sent_when_it_changes_when_a_player_joins_and_after_a_load() {
    assert!(look_send_due(None, PICKED, false, false));
    assert!(!look_send_due(Some(PICKED), PICKED, false, false));
    let changed = PlayerLook {
        pod_skin: 2,
        ..PICKED
    };
    assert!(look_send_due(Some(PICKED), changed, false, false));
    assert!(look_send_due(Some(PICKED), PICKED, true, false));
    assert!(look_send_due(Some(PICKED), PICKED, false, true));
}
