use automata_game::partner::{
    MOVING_POSITION_ERROR_LEFT_ALONE, POSITION_ERROR_LEFT_ALONE, PREDICTION_AGE_CAP_MS,
    PositionCorrection, PositionSample, SampleCorrector, position_correction_line,
};

const RUN_SPEED: f32 = 8.0;
const FULL_STICK: f32 = 1000.0;

const GUEST_LOG_READINGS: [([f32; 3], [f32; 3]); 6] = [
    (
        [-132.00867, 55.29623, 669.86566],
        [-131.91705, 55.29623, 669.6715],
    ),
    (
        [-122.4636, 58.689358, 655.0961],
        [-122.87349, 58.689358, 655.2758],
    ),
    (
        [-110.75327, 61.308376, 658.3922],
        [-111.013626, 61.308376, 658.30676],
    ),
    (
        [-98.239975, 61.30837, 672.0713],
        [-98.21369, 61.30837, 672.13654],
    ),
    (
        [-93.069954, 61.322773, 676.21826],
        [-93.07667, 61.301365, 675.6694],
    ),
    (
        [-94.18555, 60.565773, 662.03674],
        [-94.172844, 60.564713, 661.99414],
    ),
];

fn sample(sequence: u32, x: f32, input_speed: f32) -> PositionSample {
    PositionSample {
        sequence,
        position: [x, 0.0, 0.0],
        input_speed,
    }
}

#[test]
fn running_readings_from_the_guest_log_are_left_alone() {
    for (index, (body, other_player)) in GUEST_LOG_READINGS.iter().enumerate() {
        let mut corrector = SampleCorrector::default();
        let correction = corrector.correct(
            1_000,
            *body,
            PositionSample {
                sequence: index as u32,
                position: *other_player,
                input_speed: FULL_STICK,
            },
        );
        assert_eq!(
            correction,
            PositionCorrection::Leave,
            "reading {} was corrected",
            index + 2
        );
    }
}

#[test]
fn a_run_at_the_real_send_rate_is_never_pulled_back_and_never_teleported() {
    let mut corrector = SampleCorrector::default();
    let mut body_x = 0.0_f32;
    let mut sequence = 0_u32;
    let mut next_send_ms = 0_u64;
    let mut latest_sent_x = 0.0_f32;
    let mut latest_sequence = 0_u32;
    let mut largest_error_after_warmup = 0.0_f32;
    for tick in 0..300_u64 {
        let now_ms = tick * 16;
        while next_send_ms <= now_ms {
            sequence += 1;
            latest_sequence = sequence;
            latest_sent_x = RUN_SPEED * next_send_ms as f32 / 1000.0;
            next_send_ms += if sequence.is_multiple_of(2) { 33 } else { 40 };
        }
        body_x += 7.8 * 0.016;
        let before = body_x;
        let correction = corrector.correct(
            now_ms,
            [body_x, 0.0, 0.0],
            sample(latest_sequence, latest_sent_x, FULL_STICK),
        );
        match correction {
            PositionCorrection::Nudge(position) => {
                assert!(position[0] >= before, "tick {tick} pulled the body back");
                body_x = position[0];
            }
            PositionCorrection::Teleport => panic!("tick {tick} teleported"),
            PositionCorrection::Leave => {}
        }
        if now_ms > 500 {
            let true_x = RUN_SPEED * now_ms as f32 / 1000.0;
            largest_error_after_warmup = largest_error_after_warmup.max((true_x - body_x).abs());
        }
    }
    assert!(
        largest_error_after_warmup < MOVING_POSITION_ERROR_LEFT_ALONE,
        "the error reached {largest_error_after_warmup}"
    );
}

#[test]
fn a_body_that_runs_ahead_is_not_pulled_back_along_its_motion() {
    let mut corrector = SampleCorrector::default();
    corrector.correct(0, [0.0; 3], sample(1, 0.0, FULL_STICK));
    corrector.correct(33, [0.264, 0.0, 0.0], sample(2, 0.264, FULL_STICK));
    let correction = corrector.correct(49, [1.2, 0.0, 0.0], sample(2, 0.264, FULL_STICK));
    let PositionCorrection::Nudge(position) = correction else {
        panic!("the body 0.9 ahead was not nudged: {correction:?}");
    };
    assert!(position[0] >= 1.2, "{position:?}");
}

#[test]
fn a_small_error_is_left_alone_at_rest_below_the_rest_band_and_nudged_above_it() {
    let mut corrector = SampleCorrector::default();
    let inside = POSITION_ERROR_LEFT_ALONE * 0.5;
    assert_eq!(
        corrector.correct(0, [0.0; 3], sample(1, inside, 0.0)),
        PositionCorrection::Leave
    );
    let mut corrector = SampleCorrector::default();
    assert!(matches!(
        corrector.correct(0, [0.0; 3], sample(1, 0.3, 0.0)),
        PositionCorrection::Nudge(_)
    ));
}

