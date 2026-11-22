use automata_game::address::{
    APPLY_STICK_TO_ANGLES_FUNCTION, CONTROLLER_INPUT_BLOCKED_FUNCTION, GET_CAMERA_STICK_FUNCTION,
};
use automata_game::exe::ExeVariant;
use automata_game::raw_mouse::{
    CAP_PROBE_MIN_GAP_BETWEEN_LINES_MS, CAP_PROBE_WINDOW_REPORT_GAP_MS, CameraAngles, CapProbe,
    EaseArmSource, GAME_PITCH_LIMIT_RADIANS, LOOK_HOLD_AFTER_LAST_COUNTS_MS, LookHold, ModeLimit,
    RawLook, RawMouseAccumulator, RawMouseHook, SWING_PROBE_MIN_GAP_BETWEEN_LINES_MS,
    SWING_PROBE_TAIL_MS, SWING_PROBE_YAW_STEP_RADIANS, ScreenRect, SwingProbe, apply_raw_look,
    camera_field_writes, clip_cursor_is_set, counts_of_device_move, ease_arm_writes,
    eye_to_look_distance, pitch_inside_ease_limit, prologue_matches, radians_per_count,
    raw_camera_plan, sensitivity_multiplier, x_sensitivity_value, yaw_step_radians,
};

const DEFAULT_MULTIPLIER: f32 = 0.82;
const DEFAULT_RADIANS_PER_COUNT: f32 = 0.07 * core::f32::consts::PI / 180.0;

const NO_LIMITS_LOOK: RawLook = RawLook {
    yaw_multiplier: DEFAULT_MULTIPLIER,
    pitch_multiplier: DEFAULT_MULTIPLIER,
    invert_pitch: false,
    invert_yaw: false,
    pitch_enabled: true,
    yaw_enabled: true,
    lower_limit: ModeLimit::OFF,
    upper_limit: ModeLimit::OFF,
};

const LEVEL: CameraAngles = CameraAngles {
    pitch: 0.0,
    yaw: 0.0,
};

#[test]
fn counts_between_two_frames_are_summed() {
    let mut accumulator = RawMouseAccumulator::default();
    accumulator.add(3, -2);
    accumulator.add(4, 5);
    accumulator.begin_frame();
    assert_eq!(accumulator.take_frame_counts(), (7, 3));
}

#[test]
fn a_frame_reads_only_what_arrived_before_it_began() {
    let mut accumulator = RawMouseAccumulator::default();
    accumulator.add(10, 10);
    accumulator.begin_frame();
    accumulator.add(1, 1);
    assert_eq!(accumulator.take_frame_counts(), (10, 10));
    accumulator.begin_frame();
    assert_eq!(accumulator.take_frame_counts(), (1, 1));
}

#[test]
fn counts_are_gone_after_they_are_taken_so_a_second_camera_step_gets_none() {
    let mut accumulator = RawMouseAccumulator::default();
    accumulator.add(5, 5);
    accumulator.begin_frame();
    assert_eq!(accumulator.take_frame_counts(), (5, 5));
    assert_eq!(accumulator.take_frame_counts(), (0, 0));
}

#[test]
fn a_frame_nobody_took_is_dropped_by_the_next_one() {
    let mut accumulator = RawMouseAccumulator::default();
    accumulator.add(9, 9);
    accumulator.begin_frame();
    accumulator.begin_frame();
    assert_eq!(accumulator.take_frame_counts(), (0, 0));
}

#[test]
fn discarding_empties_both_the_waiting_and_the_frame_counts() {
    let mut accumulator = RawMouseAccumulator::default();
    accumulator.add(2, 2);
    accumulator.begin_frame();
    accumulator.add(3, 3);
    accumulator.discard_all();
    assert_eq!(accumulator, RawMouseAccumulator::default());
}

#[test]
fn a_huge_sum_saturates_instead_of_wrapping() {
    let mut accumulator = RawMouseAccumulator::default();
    accumulator.add(i32::MAX, 0);
    accumulator.add(5, 0);
    accumulator.begin_frame();
    assert_eq!(accumulator.take_frame_counts().0, i32::MAX);
}

