use replicant_game::move_log::{
    BODY_SPEED_RATE_LEAST, BODY_SPEED_RATE_MOST, FeetDrive, FeetDriveReading, FeetDriveStep,
    FeetDriveTally, GAMES_OWN_BODY_SPEED_RATE, FALLBACK_INDOOR_PACE_PER_SECOND,
    HOUSE_PACE_TICKS_BEFORE_THE_FEET_DRIVE, HostPaceWatch, PaceFeedback, RUN_LOOP_MOTION_NAME,
    STATE_ACTION_MOVE_RVA, house_pace_in_play, starting_body_speed_rate,
};

const DODGE_STATE_RVA: u32 = 0xC3_0028;
const ATTACK_STATE_RVA: u32 = 0xC2_DE10;
const HOUSE_SPEED_CAP: f32 = 0.01;
const TICKS_IN_TEN_SECONDS: usize = 600;

fn the_walk_start_at_08_27_34_247() -> FeetDriveReading<'static> {
    // run-1116a guest run2, 08:27:34.232 "move: hold released +0s ... state=StateActionMove
    // speed=0.06 step_per_s=0.05 ... rates=10:2=20.00" (5:2 back at the house value 0.01 the next
    // tick) and 08:27:34.247 "indoor feet: her body plays KAINE015_idle_lp ... step 0.03/s"
    FeetDriveReading {
        kaine_swapped: true,
        state_rva: Some(STATE_ACTION_MOVE_RVA),
        speed_cap: HOUSE_SPEED_CAP,
        event_scene: false,
        talk: false,
        control_masked: false,
        step_per_second: Some(0.03),
        ground_speed: 0.06,
        body_motion: Some("KAINE015_idle_lp"),
        host_pace_per_second: None,
    }
}

fn the_run_loop_under_her_at_08_27_36_009() -> FeetDriveReading<'static> {
    // 08:27:36.009 "indoor feet: her body plays KAINE015_run_lp (index 226) at frame 7.1 of 39,
    // motion speed 0.50, step 9.03/s" under the old lift; without it the game holds 5:2 at 0.01
    // and her ground speed is 0.06, as the first body's is indoors (host run2 08:29:21.731)
    FeetDriveReading {
        step_per_second: Some(9.03),
        body_motion: Some(RUN_LOOP_MOTION_NAME),
        ..the_walk_start_at_08_27_34_247()
    }
}

fn settled(drive: &mut FeetDrive, reading: &FeetDriveReading<'_>) -> FeetDriveStep {
    let mut step = FeetDriveStep::LEAVE;
    for _ in 0..HOUSE_PACE_TICKS_BEFORE_THE_FEET_DRIVE {
        step = drive.tick(reading);
    }
    step
}

fn pace_after(travel_per_second_at_full_rate: f32, ticks: usize) -> (f32, f32) {
    let mut pace = PaceFeedback::default();
    let mut step = travel_per_second_at_full_rate * pace.rate();
    for _ in 0..ticks {
        let rate = pace.observe(step);
        step = travel_per_second_at_full_rate * rate;
    }
    (pace.rate(), step)
}

#[test]
fn a_walk_start_under_a_held_stick_gets_her_run_loop_at_once() {
    // the old drive waited for a step of 1.0/s, which root travel alone never gives on idle_lp:
    // "the drive did nothing" at 08:27:34.247 while the lift slid her on idle
    let mut drive = FeetDrive::default();
    assert_eq!(
        settled(&mut drive, &the_walk_start_at_08_27_34_247()),
        FeetDriveStep {
            play_run_loop: true,
            body_speed_rate: starting_body_speed_rate(),
        }
    );
}

#[test]
fn the_logged_nine_per_second_walk_is_brought_down_to_the_first_bodys_pace() {
    // the old drive held 0.50 against a step of 9.03/s and the lift kept pushing 3.0/s on top
    let mut drive = FeetDrive::default();
    let reading = the_run_loop_under_her_at_08_27_36_009();
    let first = settled(&mut drive, &reading);
    assert!(!first.play_run_loop);
    assert!((first.body_speed_rate - starting_body_speed_rate()).abs() < 1e-6);
    for _ in 0..TICKS_IN_TEN_SECONDS {
        drive.tick(&reading);
    }
    assert!(drive.pace().rate() < first.body_speed_rate);
    assert!(drive.pace().rate() >= BODY_SPEED_RATE_LEAST);
}

