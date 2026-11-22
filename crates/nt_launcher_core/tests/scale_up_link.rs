use std::thread;
use std::time::Duration;

use nt_launcher_core::{
    AUTOMATA, LauncherConfig, LauncherCore, LauncherEvent, LauncherNet, NetState,
};
use nt_net::LoopbackNetwork;
use replicant_proto::{
    LOBBY_MODE_CONTINUE, LOBBY_MODE_NEW_GAME, Message, ScaleUpAnswer, ScaleUpAsk,
};

use crate::bridge::{FakeGame, loopback_factory, wait_until};

const QUIET_MILLIS: u64 = 300;
const GUEST_SLOT: u8 = 1;

struct Member {
    core: LauncherCore,
    events: Vec<LauncherEvent>,
}

impl Member {
    fn new(network: &LoopbackNetwork, name: &str) -> Self {
        let core =
            LauncherCore::new(LauncherConfig::new(name), loopback_factory(network)).expect("core");
        Self {
            core,
            events: Vec::new(),
        }
    }

    fn gather(&mut self) {
        self.events.extend(self.core.drain_events());
    }

    fn starts(&mut self) -> Vec<u8> {
        self.gather();
        self.events
            .iter()
            .filter_map(|event| match event {
                LauncherEvent::LobbyStartReceived { mode } => Some(*mode),
                _ => None,
            })
            .collect()
    }

    fn answers(&mut self) -> Vec<(u8, bool)> {
        self.gather();
        self.events
            .iter()
            .filter_map(|event| match event {
                LauncherEvent::ScaleUpAnswered { slot, yes } => Some((*slot, *yes)),
                _ => None,
            })
            .collect()
    }

    fn host_asked(&mut self) -> Vec<ScaleUpAsk> {
        self.gather();
        self.events
            .iter()
            .filter_map(|event| match event {
                LauncherEvent::HostAsked { ask } => Some(ask.clone()),
                _ => None,
            })
            .collect()
    }

    fn is_connected(&self) -> bool {
        matches!(self.core.status().state, NetState::Connected { .. })
    }
}

fn settle(members: &mut [&mut Member]) {
    thread::sleep(Duration::from_millis(QUIET_MILLIS));
    for member in members {
        member.gather();
    }
}

fn open_ask() -> ScaleUpAsk {
    ScaleUpAsk::Open {
        slot: GUEST_SLOT,
        name: "Guesty".to_owned(),
    }
}

fn withdrawn_ask() -> ScaleUpAsk {
    ScaleUpAsk::Withdrawn { slot: GUEST_SLOT }
}

fn asks_the_game_saw(game: &mut FakeGame) -> Vec<ScaleUpAsk> {
    game.pump();
    game.received
        .iter()
        .filter_map(|message| match message {
            Message::ScaleUpAsk(ask) => Some(ask.clone()),
            _ => None,
        })
        .collect()
}

struct Table {
    host: Member,
    guest: Member,
    host_game: FakeGame,
    guest_game: FakeGame,
}

fn seated_table(name: &str) -> Table {
    let network = LoopbackNetwork::new();
    let host = Member::new(&network, &format!("{name}Host"));
    let guest = Member::new(&network, "Guesty");
    let host_game = FakeGame::attach(&host.core, &LauncherConfig::new("x"));
    let guest_game = FakeGame::attach(&guest.core, &LauncherConfig::new("x"));
    assert!(wait_until(|| host.core.link_status().game_linked));
    assert!(wait_until(|| guest.core.link_status().game_linked));
    let ticket = host.core.host().expect("ticket");
    guest.core.join(&ticket).expect("join");
    assert!(wait_until(|| host.is_connected() && guest.is_connected()));
    Table {
        host,
        guest,
        host_game,
        guest_game,
    }
}

#[test]
fn the_question_reaches_the_hosts_game_and_tells_the_guest_its_host_is_being_asked() {
    let mut table = seated_table("ask");
    table.host.core.send_scale_up_ask(open_ask());
    assert!(wait_until(|| {
        asks_the_game_saw(&mut table.host_game) == vec![open_ask()]
    }));
    assert!(wait_until(|| table.guest.host_asked() == vec![open_ask()]));
    settle(&mut [&mut table.host, &mut table.guest]);
    assert!(
        asks_the_game_saw(&mut table.guest_game).is_empty(),
        "the guest's own game is never asked"
    );
    assert_eq!(asks_the_game_saw(&mut table.host_game), vec![open_ask()]);
}

