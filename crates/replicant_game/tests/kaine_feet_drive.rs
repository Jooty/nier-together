use replicant_game::move_log::{
    FeetDriveReading, FeetDriveStep, FeetDriveTally, INDOOR_FEET_LEAST_MOTION_SPEED,
    INDOOR_FEET_MOST_MOTION_SPEED, RUN_LOOP_MOTION_NAME, SPEED_CAP_LIFTED_VALUE,
    STATE_ACTION_MOVE_RVA, feet_drive_step, house_pace_in_play, run_loop_speed_for,
};

const SOME_OTHER_STATE_RVA: u32 = 0xC9_3F28;

fn the_slide_at_01_49_00() -> FeetDriveReading<'static> {
    // run-1115j guest run1, 01:49:00.175: "indoor walk: swapped body, phase 1: 5:2 0.50 (1 lifts
    // this phase), step 3.00/s at ground speed 3.00, state StateActionMove" and "indoor feet: her
    // body plays KAINE015_idle_lp (index 93) at frame 439.8 of 646, motion speed 1.00, step 3.00/s"
    FeetDriveReading {
        kaine_swapped: true,
        state_rva: Some(STATE_ACTION_MOVE_RVA),
        speed_cap: SPEED_CAP_LIFTED_VALUE,
        lifts_this_phase: 1,
        event_scene: false,
        talk: false,
        control_masked: false,
        step_per_second: Some(3.0),
        ground_speed: 3.0,
        body_motion: Some("KAINE015_idle_lp"),
        body_motion_speed: Some(1.0),
    }
}

#[test]
fn the_logged_slide_gets_her_run_loop_at_half_speed() {
    assert_eq!(
        feet_drive_step(&the_slide_at_01_49_00()),
        FeetDriveStep::PlayRunLoop { speed: 0.5 }
    );
}

#[test]
fn the_walk_stop_clip_under_a_moving_body_is_replaced_too() {
    // 01:49:04.186: walk_ed_1 at frame 78.2 of 171, motion speed 1.00, step 2.59/s
    let reading = FeetDriveReading {
        step_per_second: Some(2.59),
        body_motion: Some("KAINE015_walk_ed_1"),
        ..the_slide_at_01_49_00()
    };
    assert_eq!(
        feet_drive_step(&reading),
        FeetDriveStep::PlayRunLoop { speed: 0.5 }
    );
}

#[test]
fn the_run_loop_at_the_wrong_speed_has_its_speed_held() {
    let reading = FeetDriveReading {
        body_motion: Some(RUN_LOOP_MOTION_NAME),
        body_motion_speed: Some(1.0),
        ..the_slide_at_01_49_00()
    };
    assert_eq!(
        feet_drive_step(&reading),
        FeetDriveStep::HoldSpeed { speed: 0.5 }
    );
}

#[test]
fn the_run_loop_at_the_pace_speed_is_left_alone() {
    let reading = FeetDriveReading {
        body_motion: Some("kaine015_RUN_LP"),
        body_motion_speed: Some(0.5),
        ..the_slide_at_01_49_00()
    };
    assert_eq!(feet_drive_step(&reading), FeetDriveStep::Leave);
}

#[test]
fn a_body_that_stands_still_keeps_the_games_own_motion() {
    // 01:48:59.617: step 0.05/s at ground speed 0.06, idle_lp
    let reading = FeetDriveReading {
        step_per_second: Some(0.05),
        ground_speed: 0.06,
        ..the_slide_at_01_49_00()
    };
    assert_eq!(feet_drive_step(&reading), FeetDriveStep::Leave);
    assert_eq!(
        feet_drive_step(&FeetDriveReading {
            step_per_second: None,
            ..the_slide_at_01_49_00()
        }),
        FeetDriveStep::Leave
    );
}

#[test]
fn the_drive_runs_only_on_the_house_pace_in_the_plain_move_state() {
    let slide = the_slide_at_01_49_00();
    let outdoors = FeetDriveReading {
        speed_cap: 1.0,
        ground_speed: 6.0,
        ..slide
    };
    assert!(!house_pace_in_play(&outdoors));
    let before_any_lift = FeetDriveReading {
        lifts_this_phase: 0,
        speed_cap: 0.01,
        ..slide
    };
    assert!(!house_pace_in_play(&before_any_lift));
    let house_value_this_tick = FeetDriveReading {
        speed_cap: 0.01,
        ..slide
    };
    assert!(house_pace_in_play(&house_value_this_tick));
    let another_state = FeetDriveReading {
        state_rva: Some(SOME_OTHER_STATE_RVA),
        ..slide
    };
    assert!(!house_pace_in_play(&another_state));
    let first_body = FeetDriveReading {
        kaine_swapped: false,
        ..slide
    };
    assert!(!house_pace_in_play(&first_body));
    for held in [
        FeetDriveReading {
            event_scene: true,
            ..slide
        },
        FeetDriveReading {
            talk: true,
            ..slide
        },
        FeetDriveReading {
            control_masked: true,
            ..slide
        },
    ] {
        assert_eq!(feet_drive_step(&held), FeetDriveStep::Leave);
    }
}

#[test]
fn the_run_loop_speed_is_the_pace_over_her_full_run_within_bounds() {
    assert!((run_loop_speed_for(3.0) - 0.5).abs() < 1e-6);
    assert!((run_loop_speed_for(6.0) - INDOOR_FEET_MOST_MOTION_SPEED).abs() < 1e-6);
    assert!((run_loop_speed_for(12.0) - INDOOR_FEET_MOST_MOTION_SPEED).abs() < 1e-6);
    assert!((run_loop_speed_for(0.0) - INDOOR_FEET_LEAST_MOTION_SPEED).abs() < 1e-6);
}

#[test]
fn the_tally_counts_plays_and_holds_and_empties_when_taken() {
    let mut tally = FeetDriveTally::default();
    tally.count(FeetDriveStep::PlayRunLoop { speed: 0.5 });
    tally.count(FeetDriveStep::HoldSpeed { speed: 0.5 });
    tally.count(FeetDriveStep::HoldSpeed { speed: 0.5 });
    tally.count(FeetDriveStep::Leave);
    assert_eq!(
        tally.take(),
        FeetDriveTally {
            plays: 1,
            speed_holds: 2
        }
    );
    assert_eq!(tally, FeetDriveTally::default());
}
