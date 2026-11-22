use std::thread;
use std::time::Duration;

use nt_launcher_core::{
    AUTOMATA, FREE_ROAM, LauncherConfig, LauncherCore, LauncherEvent, LauncherNet, NetState,
};
use nt_net::LoopbackNetwork;
use replicant_proto::{LOBBY_MODE_CONTINUE, LOBBY_MODE_NEW_GAME};

use crate::bridge::{FakeGame, loopback_factory, wait_until};

const QUIET_MILLIS: u64 = 300;

struct Member {
    core: LauncherCore,
    events: Vec<LauncherEvent>,
}

impl Member {
    fn new(network: &LoopbackNetwork, name: &str, automata: bool) -> Self {
        let core = if automata {
            LauncherCore::for_game(&AUTOMATA, name, loopback_factory(network)).expect("core")
        } else {
            LauncherCore::new(LauncherConfig::new(name), loopback_factory(network)).expect("core")
        };
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

fn both_games() -> [bool; 2] {
    [false, true]
}

#[test]
fn a_member_who_joins_after_the_start_is_told_the_start_once_with_the_options_before_it() {
    for automata in both_games() {
        let network = LoopbackNetwork::new();
        let mut host = Member::new(&network, "Hosty", automata);
        let mut guest = Member::new(&network, "Guesty", automata);
        let ticket = host.core.host().expect("ticket");
        host.core.start_lobby(LOBBY_MODE_CONTINUE);
        guest.core.join(&ticket).expect("join");
        assert!(wait_until(|| guest.starts() == vec![LOBBY_MODE_CONTINUE]));
        settle(&mut [&mut host, &mut guest]);
        assert_eq!(guest.starts(), vec![LOBBY_MODE_CONTINUE], "once, not twice");
        if !automata {
            let options_at = guest
                .events
                .iter()
                .position(|event| matches!(event, LauncherEvent::SessionOptionsChanged { .. }));
            let start_at = guest
                .events
                .iter()
                .position(|event| matches!(event, LauncherEvent::LobbyStartReceived { .. }));
            assert!(
                options_at.is_some() && options_at < start_at,
                "the host's options reach the member before the start"
            );
        }
    }
}

#[test]
fn nothing_is_sent_to_a_member_when_the_host_has_not_started() {
    for automata in both_games() {
        let network = LoopbackNetwork::new();
        let mut host = Member::new(&network, "Hosty", automata);
        let mut guest = Member::new(&network, "Guesty", automata);
        let ticket = host.core.host().expect("ticket");
        guest.core.join(&ticket).expect("join");
        assert!(wait_until(|| guest.is_connected()));
        settle(&mut [&mut host, &mut guest]);
        assert!(guest.starts().is_empty());
    }
}

#[test]
fn a_start_pressed_with_a_member_present_is_not_sent_again_to_that_member_later() {
    let network = LoopbackNetwork::new();
    let mut host = Member::new(&network, "Hosty", false);
    let mut guest = Member::new(&network, "Guesty", false);
    let ticket = host.core.host().expect("ticket");
    guest.core.join(&ticket).expect("join");
    assert!(wait_until(|| host.is_connected() && guest.is_connected()));
    host.core.start_lobby(LOBBY_MODE_NEW_GAME);
    assert!(wait_until(|| guest.starts() == vec![LOBBY_MODE_NEW_GAME]));
    settle(&mut [&mut host, &mut guest]);
    assert_eq!(guest.starts(), vec![LOBBY_MODE_NEW_GAME]);
}

#[test]
fn two_members_joining_one_after_the_other_each_get_the_start_and_the_first_gets_no_second_one() {
    let network = LoopbackNetwork::new();
    let mut host = Member::new(&network, "Hosty", true);
    let mut first = Member::new(&network, "First", true);
    let mut second = Member::new(&network, "Second", true);
    host.core.set_rules(FREE_ROAM.rules());
    let ticket = host.core.host().expect("ticket");
    first.core.join(&ticket).expect("join");
    assert!(wait_until(|| first.is_connected()));
    host.core.start_lobby(0);
    assert!(wait_until(|| first.starts() == vec![0]));
    second.core.join(&ticket).expect("join");
    assert!(wait_until(|| second.starts() == vec![0]));
    settle(&mut [&mut host, &mut first, &mut second]);
    assert_eq!(first.starts(), vec![0]);
    assert_eq!(second.starts(), vec![0]);
}

#[test]
fn two_members_joining_at_once_after_the_start_each_get_it_once() {
    let network = LoopbackNetwork::new();
    let mut host = Member::new(&network, "Hosty", true);
    let mut first = Member::new(&network, "First", true);
    let mut second = Member::new(&network, "Second", true);
    host.core.set_rules(FREE_ROAM.rules());
    let ticket = host.core.host().expect("ticket");
    host.core.start_lobby(0);
    first.core.join(&ticket).expect("join");
    second.core.join(&ticket).expect("join");
    assert!(wait_until(
        || first.starts() == vec![0] && second.starts() == vec![0]
    ));
    settle(&mut [&mut host, &mut first, &mut second]);
    assert_eq!(first.starts(), vec![0]);
    assert_eq!(second.starts(), vec![0]);
}

#[test]
fn a_member_who_leaves_and_joins_again_with_the_same_ticket_is_told_the_start_again() {
    let network = LoopbackNetwork::new();
    let mut host = Member::new(&network, "Hosty", false);
    let mut guest = Member::new(&network, "Guesty", false);
    let ticket = host.core.host().expect("ticket");
    host.core.start_lobby(LOBBY_MODE_CONTINUE);
    guest.core.join(&ticket).expect("join");
    assert!(wait_until(|| guest.starts().len() == 1));
    guest.core.leave();
    assert!(wait_until(|| {
        host.gather();
        host.events
            .iter()
            .any(|event| matches!(event, LauncherEvent::MemberLeft { .. }))
    }));
    guest.events.clear();
    guest.core.join(&ticket).expect("join again");
    assert!(wait_until(|| guest.starts() == vec![LOBBY_MODE_CONTINUE]));
}

#[test]
fn a_host_whose_game_closed_does_not_start_anyone_who_joins_afterwards() {
    let network = LoopbackNetwork::new();
    let mut host = Member::new(&network, "Hosty", false);
    let mut guest = Member::new(&network, "Guesty", false);
    let game = FakeGame::attach(&host.core, &LauncherConfig::new("x"));
    assert!(wait_until(|| host.core.link_status().game_linked));
    let ticket = host.core.host().expect("ticket");
    host.core.start_lobby(LOBBY_MODE_NEW_GAME);
    drop(game);
    assert!(wait_until(|| !host.core.link_status().game_linked));
    guest.core.join(&ticket).expect("join");
    assert!(wait_until(|| guest.is_connected()));
    settle(&mut [&mut host, &mut guest]);
    assert!(guest.starts().is_empty());
}

#[test]
fn a_new_session_never_carries_the_last_sessions_start() {
    let network = LoopbackNetwork::new();
    let mut host = Member::new(&network, "Hosty", false);
    let mut guest = Member::new(&network, "Guesty", false);
    host.core.host().expect("ticket");
    host.core.start_lobby(LOBBY_MODE_NEW_GAME);
    host.core.leave();
    let ticket = host.core.host().expect("a second ticket");
    guest.core.join(&ticket).expect("join");
    assert!(wait_until(|| guest.is_connected()));
    settle(&mut [&mut host, &mut guest]);
    assert!(guest.starts().is_empty());
}

#[test]
fn a_full_replicant_session_refuses_a_third_without_disturbing_the_second() {
    let network = LoopbackNetwork::new();
    let mut host = Member::new(&network, "Hosty", false);
    let mut guest = Member::new(&network, "Guesty", false);
    let mut third = Member::new(&network, "Third", false);
    let ticket = host.core.host().expect("ticket");
    host.core.start_lobby(LOBBY_MODE_CONTINUE);
    guest.core.join(&ticket).expect("join");
    assert!(wait_until(|| guest.starts().len() == 1));
    third.core.join(&ticket).expect("join starts");
    settle(&mut [&mut host, &mut guest, &mut third]);
    assert!(third.starts().is_empty());
    assert_eq!(guest.starts().len(), 1);
    assert!(guest.is_connected());
}
