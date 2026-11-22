use automata_proto::{Message, PlayerLook, WireError, decode, encode};
use nt_wire::GameMessage;

const LOOK_VARIANT_TAG: u8 = 41;
const LOOK_RULE_VARIANT_TAG: u8 = 42;
const BOSS_WAIT_VARIANT_TAG: u8 = 43;

fn a_look() -> PlayerLook {
    PlayerLook {
        outfit_variant: 1,
        accessory_code: 19,
        hair_colour: 18,
        pod_skin: 9,
        gun_skin: 2,
    }
}

#[test]
fn a_look_comes_after_every_existing_variant_and_round_trips() {
    let message = Message::Look(a_look());
    let bytes = encode(&message).expect("encodes");
    assert_eq!(bytes[0], LOOK_VARIANT_TAG);
    assert_eq!(decode(&bytes).expect("decodes"), message);
    assert_eq!(message.kind_name(), "Look");
}

#[test]
fn the_hosts_rule_comes_after_the_look_and_round_trips() {
    for any_look_allowed in [false, true] {
        let message = Message::LookRule { any_look_allowed };
        let bytes = encode(&message).expect("encodes");
        assert_eq!(bytes[0], LOOK_RULE_VARIANT_TAG);
        assert_eq!(decode(&bytes).expect("decodes"), message);
        assert_eq!(message.kind_name(), "LookRule");
    }
}

#[test]
fn a_look_and_the_rule_go_through_the_host_to_every_other_player() {
    let look = Message::Look(a_look());
    assert!(look.is_relayed());
    assert!(GameMessage::forwarded_to_other_guests(&look));
    assert!(!look.is_latest_wins());
    let rule = Message::LookRule {
        any_look_allowed: true,
    };
    assert!(rule.is_relayed());
    assert!(!GameMessage::forwarded_to_other_guests(&rule));
}

#[test]
fn a_look_with_an_unpicked_kind_is_valid_and_one_past_the_highest_is_refused() {
    assert!(Message::Look(PlayerLook::UNPICKED).validate().is_ok());
    let past = PlayerLook {
        hair_colour: 19,
        ..a_look()
    };
    assert_eq!(
        Message::Look(past).validate(),
        Err(WireError::OutOfRange { field: "look" })
    );
}

#[test]
fn a_refused_look_does_not_decode_off_the_wire() {
    let past = PlayerLook {
        pod_skin: 10,
        ..a_look()
    };
    let bytes = postcard::to_stdvec(&Message::Look(past)).expect("serializes");
    assert!(decode(&bytes).is_err());
}

#[test]
fn the_boss_wait_comes_after_the_rule_is_relayed_and_checks_its_slot() {
    let message = Message::BossWait {
        slot: 2,
        waiting: true,
    };
    let bytes = encode(&message).expect("encodes");
    assert_eq!(bytes[0], BOSS_WAIT_VARIANT_TAG);
    assert_eq!(decode(&bytes).expect("decodes"), message);
    assert!(message.is_relayed());
    let out_of_range = Message::BossWait {
        slot: u8::MAX,
        waiting: false,
    };
    assert!(out_of_range.validate().is_err());
}
