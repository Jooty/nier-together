use replicant_game::stay_together::PARTNER_TIMEOUT_MS;
use replicant_game::talk_guard::{
    IMPORTANT_TALK_BASES, IMPORTANT_TALK_IDS, PartnerSideView, PressAsk, PressVerdict,
    SIDE_VIEW_STATE_RESENT_EVERY_MS, SideViewReporter, TalkRegistry, important_base_of_talk_id,
    is_important_talk, judge_press, partner_inside, talk_needs_the_partner_in_the_house,
};
use replicant_game::zone_rules::{Companion, Placement, ZoneRadius};
use replicant_proto::SideViewState;

const IMPORTANT_START: &str = "EID_0100_a0038g_Start";
const ORDINARY_START: &str = "EID_0100_zzzz_Start";
const PHASE: u16 = 3;
const LOCAL: Placement = Placement {
    phase: PHASE,
    position: [0.0, 0.0, 0.0],
};
const ACTOR: i32 = 7;

fn partner_at(x: f32, phase: u16, age_ms: u64) -> Companion {
    Companion {
        name: "Sam".to_owned(),
        placement: Some(Placement {
            phase,
            position: [x, 0.0, 0.0],
        }),
        age_ms,
    }
}

fn verdict_with(connected: bool, game_open: bool, partner: Option<&Companion>) -> PressVerdict {
    judge_press(&PressAsk {
        connected,
        partner_game_open: game_open,
        local: Some(LOCAL),
        partner,
        radius: ZoneRadius::default(),
        talk_id: None,
        partner_side_view: None,
    })
}

#[test]
fn the_generated_table_holds_the_next_day_cutscene() {
    assert!(IMPORTANT_TALK_BASES.contains(&"EID_0100_a0038g"));
    assert!(is_important_talk(IMPORTANT_START));
}

#[test]
fn the_generated_table_is_sorted_and_free_of_duplicates() {
    assert!(
        IMPORTANT_TALK_BASES
            .windows(2)
            .all(|pair| pair[0] < pair[1])
    );
}

#[test]
fn a_talk_that_is_not_listed_is_not_important() {
    assert!(!is_important_talk(ORDINARY_START));
}

#[test]
fn an_important_press_with_a_near_partner_is_allowed() {
    let near = partner_at(5.0, PHASE, 0);
    assert_eq!(verdict_with(true, true, Some(&near)), PressVerdict::Allow);
}

#[test]
fn an_important_press_with_a_far_partner_waits() {
    let far = partner_at(100.0, PHASE, 0);
    let PressVerdict::Wait {
        partner,
        distance_units,
    } = verdict_with(true, true, Some(&far))
    else {
        panic!("expected a wait");
    };
    assert_eq!(partner, "Sam");
    assert!((distance_units - 100.0).abs() < 0.01);
}

#[test]
fn the_wait_text_names_the_partner() {
    let far = partner_at(100.0, PHASE, 0);
    let text = verdict_with(true, true, Some(&far)).prompt_text();
    assert!(text.is_some_and(|text| text.contains("Sam")));
}

#[test]
fn a_partner_in_another_phase_never_makes_the_press_wait() {
    let elsewhere = partner_at(100.0, PHASE + 1, 0);
    assert_eq!(
        verdict_with(true, true, Some(&elsewhere)),
        PressVerdict::Allow
    );
}

#[test]
fn an_offline_closed_or_silent_partner_never_makes_the_press_wait() {
    let far = partner_at(100.0, PHASE, 0);
    assert_eq!(verdict_with(false, true, Some(&far)), PressVerdict::Allow);
    assert_eq!(verdict_with(true, false, Some(&far)), PressVerdict::Allow);
    let silent = partner_at(100.0, PHASE, PARTNER_TIMEOUT_MS + 1);
    assert_eq!(verdict_with(true, true, Some(&silent)), PressVerdict::Allow);
    assert_eq!(verdict_with(true, true, None), PressVerdict::Allow);
}