#[test]
fn the_starting_rate_puts_her_run_loop_at_the_first_bodys_indoor_pace() {
    // her run loop at the game's full rate is her 6.0/s outdoor run (K6: ground 6.0 x 5:2 1.0)
    assert!((starting_body_speed_rate() * 6.0 - FALLBACK_INDOOR_PACE_PER_SECOND).abs() < 0.01);
}

#[test]
fn the_pace_settles_at_the_first_bodys_whatever_her_run_loop_travels_at_full_rate() {
    // 6.0/s: her outdoor run; 9.03/s: the fastest step logged with the loop under her indoors
    for travel in [4.5_f32, 6.0, 7.5, 9.03] {
        let (rate, step) = pace_after(travel, TICKS_IN_TEN_SECONDS);
        assert!(
            (step - FALLBACK_INDOOR_PACE_PER_SECOND).abs() < 0.15,
            "travel {travel}: rate {rate}, pace {step}"
        );
    }
}

#[test]
fn the_rate_never_goes_above_the_games_own_so_she_never_gains_speed() {
    let (rate, _) = pace_after(2.0, TICKS_IN_TEN_SECONDS);
    assert!((rate - BODY_SPEED_RATE_MOST).abs() < 1e-6);
    const { assert!(BODY_SPEED_RATE_MOST <= GAMES_OWN_BODY_SPEED_RATE) };
}

#[test]
fn a_body_pressed_against_a_wall_leaves_the_rate_where_it_was() {
    // 08:27:46.003 and 08:27:49.008: run_lp under her, step 0.00/s at the landing of the stairs
    let mut drive = FeetDrive::default();
    let blocked = FeetDriveReading {
        step_per_second: Some(0.0),
        ..the_run_loop_under_her_at_08_27_36_009()
    };
    settled(&mut drive, &blocked);
    for _ in 0..TICKS_IN_TEN_SECONDS {
        drive.tick(&blocked);
    }
    assert!((drive.pace().rate() - starting_body_speed_rate()).abs() < 1e-6);
}

#[test]
fn standing_still_keeps_the_games_own_motion_and_rate() {
    // 08:27:39.010: idle_lp, step 0.01/s at ground speed 0.00
    let mut drive = FeetDrive::default();
    let standing = FeetDriveReading {
        step_per_second: Some(0.01),
        ground_speed: 0.0,
        ..the_walk_start_at_08_27_34_247()
    };
    assert_eq!(settled(&mut drive, &standing), FeetDriveStep::LEAVE);
}

#[test]
fn the_stop_clip_after_her_run_gets_the_games_own_rate_back() {
    // 08:27:35.003: walk_ed_1, step 0.95/s at ground speed 0.00 with the stick let go
    let mut drive = FeetDrive::default();
    settled(&mut drive, &the_run_loop_under_her_at_08_27_36_009());
    let stop = FeetDriveReading {
        step_per_second: Some(0.95),
        ground_speed: 0.0,
        body_motion: Some("KAINE015_walk_ed_1"),
        ..the_walk_start_at_08_27_34_247()
    };
    assert_eq!(drive.tick(&stop), FeetDriveStep::LEAVE);
}

#[test]
fn a_dodge_a_dash_or_an_attack_indoors_keeps_the_games_own_rate() {
    // 08:27:28.113 to 08:27:31.001: sws_dodge_f and sws_dashFromF_st_f in their own states
    let mut drive = FeetDrive::default();
    settled(&mut drive, &the_run_loop_under_her_at_08_27_36_009());
    for state in [DODGE_STATE_RVA, ATTACK_STATE_RVA] {
        let reading = FeetDriveReading {
            state_rva: Some(state),
            step_per_second: Some(10.97),
            body_motion: Some("KAINE015_sws_dashFromF_st_f"),
            ..the_walk_start_at_08_27_34_247()
        };
        assert_eq!(drive.tick(&reading), FeetDriveStep::LEAVE);
    }
}

