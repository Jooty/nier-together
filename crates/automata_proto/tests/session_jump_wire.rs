use automata_game::session_jump::SessionJumpOrder;
use automata_game::start::MAX_CHAPTER_RECORDS;
use automata_proto::{Message, WireError, decode, encode};
use nt_wire::GameMessage;

const SESSION_JUMP_VARIANT_TAG: u8 = 40;

fn an_order(start_index: u16) -> Message {
    Message::SessionJump(SessionJumpOrder { start_index })
}

#[test]
fn the_session_jump_message_comes_after_every_existing_variant() {
    let bytes = encode(&an_order(3)).expect("encodes");
    assert_eq!(bytes[0], SESSION_JUMP_VARIANT_TAG);
    assert_eq!(an_order(3).kind_name(), "SessionJump");
}

#[test]
fn a_session_jump_order_round_trips() {
    for start_index in [0, 3, 41, 1023] {
        let bytes = encode(&an_order(start_index)).expect("encodes");
        assert_eq!(decode(&bytes).expect("decodes"), an_order(start_index));
    }
}

#[test]
fn a_session_jump_order_is_relayed_reliable_and_not_latest_wins() {
    let message = an_order(3);
    assert!(message.is_relayed());
    assert!(!message.is_latest_wins());
    assert!(!GameMessage::forwarded_to_other_guests(&message));
    assert_eq!(GameMessage::ordered_tick(&message), None);
}

#[test]
fn a_start_index_beyond_the_chapter_table_is_refused() {
    let beyond = u16::try_from(MAX_CHAPTER_RECORDS).expect("the table size fits");
    assert_eq!(
        an_order(beyond).validate(),
        Err(WireError::OutOfRange {
            field: "session_jump"
        })
    );
    assert_eq!(an_order(beyond - 1).validate(), Ok(()));
}
