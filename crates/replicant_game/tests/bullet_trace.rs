use replicant_game::bullet_flights::MAX_EXTRAPOLATION_MS;
use replicant_game::enemy_bullets::{
    BULLET_TRACE_FRAME_GAP_MS, GuestBulletTrace, HostBulletTrace, HostSighting,
    MAX_BULLET_TRACE_LINES, MAX_TRACED_BULLETS, SpriteAddView, SpriteRefusal, SpriteRefusalTally,
    TracedDraw, read_bullet_motion_fields, sprite_refusal,
};
use replicant_game::offsets::{
    ACTOR_APPLIED_DISPLACEMENT_OFFSET, ACTOR_OWN_VELOCITY_OFFSET, ACTOR_PENDING_DISPLACEMENT_OFFSET,
};

use crate::fake_memory::FakeMemory;

const ACTOR: u64 = 0x2_4000_0000;
const ORIGIN: [f32; 3] = [0.0, 0.0, 0.0];

fn sighting(slot: u16, x: f32) -> HostSighting {
    HostSighting {
        slot,
        hit_id: 7,
        position: [x, 0.0, 0.0],
        fields: None,
    }
}

fn floats(values: [f32; 3]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|value| value.to_le_bytes())
        .collect()
}

#[test]
fn the_host_follows_the_first_bullets_up_to_the_limit_and_no_more() {
    let mut trace = HostBulletTrace::default();
    for slot in 0..10 {
        trace.start(slot, ORIGIN, 0);
    }
    assert_eq!(trace.slots().len(), MAX_TRACED_BULLETS);
    assert_eq!(trace.slots(), vec![0, 1, 2]);
}

#[test]
fn a_bullet_that_ends_gives_its_place_to_the_next_one_that_starts() {
    let mut trace = HostBulletTrace::default();
    for slot in 0..3 {
        trace.start(slot, ORIGIN, 0);
    }
    trace.stop(1);
    trace.start(9, ORIGIN, 100);
    assert_eq!(trace.slots(), vec![0, 2, 9]);
}

#[test]
fn a_host_line_gives_the_move_since_the_last_scan_and_the_speed() {
    let mut trace = HostBulletTrace::default();
    trace.start(4, ORIGIN, 1000);
    let line = trace.line(&sighting(4, 1.0), false, 1050).expect("traced");
    assert!(
        line.contains("slot 4 hit 7 50 ms after its first message"),
        "{line}"
    );
    assert!(
        line.contains("moved (1.000, 0.000, 0.000) in 50 ms (20.00 m/s)"),
        "{line}"
    );
    assert!(line.contains("not sent this scan"), "{line}");
    let next = trace.line(&sighting(4, 1.5), true, 1100).expect("traced");
    assert!(
        next.contains("moved (0.500, 0.000, 0.000) in 50 ms (10.00 m/s)"),
        "{next}"
    );
    assert!(next.contains("sent now"), "{next}");
}

#[test]
fn a_bullet_the_host_does_not_follow_gets_no_line() {
    let mut trace = HostBulletTrace::default();
    assert!(trace.line(&sighting(4, 1.0), true, 1050).is_none());
}

#[test]
fn the_game_motion_fields_are_read_from_the_actor() {
    let mut memory = FakeMemory::new();
    memory.put(
        ACTOR + ACTOR_PENDING_DISPLACEMENT_OFFSET,
        &floats([1.0, 2.0, 3.0]),
    );
    memory.put(
        ACTOR + ACTOR_APPLIED_DISPLACEMENT_OFFSET,
        &floats([4.0, 5.0, 6.0]),
    );
    memory.put(ACTOR + ACTOR_OWN_VELOCITY_OFFSET, &floats([7.0, 8.0, 9.0]));
    let fields = read_bullet_motion_fields(&memory, ACTOR).expect("readable");
    assert_eq!(fields.pending_step, [1.0, 2.0, 3.0]);
    assert_eq!(fields.applied_step, [4.0, 5.0, 6.0]);
    assert_eq!(fields.own_velocity, [7.0, 8.0, 9.0]);
}

#[test]
fn motion_fields_that_cannot_be_read_give_none_and_the_line_says_so() {
    let memory = FakeMemory::new();
    assert!(read_bullet_motion_fields(&memory, ACTOR).is_none());
    let mut trace = HostBulletTrace::default();
    trace.start(4, ORIGIN, 0);
    let line = trace.line(&sighting(4, 0.0), true, 50).expect("traced");
    assert!(line.contains("game fields unreadable"), "{line}");
}

