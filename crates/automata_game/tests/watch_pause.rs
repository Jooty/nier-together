use automata_game::event_follow::SkipPrompt;
use automata_game::watch_pause::{
    CLOSE_PRESSES_ARE_SPACED_MS, FIRST_CLOSE_PRESS_AFTER_THE_HOSTS_PAUSE_ENDS_MS,
    HOST_STATE_COUNTS_AS_FRESH_FOR_MS, MOST_CLOSE_PRESSES,
    OWN_MENU_IS_CHECKED_AFTER_THE_HOSTS_PAUSE_ENDS_MS, OWN_MENU_SHOULD_HAVE_OPENED_WITHIN_MS,
    WatchPause, WatchPauseReadings, WatchPauseStep,
};

fn watching(now_ms: u64, host_paused: bool) -> WatchPauseReadings {
    WatchPauseReadings {
        now_ms,
        watches: true,
        host_paused: Some((host_paused, 0)),
        a_cutscene_plays_here: false,
        own_menu_open: false,
    }
}

fn has_line(step: &WatchPauseStep, text: &str) -> bool {
    step.logs.iter().any(|line| line.contains(text))
}

fn held_machine() -> WatchPause {
    let mut machine = WatchPause::default();
    machine.step(&watching(0, false));
    machine.step(&watching(100, true));
    machine
}

#[test]
fn the_hosts_pause_holds_the_watching_guest_and_shows_the_note_until_the_host_resumes() {
    let mut machine = WatchPause::default();
    let idle = machine.step(&watching(0, false));
    assert!(!idle.hold && !idle.note && idle.logs.is_empty());
    let applied = machine.step(&watching(100, true));
    assert!(applied.hold && applied.note);
    assert!(has_line(
        &applied,
        "the host paused its game, this copy's game is paused with it"
    ));
    let long_after = machine.step(&WatchPauseReadings {
        own_menu_open: true,
        ..watching(10_000_000, true)
    });
    assert!(long_after.hold && long_after.note);
    let resumed = machine.step(&WatchPauseReadings {
        own_menu_open: true,
        ..watching(10_000_100, false)
    });
    assert!(!resumed.hold && !resumed.note);
    assert!(has_line(
        &resumed,
        "the host's pause ends here after 10000000 ms: the host resumed"
    ));
}

#[test]
fn the_own_menu_opening_is_logged_once_and_a_menu_that_does_not_open_is_logged_once() {
    let mut machine = held_machine();
    let opened = machine.step(&WatchPauseReadings {
        own_menu_open: true,
        ..watching(150, true)
    });
    assert!(has_line(
        &opened,
        "this copy's pause menu opened 50 ms after"
    ));
    let again = machine.step(&WatchPauseReadings {
        own_menu_open: true,
        ..watching(200, true)
    });
    assert!(again.logs.is_empty());

    let mut never_opens = held_machine();
    let early = never_opens.step(&watching(
        100 + OWN_MENU_SHOULD_HAVE_OPENED_WITHIN_MS - 1,
        true,
    ));
    assert!(early.logs.is_empty());
    let missed = never_opens.step(&watching(100 + OWN_MENU_SHOULD_HAVE_OPENED_WITHIN_MS, true));
    assert!(has_line(&missed, "this copy's pause menu has not opened"));
    let later = never_opens.step(&watching(5_000, true));
    assert!(later.logs.is_empty() && later.hold);
}

#[test]
fn after_the_hosts_resume_the_own_menu_is_reported_closed_or_still_open() {
    let mut machine = held_machine();
    machine.step(&watching(200, false));
    let closed = machine.step(&watching(250, false));
    assert!(has_line(&closed, "pause menu closed by itself"));

    let mut stays = held_machine();
    stays.step(&WatchPauseReadings {
        own_menu_open: true,
        ..watching(200, false)
    });
    let soon = stays.step(&WatchPauseReadings {
        own_menu_open: true,
        ..watching(
            200 + OWN_MENU_IS_CHECKED_AFTER_THE_HOSTS_PAUSE_ENDS_MS - 1,
            false,
        )
    });
    assert!(soon.logs.is_empty());
    let still = stays.step(&WatchPauseReadings {
        own_menu_open: true,
        ..watching(
            200 + OWN_MENU_IS_CHECKED_AFTER_THE_HOSTS_PAUSE_ENDS_MS,
            false,
        )
    });
    assert!(has_line(
        &still,
        "pause menu is still open 500 ms after the host's pause ended; the pause key here closes it"
    ));
    assert!(
        stays
            .step(&WatchPauseReadings {
                own_menu_open: true,
                ..watching(5_000, false)
            })
            .logs
            .is_empty()
    );
}

