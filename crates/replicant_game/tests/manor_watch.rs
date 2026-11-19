use replicant_game::callback_gate::{
    CallbackGate, EndArrival, EndFollow, GateRole, HOST_STEPS_THAT_PLACE_OR_MOVE_A_STORY_ACTOR,
    PartnerFire, control_slots_free, end_follow_around_a_stretch,
};
use replicant_game::host_led_stretch::{
    HostLedStretchState, StretchChange, StretchEnd, StretchPlace, started_from_the_story_line,
    stretch_the_story_is_inside,
};
use replicant_game::npcs::{NpcRoster, NpcSighting};
use replicant_proto::NpcId;

const FIRST_PACK: &str = "A_SOUTH_MANSION_01";
const LAST_PACK: &str = "A_SOUTH_MANSION_02";
const FIELD_PACK: &str = "A_SOUTH_FIELD_01";
const MANOR_PHASE: u16 = 20;
const ARMING_STEP: &str = "ScriptMode_to_Kaine_03_04";
const DOOR_STEP: &str = "Open_Door_01";
const CUTSCENE_START: &str = "EID_1100_a0080_01_Start";
const CUTSCENE_END: &str = "EID_1100_a0080_01_End";
const ROOM_ACTORS_END: &str = "EID_1100_a0064_01_End";
const JOIN_END: &str = "EID_1100_a0070_01_End";
const STORY_ON_FLAG: u16 = 504;
const STORY_OFF_FLAG: u16 = 511;
const ARMED_MS: u64 = 2_588_778;
const STARTED_MS: u64 = 2_634_686;
const JOIN_END_MS: u64 = 3_348_339;
const CUTSCENE_START_MS: u64 = 3_802_809;
const CUTSCENE_END_MS: u64 = 3_820_407;
const FIELD_ENTRY_MS: u64 = 4_086_833;
const GUEST_SLOTS_WHILE_WATCHING: [u32; 4] = [0x80, 0, 0, 0];
const GATE_PUT_STEP: &str = "EID_2000_a0026g_02_Start";
const GATE_WALK_STEP: &str = "EID_2000_a0026g_03_Start";
const GATE_FIRST_STEP: &str = "EID_2000_a0026g_Start";
const GATE_PUT_MS: u64 = 4_579_407;
const COMPANION_GROUP: u16 = 30;
const COMPANION_CLASS: u16 = 4;
const BUTLER_GROUP: u16 = 31;
const BUTLER_CLASS: u16 = 133;

fn started_state() -> HostLedStretchState {
    let mut state = HostLedStretchState::new();
    state.host_step(FIRST_PACK, ARMING_STEP, ARMED_MS);
    state.host_step(FIRST_PACK, DOOR_STEP, STARTED_MS);
    state
}

fn flags(on: &[u16]) -> impl Fn(u16) -> bool + '_ {
    move |id| on.contains(&id)
}

fn guest_arrival(phase: u16) -> EndArrival {
    EndArrival {
        message_phase: phase,
        shared_phase: Some(phase),
        local_phase: Some(phase),
        local_in_own_talk: false,
        control_mask_settled: control_slots_free(GUEST_SLOTS_WHILE_WATCHING, false),
        already_ran_here: false,
        own_copy_of_same_event_running: false,
    }
}

fn sighting(actor: u64, group: u16, class: u16, serial: i32) -> NpcSighting {
    NpcSighting {
        actor,
        spawn_serial: serial,
        group,
        class,
        position: [-10.1, 0.0, -28.7],
        yaw_radians: 0.0,
        motion: None,
        visible: true,
        event: None,
    }
}

#[test]
fn the_manor_watch_ends_when_the_fights_cutscene_starts_not_at_its_end() {
    let mut state = started_state();
    assert_eq!(state.check(Some(LAST_PACK), CUTSCENE_START_MS), None);
    let change = state.host_step(LAST_PACK, CUTSCENE_START, CUTSCENE_START_MS);
    let Some(StretchChange::Ended(stretch, StretchEnd::LastStep)) = change else {
        panic!("the stretch did not end at the cutscene's start: {change:?}");
    };
    assert!(stretch.the_guest_plays_from_its_last_step);
    assert_eq!(state.current(), None);
    assert_eq!(
        state.host_step(LAST_PACK, CUTSCENE_END, CUTSCENE_END_MS),
        None
    );
}

#[test]
fn a_load_or_a_jump_inside_the_manor_stretch_starts_it_from_the_story_flags() {
    for pack in [FIRST_PACK, LAST_PACK] {
        let mut state = HostLedStretchState::new();
        let change =
            state.start_from_the_story(pack, flags(&[STORY_ON_FLAG]), CUTSCENE_START_MS - 1);
        assert!(
            matches!(change, Some(StretchChange::StartedFromTheStory(stretch)) if stretch.pack == FIRST_PACK),
            "{pack}: {change:?}"
        );
        assert_eq!(state.door_ban().map(|s| s.first_step), Some(DOOR_STEP));
    }
}

#[test]
fn the_story_flags_outside_the_stretch_or_its_packs_start_nothing() {
    assert!(stretch_the_story_is_inside(FIRST_PACK, flags(&[])).is_none());
    assert!(
        stretch_the_story_is_inside(LAST_PACK, flags(&[STORY_ON_FLAG, STORY_OFF_FLAG])).is_none()
    );
    assert!(stretch_the_story_is_inside(FIELD_PACK, flags(&[STORY_ON_FLAG])).is_none());
    assert!(stretch_the_story_is_inside("A_SEASIDE_TOWN_01", flags(&[STORY_ON_FLAG])).is_none());
}

