use std::sync::{Arc, PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::thread;
use std::time::{Duration, Instant};

use automata_game::build::MAIN_PE_IDENTITY;
use automata_game::character::Character;
use automata_game::exe::ExeVariant;
use automata_hook::net::{
    AutomataNet, push_announced_line, take_announced_lines_where,
    take_postable_announced_lines_where,
};
use nt_hook::net::{LinkPeer, NetThread};
use nt_launcher_core::{AUTOMATA, LauncherCore, LauncherNet, TransportFactory};
use nt_net::{LocalLinkTransport, LoopbackNetwork, Transport};
use nt_wire::{BuildGate, STORY_MODE, SessionRules};

const WAIT_LIMIT: Duration = Duration::from_secs(20);
const SETTLE_TIME: Duration = Duration::from_millis(400);
const THREE_PLAYER_RULES: SessionRules = SessionRules {
    mode: STORY_MODE,
    max_players: 3,
    map: 0,
};
const MAIN_BUILD: BuildGate = BuildGate {
    time_date_stamp: MAIN_PE_IDENTITY.time_date_stamp,
    size_of_image: MAIN_PE_IDENTITY.size_of_image,
};

static THE_LINE_QUEUE: RwLock<()> = RwLock::new(());

struct HostAlone {
    launcher: LauncherCore,
    _game: NetThread<AutomataNet>,
    ticket: String,
    network: LoopbackNetwork,
}

fn sharing_the_queue() -> RwLockReadGuard<'static, ()> {
    THE_LINE_QUEUE
        .read()
        .unwrap_or_else(PoisonError::into_inner)
}

fn owning_the_queue() -> RwLockWriteGuard<'static, ()> {
    THE_LINE_QUEUE
        .write()
        .unwrap_or_else(PoisonError::into_inner)
}

