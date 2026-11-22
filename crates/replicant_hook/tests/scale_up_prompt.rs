use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, MutexGuard, PoisonError};
use std::thread;
use std::time::{Duration, Instant};

use nt_launcher_core::{
    LauncherConfig, LauncherCore, LauncherEvent, LauncherNet, TransportFactory,
};
use nt_net::{LocalLinkTransport, LoopbackNetwork, Transport};
use replicant_game::scale_up_prompt::{
    Gesture, HOLD_TO_ANSWER_MS, IGNORE_PRESSES_AFTER_SHOWN_MS, PressSample,
};
use replicant_hook::event_log::take_visible_events;
use replicant_hook::net::{NetState, NetThread};
use replicant_proto::{GameReadiness, ScaleUpAsk, scale_up_no_save_yet_line};

const WAIT_LIMIT: Duration = Duration::from_secs(20);
const REPORT_EVERY: Duration = Duration::from_millis(20);
const SETTLE_TIME: Duration = Duration::from_millis(400);
const FRAME_MS: u64 = 16;
const GUEST_SLOT: u8 = 1;
const GUEST_NAME: &str = "Quillfeather";

struct PlayingGame {
    game: Arc<NetThread>,
    safe_for_a_joiner: Arc<AtomicBool>,
    stop: Arc<AtomicBool>,
    worker: Option<thread::JoinHandle<()>>,
}