#[test]
fn the_story_flags_never_restart_the_stretch_its_cutscene_start_just_ended() {
    let mut state = started_state();
    state.host_step(LAST_PACK, CUTSCENE_START, CUTSCENE_START_MS);
    assert_eq!(
        state.start_from_the_story(LAST_PACK, flags(&[STORY_ON_FLAG]), CUTSCENE_START_MS + 16),
        None
    );
    assert_eq!(state.check(Some(FIELD_PACK), FIELD_ENTRY_MS), None);
    assert!(matches!(
        state.start_from_the_story(LAST_PACK, flags(&[STORY_ON_FLAG]), FIELD_ENTRY_MS + 1),
        Some(StretchChange::StartedFromTheStory(_))
    ));
}

#[test]
fn a_running_stretch_is_not_started_again_by_the_story_flags() {
    let mut state = started_state();
    assert_eq!(
        state.start_from_the_story(FIRST_PACK, flags(&[STORY_ON_FLAG]), JOIN_END_MS),
        None
    );
}

#[test]
fn inside_the_manor_stretch_the_hosts_ends_run_on_the_watching_guest_at_once() {
    let state = started_state();
    for end in [ROOM_ACTORS_END, JOIN_END] {
        let place = state.place_of_end(end);
        assert_eq!(place, StretchPlace::InsideItTheGuestIsNotTakenAlong);
        assert_eq!(
            end_follow_around_a_stretch(&guest_arrival(MANOR_PHASE), place),
            EndFollow::Mirror
        );
    }
}

#[test]
fn the_guest_that_missed_the_hosts_first_spawn_numbers_the_respawn_as_the_host_does() {
    let mut host = NpcRoster::new();
    let mut guest = NpcRoster::new();
    host.observe(
        MANOR_PHASE,
        &[sighting(0x100, COMPANION_GROUP, COMPANION_CLASS, 7)],
    );
    host.observe(MANOR_PHASE, &[]);
    let host_respawn = host.observe(
        MANOR_PHASE,
        &[
            sighting(0x200, COMPANION_GROUP, COMPANION_CLASS, 12),
            sighting(0x300, BUTLER_GROUP, BUTLER_CLASS, 13),
        ],
    );
    let guest_spawn = guest.observe(
        MANOR_PHASE,
        &[
            sighting(0x900, COMPANION_GROUP, COMPANION_CLASS, 4),
            sighting(0x950, BUTLER_GROUP, BUTLER_CLASS, 5),
        ],
    );
    let companion = NpcId {
        group: COMPANION_GROUP,
        class: COMPANION_CLASS,
        ordinal: 0,
    };
    assert!(host_respawn.arrived.contains(&companion));
    assert_eq!(host_respawn.arrived, guest_spawn.arrived);
}

#[test]
fn two_live_npcs_of_one_model_keep_counting_while_one_of_them_is_left() {
    let mut roster = NpcRoster::new();
    roster.observe(
        MANOR_PHASE,
        &[
            sighting(0x100, BUTLER_GROUP, BUTLER_CLASS, 1),
            sighting(0x200, BUTLER_GROUP, BUTLER_CLASS, 2),
        ],
    );
    roster.observe(
        MANOR_PHASE,
        &[sighting(0x200, BUTLER_GROUP, BUTLER_CLASS, 2)],
    );
    let update = roster.observe(
        MANOR_PHASE,
        &[
            sighting(0x200, BUTLER_GROUP, BUTLER_CLASS, 2),
            sighting(0x300, BUTLER_GROUP, BUTLER_CLASS, 3),
        ],
    );
    assert_eq!(
        update.arrived,
        vec![NpcId {
            group: BUTLER_GROUP,
            class: BUTLER_CLASS,
            ordinal: 2,
        }]
    );
}

#[test]
fn a_far_guest_follows_the_hosts_steps_that_place_and_walk_an_actor_at_the_gate() {
    let mut gate = CallbackGate::new();
    gate.note_partner_here(false);
    for step in [GATE_PUT_STEP, GATE_WALK_STEP] {
        assert!(HOST_STEPS_THAT_PLACE_OR_MOVE_A_STORY_ACTOR.contains(&step));
        assert_eq!(
            gate.partner_fire(GateRole::Guest, step, GATE_PUT_MS),
            PartnerFire::FollowHostFromAfar
        );
    }
    assert_eq!(
        gate.partner_fire(GateRole::Guest, GATE_FIRST_STEP, GATE_PUT_MS),
        PartnerFire::Noted
    );
}

#[test]
fn a_near_guest_follows_the_gate_steps_as_any_other_step() {
    let mut gate = CallbackGate::new();
    gate.note_partner_here(true);
    assert_eq!(
        gate.partner_fire(GateRole::Guest, GATE_PUT_STEP, GATE_PUT_MS),
        PartnerFire::FollowHost
    );
}

#[test]
fn the_story_start_line_names_the_flags_and_steps_only() {
    let stretch = stretch_the_story_is_inside(LAST_PACK, flags(&[STORY_ON_FLAG])).expect("inside");
    let line = started_from_the_story_line(stretch, false, LAST_PACK);
    assert!(line.contains("g504 on and g511 off"), "{line}");
    assert!(line.contains(CUTSCENE_START), "{line}");
    assert!(line.contains(LAST_PACK), "{line}");
}
