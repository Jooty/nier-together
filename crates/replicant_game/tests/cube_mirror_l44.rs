use std::collections::HashSet;

use replicant_game::enemies::{
    CUBE_FIGHT_SPAWN_FILL_IN_GRACE_MS, CUBE_GROUP, CUBE_MIRROR_DELAY_MS,
    CUBE_MIRROR_EXTRAPOLATION_MS, ENEMY_MIRROR_DELAY_MS, EnemyMirror, FIGHT_SPAWN_FILL_IN_GRACE_MS,
    FightSpawnFillIn, fill_in_grace_ms,
};
use replicant_proto::{EnemyId, EnemySnapshot, EnemyState};

const PHASE: u16 = 6;
const BODY_GROUP: u16 = 100;

fn cube(ordinal: u16) -> EnemyId {
    EnemyId {
        group: CUBE_GROUP,
        ordinal,
    }
}

fn state_at(id: EnemyId, x: f32) -> EnemyState {
    EnemyState {
        id,
        position: [x, 0.0, 0.0],
        yaw_radians: 0.0,
        motion: None,
        hp: 60,
        max_hp: 60,
    }
}

fn snapshot(tick: u32, enemies: Vec<EnemyState>) -> EnemySnapshot {
    let round = u16::try_from(enemies.len()).unwrap();
    EnemySnapshot {
        tick,
        phase: PHASE,
        part: 0,
        parts: 1,
        total: round,
        round,
        enemies,
    }
}

#[test]
fn cubes_113_to_119_of_the_l44_fight_no_longer_wait_three_seconds() {
    let waiting: Vec<EnemyId> = (113..120).map(cube).collect();
    let copyable: HashSet<EnemyId> = waiting.iter().copied().collect();
    let none = HashSet::new();
    let mut fill_in = FightSpawnFillIn::default();
    let first = fill_in.observe(43_026, &waiting, &none, &none, &copyable);
    assert_eq!(first.newly_waiting.len(), 7);
    assert!(first.due.is_empty());
    let due = fill_in.observe(
        43_026 + CUBE_FIGHT_SPAWN_FILL_IN_GRACE_MS,
        &waiting,
        &none,
        &none,
        &copyable,
    );
    assert_eq!(due.due.len(), 7);
    assert!(
        due.due
            .iter()
            .all(|(_, waited)| *waited <= CUBE_FIGHT_SPAWN_FILL_IN_GRACE_MS)
    );
}

#[test]
fn only_cubes_get_the_short_grace() {
    assert_eq!(
        fill_in_grace_ms(CUBE_GROUP),
        CUBE_FIGHT_SPAWN_FILL_IN_GRACE_MS
    );
    assert_eq!(fill_in_grace_ms(BODY_GROUP), FIGHT_SPAWN_FILL_IN_GRACE_MS);
    assert_eq!(fill_in_grace_ms(6), FIGHT_SPAWN_FILL_IN_GRACE_MS);
}

#[test]
fn a_cube_is_drawn_closer_to_the_host_than_the_body() {
    const { assert!(CUBE_MIRROR_DELAY_MS < ENEMY_MIRROR_DELAY_MS) };
    let mut mirror = EnemyMirror::new();
    let body = EnemyId {
        group: BODY_GROUP,
        ordinal: 0,
    };
    for (tick, at_ms) in [(1_u32, 1000_u64), (2, 1033)] {
        let x = (tick - 1) as f32 * 3.3;
        mirror.receive_snapshot(
            at_ms,
            &snapshot(tick, vec![state_at(cube(0), x), state_at(body, x)]),
        );
    }
    let samples = mirror.sample_with_cubes(
        1033 + CUBE_MIRROR_DELAY_MS,
        ENEMY_MIRROR_DELAY_MS,
        CUBE_MIRROR_DELAY_MS,
        CUBE_MIRROR_EXTRAPOLATION_MS,
    );
    let at = |id: EnemyId| {
        samples
            .iter()
            .find(|enemy| enemy.id == id)
            .unwrap()
            .position[0]
    };
    assert!(
        (at(cube(0)) - 3.3).abs() < 1e-3,
        "cube is at the newest state"
    );
    assert!(at(body) < 3.3, "body is still behind");
}

#[test]
fn a_cube_whose_next_state_is_late_keeps_moving_a_little_then_stops() {
    let mut mirror = EnemyMirror::new();
    mirror.receive_snapshot(1000, &snapshot(1, vec![state_at(cube(0), 0.0)]));
    mirror.receive_snapshot(1033, &snapshot(2, vec![state_at(cube(0), 3.3)]));
    let x_at = |now_ms: u64| {
        mirror
            .sample_with_cubes(
                now_ms,
                ENEMY_MIRROR_DELAY_MS,
                CUBE_MIRROR_DELAY_MS,
                CUBE_MIRROR_EXTRAPOLATION_MS,
            )
            .first()
            .unwrap()
            .position[0]
    };
    let late = x_at(1033 + CUBE_MIRROR_DELAY_MS + 33);
    assert!(
        (late - 6.6).abs() < 1e-3,
        "one more interval of movement: {late}"
    );
    let much_later = x_at(1033 + CUBE_MIRROR_DELAY_MS + 5_000);
    assert!(
        (much_later - 13.3).abs() < 1e-2,
        "stopped at the limit: {much_later}"
    );
}
