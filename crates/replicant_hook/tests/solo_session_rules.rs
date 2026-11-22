use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use nt_launcher_core::{
    LauncherConfig, LauncherCore, LauncherNet, SessionOptions, TransportFactory,
};
use nt_net::{LocalLinkTransport, LoopbackNetwork, Transport};
use nt_wire::STORY_MODE;
use replicant_hook::net::{NetState, NetThread};

const WAIT_LIMIT: Duration = Duration::from_secs(20);
const SETTLE_TIME: Duration = Duration::from_millis(300);
const NOT_THE_DEFAULT_ENEMY_HEALTH_PERCENT: u16 = 150;

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

#[test]
fn a_replicant_host_alone_keeps_the_state_the_game_had_before_this_change() {
    let network = LoopbackNetwork::new();
    let config = LauncherConfig::new("Hosty");
    let launcher = LauncherCore::new(config.clone(), loopback_factory(&network)).expect("starts");
    launcher.set_session_options(SessionOptions {
        enemy_health_percent: NOT_THE_DEFAULT_ENEMY_HEALTH_PERCENT,
        ..SessionOptions::default()
    });
    launcher.host().expect("hosts");
    let game = NetThread::spawn(LocalLinkTransport::game(config.build), launcher.link_port())
        .expect("the net worker starts");
    assert!(wait_until(|| launcher.link_status().game_linked));
    assert!(wait_until(|| game.view().state == NetState::LauncherLinked));
    thread::sleep(SETTLE_TIME);
    let view = game.view();
    assert_eq!(view.state, NetState::LauncherLinked);
    assert_eq!(view.local_slot, None);
    assert_eq!(view.rules.mode, STORY_MODE);
    assert_eq!(view.options, SessionOptions::default());
}