#[test]
fn each_release_names_its_reason_and_a_stale_host_state_releases() {
    let mut ended = held_machine();
    let step = ended.step(&WatchPauseReadings {
        watches: false,
        ..watching(200, true)
    });
    assert!(!step.hold && has_line(&step, "watching ended"));

    let mut cutscene = held_machine();
    let step = cutscene.step(&WatchPauseReadings {
        a_cutscene_plays_here: true,
        ..watching(200, true)
    });
    assert!(!step.hold && has_line(&step, "a cutscene plays here, its pause rule takes over"));

    let mut stale = held_machine();
    let step = stale.step(&WatchPauseReadings {
        host_paused: Some((true, HOST_STATE_COUNTS_AS_FRESH_FOR_MS + 1)),
        ..watching(200, true)
    });
    assert!(!step.hold && has_line(&step, "no fresh state from the host"));

    let mut gone = held_machine();
    let step = gone.step(&WatchPauseReadings {
        host_paused: None,
        ..watching(200, true)
    });
    assert!(!step.hold && has_line(&step, "no fresh state from the host"));
}

#[test]
fn a_pause_outside_watching_or_during_a_cutscene_is_never_applied() {
    let mut machine = WatchPause::default();
    let not_watching = machine.step(&WatchPauseReadings {
        watches: false,
        ..watching(0, true)
    });
    assert!(!not_watching.hold && not_watching.logs.is_empty());
    let in_a_cutscene = machine.step(&WatchPauseReadings {
        a_cutscene_plays_here: true,
        ..watching(100, true)
    });
    assert!(!in_a_cutscene.hold && in_a_cutscene.logs.is_empty());
}

#[test]
fn the_hosts_pause_can_be_applied_again_while_the_own_menu_is_still_being_checked() {
    let mut machine = held_machine();
    machine.step(&WatchPauseReadings {
        own_menu_open: true,
        ..watching(200, false)
    });
    let again = machine.step(&WatchPauseReadings {
        own_menu_open: true,
        ..watching(300, true)
    });
    assert!(again.hold && has_line(&again, "this copy's game is paused with it"));
}

#[test]
fn the_note_names_the_host() {
    assert_eq!(
        SkipPrompt::PartnerPausedTheGame.text("guy"),
        "guy paused the game"
    );
}

fn menu_open(now_ms: u64, host_paused: bool) -> WatchPauseReadings {
    WatchPauseReadings {
        own_menu_open: true,
        ..watching(now_ms, host_paused)
    }
}

#[test]
fn a_menu_the_hold_opened_gets_back_presses_after_the_hosts_resume_until_it_closes() {
    let mut machine = WatchPause::default();
    machine.step(&watching(0, false));
    machine.step(&watching(375, true));
    machine.step(&menu_open(390, true));
    machine.step(&menu_open(1_389, false));
    let early = machine.step(&menu_open(
        1_389 + FIRST_CLOSE_PRESS_AFTER_THE_HOSTS_PAUSE_ENDS_MS - 1,
        false,
    ));
    assert!(!early.press_back);
    let first = machine.step(&menu_open(
        1_389 + FIRST_CLOSE_PRESS_AFTER_THE_HOSTS_PAUSE_ENDS_MS,
        false,
    ));
    assert!(first.press_back && has_line(&first, "back press 1 is sent"));
    let waiting = machine.step(&menu_open(1_389 + 516, false));
    assert!(!waiting.press_back && waiting.logs.is_empty());
    let closed = machine.step(&watching(1_389 + 700, false));
    assert!(
        !closed.press_back
            && has_line(
                &closed,
                "closed 700 ms after the host's pause ended, after 1 back presses"
            )
    );
    assert!(machine.step(&watching(5_000, false)).logs.is_empty());
}

#[test]
fn a_menu_that_stays_open_gets_spaced_presses_and_is_then_left_to_the_player() {
    let mut machine = WatchPause::default();
    machine.step(&watching(0, false));
    machine.step(&watching(100, true));
    machine.step(&menu_open(115, true));
    machine.step(&menu_open(1_100, false));
    let mut pressed_at = Vec::new();
    for now_ms in (1_100..30_000).step_by(100) {
        let step = machine.step(&menu_open(now_ms, false));
        if step.press_back {
            pressed_at.push(now_ms);
        }
        if has_line(&step, "left to the player") {
            break;
        }
    }
    assert_eq!(pressed_at.len(), MOST_CLOSE_PRESSES as usize);
    assert!(
        pressed_at
            .windows(2)
            .all(|pair| pair[1] - pair[0] >= CLOSE_PRESSES_ARE_SPACED_MS)
    );
}

#[test]
fn a_menu_the_guest_opened_itself_is_never_pressed_closed() {
    let mut machine = WatchPause::default();
    machine.step(&menu_open(0, false));
    machine.step(&menu_open(100, true));
    machine.step(&menu_open(1_100, false));
    for now_ms in (1_100..10_000).step_by(100) {
        assert!(!machine.step(&menu_open(now_ms, false)).press_back);
    }
}