fn requeue_the_line_naming(name: &str) {
    let mut taken = Vec::new();
    assert!(
        wait_until(|| {
            lines_naming(name, &mut taken);
            !taken.is_empty()
        }),
        "no line naming {name}"
    );
    for line in taken {
        push_announced_line(line);
    }
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

fn launcher_on(network: &LoopbackNetwork, name: &str) -> LauncherCore {
    let network = network.clone();
    let factory: TransportFactory =
        Arc::new(move || Ok(Box::new(network.endpoint()) as Box<dyn Transport>));
    LauncherCore::for_game(&AUTOMATA, name, factory).expect("launcher starts")
}

fn game_linked_to(launcher: &LauncherCore) -> NetThread<AutomataNet> {
    NetThread::spawn(
        AutomataNet::new(Character::TwoB, ExeVariant::Main),
        LocalLinkTransport::game(MAIN_BUILD),
        LinkPeer::launcher_on_port(launcher.link_port()),
    )
    .expect("the net worker starts")
}

fn host_game_linked_alone(host_name: &str) -> HostAlone {
    let network = LoopbackNetwork::new();
    let launcher = launcher_on(&network, host_name);
    launcher.set_rules(THREE_PLAYER_RULES);
    let ticket = launcher.host().expect("hosts");
    let game = game_linked_to(&launcher);
    assert!(wait_until(|| launcher.link_status().game_linked));
    assert!(wait_until(|| game.view().extras.session_open));
    HostAlone {
        launcher,
        _game: game,
        ticket,
        network,
    }
}

fn lines_naming(name: &str, collected: &mut Vec<String>) {
    collected.extend(take_announced_lines_where(|line| line.contains(name)));
}

fn wait_for_lines_naming(name: &str, collected: &mut Vec<String>, count: usize) {
    assert!(
        wait_until(|| {
            lines_naming(name, collected);
            collected.len() >= count
        }),
        "lines for {name}: {collected:?}"
    );
    thread::sleep(SETTLE_TIME);
    lines_naming(name, collected);
}

#[test]
fn a_member_joining_the_lobby_is_posted_once_to_the_game_log_with_no_game_of_theirs() {
    let _queue = sharing_the_queue();
    let rig = host_game_linked_alone("Hostess");
    let guest = launcher_on(&rig.network, "Lanternwick");
    guest.join(&rig.ticket).expect("guest dials");
    let mut lines = Vec::new();
    wait_for_lines_naming("Lanternwick", &mut lines, 1);
    assert_eq!(lines, vec!["Lanternwick joined"]);
}

#[test]
fn two_members_joining_are_each_posted_once() {
    let _queue = sharing_the_queue();
    let rig = host_game_linked_alone("Hostess");
    let first = launcher_on(&rig.network, "Quillfeather");
    let second = launcher_on(&rig.network, "Brambleton");
    first.join(&rig.ticket).expect("first dials");
    let mut first_lines = Vec::new();
    wait_for_lines_naming("Quillfeather", &mut first_lines, 1);
    second.join(&rig.ticket).expect("second dials");
    let mut second_lines = Vec::new();
    wait_for_lines_naming("Brambleton", &mut second_lines, 1);
    assert_eq!(first_lines, vec!["Quillfeather joined"]);
    assert_eq!(second_lines, vec!["Brambleton joined"]);
}

#[test]
fn a_member_leaving_is_posted_once() {
    let _queue = sharing_the_queue();
    let rig = host_game_linked_alone("Hostess");
    let guest = launcher_on(&rig.network, "Thistledown");
    guest.join(&rig.ticket).expect("guest dials");
    let mut lines = Vec::new();
    wait_for_lines_naming("Thistledown", &mut lines, 1);
    guest.leave();
    wait_for_lines_naming("Thistledown", &mut lines, 2);
    assert_eq!(lines, vec!["Thistledown joined", "Thistledown left"]);
}

#[test]
fn the_joiners_game_arriving_later_adds_no_second_join_line() {
    let _queue = sharing_the_queue();
    let rig = host_game_linked_alone("Hostess");
    let guest = launcher_on(&rig.network, "Marrowby");
    guest.join(&rig.ticket).expect("guest dials");
    let mut lines = Vec::new();
    wait_for_lines_naming("Marrowby", &mut lines, 1);
    let guest_game = game_linked_to(&guest);
    assert!(wait_until(|| guest_game.view().others().next().is_some()));
    wait_for_lines_naming("Marrowby", &mut lines, 1);
    assert_eq!(lines, vec!["Marrowby joined"]);
    let mut guest_side = Vec::new();
    wait_for_lines_naming("Hostess", &mut guest_side, 1);
    assert_eq!(guest_side, vec!["Hostess joined"]);
}

#[test]
fn the_host_game_alone_posts_nothing_about_itself() {
    let _queue = sharing_the_queue();
    let rig = host_game_linked_alone("Solitaire");
    thread::sleep(SETTLE_TIME);
    assert!(take_announced_lines_where(|line| line.contains("Solitaire")).is_empty());
    assert!(rig.launcher.link_status().game_linked);
}

fn postable_naming(local_in_play: bool, name: &str) -> Vec<String> {
    take_postable_announced_lines_where(local_in_play, true, |line| line.contains(name))
}

#[test]
fn a_join_while_the_local_game_is_not_in_play_waits_and_is_posted_once_when_it_is() {
    let _queue = sharing_the_queue();
    let rig = host_game_linked_alone("Hostess");
    let guest = launcher_on(&rig.network, "Gossamer");
    guest.join(&rig.ticket).expect("guest dials");
    requeue_the_line_naming("Gossamer");
    assert!(postable_naming(false, "Gossamer").is_empty());
    assert!(postable_naming(false, "Gossamer").is_empty());
    assert_eq!(postable_naming(true, "Gossamer"), vec!["Gossamer joined"]);
    assert!(postable_naming(true, "Gossamer").is_empty());
}

#[test]
fn two_joins_and_a_leave_before_play_are_posted_in_order_and_a_session_end_loses_none() {
    let _queue = sharing_the_queue();
    let rig = host_game_linked_alone("Hostess");
    let first = launcher_on(&rig.network, "Ottermere");
    let second = launcher_on(&rig.network, "Pennywhistle");
    first.join(&rig.ticket).expect("first dials");
    thread::sleep(SETTLE_TIME);
    second.join(&rig.ticket).expect("second dials");
    thread::sleep(SETTLE_TIME);
    first.leave();
    thread::sleep(SETTLE_TIME);
    rig.launcher.leave();
    thread::sleep(SETTLE_TIME);
    assert!(postable_naming(false, "Ottermere").is_empty());
    let posted = take_postable_announced_lines_where(true, true, |line| {
        line.contains("Ottermere") || line.contains("Pennywhistle")
    });
    assert_eq!(
        posted[..3],
        ["Ottermere joined", "Pennywhistle joined", "Ottermere left"]
    );
}

#[test]
fn the_queue_keeps_the_newest_thirty_two_lines_until_play() {
    let _queue = owning_the_queue();
    for number in 0..40 {
        push_announced_line(format!("Cap{number:02} joined"));
    }
    let posted = postable_naming(true, "Cap");
    assert_eq!(posted.len(), 32);
    assert_eq!(posted.first().map(String::as_str), Some("Cap08 joined"));
    assert_eq!(posted.last().map(String::as_str), Some("Cap39 joined"));
}

#[test]
fn a_join_during_an_event_while_in_play_waits_until_the_event_ends() {
    let _queue = sharing_the_queue();
    let rig = host_game_linked_alone("Hostess");
    let guest = launcher_on(&rig.network, "Bellwether");
    guest.join(&rig.ticket).expect("guest dials");
    requeue_the_line_naming("Bellwether");
    let while_in_an_event =
        || take_postable_announced_lines_where(true, false, |line| line.contains("Bellwether"));
    assert!(while_in_an_event().is_empty());
    assert!(while_in_an_event().is_empty());
    assert_eq!(
        postable_naming(true, "Bellwether"),
        vec!["Bellwether joined"]
    );
    assert!(postable_naming(true, "Bellwether").is_empty());
}
