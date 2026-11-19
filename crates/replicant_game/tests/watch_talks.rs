use replicant_game::callback_gate::{
    EndArrival, EndDrop, EndFollow, control_slots_free, end_follow, end_follow_around_a_stretch,
};
use replicant_game::fade_sync::HostFadeSync;
use replicant_game::host_led_stretch::{
    HostLedStretchState, StretchChange, StretchEnd, StretchPlace, first_step_refused_line,
    stretch_the_guest_waits_to_start,
};

const VILLAGE: &str = "A_CENTER_VILLAGE_01";
const VILLAGE_PHASE: u16 = 1;
const DREAM_START: &str = "EID_1500_a0005_010_01_Start";
const DREAM_END: &str = "EID_1500_a0005_010_01_End";
const MORNING_STEP: &str = "EID_1500_a0005_010_01_End02";
const DREAM_START_POP_MS: u64 = 69_759_297;
const DREAM_FIRST_FADE_OUT_MS: u64 = 69_759_313;
const HOST_TICK_AFTER_THE_POP_MS: u64 = 69_759_330;
const DREAM_END_MS: u64 = 69_787_332;
const GUEST_SLOTS_AFTER_ITS_LATE_CHOICE: [u32; 4] = [0x480, 0, 0, 0];
const SEASIDE: &str = "A_SEASIDE_TOWN_01";
const SEASIDE_PHASE: u16 = 18;
const SEASIDE_TALK_START: &str = "EID_9300_a0050_0010_Start";
const SEASIDE_FIRST_END: &str = "EID_9300_a0050_0010_End";
const SEASIDE_LAST_STEP: &str = "EID_9300_a0050_0240_End";
const SEASIDE_TALK_START_MS: u64 = 69_909_623;
const SEASIDE_FIRST_END_MS: u64 = 69_911_952;

fn arrival(phase: u16, slots: [u32; 4]) -> EndArrival {
    EndArrival {
        message_phase: phase,
        shared_phase: Some(phase),
        local_phase: Some(phase),
        local_in_own_talk: false,
        control_mask_settled: control_slots_free(slots, false),
        already_ran_here: false,
        own_copy_of_same_event_running: false,
    }
}

#[test]
fn the_dreams_first_fade_out_reaches_the_guest_when_the_stretch_arms_the_sync_at_its_pop() {
    let mut armed_on_the_next_tick = HostFadeSync::new();
    assert!(
        !armed_on_the_next_tick.should_send_fade_out(DREAM_FIRST_FADE_OUT_MS),
        "run 1115f: the 5 s fade out ran 16 ms after the pop, before the tick at {HOST_TICK_AFTER_THE_POP_MS} ms armed the sync"
    );
    let mut armed_at_the_pop = HostFadeSync::new();
    armed_at_the_pop.set_host_led_stretch(true);
    assert!(armed_at_the_pop.should_send_fade_out(DREAM_FIRST_FADE_OUT_MS));
    assert!(armed_at_the_pop.should_send_fade_in());
}

#[test]
fn the_dream_is_no_stretch_the_guest_plays_its_own_copy_beside_the_host() {
    let mut guest = HostLedStretchState::new();
    assert_eq!(
        guest.host_step(VILLAGE, DREAM_START, DREAM_START_POP_MS),
        None,
        "run 1115j 01:47:57.019: the guest's own game fired the dream's start 114 ms before the host's reached it, so a watch on top of it drove the same window twice"
    );
    assert_eq!(guest.current(), None);
    assert_eq!(guest.host_step(VILLAGE, DREAM_END, DREAM_END_MS), None);
    assert_eq!(guest.place_of_end(DREAM_END), StretchPlace::NotInOne);
    let paused = arrival(VILLAGE_PHASE, GUEST_SLOTS_AFTER_ITS_LATE_CHOICE);
    assert_eq!(
        end_follow_around_a_stretch(&paused, guest.place_of_end(DREAM_END)),
        EndFollow::Drop(EndDrop::ControlMaskBusy),
        "the host's End copy waits on the guest's own pause, and the guest's own End pops in its place"
    );
    assert_eq!(guest.place_of_end(MORNING_STEP), StretchPlace::NotInOne);
}

