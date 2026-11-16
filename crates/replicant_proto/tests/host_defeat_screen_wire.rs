use replicant_proto::{
    DefeatPick, HostDefeatScreen, MAX_MESSAGE_BYTES, Message, WireError, decode, encode,
};

#[test]
fn the_hosts_defeat_screen_round_trips_and_is_relayed_to_the_guest() {
    for pick in [None, Some(DefeatPick::Continue), Some(DefeatPick::Leave)] {
        let message = Message::HostDefeatScreen(HostDefeatScreen {
            defeat: 2,
            cursor: 0,
            pick,
        });
        assert!(message.is_relayed());
        assert!(!message.is_latest_wins());
        assert_eq!(message.kind_name(), "HostDefeatScreen");
        assert_eq!(message.validate(), Ok(()));
        let bytes = encode(&message).unwrap();
        assert!(bytes.len() <= MAX_MESSAGE_BYTES);
        assert_eq!(decode(&bytes), Ok(message));
    }
}

#[test]
fn a_cursor_past_the_screens_two_items_is_refused() {
    let message = Message::HostDefeatScreen(HostDefeatScreen {
        defeat: 1,
        cursor: 2,
        pick: None,
    });
    assert_eq!(
        message.validate(),
        Err(WireError::OutOfRange {
            field: "defeat screen cursor"
        })
    );
}