#[test]
fn a_registered_talk_is_found_by_its_actor() {
    let mut registry = TalkRegistry::default();
    registry.enter_phase("A_CENTER_VILLAGE_01");
    registry.register_name(ACTOR, 1, IMPORTANT_START);
    assert_eq!(registry.important_name_of(ACTOR), Some(IMPORTANT_START));
    assert_eq!(registry.names_of(ACTOR), [None, Some(IMPORTANT_START)]);
    assert_eq!(registry.important_name_of(ACTOR + 1), None);
}

#[test]
fn a_talk_that_is_not_listed_is_found_but_not_important() {
    let mut registry = TalkRegistry::default();
    registry.register_name(ACTOR, 0, ORDINARY_START);
    assert_eq!(registry.names_of(ACTOR), [Some(ORDINARY_START), None]);
    assert_eq!(registry.important_name_of(ACTOR), None);
}

#[test]
fn registering_again_overwrites_that_mode_only() {
    let mut registry = TalkRegistry::default();
    registry.register_name(ACTOR, 0, ORDINARY_START);
    registry.register_name(ACTOR, 1, ORDINARY_START);
    registry.register_name(ACTOR, 0, IMPORTANT_START);
    assert_eq!(
        registry.names_of(ACTOR),
        [Some(IMPORTANT_START), Some(ORDINARY_START)]
    );
}

#[test]
fn an_empty_name_or_an_unknown_mode_registers_nothing() {
    let mut registry = TalkRegistry::default();
    registry.register_name(ACTOR, 0, "");
    registry.register_name(ACTOR, 2, IMPORTANT_START);
    registry.register_name(ACTOR, -1, IMPORTANT_START);
    assert_eq!(registry.names_of(ACTOR), [None, None]);
}

#[test]
fn a_new_phase_clears_the_registry() {
    let mut registry = TalkRegistry::default();
    registry.enter_phase("A_CENTER_VILLAGE_01");
    registry.register_name(ACTOR, 0, IMPORTANT_START);
    registry.enter_phase("A_CENTER_VILLAGE_01");
    assert!(registry.important_name_of(ACTOR).is_some());
    registry.enter_phase("A_CENTER_LIBRARY_01");
    assert_eq!(registry.important_name_of(ACTOR), None);
}

#[test]
fn a_press_is_logged_once_per_actor_until_the_phase_changes() {
    let mut registry = TalkRegistry::default();
    registry.enter_phase("A_CENTER_VILLAGE_01");
    assert!(registry.first_press(ACTOR));
    assert!(!registry.first_press(ACTOR));
    assert!(registry.first_press(ACTOR + 1));
    registry.enter_phase("A_CENTER_LIBRARY_01");
    assert!(registry.first_press(ACTOR));
}

#[test]
fn the_press_line_names_both_modes_and_the_talk_id() {
    let mut registry = TalkRegistry::default();
    registry.register_name(ACTOR, 0, IMPORTANT_START);
    registry.register_talk_id(ACTOR, 1, 42);
    assert_eq!(
        registry.press_line(ACTOR),
        "talk press: actor 7 mode0 EID_0100_a0038g_Start mode1 none talk id 42"
    );
    assert_eq!(
        registry.press_line(ACTOR + 1),
        "talk press: actor 8 mode0 none mode1 none talk id none"
    );
}

#[test]
fn a_numbered_talk_maps_to_its_important_base() {
    assert_eq!(important_base_of_talk_id(1031), Some("EID_0100_a0038g"));
    assert_eq!(important_base_of_talk_id(1014), Some("EID_0100_a0020"));
}

#[test]
fn a_talk_id_that_is_not_listed_is_not_important() {
    assert_eq!(important_base_of_talk_id(-5), None);
    assert_eq!(important_base_of_talk_id(1032), None);
}

#[test]
fn the_generated_id_table_is_sorted_and_free_of_duplicates() {
    assert!(IMPORTANT_TALK_IDS.windows(2).all(|pair| pair[0] < pair[1]));
}

