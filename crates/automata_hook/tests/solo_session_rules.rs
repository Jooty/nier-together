use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use automata_game::build::MAIN_PE_IDENTITY;
use automata_game::character::Character;
use automata_game::exe::ExeVariant;
use automata_game::mode::SessionMode;
use automata_game::travel::JumpRole;
use automata_hook::net::{AutomataNet, SessionModeExt, load_role_of};
use nt_hook::net::{LinkPeer, NetState, NetThread};
use nt_launcher_core::{
    AUTOMATA, FREE_ROAM_MODE, LauncherCore, LauncherNet, SessionOptions, TransportFactory,
    rules_for_mode,
};
use nt_net::{LocalLinkTransport, LoopbackNetwork, Transport};
use nt_wire::{BuildGate, STORY_MODE, SessionRules};

const WAIT_LIMIT: Duration = Duration::from_secs(20);
const SETTLE_TIME: Duration = Duration::from_millis(300);
const MAIN_BUILD: BuildGate = BuildGate {
    time_date_stamp: MAIN_PE_IDENTITY.time_date_stamp,
    size_of_image: MAIN_PE_IDENTITY.size_of_image,
};

struct SoloRig {
    launcher: LauncherCore,
    game: NetThread<AutomataNet>,
    ticket: String,
    network: LoopbackNetwork,
}

fn wait_until(mut condition: impl FnMut() -> bool) -> bool {
    let deadline = Instant::now() + WAIT_LIMIT;
    while Instant::now() < deadline {
        if condition() {
            return true;
        }
        thread::sleep(Duration::from_millis(3));
    }
    false
}

fn loopback_factory(network: &LoopbackNetwork) -> TransportFactory {
    let network = network.clone();
    Arc::new(move || Ok(Box::new(network.endpoint()) as Box<dyn Transport>))
}

fn launcher_on(network: &LoopbackNetwork, name: &str) -> LauncherCore {
    LauncherCore::for_game(&AUTOMATA, name, loopback_factory(network)).expect("launcher starts")
}

fn game_linked_to(launcher: &LauncherCore) -> NetThread<AutomataNet> {
    NetThread::spawn(
        AutomataNet::new(Character::TwoB, ExeVariant::Main),
        LocalLinkTransport::game(MAIN_BUILD),
        LinkPeer::launcher_on_port(launcher.link_port()),
    )
    .expect("the net worker starts")
}

fn free_roam_rules() -> SessionRules {
    rules_for_mode(&AUTOMATA, FREE_ROAM_MODE).expect("automata has free roam")
}

fn campaign_rules() -> SessionRules {
    SessionRules {
        mode: STORY_MODE,
        max_players: 2,
        map: 0,
    }
}

fn hosting_alone(rules: SessionRules) -> SoloRig {
    let network = LoopbackNetwork::new();
    let launcher = launcher_on(&network, "Hosty");
    launcher.set_rules(rules);
    let ticket = launcher.host().expect("hosts");
    let game = game_linked_to(&launcher);
    SoloRig {
        launcher,
        game,
        ticket,
        network,
    }
}

fn mode_of(game: &NetThread<AutomataNet>) -> SessionMode {
    game.view().session_mode()
}

fn host_looks_allowed(game: &NetThread<AutomataNet>) -> bool {
    game.view().looks_allowed()
}

fn wait_for_link(rig: &SoloRig) {
    assert!(wait_until(|| rig.launcher.link_status().game_linked));
    thread::sleep(SETTLE_TIME);
}

fn any_look_options(allowed: bool) -> SessionOptions {
    SessionOptions {
        any_look_allowed: allowed,
        ..SessionOptions::default()
    }
}

#[test]
fn a_host_alone_in_free_roam_reads_free_roam_with_every_look_allowed() {
    let rig = hosting_alone(free_roam_rules());
    wait_for_link(&rig);
    assert_eq!(mode_of(&rig.game), SessionMode::FreeRoam);
    assert!(host_looks_allowed(&rig.game));
    assert!(!rig.game.view().session_mode().story_rules_apply());
    assert!(!rig.game.view().campaign_link_is_up());
    assert_eq!(rig.game.view().rules, free_roam_rules());
}