#[test]
fn the_seaside_talk_is_a_stretch_the_guest_watches_from_the_hosts_talk_command() {
    let mut guest = HostLedStretchState::new();
    let started = guest.host_step(SEASIDE, SEASIDE_TALK_START, SEASIDE_TALK_START_MS);
    let Some(StretchChange::Started(stretch)) = started else {
        panic!("the seaside talk began no stretch: {started:?}");
    };
    assert_eq!(stretch.last_step, SEASIDE_LAST_STEP);
    assert!(stretch.guest_waits_for_the_hosts_first_step);
    assert_eq!(guest.current(), Some(stretch));
}

#[test]
fn inside_the_seaside_stretch_the_hosts_first_end_is_not_run_on_the_guest() {
    let mut guest = HostLedStretchState::new();
    guest.host_step(SEASIDE, SEASIDE_TALK_START, SEASIDE_TALK_START_MS);
    assert_eq!(
        guest.host_step(SEASIDE, SEASIDE_FIRST_END, SEASIDE_FIRST_END_MS),
        None
    );
    let free = arrival(SEASIDE_PHASE, [0; 4]);
    assert_eq!(
        end_follow(&free),
        EndFollow::Mirror,
        "run 1115f: the guest ran the host's first End, and its own copy opened talk 100012 by itself"
    );
    assert_eq!(
        guest.place_of_end(SEASIDE_FIRST_END),
        StretchPlace::InsideIt
    );
    assert_eq!(
        end_follow_around_a_stretch(&free, guest.place_of_end(SEASIDE_FIRST_END)),
        EndFollow::Drop(EndDrop::HostLeadsTheStretch)
    );
    assert!(!EndDrop::HostLeadsTheStretch.can_wait());
}

#[test]
fn the_seaside_stretchs_last_step_runs_on_the_guest() {
    let mut guest = HostLedStretchState::new();
    guest.host_step(SEASIDE, SEASIDE_TALK_START, SEASIDE_TALK_START_MS);
    assert!(matches!(
        guest.host_step(SEASIDE, SEASIDE_LAST_STEP, SEASIDE_FIRST_END_MS + 60_000),
        Some(StretchChange::Ended(_, StretchEnd::LastStep))
    ));
    let paused = arrival(SEASIDE_PHASE, [0xa0, 0, 0, 0]);
    assert_eq!(
        end_follow_around_a_stretch(&paused, guest.place_of_end(SEASIDE_LAST_STEP)),
        EndFollow::Mirror
    );
}

#[test]
fn an_end_of_the_host_in_another_area_is_never_run_here_by_a_stretch() {
    let mut guest = HostLedStretchState::new();
    guest.host_step(SEASIDE, SEASIDE_TALK_START, SEASIDE_TALK_START_MS);
    let elsewhere = EndArrival {
        local_phase: Some(VILLAGE_PHASE),
        ..arrival(SEASIDE_PHASE, [0; 4])
    };
    for place in [StretchPlace::InsideIt, StretchPlace::ItsLastStep] {
        assert_eq!(
            end_follow_around_a_stretch(&elsewhere, place),
            EndFollow::Drop(EndDrop::OtherPhase)
        );
    }
}

#[test]
fn the_guest_leaves_the_seaside_talk_to_the_host_and_not_the_dream() {
    let seaside = stretch_the_guest_waits_to_start(SEASIDE, SEASIDE_TALK_START);
    let Some(seaside) = seaside else {
        panic!("the guest's own seaside talk start is not left to the host");
    };
    assert!(first_step_refused_line(seaside).contains(SEASIDE_TALK_START));
    assert_eq!(stretch_the_guest_waits_to_start(VILLAGE, DREAM_START), None);
    assert_eq!(
        stretch_the_guest_waits_to_start(SEASIDE, SEASIDE_FIRST_END),
        None
    );
}

#[test]
fn a_new_stretch_forgets_the_last_one_s_last_step() {
    let mut guest = HostLedStretchState::new();
    guest.host_step(VILLAGE, DREAM_START, DREAM_START_POP_MS);
    guest.host_step(VILLAGE, DREAM_END, DREAM_END_MS);
    guest.host_step(SEASIDE, SEASIDE_TALK_START, SEASIDE_TALK_START_MS);
    assert_eq!(guest.place_of_end(DREAM_END), StretchPlace::InsideIt);
    guest.end_with_the_session();
    assert_eq!(guest.place_of_end(DREAM_END), StretchPlace::NotInOne);
}
