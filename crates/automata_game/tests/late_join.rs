use automata_game::late_join::{
    ARRIVAL_BESIDE_AFTER_BOTH_FREE_MS, JoinerTakeIn, LANDING_AFTER_THE_PLACES_DIFFER_MS,
    LandingReadings, LandingStep, LateLanding, TAKE_IN_WAIT_LOG_INTERVAL_MS, safe_moment_refusal,
    start_index_of_place,
};
use automata_game::load_gate::StoryPlace;
use automata_game::session_jump::{SessionJumpReadings, SessionJumpRefusal};
use automata_game::start::{ChapterRecord, PlayReadings};
use automata_game::story_follow::{
    FollowMachine, FollowReadings, RequestReadiness, STORY_PLACE_CATCH_UP_AFTER_MS, StoryRequest,
};
use automata_game::travel::JumpRole;

fn in_play() -> PlayReadings {
    PlayReadings {
        scene_state: 1,
        phase_id: 0x200,
        scene_loaded: 1,
        player_present: true,
    }
}

fn calm() -> SessionJumpReadings {
    SessionJumpReadings {
        play: in_play(),
        role: JumpRole::Solo,
        not_free_reason: None,
        defeat_flow_active: false,
        hero_is_down: false,
        load_under_way: false,
        jump_under_way: false,
    }
}

#[test]
fn a_calm_free_moment_is_safe_for_a_joiner_whatever_the_role() {
    for role in [JumpRole::Solo, JumpRole::Host, JumpRole::Guest] {
        let readings = SessionJumpReadings { role, ..calm() };
        assert_eq!(safe_moment_refusal(&readings), None);
    }
}

#[test]
fn a_load_a_scene_a_defeat_or_a_jump_is_no_moment_to_take_a_joiner() {
    let loading = SessionJumpReadings {
        play: PlayReadings {
            scene_state: 0,
            ..in_play()
        },
        ..calm()
    };
    assert_eq!(
        safe_moment_refusal(&loading),
        Some(SessionJumpRefusal::NotInPlay)
    );
    let in_a_cutscene = SessionJumpReadings {
        not_free_reason: Some("the scene input-off bit is set".to_owned()),
        ..calm()
    };
    assert!(matches!(
        safe_moment_refusal(&in_a_cutscene),
        Some(SessionJumpRefusal::PlayerNotFree(_))
    ));
    let defeated = SessionJumpReadings {
        defeat_flow_active: true,
        ..calm()
    };
    assert_eq!(
        safe_moment_refusal(&defeated),
        Some(SessionJumpRefusal::DefeatFlowActive)
    );
    let down = SessionJumpReadings {
        hero_is_down: true,
        ..calm()
    };
    assert_eq!(
        safe_moment_refusal(&down),
        Some(SessionJumpRefusal::HeroIsDown)
    );
    let jumping = SessionJumpReadings {
        jump_under_way: true,
        ..calm()
    };
    assert_eq!(
        safe_moment_refusal(&jumping),
        Some(SessionJumpRefusal::JumpUnderWay)
    );
    let held_load = SessionJumpReadings {
        load_under_way: true,
        ..calm()
    };
    assert_eq!(
        safe_moment_refusal(&held_load),
        Some(SessionJumpRefusal::LoadUnderWay)
    );
}

#[test]
fn a_joiner_who_arrives_is_taken_in_only_at_a_safe_moment_and_the_wait_is_logged() {
    let mut take_in = JoinerTakeIn::default();
    let waiting = take_in.step(0, true, Some("a load is under way".to_owned()));
    assert!(!waiting.take_in);
    assert_eq!(waiting.logs.len(), 1);
    let same = take_in.step(1_000, true, Some("a load is under way".to_owned()));
    assert!(same.logs.is_empty());
    let changed = take_in.step(2_000, true, Some("the player is not free".to_owned()));
    assert_eq!(changed.logs.len(), 1);
    let again = take_in.step(
        2_000 + TAKE_IN_WAIT_LOG_INTERVAL_MS,
        true,
        Some("the player is not free".to_owned()),
    );
    assert_eq!(again.logs.len(), 1);
    let done = take_in.step(20_000, true, None);
    assert!(done.take_in);
    assert_eq!(done.logs.len(), 1);
}

#[test]
fn nobody_waiting_means_nothing_to_take_in_and_nothing_logged() {
    let mut take_in = JoinerTakeIn::default();
    let step = take_in.step(0, false, Some("a load is under way".to_owned()));
    assert!(!step.take_in);
    assert!(step.logs.is_empty());
}