#[test]
fn a_game_that_links_before_the_host_starts_hosting_still_reads_free_roam() {
    let network = LoopbackNetwork::new();
    let launcher = launcher_on(&network, "Hosty");
    launcher.set_rules(free_roam_rules());
    let game = game_linked_to(&launcher);
    assert!(wait_until(|| launcher.link_status().game_linked));
    assert!(wait_until(|| mode_of(&game) == SessionMode::FreeRoam));
    launcher.host().expect("hosts");
    thread::sleep(SETTLE_TIME);
    assert_eq!(mode_of(&game), SessionMode::FreeRoam);
}

#[test]
fn the_rules_change_of_a_host_alone_reaches_the_game() {
    let rig = hosting_alone(campaign_rules());
    wait_for_link(&rig);
    assert_eq!(mode_of(&rig.game), SessionMode::Campaign);
    rig.launcher.set_rules(free_roam_rules());
    assert!(wait_until(|| mode_of(&rig.game) == SessionMode::FreeRoam));
    rig.launcher.set_rules(campaign_rules());
    assert!(wait_until(|| mode_of(&rig.game) == SessionMode::Campaign));
}

#[test]
fn a_host_alone_in_the_campaign_has_looks_only_while_the_switch_is_on() {
    let rig = hosting_alone(campaign_rules());
    wait_for_link(&rig);
    assert_eq!(mode_of(&rig.game), SessionMode::Campaign);
    assert!(rig.game.view().session_mode().story_rules_apply());
    assert!(!host_looks_allowed(&rig.game));
    rig.launcher.set_session_options(any_look_options(true));
    assert!(wait_until(|| host_looks_allowed(&rig.game)));
    rig.launcher.set_session_options(any_look_options(false));
    assert!(wait_until(|| !host_looks_allowed(&rig.game)));
}

#[test]
fn a_switch_set_before_the_game_links_is_there_when_it_links() {
    let network = LoopbackNetwork::new();
    let launcher = launcher_on(&network, "Hosty");
    launcher.set_rules(campaign_rules());
    launcher.set_session_options(any_look_options(true));
    launcher.host().expect("hosts");
    let game = game_linked_to(&launcher);
    assert!(wait_until(|| host_looks_allowed(&game)));
    assert_eq!(mode_of(&game), SessionMode::Campaign);
}

#[test]
fn free_roam_stays_free_roam_while_a_guest_joins_and_leaves() {
    let rig = hosting_alone(free_roam_rules());
    wait_for_link(&rig);
    let guest = launcher_on(&rig.network, "Guesty");
    guest.join(&rig.ticket).expect("guest dials");
    assert!(wait_until(|| rig.game.view().others().next().is_some()));
    assert_eq!(mode_of(&rig.game), SessionMode::FreeRoam);
    assert!(host_looks_allowed(&rig.game));
    guest.leave();
    assert!(wait_until(|| rig.game.view().others().next().is_none()));
    thread::sleep(SETTLE_TIME);
    assert_eq!(mode_of(&rig.game), SessionMode::FreeRoam);
    assert!(host_looks_allowed(&rig.game));
}
#[test]
fn a_host_alone_in_the_campaign_is_not_connected_and_a_guest_makes_it_so() {
    let rig = hosting_alone(campaign_rules());
    wait_for_link(&rig);
    let alone = rig.game.view();
    assert_ne!(alone.state, NetState::Connected);
    assert!(!alone.campaign_link_is_up());
    assert_eq!(load_role_of(&alone), JumpRole::Solo);
    let guest = launcher_on(&rig.network, "Guesty");
    guest.join(&rig.ticket).expect("guest dials");
    assert!(wait_until(|| rig.game.view().campaign_link_is_up()));
    assert_ne!(load_role_of(&rig.game.view()), JumpRole::Solo);
    guest.leave();
    assert!(wait_until(|| !rig.game.view().campaign_link_is_up()));
    assert_eq!(load_role_of(&rig.game.view()), JumpRole::Solo);
}

fn session_open(game: &NetThread<AutomataNet>) -> bool {
    game.view().extras.session_open
}