#[test]
fn a_guest_message_line_gives_how_far_the_copy_was_from_the_host() {
    let mut trace = GuestBulletTrace::default();
    trace.start(2, 5000);
    let line = trace
        .message(2, 5300, [3.0, 0.0, 0.0], [3.0, 4.0, 0.0], [10.0, 0.0, 0.0])
        .expect("traced");
    assert!(
        line.contains("message 1 came 300 ms after the last"),
        "{line}"
    );
    assert!(line.contains("off by 4.000 m"), "{line}");
    assert!(line.contains("(10.00 m/s)"), "{line}");
}

#[test]
fn a_guest_frame_line_says_when_the_copy_stopped_moving_on() {
    let mut trace = GuestBulletTrace::default();
    trace.start(2, 5000);
    let early = trace
        .frame(
            2,
            5000 + BULLET_TRACE_FRAME_GAP_MS,
            ORIGIN,
            Some(TracedDraw::Taken),
        )
        .expect("traced");
    assert!(!early.contains("extrapolation stopped"), "{early}");
    assert!(early.contains("the batch took it"), "{early}");
    let late = trace
        .frame(
            2,
            5000 + MAX_EXTRAPOLATION_MS + BULLET_TRACE_FRAME_GAP_MS,
            ORIGIN,
            Some(TracedDraw::Refused(SpriteRefusal::OffScreen)),
        )
        .expect("traced");
    assert!(late.contains("extrapolation stopped"), "{late}");
    assert!(late.contains("the batch refused it: off screen"), "{late}");
}

#[test]
fn guest_frame_lines_come_at_most_once_per_gap() {
    let mut trace = GuestBulletTrace::default();
    trace.start(2, 5000);
    assert!(trace.frame(2, 5010, ORIGIN, None).is_none());
    assert!(
        trace
            .frame(2, 5000 + BULLET_TRACE_FRAME_GAP_MS, ORIGIN, None)
            .is_some()
    );
}

#[test]
fn a_guest_end_line_frees_the_place_and_counts_the_messages() {
    let mut trace = GuestBulletTrace::default();
    trace.start(2, 5000);
    trace.message(2, 5100, ORIGIN, ORIGIN, ORIGIN);
    let line = trace.ended(2, 5800, "timed out").expect("traced");
    assert!(
        line.contains("timed out after 1 messages, 700 ms after the last"),
        "{line}"
    );
    assert!(!trace.is_traced(2));
    assert!(trace.slots().is_empty());
}

#[test]
fn the_trace_stops_writing_after_its_line_budget() {
    let mut trace = GuestBulletTrace::default();
    trace.start(2, 0);
    let written = (0..MAX_BULLET_TRACE_LINES + 10)
        .filter(|_| trace.message(2, 1, ORIGIN, ORIGIN, ORIGIN).is_some())
        .count();
    assert_eq!(written, MAX_BULLET_TRACE_LINES as usize);
}

#[test]
fn restarting_the_line_budget_lets_the_next_fight_write_again() {
    let mut trace = GuestBulletTrace::default();
    trace.start(2, 0);
    for _ in 0..MAX_BULLET_TRACE_LINES {
        trace.message(2, 1, ORIGIN, ORIGIN, ORIGIN);
    }
    assert!(trace.message(2, 1, ORIGIN, ORIGIN, ORIGIN).is_none());
    trace.restart_line_budget();
    assert!(trace.message(2, 1, ORIGIN, ORIGIN, ORIGIN).is_some());
}

fn view(batch_full: bool, in_view: bool, distance_squared: f32) -> SpriteAddView {
    SpriteAddView {
        batch_full,
        in_view,
        camera_distance_squared: distance_squared,
        nearest_distance_squared: 4.0,
    }
}

#[test]
fn a_refused_sprite_is_put_down_to_the_first_check_the_game_makes() {
    assert_eq!(
        sprite_refusal(view(true, false, 1.0)),
        SpriteRefusal::BatchFull
    );
    assert_eq!(
        sprite_refusal(view(false, false, 1.0)),
        SpriteRefusal::OffScreen
    );
    assert_eq!(
        sprite_refusal(view(false, true, 1.0)),
        SpriteRefusal::NearCamera
    );
    assert_eq!(sprite_refusal(view(false, true, 9.0)), SpriteRefusal::Other);
}

#[test]
fn the_refusal_line_counts_each_reason_and_clears() {
    let mut tally = SpriteRefusalTally::default();
    assert!(tally.take_line().is_none());
    for refusal in [
        SpriteRefusal::OffScreen,
        SpriteRefusal::OffScreen,
        SpriteRefusal::NearCamera,
        SpriteRefusal::BatchFull,
        SpriteRefusal::Other,
    ] {
        tally.note(refusal);
    }
    let line = tally.take_line().expect("something refused");
    assert!(
        line.contains(
            "refused 5 host sprites: 2 off screen, 1 near the camera, 1 batch full, 1 other"
        ),
        "{line}"
    );
    assert!(tally.take_line().is_none());
}
