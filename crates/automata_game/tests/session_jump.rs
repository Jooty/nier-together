use automata_game::character::Character;
use automata_game::session_jump::{
    SessionJumpAction, SessionJumpCharacters, SessionJumpMachine, SessionJumpOrder,
    SessionJumpOrigin, SessionJumpReadings, SessionJumpRefusal, decide_session_jump,
    record_of_start_index,
};
use automata_game::start::{ArrivalReadings, ChapterRecord, PlayReadings, StartCall};
use automata_game::travel::JumpRole;

fn record(chapter: u8, group: u8, phase_id: i16, name: &str) -> ChapterRecord {
    ChapterRecord {
        chapter,
        group,
        route: 0,
        hero_mask: 1,
        phase_id,
        phase_name: name.to_owned(),
    }
}

fn table() -> Vec<ChapterRecord> {
    vec![
        record(0, 1, 0x100, "opening_Prologue"),
        record(1, 2, 0x200, "first_start"),
        record(2, 3, 0x300, "second_start"),
    ]
}

fn characters() -> SessionJumpCharacters {
    SessionJumpCharacters {
        hero: Character::TwoB,
        partner: Character::NineS,
        route_lead: Character::TwoB,
    }
}

fn readings() -> SessionJumpReadings {
    SessionJumpReadings {
        play: PlayReadings {
            scene_state: 1,
            phase_id: 0x200,
            scene_loaded: 1,
            player_present: true,
        },
        role: JumpRole::Host,
        not_free_reason: None,
        defeat_flow_active: false,
        hero_is_down: false,
        load_under_way: false,
        jump_under_way: false,
    }
}

fn arrival(scene_state: i32, phase_id: i32, held_by: Option<&str>) -> ArrivalReadings {
    ArrivalReadings {
        scene_state,
        phase_id,
        phase_name: "made_up".to_owned(),
        scene_loaded: 1,
        player_present: true,
        player_held_by: held_by.map(str::to_owned),
    }
}

fn begun_machine() -> SessionJumpMachine {
    let mut machine = SessionJumpMachine::default();
    let chosen = record_of_start_index(&table(), 0).expect("index 0 exists");
    machine.begin(&chosen, characters(), SessionJumpOrigin::ThisPlayer);
    machine
}

fn machine_accepted_at(now_ms: u64) -> SessionJumpMachine {
    let mut machine = begun_machine();
    assert!(matches!(
        machine.step(0, None).action,
        SessionJumpAction::RunPlan(_)
    ));
    assert!(matches!(
        machine.step(10, None).action,
        SessionJumpAction::TryPhaseStart(_)
    ));
    machine.phase_start_returned(now_ms, true);
    machine
}

#[test]
fn a_start_index_picks_the_record_the_launch_start_picks() {
    let chosen = record_of_start_index(&table(), 0).unwrap();
    assert_eq!(chosen.phase_id, 0x200);
    assert_eq!(record_of_start_index(&table(), 1).unwrap().phase_id, 0x300);
    assert_eq!(record_of_start_index(&table(), 2).unwrap().phase_id, 0x100);
    assert!(record_of_start_index(&table(), 99).is_err());
}

#[test]
fn a_jump_in_a_free_scene_is_allowed_for_a_solo_player_and_the_host() {
    for role in [JumpRole::Solo, JumpRole::Host] {
        let ready = SessionJumpReadings { role, ..readings() };
        assert_eq!(
            decide_session_jump(SessionJumpOrigin::ThisPlayer, &ready),
            Ok(())
        );
    }
}

#[test]
fn only_the_host_asks_and_only_the_guest_follows() {
    let guest = SessionJumpReadings {
        role: JumpRole::Guest,
        ..readings()
    };
    assert_eq!(
        decide_session_jump(SessionJumpOrigin::ThisPlayer, &guest),
        Err(SessionJumpRefusal::OnlyTheHostAsks)
    );
    assert_eq!(
        decide_session_jump(SessionJumpOrigin::TheHostsOrder, &guest),
        Ok(())
    );
    assert_eq!(
        decide_session_jump(SessionJumpOrigin::TheHostsOrder, &readings()),
        Err(SessionJumpRefusal::OnlyTheGuestFollows)
    );
}