#[test]
fn with_the_setting_off_the_games_own_mouse_path_is_left_alone() {
    let plan = raw_camera_plan(false, false, (40, 40));
    assert!(!plan.replace_game_mouse);
    assert_eq!(plan.counts, (0, 0));
}

#[test]
fn with_the_setting_on_the_game_mouse_is_replaced_and_counts_pass() {
    let plan = raw_camera_plan(true, false, (7, -3));
    assert!(plan.replace_game_mouse);
    assert_eq!(plan.counts, (7, -3));
}

#[test]
fn an_open_menu_gives_the_camera_no_movement() {
    let plan = raw_camera_plan(true, true, (7, -3));
    assert_eq!(plan.counts, (0, 0));
}

#[test]
fn moving_right_turns_the_yaw_down_and_moving_down_turns_the_pitch_down() {
    let moved = apply_raw_look(LEVEL, (10, 10), &NO_LIMITS_LOOK).expect("moved");
    assert!((moved.yaw + 10.0 * DEFAULT_RADIANS_PER_COUNT).abs() < 1e-6);
    assert!((moved.pitch + 10.0 * DEFAULT_RADIANS_PER_COUNT).abs() < 1e-6);
}

#[test]
fn the_turn_is_one_to_one_so_twice_the_counts_is_twice_the_angle() {
    let once = apply_raw_look(LEVEL, (5, 0), &NO_LIMITS_LOOK).expect("moved");
    let twice = apply_raw_look(LEVEL, (10, 0), &NO_LIMITS_LOOK).expect("moved");
    assert!((twice.yaw - 2.0 * once.yaw).abs() < 1e-6);
}

#[test]
fn a_fast_flick_is_not_capped() {
    let moved = apply_raw_look(LEVEL, (-200, 0), &NO_LIMITS_LOOK).expect("moved");
    assert!((moved.yaw - 200.0 * DEFAULT_RADIANS_PER_COUNT).abs() < 1e-5);
}

#[test]
fn inverted_axes_turn_the_other_way() {
    let look = RawLook {
        invert_pitch: true,
        invert_yaw: true,
        ..NO_LIMITS_LOOK
    };
    let moved = apply_raw_look(LEVEL, (10, 10), &look).expect("moved");
    assert!(moved.yaw > 0.0);
    assert!(moved.pitch > 0.0);
}

#[test]
fn no_counts_means_nothing_is_written() {
    assert_eq!(apply_raw_look(LEVEL, (0, 0), &NO_LIMITS_LOOK), None);
}

#[test]
fn a_disabled_axis_does_not_move() {
    let look = RawLook {
        pitch_enabled: false,
        ..NO_LIMITS_LOOK
    };
    let moved = apply_raw_look(LEVEL, (10, 10), &look).expect("moved");
    assert_eq!(moved.pitch, 0.0);
    assert_eq!(apply_raw_look(LEVEL, (0, 10), &look), None);
}

#[test]
fn yaw_wraps_around_the_back_of_the_camera() {
    let near_the_edge = CameraAngles {
        pitch: 0.0,
        yaw: -core::f32::consts::PI + 0.001,
    };
    let moved = apply_raw_look(near_the_edge, (10, 0), &NO_LIMITS_LOOK).expect("moved");
    assert!(moved.yaw > 3.0);
}

#[test]
fn pitch_stops_at_the_games_own_limit() {
    let moved = apply_raw_look(LEVEL, (0, -100_000), &NO_LIMITS_LOOK).expect("moved");
    assert_eq!(moved.pitch, GAME_PITCH_LIMIT_RADIANS);
    let moved = apply_raw_look(LEVEL, (0, 100_000), &NO_LIMITS_LOOK).expect("moved");
    assert_eq!(moved.pitch, -GAME_PITCH_LIMIT_RADIANS);
}

