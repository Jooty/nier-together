use replicant_game::enemies::{
    FIGHT_ACTOR_PROBE_INTERVAL_MS, FIGHT_ACTOR_PROBE_NEW_KEY_GAP_MS, FightActorProbe, ProbedActor,
    ProbedFight, probed_fight, raw_group_count_line,
};

fn actor(group: u16, class_id: i32, address: u64) -> ProbedActor {
    ProbedActor {
        group,
        actor: address,
        class_id,
        kind_mask: 0x4,
        model_slots_filled: 1,
        position: [1.0, 2.0, 3.0],
    }
}

#[test]
fn only_fight_7_and_fight_11_bosses_start_the_probe() {
    assert_eq!(probed_fight(100, 0x51), Some(ProbedFight::Seven));
    assert_eq!(probed_fight(100, 0x50), None);
    assert_eq!(probed_fight(800, 0x20), Some(ProbedFight::Eleven));
    assert_eq!(probed_fight(901, 0x20), Some(ProbedFight::Eleven));
    assert_eq!(probed_fight(1001, 0x52), None);
}

#[test]
fn the_dump_groups_actors_by_kind_and_counts_them() {
    let mut probe = FightActorProbe::default();
    let actors = [
        actor(100, 0x51, 0x10),
        actor(1001, 0x52, 0x20),
        actor(1001, 0x52, 0x30),
    ];
    let lines = probe.observe("host", 0, &actors);
    assert_eq!(lines.len(), 3);
    assert!(lines[0].contains("3 live actors in 2 kinds"));
    assert!(lines[2].contains("group 1001 class 0x52") && lines[2].contains("count 2"));
}

#[test]
fn a_second_dump_waits_five_seconds_unless_a_new_kind_appears_after_one() {
    let mut probe = FightActorProbe::default();
    let mut actors = vec![actor(100, 0x51, 0x10)];
    assert!(!probe.observe("host", 0, &actors).is_empty());
    assert!(probe.observe("host", 500, &actors).is_empty());
    actors.push(actor(1001, 0x52, 0x20));
    assert!(
        probe
            .observe("host", FIGHT_ACTOR_PROBE_NEW_KEY_GAP_MS - 1, &actors)
            .is_empty()
    );
    assert!(
        !probe
            .observe("host", FIGHT_ACTOR_PROBE_NEW_KEY_GAP_MS, &actors)
            .is_empty()
    );
    assert!(
        probe
            .observe("host", FIGHT_ACTOR_PROBE_NEW_KEY_GAP_MS + 1000, &actors)
            .is_empty()
    );
    assert!(
        !probe
            .observe(
                "host",
                FIGHT_ACTOR_PROBE_NEW_KEY_GAP_MS + FIGHT_ACTOR_PROBE_INTERVAL_MS,
                &actors
            )
            .is_empty()
    );
}

#[test]
fn the_raw_count_line_shows_the_manager_count_beside_the_scanned_count() {
    let mut counts = vec![0_i32; 0x400];
    counts[100] = 1;
    counts[1001] = 300;
    let actors = [
        actor(100, 0x51, 0x10),
        actor(1001, 0x52, 0x20),
        actor(1001, 0x52, 0x30),
    ];
    let line = raw_group_count_line("host", &counts, &actors);
    assert!(line.contains("100: manager 1, live scanned 1;"));
    assert!(line.contains("1001: manager 300, live scanned 2 (scan cap hit)"));
}