#[test]
fn a_few_house_value_ticks_outdoors_do_not_start_the_drive() {
    // run0 08:36:53.384 to 08:36:57.550, outdoors in phase 2: 5:2 touched the house value for one
    // tick at a time between readings of 1.00, and the old drive played her run loop there
    let mut drive = FeetDrive::default();
    let dip = FeetDriveReading {
        ground_speed: 4.7,
        step_per_second: Some(4.7),
        body_motion: Some("KAINE015_walk_lp"),
        ..the_walk_start_at_08_27_34_247()
    };
    let outdoors = FeetDriveReading {
        speed_cap: 1.0,
        ..dip
    };
    for _ in 0..TICKS_IN_TEN_SECONDS {
        for _ in 1..HOUSE_PACE_TICKS_BEFORE_THE_FEET_DRIVE {
            assert_eq!(drive.tick(&dip), FeetDriveStep::LEAVE);
        }
        assert_eq!(drive.tick(&outdoors), FeetDriveStep::LEAVE);
    }
}

#[test]
fn the_drive_runs_only_for_her_swapped_body_free_to_move_on_the_house_value() {
    let walk = the_walk_start_at_08_27_34_247();
    assert!(house_pace_in_play(&walk));
    for held in [
        FeetDriveReading {
            speed_cap: 1.0,
            ..walk
        },
        FeetDriveReading {
            speed_cap: 0.0,
            ..walk
        },
        FeetDriveReading {
            kaine_swapped: false,
            ..walk
        },
        FeetDriveReading {
            state_rva: None,
            ..walk
        },
        FeetDriveReading {
            event_scene: true,
            ..walk
        },
        FeetDriveReading { talk: true, ..walk },
        FeetDriveReading {
            control_masked: true,
            ..walk
        },
    ] {
        assert!(!house_pace_in_play(&held));
        let mut drive = FeetDrive::default();
        assert_eq!(settled(&mut drive, &held), FeetDriveStep::LEAVE);
    }
}

#[test]
fn the_tally_counts_plays_and_rate_writes_and_empties_when_taken() {
    let mut tally = FeetDriveTally::default();
    tally.count_play();
    tally.count_rate_write();
    tally.count_rate_write();
    assert_eq!(
        tally.take(),
        FeetDriveTally {
            plays: 1,
            rate_writes: 2
        }
    );
    assert_eq!(tally, FeetDriveTally::default());
}

const MANOR_HOST_PACE: f32 = 4.07;

fn the_manor_walk() -> FeetDriveReading<'static> {
    // L33 run1, A_SOUTH_MANSION_02: the host walks the manor at 4.07/s (15 samples at ground
    // speed 0.1 or less)
    FeetDriveReading {
        host_pace_per_second: Some(MANOR_HOST_PACE),
        ..the_run_loop_under_her_at_08_27_36_009()
    }
}

#[test]
fn in_the_manor_her_pace_settles_at_the_hosts_four_per_second_not_the_homes_two_nine() {
    let mut drive = FeetDrive::default();
    let reading = the_manor_walk();
    let mut step = settled(&mut drive, &reading);
    for _ in 0..TICKS_IN_TEN_SECONDS {
        step = drive.tick(&FeetDriveReading {
            step_per_second: Some(6.0 * step.body_speed_rate),
            ..reading
        });
    }
    let pace = 6.0 * step.body_speed_rate;
    assert!((pace - MANOR_HOST_PACE).abs() < 0.15, "pace {pace}");
}