#[test]
fn the_answer_from_the_hosts_game_reaches_the_launcher_once() {
    let mut table = seated_table("answer");
    table.host.core.send_scale_up_ask(open_ask());
    assert!(wait_until(
        || !asks_the_game_saw(&mut table.host_game).is_empty()
    ));
    let yes = Message::ScaleUpAnswer(ScaleUpAnswer {
        slot: GUEST_SLOT,
        yes: true,
    });
    table.host_game.send_message(&yes);
    table.host_game.send_message(&yes);
    assert!(wait_until(
        || table.host.answers() == vec![(GUEST_SLOT, true)]
    ));
    settle(&mut [&mut table.host, &mut table.guest]);
    assert_eq!(
        table.host.answers(),
        vec![(GUEST_SLOT, true)],
        "one answer is one action"
    );
}

#[test]
fn an_answer_nobody_asked_for_does_nothing() {
    let mut table = seated_table("unasked");
    let answer = Message::ScaleUpAnswer(ScaleUpAnswer {
        slot: GUEST_SLOT,
        yes: true,
    });
    table.host_game.send_message(&answer);
    settle(&mut [&mut table.host, &mut table.guest]);
    assert!(table.host.answers().is_empty());
}

#[test]
fn an_answer_about_another_guest_than_the_one_asked_about_does_nothing() {
    let mut table = seated_table("other");
    table.host.core.send_scale_up_ask(open_ask());
    assert!(wait_until(
        || !asks_the_game_saw(&mut table.host_game).is_empty()
    ));
    table
        .host_game
        .send_message(&Message::ScaleUpAnswer(ScaleUpAnswer {
            slot: 0,
            yes: true,
        }));
    settle(&mut [&mut table.host, &mut table.guest]);
    assert!(table.host.answers().is_empty());
}

#[test]
fn a_withdrawn_question_reaches_the_game_and_the_guest_and_a_late_answer_does_nothing() {
    let mut table = seated_table("withdraw");
    table.host.core.send_scale_up_ask(open_ask());
    assert!(wait_until(
        || asks_the_game_saw(&mut table.host_game) == vec![open_ask()]
    ));
    table.host.core.send_scale_up_ask(withdrawn_ask());
    assert!(wait_until(|| {
        asks_the_game_saw(&mut table.host_game) == vec![open_ask(), withdrawn_ask()]
    }));
    assert!(wait_until(
        || table.guest.host_asked() == vec![open_ask(), withdrawn_ask()]
    ));
    table
        .host_game
        .send_message(&Message::ScaleUpAnswer(ScaleUpAnswer {
            slot: GUEST_SLOT,
            yes: true,
        }));
    settle(&mut [&mut table.host, &mut table.guest]);
    assert!(table.host.answers().is_empty());
}

#[test]
fn withdrawing_a_question_that_is_not_open_says_nothing() {
    let mut table = seated_table("nothing_open");
    table.host.core.send_scale_up_ask(withdrawn_ask());
    settle(&mut [&mut table.host, &mut table.guest]);
    assert!(asks_the_game_saw(&mut table.host_game).is_empty());
    assert!(table.guest.host_asked().is_empty());
}

#[test]
fn a_guest_who_leaves_withdraws_the_question_in_the_hosts_game() {
    let mut table = seated_table("leaves");
    table.host.core.send_scale_up_ask(open_ask());
    assert!(wait_until(
        || asks_the_game_saw(&mut table.host_game) == vec![open_ask()]
    ));
    table.guest.core.leave();
    assert!(wait_until(|| {
        asks_the_game_saw(&mut table.host_game) == vec![open_ask(), withdrawn_ask()]
    }));
    table
        .host_game
        .send_message(&Message::ScaleUpAnswer(ScaleUpAnswer {
            slot: GUEST_SLOT,
            yes: false,
        }));
    settle(&mut [&mut table.host, &mut table.guest]);
    assert!(
        table.host.answers().is_empty(),
        "an answer for a guest who left"
    );
}

#[test]
fn a_host_who_closes_the_lobby_withdraws_the_question_in_its_game() {
    let mut table = seated_table("closes");
    table.host.core.send_scale_up_ask(open_ask());
    assert!(wait_until(
        || asks_the_game_saw(&mut table.host_game) == vec![open_ask()]
    ));
    table.host.core.leave();
    assert!(wait_until(|| {
        asks_the_game_saw(&mut table.host_game) == vec![open_ask(), withdrawn_ask()]
    }));
}