fn story_place(phase_id: i32, name: &str) -> StoryPlace {
    StoryPlace::new(phase_id, name)
}

fn landing(now_ms: u64, own: StoryPlace, host: StoryPlace) -> LandingReadings {
    LandingReadings {
        now_ms,
        applies: true,
        can_jump: true,
        own_place: own,
        host_place: host,
        host_in_play: true,
        safe_moment: true,
        host_free: true,
        host_body_present: true,
    }
}

fn armed() -> LateLanding {
    let mut late = LateLanding::default();
    late.arm();
    late
}

#[test]
fn a_game_that_was_not_armed_never_lands_or_walks() {
    let own = story_place(0x200, "Start");
    let host = story_place(0x300, "Later");
    let mut late = LateLanding::default();
    assert!(!late.is_armed());
    for now_ms in [0, 10_000, 20_000] {
        assert_eq!(
            late.step(&landing(now_ms, own, host)),
            LandingStep::default()
        );
    }
    let same = story_place(0x300, "Start");
    for now_ms in [30_000, 40_000] {
        assert_eq!(
            late.step(&landing(now_ms, same, host)),
            LandingStep::default()
        );
    }
}

#[test]
fn a_newcomer_in_another_chapter_lands_at_the_host_after_the_places_have_differed_a_while() {
    let own = story_place(0x200, "Start");
    let host = story_place(0x300, "Later");
    let mut late = armed();
    assert!(late.step(&landing(1_000, own, host)).land_at.is_none());
    let early = late.step(&landing(
        1_000 + LANDING_AFTER_THE_PLACES_DIFFER_MS - 1,
        own,
        host,
    ));
    assert!(early.land_at.is_none());
    let step = late.step(&landing(
        1_000 + LANDING_AFTER_THE_PLACES_DIFFER_MS,
        own,
        host,
    ));
    assert_eq!(step.land_at, Some(host));
}

#[test]
fn a_campaign_newcomer_never_jumps_and_waits_for_the_story_catch_up() {
    let own = story_place(0x200, "Start");
    let host = story_place(0x300, "Later");
    let mut late = armed();
    for now_ms in [0, 6_000, 12_000] {
        let step = late.step(&LandingReadings {
            can_jump: false,
            ..landing(now_ms, own, host)
        });
        assert!(step.land_at.is_none());
    }
    assert!(late.is_armed());
}

#[test]
fn a_newcomer_in_the_hosts_phase_walks_to_the_host_once_when_both_are_free() {
    let own = story_place(0x300, "Start");
    let host = story_place(0x300, "Later");
    let mut late = armed();
    assert!(!late.step(&landing(1_000, own, host)).go_to_partner);
    assert!(!late.step(&landing(1_500, own, host)).go_to_partner);
    let walk = late.step(&landing(
        1_500 + ARRIVAL_BESIDE_AFTER_BOTH_FREE_MS,
        own,
        host,
    ));
    assert!(walk.go_to_partner);
    assert!(!late.is_armed());
    for now_ms in [10_000, 20_000, 30_000] {
        assert_eq!(
            late.step(&landing(now_ms, own, host)),
            LandingStep::default()
        );
    }
}

#[test]
fn the_walk_waits_while_either_copy_is_not_free_or_the_hosts_body_is_missing() {
    let own = story_place(0x300, "Start");
    let host = story_place(0x300, "Later");
    let mut late = armed();
    late.step(&landing(0, own, host));
    let busy_here = LandingReadings {
        safe_moment: false,
        ..landing(5_000, own, host)
    };
    assert!(!late.step(&busy_here).go_to_partner);
    let busy_there = LandingReadings {
        host_free: false,
        ..landing(6_000, own, host)
    };
    assert!(!late.step(&busy_there).go_to_partner);
    let no_body = LandingReadings {
        host_body_present: false,
        ..landing(7_000, own, host)
    };
    assert!(!late.step(&no_body).go_to_partner);
    late.step(&landing(8_000, own, host));
    let walk = late.step(&landing(
        8_000 + ARRIVAL_BESIDE_AFTER_BOTH_FREE_MS,
        own,
        host,
    ));
    assert!(walk.go_to_partner);
}

