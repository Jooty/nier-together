use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::thread;
use std::time::{Duration, Instant};

use nt_launcher_core::{LauncherConfig, LauncherCore, LauncherNet, TransportFactory};
use nt_net::{LocalLinkTransport, LoopbackNetwork, Transport};
use replicant_hook::event_log::take_visible_events;
use replicant_hook::net::{NetState, NetThread};

const WAIT_LIMIT: Duration = Duration::from_secs(20);
pub static ONE_JOIN_TEST_AT_A_TIME: Mutex<()> = Mutex::new(());
const SETTLE_TIME: Duration = Duration::from_millis(400);

struct HostAlone {
    _turn: MutexGuard<'static, ()>,
    launcher: LauncherCore,
    _game: NetThread,
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

fn launcher_on(network: &LoopbackNetwork, name: &str) -> (LauncherCore, LauncherConfig) {
    let config = LauncherConfig::new(name);
    let network = network.clone();
    let factory: TransportFactory =
        Arc::new(move || Ok(Box::new(network.endpoint()) as Box<dyn Transport>));
    let launcher = LauncherCore::new(config.clone(), factory).expect("launcher starts");
    (launcher, config)
}

fn game_linked_to(launcher: &LauncherCore, config: &LauncherConfig) -> NetThread {
    NetThread::spawn(LocalLinkTransport::game(config.build), launcher.link_port())
        .expect("the net worker starts")
}

fn host_game_linked_alone(host_name: &str) -> HostAlone {
    let turn = ONE_JOIN_TEST_AT_A_TIME
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    let network = LoopbackNetwork::new();
    let (launcher, config) = launcher_on(&network, host_name);
    let ticket = launcher.host().expect("hosts");
    let game = game_linked_to(&launcher, &config);
    assert!(wait_until(|| launcher.link_status().game_linked));
    assert!(wait_until(|| game.view().state == NetState::LauncherLinked));
    HostAlone {
        _turn: turn,
        launcher,
        _game: game,
        ticket,
        network,
    }
}

fn lines_naming(name: &str) -> Vec<String> {
    take_visible_events()
        .into_iter()
        .map(|event| event.text)
        .filter(|text| text.contains(name))
        .collect()
}

fn keep_new_lines(seen: &mut Vec<String>) {
    let snapshot: Vec<String> = take_visible_events()
        .into_iter()
        .map(|event| event.text)
        .collect();
    for text in &snapshot {
        let shown = snapshot.iter().filter(|line| *line == text).count();
        while seen.iter().filter(|line| *line == text).count() < shown {
            seen.push(text.clone());
        }
    }
}

fn kept_lines_naming(seen: &[String], name: &str) -> Vec<String> {
    seen.iter()
        .filter(|text| text.contains(name))
        .cloned()
        .collect()
}

fn wait_for_lines_naming(seen: &mut Vec<String>, name: &str, count: usize) -> Vec<String> {
    let found = wait_until(|| {
        keep_new_lines(seen);
        kept_lines_naming(seen, name).len() >= count
    });
    assert!(found, "lines for {name}: {seen:?}");
    thread::sleep(SETTLE_TIME);
    keep_new_lines(seen);
    kept_lines_naming(seen, name)
}

#[test]
fn a_member_joining_the_lobby_after_a_solo_start_is_in_the_event_log_once() {
    let rig = host_game_linked_alone("Hostess");
    assert!(lines_naming("Lanternwick").is_empty());
    let (guest, _) = launcher_on(&rig.network, "Lanternwick");
    guest.join(&rig.ticket).expect("guest dials");
    let mut seen = Vec::new();
    assert_eq!(
        wait_for_lines_naming(&mut seen, "Lanternwick", 1),
        vec!["Lanternwick joined"]
    );
}

#[test]
fn a_line_stays_in_the_log_after_it_is_posted_until_the_game_draws_it() {
    let rig = host_game_linked_alone("Hostess");
    let (guest, _) = launcher_on(&rig.network, "Quillfeather");
    guest.join(&rig.ticket).expect("guest dials");
    assert!(wait_until(|| !lines_naming("Quillfeather").is_empty()));
    thread::sleep(Duration::from_millis(100));
    assert_eq!(lines_naming("Quillfeather"), vec!["Quillfeather joined"]);
}

#[test]
fn a_member_who_joins_after_another_has_left_is_in_the_event_log_once() {
    let rig = host_game_linked_alone("Hostess");
    let (first, _) = launcher_on(&rig.network, "Marrowby");
    first.join(&rig.ticket).expect("first dials");
    let mut seen = Vec::new();
    wait_for_lines_naming(&mut seen, "Marrowby", 1);
    first.leave();
    wait_for_lines_naming(&mut seen, "Marrowby", 2);
    let (second, _) = launcher_on(&rig.network, "Brambleton");
    second.join(&rig.ticket).expect("second dials");
    assert_eq!(
        wait_for_lines_naming(&mut seen, "Brambleton", 1),
        vec!["Brambleton joined"]
    );
    assert_eq!(
        kept_lines_naming(&seen, "Marrowby"),
        vec!["Marrowby joined", "Marrowby left"]
    );
    let expected = ["Marrowby joined".to_string(), "Marrowby left".to_string()];
    assert!(
        expected.ends_with(&lines_naming("Marrowby")),
        "live lines are not the end of the expected ones"
    );
}

#[test]
fn a_member_leaving_is_in_the_event_log_once() {
    let rig = host_game_linked_alone("Hostess");
    let (guest, _) = launcher_on(&rig.network, "Thistledown");
    guest.join(&rig.ticket).expect("guest dials");
    let mut seen = Vec::new();
    wait_for_lines_naming(&mut seen, "Thistledown", 1);
    guest.leave();
    assert_eq!(
        wait_for_lines_naming(&mut seen, "Thistledown", 2),
        vec!["Thistledown joined", "Thistledown left"]
    );
    assert!(rig.launcher.link_status().game_linked);
}
