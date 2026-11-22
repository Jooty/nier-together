use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, MutexGuard, PoisonError};
use std::thread;
use std::time::{Duration, Instant};

use nt_launcher_core::{
    LauncherConfig, LauncherCore, LauncherNet, SessionOptions, TransportFactory,
};
use nt_net::{LocalLinkTransport, LoopbackNetwork, Transport};
use replicant_hook::net::{LocalSnapshot, NetSharedExt, NetState, NetThread};
use replicant_proto::{FollowPhase, GameReadiness, Message, StoryRequest};

const WAIT_LIMIT: Duration = Duration::from_secs(20);
const REPORT_EVERY: Duration = Duration::from_millis(20);
const SETTLE_TIME: Duration = Duration::from_millis(500);
const NOT_THE_DEFAULT_ENEMY_HEALTH_PERCENT: u16 = 150;

struct PlayingGame {
    game: Arc<NetThread>,
    safe_for_a_joiner: Arc<AtomicBool>,
    stop: Arc<AtomicBool>,
    worker: Option<thread::JoinHandle<()>>,
}

impl PlayingGame {
    fn start(game: NetThread, in_play: bool, safe_for_a_joiner: bool) -> Self {
        let game = Arc::new(game);
        let safe = Arc::new(AtomicBool::new(safe_for_a_joiner));
        let stop = Arc::new(AtomicBool::new(false));
        let worker = {
            let (game, safe, stop) = (game.clone(), safe.clone(), stop.clone());
            thread::spawn(move || {
                while !stop.load(Ordering::Acquire) {
                    if in_play {
                        game.report_readiness(
                            GameReadiness::InPlay,
                            Default::default(),
                            safe.load(Ordering::Acquire),
                        );
                        if game.wants_snapshots() {
                            game.publish(snapshot());
                        }
                    }
                    thread::sleep(REPORT_EVERY);
                }
            })
        };
        Self {
            game,
            safe_for_a_joiner: safe,
            stop,
            worker: Some(worker),
        }
    }

    fn the_game_reaches_a_safe_moment(&self) {
        self.safe_for_a_joiner.store(true, Ordering::Release);
    }
}

impl Drop for PlayingGame {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

struct Rig {
    _turn: MutexGuard<'static, ()>,
    network: LoopbackNetwork,
    host_launcher: LauncherCore,
    host: PlayingGame,
    ticket: String,
}

fn snapshot() -> LocalSnapshot {
    LocalSnapshot {
        phase: 3,
        position: [1.0, 0.0, 0.0],
        yaw_radians: 0.0,
        hp: 10,
        max_hp: 10,
        body_motion: None,
        body_motion_hash: 0,
        weapon: None,
        activity: Default::default(),
        mp: 0.0,
        max_mp: 0.0,
        level: 0,
        ridden_boar: None,
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

fn host_playing(options: SessionOptions, host_in_play: bool, safe: bool) -> Rig {
    let turn = crate::join_line::ONE_JOIN_TEST_AT_A_TIME
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    let network = LoopbackNetwork::new();
    let (host_launcher, config) = launcher_on(&network, "Hostess");
    host_launcher.set_session_options(options);
    let ticket = host_launcher.host().expect("hosts");
    let game = game_linked_to(&host_launcher, &config);
    assert!(wait_until(|| host_launcher.link_status().game_linked));
    let host = PlayingGame::start(game, host_in_play, safe);
    if host_in_play {
        assert!(wait_until(|| host
            .game
            .view()
            .effective_local_readiness()
            .is_in_play()));
    }
    Rig {
        _turn: turn,
        network,
        host_launcher,
        host,
        ticket,
    }
}

fn late_guest(rig: &Rig, name: &str, in_play: bool) -> (LauncherCore, PlayingGame) {
    let (launcher, config) = launcher_on(&rig.network, name);
    launcher.join(&rig.ticket).expect("guest dials");
    let game = game_linked_to(&launcher, &config);
    assert!(wait_until(|| launcher.link_status().game_linked));
    (launcher, PlayingGame::start(game, in_play, false))
}

fn follow_order() -> Message {
    Message::FollowPhase(FollowPhase {
        phase: 3,
        point: 1,
        in_side_view: false,
        side_view_door: 0,
        last_normal_view: None,
    })
}

#[test]
fn a_late_joiner_is_not_in_play_for_a_host_in_a_scene_until_the_scene_ends() {
    let rig = host_playing(SessionOptions::default(), true, false);
    let (_guest_launcher, guest) = late_guest(&rig, "Oakhollow", true);
    assert!(wait_until(
        || rig.host.game.view().state == NetState::Connected
    ));
    assert!(wait_until(|| guest.game.view().partner_in_play()));
    thread::sleep(SETTLE_TIME);
    let view = rig.host.game.view();
    assert!(view.partner_joining());
    assert!(!view.partner_in_play());
    assert!(view.partner_state().is_none());
    rig.host.the_game_reaches_a_safe_moment();
    assert!(wait_until(|| rig.host.game.view().partner_in_play()));
}

#[test]
fn a_late_joiner_does_not_hold_up_the_hosts_zone_changes_before_it_is_in_play() {
    let rig = host_playing(SessionOptions::default(), true, true);
    let (_guest_launcher, _guest) = late_guest(&rig, "Pennyroyal", false);
    assert!(wait_until(
        || rig.host.game.view().state == NetState::Connected
    ));
    thread::sleep(SETTLE_TIME);
    assert!(rig.host.game.view().partner_joining());
    assert!(!rig.host.game.view().partner_game_binds_the_rules());
}

#[test]
fn a_partner_who_joined_before_the_host_played_still_binds_the_hosts_zone_changes() {
    let rig = host_playing(SessionOptions::default(), false, false);
    let (_guest_launcher, _guest) = late_guest(&rig, "Fernsby", false);
    assert!(wait_until(
        || rig.host.game.view().state == NetState::Connected
    ));
    thread::sleep(SETTLE_TIME);
    assert!(rig.host.game.view().partner_joining());
    assert!(rig.host.game.view().partner_game_binds_the_rules());
}

#[test]
fn a_taken_in_late_joiner_reaches_the_hosts_follow_order_and_asks_for_the_story() {
    let rig = host_playing(SessionOptions::default(), true, true);
    let (_guest_launcher, guest) = late_guest(&rig, "Wrenfield", true);
    assert!(wait_until(|| rig.host.game.view().partner_in_play()));
    assert!(wait_until(|| guest.game.view().partner_in_play()));
    assert!(rig.host.game.view().partner_game_binds_the_rules());
    rig.host.game.send_relayed(follow_order());
    assert!(wait_until(|| {
        !guest
            .game
            .take_relayed(|message| *message == follow_order())
            .is_empty()
    }));
    guest.game.send_relayed(Message::StoryRequest(StoryRequest));
    assert!(wait_until(|| {
        !rig.host
            .game
            .take_relayed(|message| matches!(message, Message::StoryRequest(_)))
            .is_empty()
    }));
}

#[test]
fn the_hosts_options_reach_a_game_that_joins_late() {
    let options = SessionOptions {
        enemy_health_percent: NOT_THE_DEFAULT_ENEMY_HEALTH_PERCENT,
        ..SessionOptions::default()
    };
    let rig = host_playing(options, true, true);
    let (_guest_launcher, guest) = late_guest(&rig, "Sedgewick", true);
    assert!(wait_until(|| guest.game.view().options == options));
    let _ = &rig.host_launcher;
}
