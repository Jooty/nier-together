use automata_game::late_join::{
    BOSS_GAUGE_READ_LEN, BOSS_SCAN_EVERY_MS, BossFight, BossFightConditions, BossGaugeKind,
    BossGaugeObject, BossReading, BossWait, ENTITY_HEADER_READ_LEN, JoinerNotice, TargetReading,
    WaitingJoiner, boss_reading_of, boss_wait_line, conditions_of, entity_body_pointer,
    handle_slot_index, slot_holds_handle,
};

const SHOWN: u32 = 0x4000_0000;
const TARGET: u32 = 0x1_2345;

fn gauge(state: u32, flags: u32) -> BossGaugeObject {
    BossGaugeObject {
        kind: BossGaugeKind::PerEnemy,
        state,
        flags,
        handles: [TARGET, 0],
        second_handle_word: 0,
    }
}

fn names(list: &[&str]) -> Vec<WaitingJoiner> {
    (1..)
        .zip(list)
        .map(|(slot, name)| WaitingJoiner {
            slot,
            name: (*name).to_owned(),
        })
        .collect()
}

fn fight() -> BossReading {
    BossReading::Fight(BossFight {
        kind: BossGaugeKind::PerEnemy,
        target_handle: TARGET,
    })
}

#[test]
fn each_of_the_four_conditions_alone_does_not_hold_a_joiner() {
    let none = BossFightConditions {
        state_is_alive: false,
        is_shown: false,
        target_resolves: false,
        target_has_health: false,
    };
    assert!(!none.all_hold());
    let ones = [
        BossFightConditions {
            state_is_alive: true,
            ..none
        },
        BossFightConditions {
            is_shown: true,
            ..none
        },
        BossFightConditions {
            target_resolves: true,
            ..none
        },
        BossFightConditions {
            target_has_health: true,
            ..none
        },
    ];
    for alone in ones {
        assert!(!alone.all_hold(), "{alone:?}");
    }
}

#[test]
fn each_single_condition_read_from_a_gauge_leaves_the_fight_unheld() {
    let alive = [TargetReading::Health(500)];
    let dead = [TargetReading::Health(0)];
    let alone = [
        (gauge(2, 0), vec![]),
        (gauge(0, SHOWN), vec![]),
        (gauge(0, 0), dead.to_vec()),
        (gauge(0, 0), alive.to_vec()),
        (gauge(4, SHOWN), alive.to_vec()),
        (gauge(2, 0), alive.to_vec()),
        (gauge(2, SHOWN), dead.to_vec()),
        (gauge(2, SHOWN), vec![TargetReading::Unresolved]),
    ];
    for (object, targets) in alone {
        assert_eq!(boss_reading_of(&[(object, targets)]), BossReading::NoBoss);
    }
}

#[test]
fn all_four_together_hold_a_joiner() {
    let object = gauge(2, SHOWN);
    let targets = vec![TargetReading::Health(1)];
    assert!(conditions_of(&object, &targets).all_hold());
    assert_eq!(boss_reading_of(&[(object, targets)]), fight());
}

#[test]
fn every_alive_state_from_one_to_three_holds_and_no_other() {
    for state in 0..8 {
        let held = boss_reading_of(&[(gauge(state, SHOWN), vec![TargetReading::Health(9)])]);
        assert_eq!(held == fight(), (1..=3).contains(&state), "state {state}");
    }
}

#[test]
fn a_dead_target_releases_a_joiner() {
    let mut wait = BossWait::default();
    let joiner = names(&["Latecomer"]);
    wait.step(0, &joiner, fight());
    assert!(wait.holding());
    let dead = boss_reading_of(&[(gauge(2, SHOWN), vec![TargetReading::Health(0)])]);
    wait.step(BOSS_SCAN_EVERY_MS, &joiner, dead);
    assert!(!wait.holding());
}

#[test]
fn an_unreadable_link_releases_and_is_logged_once() {
    let mut wait = BossWait::default();
    let joiner = names(&["Latecomer"]);
    wait.step(0, &joiner, fight());
    let unreadable = boss_reading_of(&[(gauge(2, SHOWN), vec![TargetReading::Unreadable])]);
    assert!(matches!(unreadable, BossReading::Unreadable(_)));
    let first = wait.step(BOSS_SCAN_EVERY_MS, &joiner, unreadable.clone());
    assert!(!wait.holding());
    assert!(
        first
            .logs
            .iter()
            .any(|line| line.contains("could not be read"))
    );
    let again = wait.step(2 * BOSS_SCAN_EVERY_MS, &joiner, unreadable);
    assert!(again.logs.is_empty());
}

#[test]
fn a_registry_that_cannot_be_read_never_holds_a_joiner() {
    let mut wait = BossWait::default();
    wait.step(
        0,
        &names(&["Latecomer"]),
        BossReading::Unreadable("the game keeps no UI registry".to_owned()),
    );
    assert!(!wait.holding());
}

#[test]
fn the_wait_line_is_posted_once_per_joiner_and_per_fight() {
    let mut wait = BossWait::default();
    let joiner = names(&["Latecomer"]);
    let first = wait.step(0, &joiner, fight());
    assert_eq!(first.in_game_lines, vec![boss_wait_line("Latecomer")]);
    assert_eq!(first.logs.len(), 1);
    for tick in 1..10 {
        let step = wait.step(tick * BOSS_SCAN_EVERY_MS, &joiner, fight());
        assert!(step.in_game_lines.is_empty());
        assert!(step.logs.is_empty());
    }
    let second_joiner = names(&["Latecomer", "Second"]);
    let step = wait.step(20 * BOSS_SCAN_EVERY_MS, &second_joiner, fight());
    assert_eq!(step.in_game_lines, vec![boss_wait_line("Second")]);
}