#[test]
fn the_host_pace_the_story_points_show_is_followed_and_never_lifted_past_the_cap() {
    for host in [2.7_f32, 3.73, 3.8, 4.07, 4.45] {
        let mut drive = FeetDrive::default();
        let reading = FeetDriveReading {
            host_pace_per_second: Some(host),
            ..the_run_loop_under_her_at_08_27_36_009()
        };
        let mut step = settled(&mut drive, &reading);
        for _ in 0..TICKS_IN_TEN_SECONDS {
            step = drive.tick(&FeetDriveReading {
                step_per_second: Some(6.0 * step.body_speed_rate),
                ..reading
            });
        }
        assert!(step.body_speed_rate <= BODY_SPEED_RATE_MOST);
        assert!((6.0 * step.body_speed_rate - host).abs() < 0.15, "host {host}");
    }
}

#[test]
fn with_no_host_pace_measured_she_aims_at_the_fallback() {
    let mut drive = FeetDrive::default();
    let step = settled(&mut drive, &the_run_loop_under_her_at_08_27_36_009());
    assert!((step.body_speed_rate * 6.0 - FALLBACK_INDOOR_PACE_PER_SECOND).abs() < 0.01);
}

#[test]
fn the_host_pace_is_measured_from_the_position_stream_while_he_walks() {
    let mut watch = HostPaceWatch::default();
    for tick in 0..=40 {
        watch.observe([4.07 * tick as f32 * 0.033, 0.0, 0.0], f64::from(tick) * 0.033);
    }
    assert!((watch.pace_per_second().unwrap() - 4.07).abs() < 0.05);
    for tick in 41..=80 {
        watch.observe([5.4, 0.0, 0.0], f64::from(tick) * 0.033);
    }
    assert!((watch.pace_per_second().unwrap() - 4.07).abs() < 0.6);
    for tick in 81..=120 {
        let along = 5.4 + 6.0 * (tick - 80) as f32 * 0.033;
        watch.observe([along, 0.0, 0.0], f64::from(tick) * 0.033);
    }
    assert!((watch.pace_per_second().unwrap() - 4.07).abs() < 0.6);
}

#[test]
fn only_the_rate_the_drive_wrote_is_put_back_and_never_another_writers() {
    let mut drive = FeetDrive::default();
    let walking = settled(&mut drive, &the_run_loop_under_her_at_08_27_36_009());
    let written = drive.rate_to_write(Some(1.0), walking).unwrap();
    assert!((written - walking.body_speed_rate).abs() < 1e-6);
    assert!(drive.holds_a_rate_of_ours(Some(written)));
    assert_eq!(drive.rate_to_write(Some(written), FeetDriveStep::LEAVE), Some(1.0));
    assert!(!drive.holds_a_rate_of_ours(Some(1.0)));

    // the game's own 0.75 at the stairs (0:2/1:2/2:2) or 0.60 (0:4) in this entry is not ours
    let mut drive = FeetDrive::default();
    for foreign in [0.75_f32, 0.60] {
        assert!(!drive.holds_a_rate_of_ours(Some(foreign)));
        assert_eq!(drive.rate_to_write(Some(foreign), FeetDriveStep::LEAVE), None);
    }
    drive.rate_to_write(Some(1.0), walking);
    assert_eq!(drive.rate_to_write(Some(0.6), FeetDriveStep::LEAVE), None);
    assert!(!drive.holds_a_rate_of_ours(Some(written)));
}

#[test]
fn a_talk_a_scene_a_mask_or_a_state_change_does_not_restart_the_ten_tick_wait() {
    let walk = the_run_loop_under_her_at_08_27_36_009();
    for interruption in [
        FeetDriveReading { talk: true, ..walk },
        FeetDriveReading { event_scene: true, ..walk },
        FeetDriveReading { control_masked: true, ..walk },
        FeetDriveReading { state_rva: Some(DODGE_STATE_RVA), ..walk },
    ] {
        let mut drive = FeetDrive::default();
        settled(&mut drive, &walk);
        assert_eq!(drive.tick(&interruption), FeetDriveStep::LEAVE);
        assert!(drive.tick(&walk).body_speed_rate < GAMES_OWN_BODY_SPEED_RATE);
    }
    let mut drive = FeetDrive::default();
    settled(&mut drive, &walk);
    drive.tick(&FeetDriveReading { speed_cap: 1.0, ..walk });
    assert_eq!(drive.tick(&walk), FeetDriveStep::LEAVE);
}