#[test]
fn a_game_that_links_after_the_question_was_asked_is_still_asked() {
    let network = LoopbackNetwork::new();
    let mut host = Member::new(&network, "LateGameHost");
    let guest = Member::new(&network, "Guesty");
    let ticket = host.core.host().expect("ticket");
    guest.core.join(&ticket).expect("join");
    assert!(wait_until(|| host.is_connected() && guest.is_connected()));
    host.core.send_scale_up_ask(open_ask());
    settle(&mut [&mut host]);
    let mut game = FakeGame::attach(&host.core, &LauncherConfig::new("x"));
    assert!(wait_until(
        || asks_the_game_saw(&mut game) == vec![open_ask()]
    ));
}

#[test]
fn telling_the_game_there_is_no_save_to_copy_stays_between_the_launcher_and_its_game() {
    let mut table = seated_table("nosave");
    let no_save = ScaleUpAsk::NoSaveYet {
        slot: GUEST_SLOT,
        name: "Guesty".to_owned(),
    };
    table.host.core.send_scale_up_ask(no_save.clone());
    assert!(wait_until(
        || asks_the_game_saw(&mut table.host_game) == vec![no_save.clone()]
    ));
    settle(&mut [&mut table.host, &mut table.guest]);
    assert!(table.guest.host_asked().is_empty());
    assert!(asks_the_game_saw(&mut table.guest_game).is_empty());
    table
        .host_game
        .send_message(&Message::ScaleUpAnswer(ScaleUpAnswer {
            slot: GUEST_SLOT,
            yes: true,
        }));
    settle(&mut [&mut table.host, &mut table.guest]);
    assert!(
        table.host.answers().is_empty(),
        "nothing was open to answer"
    );
}

#[test]
fn a_guest_never_answers_for_the_host_and_never_forwards_an_ask_to_its_game() {
    let mut table = seated_table("guest_side");
    table.guest.core.send_scale_up_ask(open_ask());
    table
        .guest_game
        .send_message(&Message::ScaleUpAnswer(ScaleUpAnswer {
            slot: GUEST_SLOT,
            yes: true,
        }));
    settle(&mut [&mut table.host, &mut table.guest]);
    assert!(table.guest.answers().is_empty());
    assert!(table.host.host_asked().is_empty());
    assert!(asks_the_game_saw(&mut table.guest_game).is_empty());
}

#[test]
fn a_held_start_goes_to_a_late_member_only_when_it_is_released() {
    let network = LoopbackNetwork::new();
    let mut host = Member::new(&network, "Hosty");
    let mut guest = Member::new(&network, "Guesty");
    let ticket = host.core.host().expect("ticket");
    host.core.start_lobby(LOBBY_MODE_CONTINUE);
    host.core.hold_late_starts(true);
    guest.core.join(&ticket).expect("join");
    assert!(wait_until(|| host.is_connected() && guest.is_connected()));
    settle(&mut [&mut host, &mut guest]);
    assert!(guest.starts().is_empty(), "the start is held back");
    host.core.release_start(GUEST_SLOT, None);
    assert!(wait_until(|| guest.starts() == vec![LOBBY_MODE_CONTINUE]));
    settle(&mut [&mut host, &mut guest]);
    assert_eq!(guest.starts(), vec![LOBBY_MODE_CONTINUE], "once");
}

#[test]
fn a_released_start_can_carry_another_mode() {
    let network = LoopbackNetwork::new();
    let host = Member::new(&network, "Hosty");
    let mut guest = Member::new(&network, "Guesty");
    let ticket = host.core.host().expect("ticket");
    host.core.start_lobby(LOBBY_MODE_NEW_GAME);
    host.core.hold_late_starts(true);
    guest.core.join(&ticket).expect("join");
    assert!(wait_until(|| host.is_connected() && guest.is_connected()));
    host.core
        .release_start(GUEST_SLOT, Some(LOBBY_MODE_CONTINUE));
    assert!(wait_until(|| guest.starts() == vec![LOBBY_MODE_CONTINUE]));
}

#[test]
fn releasing_nothing_that_was_started_sends_nothing() {
    let network = LoopbackNetwork::new();
    let mut host = Member::new(&network, "Hosty");
    let mut guest = Member::new(&network, "Guesty");
    let ticket = host.core.host().expect("ticket");
    guest.core.join(&ticket).expect("join");
    assert!(wait_until(|| host.is_connected() && guest.is_connected()));
    host.core
        .release_start(GUEST_SLOT, Some(LOBBY_MODE_CONTINUE));
    settle(&mut [&mut host, &mut guest]);
    assert!(guest.starts().is_empty());
}