#[test]
fn every_unsafe_moment_refuses_the_jump_with_its_reason() {
    let cases = [
        (
            SessionJumpReadings {
                jump_under_way: true,
                ..readings()
            },
            SessionJumpRefusal::JumpUnderWay,
        ),
        (
            SessionJumpReadings {
                load_under_way: true,
                ..readings()
            },
            SessionJumpRefusal::LoadUnderWay,
        ),
        (
            SessionJumpReadings {
                play: PlayReadings {
                    scene_state: 3,
                    ..readings().play
                },
                ..readings()
            },
            SessionJumpRefusal::NotInPlay,
        ),
        (
            SessionJumpReadings {
                defeat_flow_active: true,
                ..readings()
            },
            SessionJumpRefusal::DefeatFlowActive,
        ),
        (
            SessionJumpReadings {
                hero_is_down: true,
                ..readings()
            },
            SessionJumpRefusal::HeroIsDown,
        ),
        (
            SessionJumpReadings {
                not_free_reason: Some("a cutscene".to_owned()),
                ..readings()
            },
            SessionJumpRefusal::PlayerNotFree("a cutscene".to_owned()),
        ),
    ];
    for (blocked, refusal) in cases {
        assert_eq!(
            decide_session_jump(SessionJumpOrigin::ThisPlayer, &blocked),
            Err(refusal)
        );
    }
}

#[test]
fn the_title_is_not_a_place_to_jump_from() {
    let at_title = SessionJumpReadings {
        play: PlayReadings {
            phase_id: 0xF00,
            ..readings().play
        },
        ..readings()
    };
    assert_eq!(
        decide_session_jump(SessionJumpOrigin::ThisPlayer, &at_title),
        Err(SessionJumpRefusal::NotInPlay)
    );
}

#[test]
fn an_idle_machine_does_nothing_and_a_begun_one_runs_the_plan_once_then_the_phase_start() {
    let mut machine = SessionJumpMachine::default();
    assert!(!machine.is_active());
    assert_eq!(machine.step(0, None).action, SessionJumpAction::Nothing);
    let mut machine = begun_machine();
    assert!(machine.is_active());
    let SessionJumpAction::RunPlan(plan) = machine.step(0, None).action else {
        panic!("the first step runs the plan");
    };
    assert!(
        plan.before_phase_start
            .contains(&StartCall::WriteHeroModelId(Character::TwoB.object_id()))
    );
    assert_eq!(plan.phase_start.phase_id, 0x200);
    for now_ms in [10, 20, 30] {
        let SessionJumpAction::TryPhaseStart(call) = machine.step(now_ms, None).action else {
            panic!("the phase start is retried every step");
        };
        assert_eq!(call.phase_id, 0x200);
    }
}

#[test]
fn the_phase_start_is_the_only_call_made_and_only_while_it_is_being_started() {
    let mut machine = machine_accepted_at(20);
    let waiting = arrival(1, 0x200, None);
    for now_ms in [30, 40] {
        let action = machine.step(now_ms, Some(&waiting)).action;
        assert_eq!(action, SessionJumpAction::Nothing);
    }
}

#[test]
fn a_phase_start_that_keeps_returning_zero_for_twenty_seconds_is_abandoned() {
    let mut machine = begun_machine();
    machine.step(0, None);
    assert!(matches!(
        machine.step(10, None).action,
        SessionJumpAction::TryPhaseStart(_)
    ));
    machine.phase_start_returned(15, false);
    let step = machine.step(20_011, None);
    assert_eq!(step.action, SessionJumpAction::Nothing);
    assert!(step.logs.iter().any(|line| line.contains("abandoned")));
    assert!(!machine.is_active());
}

