use replicant_game::callback_gate::{
    END_RUNS_ANYWAY_AFTER_WAITING_MS, EndArrival, EndDrop, EndFollow, LastPauseOn,
    OwnStartPauseFacts, OwnStartPauseRelease, end_follow, own_start_of_end,
    own_start_pause_release,
};

const DREAM_END: &str = "EID_1500_a0005_010_01_End";
const DREAM_LAST_END: &str = "EID_1500_a0005_010_01_End02";
const DREAM_START: &str = "EID_1500_a0005_010_01_Start";
const DREAM_PAUSE: u32 = 0x480;
const HOST_END_ARRIVED_MS: u64 = 22_344;

fn dream_pause() -> LastPauseOn {
    LastPauseOn {
        step: DREAM_START.to_owned(),
        mask_after: DREAM_PAUSE,
    }
}

fn arrival(control_mask_settled: bool) -> EndArrival {
    EndArrival {
        message_phase: 1,
        shared_phase: Some(1),
        local_phase: Some(1),
        local_in_own_talk: false,
        control_mask_settled,
        already_ran_here: false,
        own_copy_of_same_event_running: false,
    }
}

struct GuestTick {
    at_ms: u64,
    pause_slot: u32,
    script_slot_idle: bool,
    talk_window_open: bool,
}

fn release_at(
    end_name: &str,
    tick: &GuestTick,
    pause: &LastPauseOn,
) -> Option<OwnStartPauseRelease> {
    own_start_pause_release(&OwnStartPauseFacts {
        end_name,
        waited_ms: tick.at_ms - HOST_END_ARRIVED_MS,
        last_pause_on: Some(pause),
        slots: [tick.pause_slot, 0, 0, 0],
        menu_open: false,
        script_slot_idle: tick.script_slot_idle,
        talk_window_open: tick.talk_window_open,
    })
}

fn verdict_at(end_name: &str, tick: &GuestTick, pause: &LastPauseOn) -> EndFollow {
    end_follow(&arrival(release_at(end_name, tick, pause).is_some()))
}

#[test]
fn run_1116a_the_hosts_dream_end_runs_on_the_guest_held_only_by_its_own_start_pause() {
    // Run 1116a guest, all at 08:30 (ms after 08:30:00): 22.344 the host's
    // EID_1500_a0005_010_01_End came and waited for "your controls are not free yet"; the pause
    // 0x480 was the guest's own: 08:29:46.877 `PauseOn by EID_1500_a0005_010_01_Start (0x480 ->
    // 0x480)`, no PauseOff after it. 22.754 slot Busy (its own Start in `_Wait`); 23.067 its own
    // 1422 window opened, 23.083 closed; 23.756 to 08:31:22.359 slot Idle, pause 0x480, no
    // window; at 08:31:22.359 it was "still waiting after 60s".
    let pause = dream_pause();
    let ticks = [
        GuestTick {
            at_ms: 22_344,
            pause_slot: DREAM_PAUSE,
            script_slot_idle: false,
            talk_window_open: false,
        },
        GuestTick {
            at_ms: 22_754,
            pause_slot: DREAM_PAUSE,
            script_slot_idle: false,
            talk_window_open: false,
        },
        GuestTick {
            at_ms: 23_067,
            pause_slot: DREAM_PAUSE,
            script_slot_idle: false,
            talk_window_open: true,
        },
        GuestTick {
            at_ms: 23_756,
            pause_slot: DREAM_PAUSE,
            script_slot_idle: true,
            talk_window_open: false,
        },
        GuestTick {
            at_ms: 24_756,
            pause_slot: DREAM_PAUSE,
            script_slot_idle: true,
            talk_window_open: false,
        },
        GuestTick {
            at_ms: 82_359,
            pause_slot: DREAM_PAUSE,
            script_slot_idle: true,
            talk_window_open: false,
        },
    ];
    for tick in &ticks {
        let settled_by_the_mask_alone = tick.pause_slot == 0;
        assert_eq!(
            end_follow(&arrival(settled_by_the_mask_alone)),
            EndFollow::Drop(EndDrop::ControlMaskBusy),
            "before H2c the end waited for a free mask at {} ms",
            tick.at_ms
        );
    }
    for tick in &ticks[..4] {
        assert_eq!(
            release_at(DREAM_END, tick, &pause),
            None,
            "too soon, or the slot is busy, or a window is open at {} ms",
            tick.at_ms
        );
        assert_eq!(
            verdict_at(DREAM_END, tick, &pause),
            EndFollow::Drop(EndDrop::ControlMaskBusy)
        );
    }
    assert_eq!(
        release_at(DREAM_END, &ticks[4], &pause),
        Some(OwnStartPauseRelease::OnlyItsOwnStartsPauseHolds)
    );
    assert_eq!(verdict_at(DREAM_END, &ticks[4], &pause), EndFollow::Mirror);
}

