use replicant_game::kaine::{
    OpenDoorStateClock, OpenDoorStateVerdict, SWAPPED_BODY_OPEN_DOOR_STATE_MOST_MS,
    body_has_a_capped_open_door_state, open_door_state_ended_text,
};
use replicant_proto::PartnerBody;

const STATE: u64 = 0x0000_0001_A5C3_7D80;
const NEXT_STATE: u64 = 0x0000_0001_A5C3_9100;

#[test]
fn only_the_swapped_body_has_its_door_state_capped() {
    assert!(body_has_a_capped_open_door_state(
        PartnerBody::SeedKaineE,
        true
    ));
    assert!(!body_has_a_capped_open_door_state(
        PartnerBody::SeedKaineE,
        false
    ));
    assert!(!body_has_a_capped_open_door_state(PartnerBody::Nier, true));
}

#[test]
fn a_door_state_that_ends_by_itself_in_a_normal_time_is_never_cut() {
    // Run 1114q host: the first body's door state lasted 2 to 3 seconds, the swapped body's about 10.
    let mut clock = OpenDoorStateClock::new();
    for frame in 0..180 {
        assert_eq!(
            clock.observe(STATE, true, 1_000 + frame * 16),
            OpenDoorStateVerdict::Run
        );
    }
}

#[test]
fn the_swapped_body_door_state_is_ended_once_it_has_run_the_most() {
    let mut clock = OpenDoorStateClock::new();
    let start = 500_000;
    let mut ended_at = None;
    for frame in 0..800 {
        let now = start + frame * 16;
        if let OpenDoorStateVerdict::EndNow { ran_ms } = clock.observe(STATE, true, now) {
            ended_at = Some((now - start, ran_ms));
            break;
        }
    }
    let (after, ran_ms) = ended_at.expect("the door state was never ended");
    assert!(ran_ms >= SWAPPED_BODY_OPEN_DOOR_STATE_MOST_MS);
    assert!(after < SWAPPED_BODY_OPEN_DOOR_STATE_MOST_MS + 32);
}

#[test]
fn the_next_door_state_starts_its_own_count() {
    let mut clock = OpenDoorStateClock::new();
    assert_eq!(
        clock.observe(STATE, true, 10_000),
        OpenDoorStateVerdict::Run
    );
    assert_eq!(
        clock.observe(STATE, true, 12_900),
        OpenDoorStateVerdict::Run
    );
    assert_eq!(
        clock.observe(NEXT_STATE, true, 12_916),
        OpenDoorStateVerdict::Run
    );
    assert_eq!(
        clock.observe(NEXT_STATE, true, 15_000),
        OpenDoorStateVerdict::Run
    );
}

#[test]
fn the_same_state_address_used_again_later_starts_a_new_count() {
    let mut clock = OpenDoorStateClock::new();
    assert_eq!(
        clock.observe(STATE, true, 10_000),
        OpenDoorStateVerdict::Run
    );
    assert_eq!(
        clock.observe(STATE, true, 60_000),
        OpenDoorStateVerdict::Run
    );
    assert_eq!(
        clock.observe(STATE, true, 62_000),
        OpenDoorStateVerdict::Run
    );
}

#[test]
fn the_first_body_is_never_counted() {
    let mut clock = OpenDoorStateClock::new();
    for frame in 0..1_000 {
        assert_eq!(
            clock.observe(STATE, false, frame * 16),
            OpenDoorStateVerdict::Run
        );
    }
}

#[test]
fn the_log_line_names_how_long_the_state_ran() {
    assert_eq!(
        open_door_state_ended_text(3_008),
        "doors: the swapped body's door state ran 3008 ms with no door motion of hers to end it, ended it now; the door's own scripts carry on"
    );
}

#[test]
fn run_1114s_the_door_closing_ends_the_state_as_it_did_for_the_first_body() {
    let mut clock = OpenDoorStateClock::new();
    let start = 500_000;
    for frame in 0..125 {
        assert_eq!(
            clock.observe(STATE, true, start + frame * 16),
            OpenDoorStateVerdict::Run
        );
    }
    clock.note_door_closed(start + 2_288);
    assert_eq!(
        clock.observe(STATE, true, start + 2_300),
        OpenDoorStateVerdict::EndNow { ran_ms: 2_300 }
    );
}

#[test]
fn a_door_close_from_before_the_state_began_does_not_end_the_next_state() {
    let mut clock = OpenDoorStateClock::new();
    clock.note_door_closed(100_000);
    assert_eq!(
        clock.observe(STATE, true, 200_000),
        OpenDoorStateVerdict::Run
    );
    assert_eq!(
        clock.observe(STATE, true, 201_000),
        OpenDoorStateVerdict::Run
    );
}