fn guest_game_in(rig: &SoloRig) -> (LauncherCore, NetThread<AutomataNet>) {
    let guest = launcher_on(&rig.network, "Guesty");
    let game = game_linked_to(&guest);
    guest.join(&rig.ticket).expect("guest dials");
    assert!(wait_until(|| game.view().others().next().is_some()));
    (guest, game)
}

#[test]
fn a_host_alone_has_an_open_session_that_closes_when_it_ends() {
    let rig = hosting_alone(free_roam_rules());
    wait_for_link(&rig);
    assert!(session_open(&rig.game));
    assert_ne!(rig.game.view().state, NetState::Connected);
    rig.launcher.leave();
    assert!(wait_until(|| !session_open(&rig.game)));
    assert_eq!(mode_of(&rig.game), SessionMode::FreeRoam);
    assert!(host_looks_allowed(&rig.game));
}

#[test]
fn the_session_stays_open_through_a_guest_joining_and_leaving() {
    let rig = hosting_alone(free_roam_rules());
    wait_for_link(&rig);
    let (guest, _guest_game) = guest_game_in(&rig);
    assert!(session_open(&rig.game));
    guest.leave();
    assert!(wait_until(|| rig.game.view().others().next().is_none()));
    thread::sleep(SETTLE_TIME);
    assert!(session_open(&rig.game));
    rig.launcher.leave();
    assert!(wait_until(|| !session_open(&rig.game)));
}

#[test]
fn a_guest_who_leaves_a_free_roam_session_keeps_playing_free_roam() {
    let rig = hosting_alone(free_roam_rules());
    wait_for_link(&rig);
    let (guest, guest_game) = guest_game_in(&rig);
    assert_eq!(mode_of(&guest_game), SessionMode::FreeRoam);
    guest.leave();
    assert!(wait_until(|| guest_game.view().others().next().is_none()));
    thread::sleep(SETTLE_TIME);
    assert_eq!(mode_of(&guest_game), SessionMode::FreeRoam);
    assert!(host_looks_allowed(&guest_game));
}

#[test]
fn a_guest_whose_host_closes_the_lobby_keeps_playing_free_roam() {
    let rig = hosting_alone(free_roam_rules());
    wait_for_link(&rig);
    let (_guest, guest_game) = guest_game_in(&rig);
    rig.launcher.leave();
    assert!(wait_until(|| guest_game.view().others().next().is_none()));
    thread::sleep(SETTLE_TIME);
    assert_eq!(mode_of(&guest_game), SessionMode::FreeRoam);
    assert!(host_looks_allowed(&guest_game));
}

#[test]
fn a_host_who_closes_the_lobby_while_playing_keeps_free_roam() {
    let rig = hosting_alone(free_roam_rules());
    wait_for_link(&rig);
    let (_guest, _guest_game) = guest_game_in(&rig);
    rig.launcher.leave();
    thread::sleep(SETTLE_TIME);
    assert_eq!(mode_of(&rig.game), SessionMode::FreeRoam);
    assert!(host_looks_allowed(&rig.game));
    rig.launcher.set_rules(campaign_rules());
    thread::sleep(SETTLE_TIME);
    assert_eq!(mode_of(&rig.game), SessionMode::FreeRoam);
}

#[test]
fn the_look_switch_and_the_rules_survive_the_last_guest_leaving_a_campaign() {
    let rig = hosting_alone(campaign_rules());
    rig.launcher.set_session_options(any_look_options(true));
    wait_for_link(&rig);
    assert!(wait_until(|| host_looks_allowed(&rig.game)));
    let (guest, guest_game) = guest_game_in(&rig);
    assert!(wait_until(|| host_looks_allowed(&guest_game)));
    guest.leave();
    assert!(wait_until(|| rig.game.view().others().next().is_none()));
    thread::sleep(SETTLE_TIME);
    assert_eq!(mode_of(&rig.game), SessionMode::Campaign);
    assert!(host_looks_allowed(&rig.game));
    rig.launcher.leave();
    thread::sleep(SETTLE_TIME);
    assert_eq!(mode_of(&rig.game), SessionMode::Campaign);
    assert!(host_looks_allowed(&rig.game));
    assert!(host_looks_allowed(&guest_game));
}