#[test]
fn run_1116a_the_mirrored_talk_after_the_dream_keeps_the_end_waiting() {
    // Guest 08:30:38.158: the host's 1329 is mirrored here (mask 0x480/0xa0, talk 1); the talk
    // bits are the watcher hold's and are taken out, but a talk window is open.
    let tick = GuestTick {
        at_ms: 38_158,
        pause_slot: DREAM_PAUSE,
        script_slot_idle: true,
        talk_window_open: true,
    };
    assert_eq!(release_at(DREAM_END, &tick, &dream_pause()), None);
}

#[test]
fn a_pause_some_other_step_set_is_not_the_starts_own_and_waits_for_the_bounded_release() {
    let door_pause = LastPauseOn {
        step: "From_2F_to_1F_03".to_owned(),
        mask_after: 0x481,
    };
    let early = GuestTick {
        at_ms: HOST_END_ARRIVED_MS + 5_000,
        pause_slot: 0x481,
        script_slot_idle: true,
        talk_window_open: false,
    };
    assert_eq!(release_at(DREAM_END, &early, &door_pause), None);
    let late = GuestTick {
        at_ms: HOST_END_ARRIVED_MS + END_RUNS_ANYWAY_AFTER_WAITING_MS,
        ..early
    };
    assert_eq!(
        release_at(DREAM_END, &late, &door_pause),
        Some(OwnStartPauseRelease::RunsAnywayAfterWaiting {
            waited_ms: END_RUNS_ANYWAY_AFTER_WAITING_MS
        })
    );
}

#[test]
fn a_bit_the_start_did_not_leave_keeps_the_end_waiting() {
    let tick = GuestTick {
        at_ms: HOST_END_ARRIVED_MS + 5_000,
        pause_slot: DREAM_PAUSE | 0x1,
        script_slot_idle: true,
        talk_window_open: false,
    };
    assert_eq!(release_at(DREAM_END, &tick, &dream_pause()), None);
}

#[test]
fn the_dreams_last_end_gets_only_the_bounded_release() {
    // Its own run comes from the guest's own seek after its End; running it early would run it
    // twice, so it waits for that run, and runs anyway only after the bound.
    let early = GuestTick {
        at_ms: HOST_END_ARRIVED_MS + 5_000,
        pause_slot: DREAM_PAUSE,
        script_slot_idle: true,
        talk_window_open: false,
    };
    assert_eq!(release_at(DREAM_LAST_END, &early, &dream_pause()), None);
    let late = GuestTick {
        at_ms: HOST_END_ARRIVED_MS + END_RUNS_ANYWAY_AFTER_WAITING_MS,
        ..early
    };
    assert!(matches!(
        release_at(DREAM_LAST_END, &late, &dream_pause()),
        Some(OwnStartPauseRelease::RunsAnywayAfterWaiting { .. })
    ));
    let busy = GuestTick {
        script_slot_idle: false,
        ..late
    };
    assert_eq!(release_at(DREAM_LAST_END, &busy, &dream_pause()), None);
}

#[test]
fn an_end_that_is_not_listed_always_waits_for_a_free_mask() {
    let tick = GuestTick {
        at_ms: HOST_END_ARRIVED_MS + 10 * END_RUNS_ANYWAY_AFTER_WAITING_MS,
        pause_slot: DREAM_PAUSE,
        script_slot_idle: true,
        talk_window_open: false,
    };
    let pause = LastPauseOn {
        step: "EID_0100_a0038g_Start".to_owned(),
        mask_after: DREAM_PAUSE,
    };
    assert_eq!(release_at("EID_0100_a0038g_End", &tick, &pause), None);
}

#[test]
fn an_end_and_a_numbered_end_name_the_start_of_their_event() {
    assert_eq!(own_start_of_end(DREAM_END).as_deref(), Some(DREAM_START));
    assert_eq!(
        own_start_of_end(DREAM_LAST_END).as_deref(),
        Some(DREAM_START)
    );
    assert_eq!(own_start_of_end("EID_1500_a0005_010_01_Start"), None);
}

#[test]
fn the_release_line_says_which_pause_held_the_guest() {
    let text = OwnStartPauseRelease::OnlyItsOwnStartsPauseHolds.describe(DREAM_END);
    assert!(text.contains(DREAM_START));
    let text =
        OwnStartPauseRelease::RunsAnywayAfterWaiting { waited_ms: 21_000 }.describe(DREAM_END);
    assert!(text.contains("21s") && text.contains("bounded release"));
}