impl PlayingGame {
    fn start(game: NetThread, safe_for_a_joiner: bool) -> Self {
        let game = Arc::new(game);
        let safe = Arc::new(AtomicBool::new(safe_for_a_joiner));
        let stop = Arc::new(AtomicBool::new(false));
        let worker = {
            let (game, safe, stop) = (game.clone(), safe.clone(), stop.clone());
            thread::spawn(move || {
                while !stop.load(Ordering::Acquire) {
                    game.report_readiness(
                        GameReadiness::InPlay,
                        Default::default(),
                        safe.load(Ordering::Acquire),
                    );
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

    fn set_safe(&self, safe: bool) {
        self.safe_for_a_joiner.store(safe, Ordering::Release);
        assert!(wait_until(
            || self.game.view().safe_to_take_a_joiner() == safe
        ));
    }

    fn close(mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
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
    launcher: LauncherCore,
    config: LauncherConfig,
    host: PlayingGame,
    clock_ms: u64,
    answers: Vec<(u8, bool)>,
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

fn open_ask() -> ScaleUpAsk {
    ScaleUpAsk::Open {
        slot: GUEST_SLOT,
        name: GUEST_NAME.to_owned(),
    }
}

fn withdrawn_ask() -> ScaleUpAsk {
    ScaleUpAsk::Withdrawn { slot: GUEST_SLOT }
}

fn keyboard(gesture: Gesture) -> PressSample {
    PressSample {
        keyboard: gesture,
        ..PressSample::default()
    }
}

fn game_linked_to(launcher: &LauncherCore, config: &LauncherConfig) -> NetThread {
    NetThread::spawn(LocalLinkTransport::game(config.build), launcher.link_port())
        .expect("the net worker starts")
}

fn host_game(safe: bool) -> Rig {
    let turn = crate::join_line::ONE_JOIN_TEST_AT_A_TIME
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    let network = LoopbackNetwork::new();
    let config = LauncherConfig::new("Hostess");
    let factory: TransportFactory = {
        let network = network.clone();
        Arc::new(move || Ok(Box::new(network.endpoint()) as Box<dyn Transport>))
    };
    let launcher = LauncherCore::new(config.clone(), factory).expect("launcher starts");
    launcher.host().expect("hosts");
    let game = game_linked_to(&launcher, &config);
    assert!(wait_until(|| launcher.link_status().game_linked));
    let host = PlayingGame::start(game, safe);
    assert!(wait_until(|| host
        .game
        .view()
        .effective_local_readiness()
        .is_in_play()));
    assert!(wait_until(
        || host.game.view().state == NetState::LauncherLinked
    ));
    Rig {
        _turn: turn,
        launcher,
        config,
        host,
        clock_ms: 100_000,
        answers: Vec::new(),
    }
}

impl Rig {
    fn ask(&self, ask: ScaleUpAsk) {
        self.launcher.send_scale_up_ask(ask);
    }

    fn prompt_is_open(&self) -> bool {
        self.host.game.view().scale_up.is_open()
    }

    fn wait_for_the_prompt(&self) {
        assert!(wait_until(|| self.prompt_is_open()));
    }

    fn frames(&mut self, input: PressSample, millis: u64) {
        let end = self.clock_ms + millis;
        while self.clock_ms < end {
            self.host.game.step_scale_up_prompt(self.clock_ms, input);
            self.clock_ms += FRAME_MS;
        }
    }

    fn show_the_prompt(&mut self) {
        self.frames(
            PressSample::default(),
            IGNORE_PRESSES_AFTER_SHOWN_MS + FRAME_MS,
        );
        assert!(self.host.game.shown_scale_up_ask().is_some());
    }

    fn launcher_answers(&mut self) -> Vec<(u8, bool)> {
        thread::sleep(SETTLE_TIME);
        for event in self.launcher.drain_events() {
            if let LauncherEvent::ScaleUpAnswered { slot, yes } = event {
                self.answers.push((slot, yes));
            }
        }
        self.answers.clone()
    }

    fn relink_the_game(&mut self, safe: bool) {
        let old = std::mem::replace(
            &mut self.host,
            PlayingGame::start(game_linked_to(&self.launcher, &self.config), safe),
        );
        old.close();
        assert!(wait_until(
            || self.host.game.view().state == NetState::LauncherLinked
        ));
        assert!(wait_until(|| self
            .host
            .game
            .view()
            .effective_local_readiness()
            .is_in_play()));
    }
}

#[test]
fn the_launchers_ask_reaches_the_prompt_and_a_yes_goes_back_to_the_launcher() {
    let mut rig = host_game(true);
    rig.ask(open_ask());
    rig.wait_for_the_prompt();
    rig.show_the_prompt();
    let shown = rig.host.game.shown_scale_up_ask().expect("shown");
    assert_eq!((shown.slot, shown.name.as_str()), (GUEST_SLOT, GUEST_NAME));
    rig.frames(keyboard(Gesture::Yes), HOLD_TO_ANSWER_MS * 2);
    assert_eq!(rig.launcher_answers(), vec![(GUEST_SLOT, true)]);
    assert!(!rig.prompt_is_open());
}

#[test]
fn a_no_goes_back_to_the_launcher_as_no() {
    let mut rig = host_game(true);
    rig.ask(open_ask());
    rig.wait_for_the_prompt();
    rig.show_the_prompt();
    let pad_no = PressSample {
        pad: Gesture::No,
        ..PressSample::default()
    };
    rig.frames(pad_no, HOLD_TO_ANSWER_MS * 2);
    assert_eq!(rig.launcher_answers(), vec![(GUEST_SLOT, false)]);
}

#[test]
fn a_click_answers_through_to_the_launcher_once() {
    let mut rig = host_game(true);
    rig.ask(open_ask());
    rig.wait_for_the_prompt();
    rig.show_the_prompt();
    let click = PressSample {
        click: Some(true),
        ..PressSample::default()
    };
    rig.frames(click, FRAME_MS * 10);
    assert_eq!(rig.launcher_answers(), vec![(GUEST_SLOT, true)]);
}

#[test]
fn a_withdrawn_ask_takes_the_prompt_away_and_a_press_then_answers_nothing() {
    let mut rig = host_game(true);
    rig.ask(open_ask());
    rig.wait_for_the_prompt();
    rig.show_the_prompt();
    rig.ask(withdrawn_ask());
    assert!(wait_until(|| !rig.prompt_is_open()));
    rig.frames(keyboard(Gesture::Yes), HOLD_TO_ANSWER_MS * 3);
    assert!(rig.host.game.shown_scale_up_ask().is_none());
    assert!(rig.launcher_answers().is_empty());
}

#[test]
fn a_guest_asked_about_again_after_a_withdrawal_gets_a_new_prompt() {
    let mut rig = host_game(true);
    rig.ask(open_ask());
    rig.wait_for_the_prompt();
    rig.ask(withdrawn_ask());
    assert!(wait_until(|| !rig.prompt_is_open()));
    rig.ask(open_ask());
    rig.wait_for_the_prompt();
    rig.show_the_prompt();
    rig.frames(keyboard(Gesture::No), HOLD_TO_ANSWER_MS * 2);
    assert_eq!(rig.launcher_answers(), vec![(GUEST_SLOT, false)]);
}

#[test]
fn an_unanswered_ask_comes_back_once_after_the_game_relinks_and_is_answered_once() {
    let mut rig = host_game(true);
    rig.ask(open_ask());
    rig.wait_for_the_prompt();
    rig.show_the_prompt();
    rig.relink_the_game(true);
    rig.wait_for_the_prompt();
    rig.show_the_prompt();
    rig.frames(keyboard(Gesture::Yes), HOLD_TO_ANSWER_MS * 2);
    assert_eq!(rig.launcher_answers(), vec![(GUEST_SLOT, true)]);
    rig.frames(keyboard(Gesture::Yes), HOLD_TO_ANSWER_MS * 2);
    assert_eq!(rig.launcher_answers(), vec![(GUEST_SLOT, true)]);
}

#[test]
fn an_answered_ask_is_not_asked_again_after_the_game_relinks() {
    let mut rig = host_game(true);
    rig.ask(open_ask());
    rig.wait_for_the_prompt();
    rig.show_the_prompt();
    rig.frames(keyboard(Gesture::No), HOLD_TO_ANSWER_MS * 2);
    assert_eq!(rig.launcher_answers(), vec![(GUEST_SLOT, false)]);
    rig.relink_the_game(true);
    thread::sleep(SETTLE_TIME);
    assert!(!rig.prompt_is_open());
}

#[test]
fn the_prompt_waits_out_a_load_a_scene_or_a_fight_and_is_still_there_after() {
    let mut rig = host_game(false);
    rig.ask(open_ask());
    rig.wait_for_the_prompt();
    rig.frames(PressSample::default(), IGNORE_PRESSES_AFTER_SHOWN_MS * 2);
    assert!(rig.host.game.shown_scale_up_ask().is_none());
    rig.frames(keyboard(Gesture::Yes), HOLD_TO_ANSWER_MS * 3);
    assert!(rig.launcher_answers().is_empty());
    assert!(rig.prompt_is_open());
    rig.host.set_safe(true);
    rig.show_the_prompt();
    rig.host.set_safe(false);
    rig.frames(PressSample::default(), FRAME_MS * 2);
    assert!(rig.host.game.shown_scale_up_ask().is_none());
    assert!(rig.prompt_is_open());
    rig.host.set_safe(true);
    rig.show_the_prompt();
    rig.frames(keyboard(Gesture::Yes), HOLD_TO_ANSWER_MS * 2);
    assert_eq!(rig.launcher_answers(), vec![(GUEST_SLOT, true)]);
}

#[test]
fn no_save_yet_posts_the_line_in_the_event_log_and_opens_no_prompt() {
    let mut rig = host_game(true);
    take_visible_events();
    rig.ask(ScaleUpAsk::NoSaveYet {
        slot: GUEST_SLOT,
        name: GUEST_NAME.to_owned(),
    });
    let line = scale_up_no_save_yet_line(GUEST_NAME);
    assert!(wait_until(|| {
        take_visible_events().iter().any(|event| event.text == line)
    }));
    rig.frames(keyboard(Gesture::Yes), HOLD_TO_ANSWER_MS * 3);
    assert!(!rig.prompt_is_open());
    assert!(rig.launcher_answers().is_empty());
}