#[test]
fn a_registered_talk_id_is_refused_when_the_partner_is_far() {
    let mut registry = TalkRegistry::default();
    registry.register_talk_id(ACTOR, 0, 1031);
    assert_eq!(
        registry.important_talk_id_of(ACTOR),
        Some((1031, "EID_0100_a0038g"))
    );
    let far = partner_at(500.0, PHASE, 0);
    assert!(matches!(
        verdict_with(true, true, Some(&far)),
        PressVerdict::Wait { .. }
    ));
}

#[test]
fn a_registered_talk_id_that_is_not_listed_passes() {
    let mut registry = TalkRegistry::default();
    registry.register_talk_id(ACTOR, 0, 1032);
    assert_eq!(registry.important_talk_id_of(ACTOR), None);
}

#[test]
fn run_1116a_the_villager_talk_that_leads_into_the_home_dream_waits_for_a_partner_outside_the_house()
 {
    const VILLAGER: i32 = 1002;
    const VILLAGE_PHASE: u16 = 1;
    const HOST_UPSTAIRS_AT_THE_PRESS: [f32; 3] = [31.0, 18.0, 11.6];
    const GUEST_OUTSIDE_AT_THE_PRESS: [f32; 3] = [98.9, 27.0, -10.4];
    const HOUSE_LOWER_FLOOR: [f32; 3] = [31.0, 14.0, 11.1];
    // Run 1116a host 08:29:25.143: actor 1002 pressed with talk 1321 registered (1322 after a
    // refusal); its chain ends in EID_1500_a0005_020_Select, the choice into the dream. Guest
    // 08:29:23.898: the guest at (98.9, 27.0, -10.4), both in phase 1; host 08:29:18.891 on the lower floor.
    for talk_id in [1321, 1322] {
        let mut registry = TalkRegistry::default();
        registry.register_talk_id(VILLAGER, 0, talk_id);
        assert_eq!(
            registry.important_talk_id_of(VILLAGER),
            Some((talk_id, "EID_1500_a0005_020"))
        );
    }
    let judge = |position: [f32; 3], in_side_view: bool| {
        let guest = Companion {
            name: "Sam".to_owned(),
            placement: Some(Placement {
                phase: VILLAGE_PHASE,
                position,
            }),
            age_ms: 0,
        };
        judge_press(&PressAsk {
            connected: true,
            partner_game_open: true,
            local: Some(Placement {
                phase: VILLAGE_PHASE,
                position: HOST_UPSTAIRS_AT_THE_PRESS,
            }),
            partner: Some(&guest),
            radius: ZoneRadius::default(),
            talk_id: Some(1321),
            partner_side_view: Some(PartnerSideView {
                phase: VILLAGE_PHASE,
                in_side_view,
                age_ms: 0,
            }),
        })
    };
    let PressVerdict::Wait { distance_units, .. } = judge(GUEST_OUTSIDE_AT_THE_PRESS, false) else {
        panic!("the press must wait for the guest outside the house");
    };
    assert!((distance_units - 71.9).abs() < 0.1);
    assert_eq!(judge(HOUSE_LOWER_FLOOR, true), PressVerdict::Allow);
}

const VILLAGE_PHASE: u16 = 1;
const HOST_UPSTAIRS_AT_THE_PRESS: [f32; 3] = [31.0, 18.0, 11.6];
const GUEST_ON_THE_DOORSTEP: [f32; 3] = [30.4, 14.0, -0.8];
const GUEST_ON_THE_LOWER_FLOOR: [f32; 3] = [31.0, 14.0, 11.1];
const GUEST_BY_THE_STAIRS: [f32; 3] = [30.9, 14.6, 18.9];

fn judge_house_talk(
    talk_id: i32,
    position: [f32; 3],
    side_view: Option<PartnerSideView>,
) -> PressVerdict {
    let guest = Companion {
        name: "Sam".to_owned(),
        placement: Some(Placement {
            phase: VILLAGE_PHASE,
            position,
        }),
        age_ms: 0,
    };
    judge_press(&PressAsk {
        connected: true,
        partner_game_open: true,
        local: Some(Placement {
            phase: VILLAGE_PHASE,
            position: HOST_UPSTAIRS_AT_THE_PRESS,
        }),
        partner: Some(&guest),
        radius: ZoneRadius::default(),
        talk_id: Some(talk_id),
        partner_side_view: side_view,
    })
}

