use automata_game::raw_mouse::{
    CameraAngles, FOLLOW_PROBE_MIN_GAP_BETWEEN_LINES_MICROS, FollowProbe, FollowSample,
    IDENTITY_BASIS, LookHold, rotate_eased_offset, rotation_basis,
};

const FRAMES_PER_SECOND_OF_GAME_TIME: f32 = 60.0;
const FOLLOW_EASE_RATE: f32 = 0.1;
const PLACEMENT_EASE: f32 = 0.1;
const DISTANCE_WEIGHT: f32 = 0.3;
const MINIMUM_DISTANCE: f32 = 0.35;
const LOOK_AT_HEIGHT: f32 = 1.2;
const LAG_PULL: f32 = 0.1;
const LAG_SPEED_CAP: f32 = 0.078;
const LAG_HEIGHT_SPEED_CAP: f32 = 0.042;
const LAG_SPEED_EASE: f32 = 0.1;
const RUN_METRES_PER_SECOND: f32 = 6.0;
const BODY_TURN_RADIANS_PER_SECOND: f32 = 0.5;
const PITCH_LIMIT: f32 = 1.256_637_1;
const FIRST_BURST_SECONDS: f32 = 2.0;
const YAW_BURST_SECONDS: f32 = 0.3;
const PITCH_BURST_SECONDS: f32 = 0.2;
const FAST_YAW_RADIANS_PER_SECOND: f32 = 12.0;
const PITCH_RADIANS_PER_SECOND: f32 = 4.0;
const STICK_YAW_RADIANS_PER_SECOND: f32 = 2.1;
const SCENARIO_SECONDS: f32 = 16.0;
const FRAME_RATES: [f32; 3] = [30.0, 60.0, 144.0];
const BASE_DISTANCE: f32 = 3.8;

type Vec3 = [f32; 3];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Camera {
    GameOnly,
    StickTurn,
    RawWithoutCut,
    RawWithCut,
    RawWithFix,
}

#[derive(Clone, Copy)]
struct Scenario {
    zone_steps: bool,
}

#[derive(Clone, Copy)]
struct Frame {
    seconds: f32,
    orbit: f32,
    placed: f32,
    jump: f32,
    look_lag: f32,
    turning: bool,
}

struct State {
    angles: CameraAngles,
    target_eye: Vec3,
    target_look: Vec3,
    eased_eye: Vec3,
    eased_look: Vec3,
    eye_to_look: f32,
    gap: f32,
    cut_frames: i32,
    lagged: Vec3,
    lag_speed: f32,
    lag_height_speed: f32,
}