#[test]
fn after_landing_the_newcomer_walks_beside_the_host_and_never_lands_twice() {
    let own = story_place(0x200, "Start");
    let host = story_place(0x300, "Later");
    let mut late = armed();
    late.step(&landing(0, own, host));
    let lands = late.step(&landing(LANDING_AFTER_THE_PLACES_DIFFER_MS, own, host));
    assert_eq!(lands.land_at, Some(host));
    let landed = story_place(0x300, "Chapter start");
    let t = LANDING_AFTER_THE_PLACES_DIFFER_MS + 1_000;
    late.step(&landing(t, landed, host));
    late.step(&landing(t + 1, landed, host));
    let walk = late.step(&landing(
        t + 1 + ARRIVAL_BESIDE_AFTER_BOTH_FREE_MS,
        landed,
        host,
    ));
    assert!(walk.go_to_partner);
    assert!(walk.land_at.is_none());
}

#[test]
fn no_landing_while_unsafe_unseen_or_in_a_system_phase_and_the_wait_restarts() {
    let own = story_place(0x200, "Start");
    let host = story_place(0x300, "Later");
    let mut late = armed();
    late.step(&landing(0, own, host));
    let unsafe_now = LandingReadings {
        safe_moment: false,
        ..landing(LANDING_AFTER_THE_PLACES_DIFFER_MS, own, host)
    };
    assert!(late.step(&unsafe_now).land_at.is_none());
    let host_away = LandingReadings {
        host_in_play: false,
        ..landing(10_000, own, host)
    };
    late.step(&host_away);
    let at_title = story_place(0xF00, "START");
    late.step(&landing(10_500, at_title, host));
    let restarted = late.step(&landing(10_600, own, host));
    assert!(restarted.land_at.is_none());
    let after = late.step(&landing(
        10_600 + LANDING_AFTER_THE_PLACES_DIFFER_MS,
        own,
        host,
    ));
    assert_eq!(after.land_at, Some(host));
}

#[test]
fn giving_up_ends_the_landing_for_good() {
    let own = story_place(0x200, "Start");
    let host = story_place(0x300, "Later");
    let mut late = armed();
    late.give_up();
    assert!(!late.is_armed());
    late.step(&landing(0, own, host));
    assert!(late.step(&landing(60_000, own, host)).land_at.is_none());
}

fn record(group: u8, phase_id: i16, name: &str) -> ChapterRecord {
    ChapterRecord {
        chapter: group,
        group,
        route: 0,
        hero_mask: 1,
        phase_id,
        phase_name: name.to_owned(),
    }
}

#[test]
fn the_hosts_place_names_the_one_chapter_start_of_its_phase() {
    let records = vec![
        record(1, 0x200, "First"),
        record(2, 0x300, "Second"),
        record(3, 0x400, "Third"),
    ];
    assert_eq!(
        start_index_of_place(&records, &story_place(0x300, "Somewhere Later")),
        Some(1)
    );
    assert_eq!(
        start_index_of_place(&records, &story_place(0x999, "Nowhere")),
        None
    );
}

#[test]
fn two_chapter_starts_in_one_phase_are_told_apart_by_name_or_not_at_all() {
    let records = vec![record(1, 0x200, "First"), record(2, 0x200, "Second")];
    assert_eq!(
        start_index_of_place(&records, &story_place(0x200, "Second")),
        Some(1)
    );
    assert_eq!(
        start_index_of_place(&records, &story_place(0x200, "Other")),
        None
    );
}

fn guest_readings(now_ms: u64, host: StoryPlace, own: StoryPlace, watches: bool) -> FollowReadings {
    FollowReadings {
        now_ms,
        role: JumpRole::Guest,
        partner_present: true,
        readiness: RequestReadiness {
            scene_state: 1,
            phase_system_state: 2,
            requested_phase_id: -1,
            pending_phase_id: 0x100,
            current_phase_id: 0x100,
            pending_name: Some(own.sub_phase()),
            current_name: Some(own.sub_phase()),
        },
        current_place: own,
        host_place: host,
        host_report_heard_at_ms: now_ms,
        host_in_play: true,
        host_waiting_at_load_end: false,
        this_copy_watches: watches,
        partner_watches: false,
        moves: Vec::new(),
        asks: Vec::new(),
        answers: Vec::new(),
    }
}

#[test]
fn a_guest_that_arrives_late_in_a_watched_stretch_catches_up_to_the_hosts_step() {
    let own = story_place(0x100, "00_30_A_RobotM_Prologue_STG1");
    let host = story_place(0x100, "00_60_A_RobotM_Prologue_STG1");
    let mut guest = FollowMachine::default();
    guest.step(&guest_readings(0, host, own, true));
    guest.step(&guest_readings(1_000, host, own, true));
    let step = guest.step(&guest_readings(
        1_000 + STORY_PLACE_CATCH_UP_AFTER_MS,
        host,
        own,
        true,
    ));
    assert!(matches!(step.make, Some(StoryRequest::SubPhase { .. })));
}