fn side_view(in_side_view: bool) -> Option<PartnerSideView> {
    Some(PartnerSideView {
        phase: VILLAGE_PHASE,
        in_side_view,
        age_ms: 0,
    })
}

#[test]
fn run_1116a_a_guest_on_the_doorstep_is_outside_the_house_and_the_dream_talk_waits() {
    // Run 1116a guest 08:27:30.275 at (30.4, 14.0, -0.8) with its side view off: 13.0 units from
    // the host's press spot, inside the 15-unit zone. 08:27:32.688 GM_IN_Door_of_NierHouse_02
    // fired and 08:27:32.705 "view: side view on here (by GM_IN_Door_of_NierHouse_02)".
    for talk_id in [1321, 1322] {
        let PressVerdict::WaitForTheHouse {
            partner,
            distance_units,
        } = judge_house_talk(talk_id, GUEST_ON_THE_DOORSTEP, side_view(false))
        else {
            panic!("talk {talk_id} must wait for a guest on the doorstep");
        };
        assert_eq!(partner, "Sam");
        assert!((distance_units - 13.0).abs() < 0.1);
        assert_eq!(
            judge_house_talk(talk_id, GUEST_ON_THE_LOWER_FLOOR, side_view(true)),
            PressVerdict::Allow
        );
        assert_eq!(
            judge_house_talk(talk_id, GUEST_BY_THE_STAIRS, side_view(true)),
            PressVerdict::Allow
        );
    }
}

#[test]
fn the_house_talk_waits_without_a_fresh_side_view_report_from_this_phase() {
    assert!(matches!(
        judge_house_talk(1321, GUEST_ON_THE_LOWER_FLOOR, None),
        PressVerdict::WaitForTheHouse { .. }
    ));
    let stale = PartnerSideView {
        phase: VILLAGE_PHASE,
        in_side_view: true,
        age_ms: PARTNER_TIMEOUT_MS + 1,
    };
    assert!(!partner_inside(VILLAGE_PHASE, Some(stale)));
    let elsewhere = PartnerSideView {
        phase: VILLAGE_PHASE + 1,
        in_side_view: true,
        age_ms: 0,
    };
    assert!(!partner_inside(VILLAGE_PHASE, Some(elsewhere)));
}

#[test]
fn the_house_wait_text_says_the_partner_is_outside() {
    let text = judge_house_talk(1321, GUEST_ON_THE_DOORSTEP, side_view(false)).prompt_text();
    assert!(text.is_some_and(|text| text.contains("inside the house") && text.contains("Sam")));
}

#[test]
fn every_other_important_talk_keeps_the_zone_radius_alone() {
    assert!(talk_needs_the_partner_in_the_house(1321));
    assert!(talk_needs_the_partner_in_the_house(1322));
    assert!(!talk_needs_the_partner_in_the_house(1031));
    assert_eq!(
        judge_house_talk(1031, GUEST_ON_THE_DOORSTEP, side_view(false)),
        PressVerdict::Allow
    );
    assert_eq!(
        judge_house_talk(1031, GUEST_ON_THE_DOORSTEP, None),
        PressVerdict::Allow
    );
}

#[test]
fn the_side_view_is_reported_on_a_change_and_again_once_a_second() {
    let inside = SideViewState {
        phase: VILLAGE_PHASE,
        in_side_view: true,
        side_view_door: 1,
    };
    let outside = SideViewState {
        in_side_view: false,
        side_view_door: 0,
        ..inside
    };
    let mut reporter = SideViewReporter::new();
    assert!(reporter.report_due(outside, 0));
    assert!(!reporter.report_due(outside, 10));
    assert!(reporter.report_due(inside, 20));
    assert!(!reporter.report_due(inside, 20 + SIDE_VIEW_STATE_RESENT_EVERY_MS - 1));
    assert!(reporter.report_due(inside, 20 + SIDE_VIEW_STATE_RESENT_EVERY_MS));
    reporter.forget();
    assert!(reporter.report_due(inside, 30 + SIDE_VIEW_STATE_RESENT_EVERY_MS));
}