#[test]
fn the_wait_line_names_the_joiner_and_no_boss() {
    let line = boss_wait_line("Latecomer");
    assert!(line.contains("Latecomer"));
    assert!(line.contains("boss fight"));
}

#[test]
fn a_new_fight_after_the_end_of_one_posts_the_line_again() {
    let mut wait = BossWait::default();
    let joiner = names(&["Latecomer"]);
    wait.step(0, &joiner, fight());
    let over = wait.step(BOSS_SCAN_EVERY_MS, &joiner, BossReading::NoBoss);
    assert_eq!(over.logs.len(), 1);
    let again = wait.step(2 * BOSS_SCAN_EVERY_MS, &joiner, fight());
    assert_eq!(again.in_game_lines.len(), 1);
}

#[test]
fn nobody_waiting_forgets_the_hold() {
    let mut wait = BossWait::default();
    wait.step(0, &names(&["Latecomer"]), fight());
    wait.step(10, &[], BossReading::NoBoss);
    assert!(!wait.holding());
    assert!(wait.scan_due(10));
}

#[test]
fn the_registry_is_scanned_at_most_every_quarter_second() {
    let mut wait = BossWait::default();
    assert!(wait.scan_due(0));
    wait.step(1_000, &names(&["Latecomer"]), BossReading::NoBoss);
    assert!(!wait.scan_due(1_000 + BOSS_SCAN_EVERY_MS - 1));
    assert!(wait.scan_due(1_000 + BOSS_SCAN_EVERY_MS));
}

#[test]
fn a_gauge_object_is_read_from_its_bytes() {
    let mut bytes = vec![0_u8; BOSS_GAUGE_READ_LEN];
    bytes[0x20..0x24].copy_from_slice(&2_u32.to_le_bytes());
    bytes[0x58..0x5C].copy_from_slice(&SHOWN.to_le_bytes());
    bytes[0x90..0x94].copy_from_slice(&TARGET.to_le_bytes());
    bytes[0x94..0x98].copy_from_slice(&0x2_0000_u32.to_le_bytes());
    let single = BossGaugeObject::from_bytes(BossGaugeKind::PerEnemy, &bytes).unwrap();
    assert!(single.state_is_alive() && single.is_shown());
    assert_eq!(single.target_handles(), vec![TARGET]);
    bytes[0xAC..0xB0].copy_from_slice(&1_u32.to_le_bytes());
    let hud = BossGaugeObject::from_bytes(BossGaugeKind::HudManager, &bytes).unwrap();
    assert_eq!(hud.target_handles(), vec![TARGET, 0x2_0000]);
    assert!(BossGaugeObject::from_bytes(BossGaugeKind::HudManager, &bytes[..8]).is_none());
}

#[test]
fn a_handle_resolves_only_to_a_slot_of_its_generation() {
    assert_eq!(handle_slot_index(0x0012_3456), 0x1234);
    assert!(slot_holds_handle(0x0012_3456, 0x0012_3400));
    assert!(!slot_holds_handle(0x0012_3456, 0x0013_3456));
}

#[test]
fn an_entity_that_is_dying_has_no_body_to_read() {
    let mut header = vec![0_u8; ENTITY_HEADER_READ_LEN];
    header[0x48..0x50].copy_from_slice(&0x1000_u64.to_le_bytes());
    assert_eq!(entity_body_pointer(&header), Some(0x1000));
    header[0x2C] = 1;
    assert_eq!(entity_body_pointer(&header), None);
    header[0x2C] = 0;
    header[0x48..0x50].fill(0);
    assert_eq!(entity_body_pointer(&header), None);
}

#[test]
fn the_joiner_is_told_when_its_wait_starts_and_when_it_ends_each_once() {
    let mut wait = BossWait::default();
    let joiner = names(&["Latecomer"]);
    let start = JoinerNotice {
        slot: 1,
        waiting: true,
    };
    assert_eq!(wait.step(0, &joiner, fight()).notices, vec![start]);
    assert!(
        wait.step(BOSS_SCAN_EVERY_MS, &joiner, fight())
            .notices
            .is_empty()
    );
    let end = wait.step(2 * BOSS_SCAN_EVERY_MS, &joiner, BossReading::NoBoss);
    assert_eq!(
        end.notices,
        vec![JoinerNotice {
            slot: 1,
            waiting: false
        }]
    );
    let after = wait.step(3 * BOSS_SCAN_EVERY_MS, &joiner, BossReading::NoBoss);
    assert!(after.notices.is_empty());
}

#[test]
fn an_unreadable_reading_ends_the_joiners_wait_notice_too() {
    let mut wait = BossWait::default();
    let joiner = names(&["Latecomer"]);
    wait.step(0, &joiner, fight());
    let step = wait.step(
        BOSS_SCAN_EVERY_MS,
        &joiner,
        BossReading::Unreadable("a target could not be read".to_owned()),
    );
    assert_eq!(step.notices.len(), 1);
    assert!(!step.notices[0].waiting);
}