fn sub(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn add(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn scale(a: Vec3, k: f32) -> Vec3 {
    [a[0] * k, a[1] * k, a[2] * k]
}

fn length(a: Vec3) -> f32 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}

fn about_x(row: Vec3, angle: f32) -> Vec3 {
    let (sin, cos) = angle.sin_cos();
    [
        row[0],
        row[1] * cos - row[2] * sin,
        row[1] * sin + row[2] * cos,
    ]
}

fn about_y(row: Vec3, angle: f32) -> Vec3 {
    let (sin, cos) = angle.sin_cos();
    [
        row[0] * cos + row[2] * sin,
        row[1],
        -row[0] * sin + row[2] * cos,
    ]
}

fn zone_distance(seconds: f32, zone_steps: bool) -> f32 {
    if !zone_steps {
        return BASE_DISTANCE;
    }
    match seconds {
        s if s < 3.0 => BASE_DISTANCE,
        s if s < 6.0 => 5.8,
        s if s < 9.0 => 5.4,
        s if s < 12.0 => 5.0,
        _ => BASE_DISTANCE,
    }
}

fn mouse_turn(seconds: f32, frames_per_second: f32) -> (f32, f32) {
    if seconds < FIRST_BURST_SECONDS {
        return (0.0, 0.0);
    }
    let since = seconds - FIRST_BURST_SECONDS;
    let slot = (since / 1.0).floor() as i32;
    let into_slot = since - slot as f32;
    let yaw_step = FAST_YAW_RADIANS_PER_SECOND / frames_per_second;
    let pitch_step = PITCH_RADIANS_PER_SECOND / frames_per_second;
    match slot % 4 {
        0 if into_slot < YAW_BURST_SECONDS => (0.0, yaw_step),
        1 if into_slot < YAW_BURST_SECONDS => (0.0, -yaw_step),
        2 if into_slot < PITCH_BURST_SECONDS => (pitch_step, 0.0),
        3 if into_slot < PITCH_BURST_SECONDS => (-pitch_step, 0.0),
        _ => (0.0, 0.0),
    }
}

fn stick_turn(seconds: f32, frames_per_second: f32) -> f32 {
    let since = seconds - FIRST_BURST_SECONDS;
    if since >= 0.0 && since % 1.0 < 0.5 {
        STICK_YAW_RADIANS_PER_SECOND / frames_per_second
    } else {
        0.0
    }
}

fn run(camera: Camera, frames_per_second: f32, scenario: Scenario) -> Vec<Frame> {
    let dt = FRAMES_PER_SECOND_OF_GAME_TIME / frames_per_second;
    let mut body = [0.0_f32; 3];
    let mut heading = 0.0_f32;
    let start_look = [0.0, LOOK_AT_HEIGHT, 0.0];
    let mut state = State {
        angles: CameraAngles {
            pitch: 0.0,
            yaw: 0.0,
        },
        target_eye: [0.0; 3],
        target_look: [0.0; 3],
        eased_eye: add(start_look, [0.0, 0.0, BASE_DISTANCE]),
        eased_look: start_look,
        eye_to_look: BASE_DISTANCE,
        gap: 0.0,
        cut_frames: 0,
        lagged: [0.0, LOOK_AT_HEIGHT, 0.0],
        lag_speed: 0.0,
        lag_height_speed: 0.0,
    };
    let mut hold = LookHold::new();
    let mut frames = Vec::new();
    let mut previous: Option<(Vec3, Vec3)> = None;
    let total = (SCENARIO_SECONDS * frames_per_second) as usize;
    for index in 0..total {
        let seconds = index as f32 / frames_per_second;
        let speed = RUN_METRES_PER_SECOND * (seconds / 0.3).min(1.0);
        if seconds > 1.0 {
            heading += BODY_TURN_RADIANS_PER_SECOND / frames_per_second;
        }
        body[0] += heading.sin() * speed / frames_per_second;
        body[2] += heading.cos() * speed / frames_per_second;
        let before = state.angles;
        let (pitch_step, yaw_step) = match camera {
            Camera::GameOnly => (0.0, 0.0),
            Camera::StickTurn => (0.0, stick_turn(seconds, frames_per_second)),
            _ => mouse_turn(seconds, frames_per_second),
        };
        state.angles = CameraAngles {
            pitch: (before.pitch + pitch_step).clamp(-PITCH_LIMIT, PITCH_LIMIT),
            yaw: before.yaw + yaw_step,
        };
        let turned = pitch_step != 0.0 || yaw_step != 0.0;
        let now_ms = (seconds * 1000.0) as u64;
        match camera {
            Camera::RawWithCut => {
                if hold.step(now_ms, turned) {
                    state.cut_frames = 1;
                }
            }
            Camera::RawWithFix if turned => {
                let offset = sub(state.eased_eye, state.eased_look);
                let turned_offset =
                    rotate_eased_offset(offset, before, state.angles, &IDENTITY_BASIS);
                state.eased_eye = add(state.eased_look, turned_offset);
            }
            _ => {}
        }
        let zone = zone_distance(seconds, scenario.zone_steps).max(MINIMUM_DISTANCE);
        let placed = place(&mut state, body, zone);
        lag_look_at(&mut state, body);
        ease(&mut state, dt);
        let look_from_body = sub(state.eased_look, body);
        let offset = sub(state.eased_eye, state.eased_look);
        let unturned = rotate_eased_offset(offset, state.angles, before, &IDENTITY_BASIS);
        let jump = previous.map_or(0.0, |(look, eye)| {
            length(add(sub(look_from_body, look), sub(unturned, eye)))
        });
        previous = Some((look_from_body, offset));
        frames.push(Frame {
            seconds,
            orbit: state.eye_to_look,
            placed,
            jump,
            look_lag: length(sub(look_from_body, [0.0, LOOK_AT_HEIGHT, 0.0])),
            turning: turned,
        });
    }
    frames
}

fn place(state: &mut State, body: Vec3, zone: f32) -> f32 {
    let look = add(body, [0.0, LOOK_AT_HEIGHT, 0.0]);
    let distance = if state.cut_frames > 0 {
        state.gap = 0.0;
        zone
    } else {
        let wanted_gap = zone - state.eye_to_look;
        let mut gap = (wanted_gap - state.gap) * PLACEMENT_EASE + state.gap;
        if wanted_gap <= 0.0 {
            gap = gap.max(wanted_gap);
        } else {
            gap = gap.min(wanted_gap);
        }
        state.gap = gap;
        gap * DISTANCE_WEIGHT + state.eye_to_look
    }
    .max(MINIMUM_DISTANCE);
    let offset = about_y(
        about_x([0.0, 0.0, distance], state.angles.pitch),
        state.angles.yaw,
    );
    state.target_eye = add(look, offset);
    state.target_look = look;
    distance
}

fn lag_look_at(state: &mut State, body: Vec3) {
    let local = about_y(sub(state.target_look, body), -state.angles.yaw);
    if state.cut_frames > 0 {
        state.lagged = local;
        state.lag_speed = 0.0;
        state.lag_height_speed = 0.0;
    } else {
        let difference = sub(local, state.lagged);
        let (pull_x, pull_z) = (difference[0] * LAG_PULL, difference[2] * LAG_PULL);
        let capped = pull_x.hypot(pull_z).min(LAG_SPEED_CAP);
        state.lag_speed += (capped - state.lag_speed) * LAG_SPEED_EASE;
        let height_pull = (difference[1] * LAG_PULL).abs().min(LAG_HEIGHT_SPEED_CAP);
        state.lag_height_speed += (height_pull - state.lag_height_speed) * LAG_SPEED_EASE;
        if capped > 0.0 {
            let along = pull_x.hypot(pull_z);
            state.lagged[0] += pull_x / along * state.lag_speed;
            state.lagged[2] += pull_z / along * state.lag_speed;
        }
        if difference[1] * LAG_PULL <= 0.0 {
            state.lagged[1] -= state.lag_height_speed;
        } else {
            state.lagged[1] += state.lag_height_speed;
        }
    }
    let moved_look = add(about_y(state.lagged, state.angles.yaw), body);
    let shift = sub(moved_look, state.target_look);
    state.target_eye = add(state.target_eye, shift);
    state.target_look = moved_look;
}

fn ease(state: &mut State, dt: f32) {
    if state.cut_frames > 0 {
        state.cut_frames -= 1;
        state.eased_eye = state.target_eye;
        state.eased_look = state.target_look;
    } else {
        let step = (dt * FOLLOW_EASE_RATE).min(1.0);
        let towards = |eased: Vec3, target: Vec3| add(eased, scale(sub(target, eased), step));
        state.eased_look = towards(state.eased_look, state.target_look);
        state.eased_eye = towards(state.eased_eye, state.target_eye);
    }
    state.eye_to_look = length(sub(state.eased_eye, state.eased_look));
}

fn after_warm_up(frames: &[Frame]) -> impl Iterator<Item = &Frame> {
    frames
        .iter()
        .filter(|frame| frame.seconds >= FIRST_BURST_SECONDS)
}

fn smallest_placed(frames: &[Frame]) -> f32 {
    after_warm_up(frames)
        .map(|frame| frame.placed)
        .fold(f32::MAX, f32::min)
}

fn largest_jump(frames: &[Frame]) -> f32 {
    after_warm_up(frames)
        .map(|frame| frame.jump)
        .fold(0.0, f32::max)
}

fn largest_jump_while_turning_or_just_after(frames: &[Frame]) -> f32 {
    let mut largest = 0.0_f32;
    let mut last_turn = f32::MIN;
    for frame in after_warm_up(frames) {
        if frame.turning {
            last_turn = frame.seconds;
        }
        if frame.seconds - last_turn < 0.6 {
            largest = largest.max(frame.jump);
        }
    }
    largest
}

const NO_ZONE_STEPS: Scenario = Scenario { zone_steps: false };
const WITH_ZONE_STEPS: Scenario = Scenario { zone_steps: true };

#[test]
fn round_43_without_the_cut_raw_turns_pull_the_camera_in_to_about_one_metre() {
    for scenario in [NO_ZONE_STEPS, WITH_ZONE_STEPS] {
        let frames = run(Camera::RawWithoutCut, 60.0, scenario);
        let nearest = smallest_placed(&frames);
        assert!((0.8..=1.5).contains(&nearest), "nearest placed {nearest}");
    }
    for fps in FRAME_RATES {
        let nearest = smallest_placed(&run(Camera::RawWithoutCut, fps, NO_ZONE_STEPS));
        assert!(nearest < 1.6, "{fps} fps nearest placed {nearest}");
    }
}

#[test]
fn the_game_alone_and_the_stick_never_pull_in_that_far() {
    for fps in FRAME_RATES {
        let game = run(Camera::GameOnly, fps, NO_ZONE_STEPS);
        let stick = run(Camera::StickTurn, fps, NO_ZONE_STEPS);
        assert!(smallest_placed(&game) > BASE_DISTANCE - 0.01);
        assert!(smallest_placed(&stick) > 3.4, "{fps} fps stick");
    }
}

#[test]
fn round_48_the_cut_hops_the_camera_at_each_switch() {
    for fps in FRAME_RATES {
        let game = largest_jump(&run(Camera::GameOnly, fps, NO_ZONE_STEPS));
        let cut = largest_jump(&run(Camera::RawWithCut, fps, NO_ZONE_STEPS));
        assert!(cut > 0.5, "{fps} fps hop {cut}");
        assert!(
            cut > game * 5.0,
            "{fps} fps hop {cut} against the game's {game}"
        );
    }
}

fn orbit_is_within_two_percent_of_the_unturned_camera(
    camera: Camera,
    fps: f32,
    scenario: Scenario,
) -> bool {
    let game = run(Camera::GameOnly, fps, scenario);
    run(camera, fps, scenario)
        .iter()
        .zip(&game)
        .all(|(turned, unturned)| (0.98..=1.02).contains(&(turned.orbit / unturned.orbit)))
}

fn body_to_camera_never_jumps_more_than_the_unturned_camera(
    camera: Camera,
    fps: f32,
    scenario: Scenario,
) -> bool {
    let game = largest_jump(&run(Camera::GameOnly, fps, scenario));
    let turned = run(camera, fps, scenario);
    largest_jump(&turned) <= game + 1e-3
        && largest_jump_while_turning_or_just_after(&turned) <= game + 1e-3
}

#[test]
fn the_fix_keeps_the_orbit_distance_of_the_camera_nobody_turns() {
    for scenario in [NO_ZONE_STEPS, WITH_ZONE_STEPS] {
        for fps in FRAME_RATES {
            assert!(
                orbit_is_within_two_percent_of_the_unturned_camera(
                    Camera::RawWithFix,
                    fps,
                    scenario
                ),
                "{fps} fps"
            );
        }
    }
}

#[test]
fn the_pull_in_of_round_43_breaks_the_orbit_distance_check() {
    for fps in FRAME_RATES {
        assert!(!orbit_is_within_two_percent_of_the_unturned_camera(
            Camera::RawWithoutCut,
            fps,
            NO_ZONE_STEPS
        ));
    }
}

#[test]
fn the_hop_of_round_48_breaks_the_body_to_camera_check() {
    for scenario in [NO_ZONE_STEPS, WITH_ZONE_STEPS] {
        for fps in FRAME_RATES {
            assert!(!body_to_camera_never_jumps_more_than_the_unturned_camera(
                Camera::RawWithCut,
                fps,
                scenario
            ));
        }
    }
}

#[test]
fn the_pull_in_of_round_43_also_breaks_the_body_to_camera_check() {
    for fps in FRAME_RATES {
        assert!(!body_to_camera_never_jumps_more_than_the_unturned_camera(
            Camera::RawWithoutCut,
            fps,
            NO_ZONE_STEPS
        ));
    }
}

#[test]
fn the_fix_keeps_at_least_the_orbit_distance_of_the_stick_turned_camera() {
    for fps in FRAME_RATES {
        let stick = run(Camera::StickTurn, fps, NO_ZONE_STEPS);
        let fixed = run(Camera::RawWithFix, fps, NO_ZONE_STEPS);
        let nearest = |frames: &[Frame]| {
            after_warm_up(frames)
                .map(|f| f.orbit)
                .fold(f32::MAX, f32::min)
        };
        assert!(nearest(&fixed) >= nearest(&stick), "{fps} fps");
    }
}

#[test]
fn the_fix_never_changes_the_body_to_camera_vector_more_than_the_game_alone_does() {
    for scenario in [NO_ZONE_STEPS, WITH_ZONE_STEPS] {
        for fps in FRAME_RATES {
            assert!(
                body_to_camera_never_jumps_more_than_the_unturned_camera(
                    Camera::RawWithFix,
                    fps,
                    scenario
                ),
                "{fps} fps"
            );
        }
    }
}

#[test]
fn the_fix_leaves_the_follow_lag_to_the_game_and_the_cut_does_not() {
    for fps in FRAME_RATES {
        let game = run(Camera::GameOnly, fps, NO_ZONE_STEPS);
        let fixed = run(Camera::RawWithFix, fps, NO_ZONE_STEPS);
        let cut = run(Camera::RawWithCut, fps, NO_ZONE_STEPS);
        let largest_difference = |other: &[Frame]| {
            game.iter()
                .zip(other)
                .map(|(a, b)| (a.look_lag - b.look_lag).abs())
                .fold(0.0, f32::max)
        };
        assert!(largest_difference(&fixed) < 0.02, "{fps} fps fix");
        assert!(largest_difference(&cut) > 0.5, "{fps} fps cut");
    }
}

#[test]
fn a_camera_basis_that_is_not_a_rotation_is_refused() {
    assert!(rotation_basis(IDENTITY_BASIS).is_some());
    assert!(rotation_basis([[2.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]).is_none());
}

#[test]
fn turning_the_offset_by_a_step_matches_placing_it_at_the_new_angles() {
    let before = CameraAngles {
        pitch: 0.2,
        yaw: 0.7,
    };
    let after = CameraAngles {
        pitch: -0.3,
        yaw: 2.0,
    };
    let place = |angles: CameraAngles| about_y(about_x([0.0, 0.0, 3.8], angles.pitch), angles.yaw);
    let turned = rotate_eased_offset(place(before), before, after, &IDENTITY_BASIS);
    let wanted = place(after);
    for axis in 0..3 {
        assert!((turned[axis] - wanted[axis]).abs() < 1e-4, "axis {axis}");
    }
}

#[test]
fn turning_by_no_step_leaves_the_offset_alone() {
    let angles = CameraAngles {
        pitch: 0.4,
        yaw: -1.1,
    };
    let offset = [1.0, 2.0, 3.0];
    let same = rotate_eased_offset(offset, angles, angles, &IDENTITY_BASIS);
    for axis in 0..3 {
        assert!((same[axis] - offset[axis]).abs() < 1e-5);
    }
}

#[test]
fn a_turned_basis_is_undone_and_redone_around_the_step() {
    let basis = rotation_basis([[0.0, 0.0, 1.0], [0.0, 1.0, 0.0], [-1.0, 0.0, 0.0]]).unwrap();
    let before = CameraAngles {
        pitch: 0.1,
        yaw: 0.2,
    };
    let after = CameraAngles {
        pitch: 0.3,
        yaw: 0.9,
    };
    let row_times = |row: Vec3| {
        [
            row[0] * basis[0][0] + row[1] * basis[1][0] + row[2] * basis[2][0],
            row[0] * basis[0][1] + row[1] * basis[1][1] + row[2] * basis[2][1],
            row[0] * basis[0][2] + row[1] * basis[1][2] + row[2] * basis[2][2],
        ]
    };
    let place = |angles: CameraAngles| {
        row_times(about_y(about_x([0.0, 0.0, 3.8], angles.pitch), angles.yaw))
    };
    let turned = rotate_eased_offset(place(before), before, after, &basis);
    let wanted = place(after);
    for axis in 0..3 {
        assert!((turned[axis] - wanted[axis]).abs() < 1e-4, "axis {axis}");
    }
}

fn probe_sample(body_x: f32, raw_turned: bool, look_x: f32) -> FollowSample {
    FollowSample {
        body: [body_x, 0.0, 0.0],
        look_at_from_body: [look_x, 1.2, 0.0],
        eye_from_look_at_turn_removed: [0.0, 0.0, 3.8],
        raw_turned,
    }
}

#[test]
fn the_follow_probe_reports_the_largest_change_once_a_second_while_the_hero_moves() {
    let mut probe = FollowProbe::new();
    let mut lines = Vec::new();
    for frame in 0..200_u64 {
        let micros = frame * 16_667;
        let look = if frame == 90 { 0.9 } else { 0.0 };
        if let Some(line) = probe.step(micros, probe_sample(frame as f32 * 0.1, frame > 70, look)) {
            lines.push(line);
        }
    }
    assert_eq!(lines.len(), 3);
    assert!((lines[1].largest_jump - 0.9).abs() < 1e-4);
    assert!(lines[1].raw_turned);
    assert!(!lines[0].raw_turned);
    assert!(lines[0].largest_frame_ms > 16.0 && lines[0].largest_frame_ms < 17.0);
    assert_eq!(FOLLOW_PROBE_MIN_GAP_BETWEEN_LINES_MICROS, 1_000_000);
}

#[test]
fn the_follow_probe_is_quiet_while_the_hero_stands_still() {
    let mut probe = FollowProbe::new();
    let lines = (0..300_u64)
        .filter_map(|frame| probe.step(frame * 16_667, probe_sample(0.0, true, 0.0)))
        .count();
    assert_eq!(lines, 0);
}