#[test]
fn an_enabled_mode_limit_stops_the_pitch_sooner() {
    let look = RawLook {
        upper_limit: ModeLimit {
            value: 0.2,
            enable: 1.0,
        },
        lower_limit: ModeLimit {
            value: -0.1,
            enable: 1.0,
        },
        ..NO_LIMITS_LOOK
    };
    assert_eq!(
        apply_raw_look(LEVEL, (0, -1000), &look)
            .expect("moved")
            .pitch,
        0.2
    );
    assert_eq!(
        apply_raw_look(LEVEL, (0, 1000), &look)
            .expect("moved")
            .pitch,
        -0.1
    );
}

#[test]
fn a_mode_limit_that_is_not_enabled_is_ignored() {
    let look = RawLook {
        upper_limit: ModeLimit {
            value: 0.01,
            enable: 0.0,
        },
        ..NO_LIMITS_LOOK
    };
    let moved = apply_raw_look(LEVEL, (0, -100), &look).expect("moved");
    assert!(moved.pitch > 0.01);
}

#[test]
fn the_three_hooks_have_an_address_in_both_exes() {
    assert_eq!(
        APPLY_STICK_TO_ANGLES_FUNCTION.rva(ExeVariant::Main),
        0x72_9960
    );
    assert_eq!(
        APPLY_STICK_TO_ANGLES_FUNCTION.rva(ExeVariant::Compat),
        0x72_1B90
    );
    assert_eq!(GET_CAMERA_STICK_FUNCTION.rva(ExeVariant::Compat), 0x4F_77A0);
    assert_eq!(
        CONTROLLER_INPUT_BLOCKED_FUNCTION.rva(ExeVariant::Compat),
        0x49_85A0
    );
    for hook in RawMouseHook::ALL {
        assert_ne!(hook.address().main, hook.address().compat);
        assert!(!hook.label().is_empty());
    }
}

#[test]
fn a_prologue_check_accepts_the_noted_bytes_and_refuses_others() {
    for hook in RawMouseHook::ALL {
        let mut bytes = hook.expected_prologue().to_vec();
        bytes.extend_from_slice(&[0x90; 4]);
        assert!(prologue_matches(hook, &bytes));
        bytes[0] ^= 0xFF;
        assert!(!prologue_matches(hook, &bytes));
    }
}

#[test]
fn the_sensitivity_curve_is_linear_up_to_ten_and_steeper_after() {
    assert!((sensitivity_multiplier(0) - 0.1).abs() < 1e-6);
    assert!((sensitivity_multiplier(8) - 0.82).abs() < 1e-6);
    assert!((sensitivity_multiplier(10) - 1.0).abs() < 1e-6);
    assert!((sensitivity_multiplier(11) - 1.2).abs() < 1e-6);
    assert!((sensitivity_multiplier(20) - 3.0).abs() < 1e-6);
}

#[test]
fn the_x_floor_only_lifts_low_values_and_only_with_its_flag() {
    assert_eq!(x_sensitivity_value(2, 0x4000), 4);
    assert_eq!(x_sensitivity_value(2, 0), 2);
    assert_eq!(x_sensitivity_value(8, 0x4000), 8);
    assert_eq!(x_sensitivity_value(12, 0x4000), 12);
    assert_eq!(x_sensitivity_value(2, 0x8000), 2);
}

#[test]
fn the_default_x_sensitivity_turns_seven_hundredths_of_a_degree_per_count() {
    let radians = radians_per_count(sensitivity_multiplier(8));
    assert!((radians.to_degrees() - 0.07).abs() < 1e-5);
}

#[test]
fn a_higher_sensitivity_turns_proportionally_more_per_count() {
    let slow = radians_per_count(sensitivity_multiplier(5));
    let fast = radians_per_count(sensitivity_multiplier(15));
    assert!((fast / slow - sensitivity_multiplier(15) / sensitivity_multiplier(5)).abs() < 1e-4);
}

#[test]
fn each_axis_uses_its_own_multiplier() {
    let look = RawLook {
        yaw_multiplier: 1.0,
        pitch_multiplier: 2.0,
        ..NO_LIMITS_LOOK
    };
    let moved = apply_raw_look(LEVEL, (10, 10), &look).expect("moved");
    assert!((moved.pitch / moved.yaw - 2.0).abs() < 1e-4);
}