#[test]
fn a_hold_ends_with_the_session() {
    let network = LoopbackNetwork::new();
    let host = Member::new(&network, "Hosty");
    let mut guest = Member::new(&network, "Guesty");
    host.core.host().expect("ticket");
    host.core.start_lobby(LOBBY_MODE_CONTINUE);
    host.core.hold_late_starts(true);
    host.core.leave();
    let ticket = host.core.host().expect("a second ticket");
    host.core.start_lobby(LOBBY_MODE_CONTINUE);
    guest.core.join(&ticket).expect("join");
    assert!(wait_until(|| guest.starts() == vec![LOBBY_MODE_CONTINUE]));
}

#[test]
fn a_game_that_keeps_no_campaign_saves_is_never_held() {
    let network = LoopbackNetwork::new();
    let mut host = Member {
        core: LauncherCore::for_game(&AUTOMATA, "Hosty", loopback_factory(&network)).expect("core"),
        events: Vec::new(),
    };
    let mut guest = Member {
        core: LauncherCore::for_game(&AUTOMATA, "Guesty", loopback_factory(&network))
            .expect("core"),
        events: Vec::new(),
    };
    let ticket = host.core.host().expect("ticket");
    host.core.start_lobby(0);
    host.core.hold_late_starts(true);
    guest.core.join(&ticket).expect("join");
    assert!(wait_until(|| guest.starts() == vec![0]));
    host.core.release_start(GUEST_SLOT, Some(0));
    settle(&mut [&mut host, &mut guest]);
    assert_eq!(guest.starts(), vec![0], "no second start either");
}

#[test]
fn a_no_save_note_for_a_game_that_is_not_linked_yet_is_delivered_once_when_it_links() {
    let network = LoopbackNetwork::new();
    let host = Member::new(&network, "NoteHost");
    let guest = Member::new(&network, "Guesty");
    let ticket = host.core.host().expect("ticket");
    guest.core.join(&ticket).expect("join");
    assert!(wait_until(|| host.is_connected() && guest.is_connected()));
    let note = ScaleUpAsk::NoSaveYet {
        slot: GUEST_SLOT,
        name: "Guesty".to_owned(),
    };
    host.core.send_scale_up_ask(note.clone());
    let mut game = FakeGame::attach(&host.core, &LauncherConfig::new("x"));
    assert!(wait_until(
        || asks_the_game_saw(&mut game) == vec![note.clone()]
    ));
    let mut again = FakeGame::attach(&host.core, &LauncherConfig::new("x"));
    thread::sleep(Duration::from_millis(QUIET_MILLIS));
    assert!(asks_the_game_saw(&mut again).is_empty(), "delivered once");
}

#[test]
fn a_question_that_was_answered_can_still_be_withdrawn_for_the_guest() {
    let mut table = seated_table("answered_then_withdrawn");
    table.host.core.send_scale_up_ask(open_ask());
    assert!(wait_until(
        || asks_the_game_saw(&mut table.host_game) == vec![open_ask()]
    ));
    table
        .host_game
        .send_message(&Message::ScaleUpAnswer(ScaleUpAnswer {
            slot: GUEST_SLOT,
            yes: true,
        }));
    assert!(wait_until(
        || table.host.answers() == vec![(GUEST_SLOT, true)]
    ));
    table.host.core.send_scale_up_ask(withdrawn_ask());
    assert!(wait_until(
        || table.guest.host_asked() == vec![open_ask(), withdrawn_ask()]
    ));
}

#[test]
fn releasing_the_start_closes_the_open_question_without_telling_anyone_again() {
    let mut table = seated_table("release_closes");
    table.host.core.start_lobby(LOBBY_MODE_CONTINUE);
    table.host.core.send_scale_up_ask(open_ask());
    assert!(wait_until(
        || asks_the_game_saw(&mut table.host_game) == vec![open_ask()]
    ));
    table.host.core.release_start(GUEST_SLOT, None);
    assert!(wait_until(
        || table.guest.starts() == vec![LOBBY_MODE_CONTINUE]
    ));
    table.host.core.send_scale_up_ask(withdrawn_ask());
    settle(&mut [&mut table.host, &mut table.guest]);
    assert_eq!(asks_the_game_saw(&mut table.host_game), vec![open_ask()]);
}