#[test]
fn the_dead_band_is_wider_while_the_body_runs_on_fed_input() {
    let mut corrector = SampleCorrector::default();
    assert_eq!(
        corrector.correct(0, [0.0; 3], sample(1, 0.3, FULL_STICK)),
        PositionCorrection::Leave
    );
    let mut corrector = SampleCorrector::default();
    assert!(matches!(
        corrector.correct(0, [0.0; 3], sample(1, 0.8, FULL_STICK)),
        PositionCorrection::Nudge(_)
    ));
}

#[test]
fn the_target_is_the_newest_position_plus_its_velocity_times_the_sample_age() {
    let mut corrector = SampleCorrector::default();
    corrector.correct(0, [0.0; 3], sample(1, 0.0, FULL_STICK));
    corrector.correct(33, [0.0; 3], sample(2, 0.264, FULL_STICK));
    corrector.correct(49, [0.0; 3], sample(2, 0.264, FULL_STICK));
    let target = corrector.target().expect("a target after a correction");
    let blended_velocity = 4.0;
    assert!(
        (target[0] - (0.264 + blended_velocity * 0.016)).abs() < 1e-3,
        "{target:?}"
    );
}

#[test]
fn the_age_used_for_the_prediction_is_capped() {
    let mut corrector = SampleCorrector::default();
    corrector.correct(0, [0.0; 3], sample(1, 0.0, FULL_STICK));
    corrector.correct(33, [0.0; 3], sample(2, 0.264, FULL_STICK));
    corrector.correct(5_033, [0.0; 3], sample(2, 0.264, FULL_STICK));
    let target = corrector.target().expect("a target")[0];
    let capped_age_seconds = PREDICTION_AGE_CAP_MS as f32 / 1000.0;
    assert!(
        (target - (0.264 + 4.0 * capped_age_seconds)).abs() < 1e-3,
        "{target}"
    );
}

#[test]
fn a_correction_is_spread_over_the_time_to_the_next_sample() {
    let mut corrector = SampleCorrector::default();
    let PositionCorrection::Nudge(first) = corrector.correct(0, [0.0; 3], sample(1, 0.9, 0.0))
    else {
        panic!("the first tick did not nudge");
    };
    assert!(first[0] > 0.0 && first[0] < 0.9, "{first:?}");
    let PositionCorrection::Nudge(second) = corrector.correct(16, first, sample(1, 0.9, 0.0))
    else {
        panic!("the second tick did not nudge");
    };
    assert!(second[0] > first[0] && second[0] < 0.9 + 1e-4, "{second:?}");
}

#[test]
fn a_small_error_is_not_judged_again_when_no_new_sample_arrives() {
    let mut corrector = SampleCorrector::default();
    corrector.correct(0, [0.0; 3], sample(1, 0.4, 0.0));
    assert_eq!(
        corrector.correct(500, [0.0; 3], sample(1, 0.4, 0.0)),
        PositionCorrection::Leave
    );
}

#[test]
fn a_large_error_teleports_but_not_again_within_half_a_second() {
    let mut corrector = SampleCorrector::default();
    let far = |sequence| sample(sequence, 5.0, 0.0);
    assert_eq!(
        corrector.correct(1_000, [0.0; 3], far(1)),
        PositionCorrection::Teleport
    );
    assert!(matches!(
        corrector.correct(1_100, [0.0; 3], far(1)),
        PositionCorrection::Nudge(_)
    ));
    assert_eq!(
        corrector.correct(1_500, [0.0; 3], far(1)),
        PositionCorrection::Teleport
    );
}

#[test]
fn a_position_that_is_not_a_number_is_left_alone() {
    let mut corrector = SampleCorrector::default();
    assert_eq!(
        corrector.correct(0, [0.0; 3], sample(1, f32::NAN, 0.0)),
        PositionCorrection::Leave
    );
}

#[test]
fn the_report_counts_each_kind_the_largest_error_and_the_sample_rate_once_a_second() {
    let mut corrector = SampleCorrector::default();
    let mut sequence = 0;
    for tick in 0..70_u64 {
        if tick % 2 == 0 {
            sequence += 1;
        }
        corrector.correct(tick * 16, [0.0; 3], sample(sequence, 0.0, 0.0));
        if tick == 10 {
            corrector.correct(tick * 16 + 1, [0.0; 3], sample(sequence + 1000, 0.4, 0.0));
        }
        if tick < 62 {
            assert!(corrector.take_report(tick * 16).is_none());
        }
    }
    let report = corrector
        .take_report(1_100)
        .expect("a report after a second");
    assert!(report.nudged >= 1);
    assert!(report.left_alone > 20);
    assert_eq!(report.teleported, 0);
    assert!(report.largest_error >= 0.3);
    assert!(
        report.samples_per_second > 25.0,
        "{}",
        report.samples_per_second
    );
    let line = position_correction_line("partner", &report);
    assert!(line.starts_with("partner: position corrections"), "{line}");
    assert!(corrector.take_report(1_200).is_none());
}
