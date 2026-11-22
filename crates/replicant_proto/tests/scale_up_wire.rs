use replicant_proto::{
    Message, SCALE_UP_NO_LABEL, SCALE_UP_YES_LABEL, ScaleUpAnswer, ScaleUpAsk, decode, encode,
    scale_up_body, scale_up_no_save_yet_line, scale_up_title,
};

const SCALE_UP_ASK_TAG: u8 = 132;
const SCALE_UP_ANSWER_TAG: u8 = 133;

fn open_ask() -> ScaleUpAsk {
    ScaleUpAsk::Open {
        slot: 1,
        name: "Guesty".to_owned(),
    }
}

#[test]
fn the_two_scale_up_messages_come_after_every_older_variant() {
    let ask = encode(&Message::ScaleUpAsk(open_ask())).expect("encodes");
    let answer = encode(&Message::ScaleUpAnswer(ScaleUpAnswer {
        slot: 1,
        yes: true,
    }))
    .expect("encodes");
    assert_eq!(ask[0], SCALE_UP_ASK_TAG);
    assert_eq!(answer[0], SCALE_UP_ANSWER_TAG);
}

#[test]
fn every_kind_of_ask_and_both_answers_round_trip() {
    let asks = [
        open_ask(),
        ScaleUpAsk::Withdrawn { slot: 1 },
        ScaleUpAsk::NoSaveYet {
            slot: 1,
            name: "Guesty".to_owned(),
        },
    ];
    for ask in asks {
        let message = Message::ScaleUpAsk(ask);
        assert_eq!(decode(&encode(&message).expect("encodes")), Ok(message));
    }
    for yes in [true, false] {
        let message = Message::ScaleUpAnswer(ScaleUpAnswer { slot: 1, yes });
        assert_eq!(decode(&encode(&message).expect("encodes")), Ok(message));
    }
}

#[test]
fn the_ask_reaches_the_guest_and_the_answer_stays_on_the_local_link() {
    assert!(Message::ScaleUpAsk(open_ask()).is_relayed());
    let answer = Message::ScaleUpAnswer(ScaleUpAnswer { slot: 1, yes: true });
    assert!(!answer.is_relayed());
}

#[test]
fn a_slot_beyond_the_players_or_a_bad_name_is_refused() {
    let far_slot = Message::ScaleUpAnswer(ScaleUpAnswer { slot: 9, yes: true });
    assert!(far_slot.validate().is_err());
    let nameless = Message::ScaleUpAsk(ScaleUpAsk::Open {
        slot: 1,
        name: String::new(),
    });
    assert!(nameless.validate().is_err());
    let controlled = Message::ScaleUpAsk(ScaleUpAsk::Open {
        slot: 1,
        name: "a\nb".to_owned(),
    });
    assert!(controlled.validate().is_err());
    assert_eq!(Message::ScaleUpAsk(open_ask()).validate(), Ok(()));
    let withdrawn = Message::ScaleUpAsk(ScaleUpAsk::Withdrawn { slot: 1 });
    assert_eq!(withdrawn.validate(), Ok(()));
}

#[test]
fn the_words_are_the_launchers_scale_up_prompt() {
    assert_eq!(scale_up_title("Ann"), "No save for Ann");
    assert_eq!(
        scale_up_body("Ann"),
        "Ann has no save in this campaign. Give them a copy of your save under their name?"
    );
    assert_eq!(SCALE_UP_YES_LABEL, "Scale up");
    assert_eq!(SCALE_UP_NO_LABEL, "Not now");
    assert!(scale_up_no_save_yet_line("Ann").starts_with("Ann has no save here"));
}