#[test]
fn device_floats_become_whole_counts() {
    assert_eq!(counts_of_device_move(3.0, -2.0), (3, -2));
    assert_eq!(counts_of_device_move(0.4, 0.6), (0, 1));
    assert_eq!(counts_of_device_move(f32::NAN, 0.0), (0, 0));
}

#[test]
fn the_look_hold_starts_with_counts_and_lasts_until_250_ms_after_the_last_ones() {
    let mut hold = LookHold::new();
    assert!(!hold.step(0, false));
    assert!(hold.step(1000, true));
    assert!(hold.step(1000 + LOOK_HOLD_AFTER_LAST_COUNTS_MS, false));
    assert!(!hold.step(1001 + LOOK_HOLD_AFTER_LAST_COUNTS_MS, false));
}

#[test]
fn new_counts_push_the_look_hold_end_out() {
    let mut hold = LookHold::new();
    assert!(hold.step(0, true));
    assert!(hold.step(200, true));
    assert!(hold.step(400, false));
    assert!(!hold.step(451, false));
}

#[test]
fn a_cleared_look_hold_is_not_held() {
    let mut hold = LookHold::new();
    assert!(hold.step(10, true));
    hold.clear();
    assert!(!hold.step(11, false));
}

#[test]
fn the_probe_stays_quiet_for_small_steps() {
    let mut probe = SwingProbe::new();
    assert!(!probe.step(0, SWING_PROBE_YAW_STEP_RADIANS));
    assert!(!probe.step(16, 0.0));
}

#[test]
fn a_big_yaw_step_starts_the_probe_in_either_direction() {
    let mut probe = SwingProbe::new();
    assert!(probe.step(0, SWING_PROBE_YAW_STEP_RADIANS + 0.01));
    let mut probe = SwingProbe::new();
    assert!(probe.step(0, -SWING_PROBE_YAW_STEP_RADIANS - 0.01));
}

#[test]
fn the_probe_logs_at_most_one_line_a_second() {
    let mut probe = SwingProbe::new();
    probe.step(0, 1.0);
    let lines = (1..2000_u64).filter(|ms| probe.step(*ms, 0.0)).count();
    assert_eq!(SWING_PROBE_MIN_GAP_BETWEEN_LINES_MS, 1000);
    assert_eq!(lines, 1);
}

#[test]
fn the_probe_goes_quiet_two_seconds_after_the_last_big_step() {
    let mut probe = SwingProbe::new();
    probe.step(0, 1.0);
    assert!(probe.step(SWING_PROBE_TAIL_MS, 0.0));
    assert!(!probe.step(SWING_PROBE_TAIL_MS + 1000, 0.0));
}

#[test]
fn a_new_big_step_restarts_the_probe_tail() {
    let mut probe = SwingProbe::new();
    probe.step(0, 1.0);
    probe.step(1500, 1.0);
    assert!(probe.step(3000, 0.0));
}

#[test]
fn a_yaw_step_across_the_wrap_is_the_short_way_round() {
    let step = yaw_step_radians(3.1, -3.1);
    assert!((step - (core::f32::consts::TAU - 6.2)).abs() < 1e-4);
    assert!((yaw_step_radians(0.5, 0.25) + 0.25).abs() < 1e-6);
}

#[test]
fn arming_the_ease_copies_the_saved_angles_and_rates_and_opens_both_windows() {
    let writes = ease_arm_writes(EaseArmSource {
        pitch_to_ease_from: 11,
        yaw_to_ease_from: 22,
        pitch_rate: 33,
        yaw_rate: 44,
    });
    let value_at = |offset: u64| writes.iter().find(|w| w.offset == offset).map(|w| w.value);
    assert_eq!(value_at(0x8810), Some(11));
    assert_eq!(value_at(0x8814), Some(22));
    assert_eq!(value_at(0x87E8), Some(1));
    assert_eq!(value_at(0x87EC), Some(1));
    assert_eq!(value_at(0x881C), Some(33));
    assert_eq!(value_at(0x8818), Some(33));
    assert_eq!(value_at(0x8824), Some(44));
    assert_eq!(value_at(0x8820), Some(44));
}