#[test]
fn an_accepted_phase_start_waits_for_arrival_readings() {
    let mut machine = machine_accepted_at(20);
    assert!(machine.wants_arrival_readings());
    let step = machine.step(30, None);
    assert!(!step.landed);
    assert!(machine.is_active());
}

#[test]
fn landing_needs_the_scene_to_have_changed_even_into_the_phase_already_running() {
    let mut machine = machine_accepted_at(20);
    let before_the_change = arrival(1, 0x200, None);
    assert!(!machine.step(30, Some(&before_the_change)).landed);
    let loading = arrival(3, 0x200, Some("not in play"));
    assert!(!machine.step(40, Some(&loading)).landed);
    let step = machine.step(50, Some(&before_the_change));
    assert!(step.landed);
    assert!(step.logs.iter().any(|line| line.contains("landed")));
    assert!(!machine.is_active());
}

#[test]
fn a_new_phase_lands_once_the_player_is_free_however_long_a_cutscene_runs() {
    let mut machine = machine_accepted_at(20);
    machine.step(25, Some(&arrival(3, 0x200, Some("not in play"))));
    assert!(
        !machine
            .step(30, Some(&arrival(1, 0x200, Some("a cutscene"))))
            .landed
    );
    assert!(machine.is_active());
    let step = machine.step(1_000_000, Some(&arrival(1, 0x200, Some("a cutscene"))));
    assert!(!step.landed);
    assert!(machine.is_active());
    assert!(
        machine
            .step(1_000_010, Some(&arrival(1, 0x200, None)))
            .landed
    );
}

#[test]
fn an_arrival_that_never_comes_is_abandoned_after_five_minutes() {
    let mut machine = machine_accepted_at(20);
    let step = machine.step(300_021, Some(&arrival(3, 0x777, Some("not in play"))));
    assert!(step.logs.iter().any(|line| line.contains("abandoned")));
    assert!(!machine.is_active());
}

#[test]
fn a_second_jump_runs_after_the_first_landed_and_is_numbered() {
    let mut machine = machine_accepted_at(20);
    machine.step(30, Some(&arrival(3, 0x200, Some("not in play"))));
    assert!(machine.step(40, Some(&arrival(1, 0x200, None))).landed);
    let chosen = record_of_start_index(&table(), 1).unwrap();
    let logs = machine.begin(&chosen, characters(), SessionJumpOrigin::TheHostsOrder);
    assert_eq!(machine.number(), 2);
    assert!(logs[0].contains("#2"));
    assert!(logs[0].contains("the host's order"));
}

#[test]
fn a_jump_cannot_begin_while_another_is_under_way() {
    let mut machine = begun_machine();
    let chosen = record_of_start_index(&table(), 1).unwrap();
    let logs = machine.begin(&chosen, characters(), SessionJumpOrigin::ThisPlayer);
    assert!(logs[0].contains("still under way"));
    assert_eq!(machine.number(), 1);
}

#[test]
fn a_failed_call_abandons_the_jump_with_its_reason() {
    let mut machine = begun_machine();
    let logs = machine.call_failed("a memory write failed");
    assert!(logs[0].contains("a memory write failed"));
    assert!(!machine.is_active());
}

#[test]
fn the_begin_line_says_no_restore_point_is_made() {
    let mut machine = SessionJumpMachine::default();
    let chosen = record_of_start_index(&table(), 0).unwrap();
    let logs = machine.begin(&chosen, characters(), SessionJumpOrigin::ThisPlayer);
    assert!(logs[0].contains("session jump #1"));
    assert!(logs[0].contains("no restore point"));
}

#[test]
fn an_order_names_a_start_index_inside_the_table_only() {
    assert!(SessionJumpOrder::of_start_index(41).is_some());
    assert!(SessionJumpOrder::of_start_index(1024).is_none());
    assert!(SessionJumpOrder::of_start_index(70_000).is_none());
}