#[test]
fn the_ease_pitch_clamp_needs_a_running_direction_timer_and_a_real_limit() {
    assert_eq!(pitch_inside_ease_limit(1.0, 0.5, 0.3), Some(0.5));
    assert_eq!(pitch_inside_ease_limit(-1.0, 0.5, 0.3), Some(-0.5));
    assert_eq!(pitch_inside_ease_limit(0.2, 0.5, 0.3), None);
    assert_eq!(pitch_inside_ease_limit(1.0, 0.5, 0.0), None);
    assert_eq!(pitch_inside_ease_limit(1.0, 0.0, 0.3), None);
    assert_eq!(pitch_inside_ease_limit(1.0, f32::NAN, 0.3), None);
}

#[test]
fn with_the_setting_off_no_camera_field_is_written() {
    assert!(!camera_field_writes(false).suppress_auto_follow);
}

#[test]
fn with_the_setting_on_auto_follow_is_suppressed_on_every_frame() {
    assert!(camera_field_writes(true).suppress_auto_follow);
}

#[test]
fn the_eye_to_look_distance_is_the_straight_line_length() {
    assert!((eye_to_look_distance([1.0, 2.0, 3.0], [4.0, 6.0, 3.0]) - 5.0).abs() < 1e-6);
}

#[test]
fn the_cap_probe_keeps_the_largest_count_of_each_axis_with_its_sign() {
    let mut probe = CapProbe::new();
    assert!(probe.step(0, (3, -2), 0.1).is_some());
    probe.step(10, (-9, 4), -0.3);
    probe.step(20, (5, -7), 0.2);
    let line = probe
        .step(CAP_PROBE_MIN_GAP_BETWEEN_LINES_MS, (1, 1), 0.0)
        .unwrap();
    assert_eq!((line.largest_lx, line.largest_ly), (-9, -7));
    assert!((line.largest_yaw_step + 0.3).abs() < 1e-6);
    assert_eq!(line.frames, 3);
}

#[test]
fn the_cap_probe_window_starts_over_after_each_line() {
    let mut probe = CapProbe::new();
    probe.step(0, (50, 50), 1.0);
    let line = probe
        .step(CAP_PROBE_MIN_GAP_BETWEEN_LINES_MS, (2, 3), 0.01)
        .unwrap();
    assert_eq!((line.largest_lx, line.largest_ly, line.frames), (2, 3, 1));
}

#[test]
fn the_cap_probe_ignores_frames_without_counts() {
    let mut probe = CapProbe::new();
    assert!(probe.step(0, (0, 0), 0.0).is_none());
    assert!(probe.step(500, (0, 0), 0.0).is_none());
}

#[test]
fn the_cap_probe_logs_at_most_one_line_a_second() {
    let mut probe = CapProbe::new();
    let lines = (0..3000_u64)
        .filter(|ms| probe.step(*ms, (1, 1), 0.01).is_some())
        .count();
    assert_eq!(CAP_PROBE_MIN_GAP_BETWEEN_LINES_MS, 1000);
    assert_eq!(lines, 3);
}

#[test]
fn the_cap_probe_adds_the_window_report_once_a_second() {
    let mut probe = CapProbe::new();
    let reports = (0..3000_u64)
        .filter_map(|ms| probe.step(ms, (1, 1), 0.01))
        .filter(|line| line.with_window_report)
        .count();
    assert_eq!(CAP_PROBE_WINDOW_REPORT_GAP_MS, 1000);
    assert_eq!(reports, 3);
}

#[test]
fn a_clip_rectangle_is_set_unless_it_is_the_whole_screen() {
    let screen = ScreenRect {
        left: 0,
        top: 0,
        right: 1920,
        bottom: 1080,
    };
    let client = ScreenRect {
        left: 100,
        top: 100,
        right: 900,
        bottom: 700,
    };
    assert!(!clip_cursor_is_set(screen, screen));
    assert!(clip_cursor_is_set(client, screen));
}
