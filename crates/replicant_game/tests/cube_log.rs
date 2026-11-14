use replicant_game::enemies::{
    CUBE_GROUP, CubeStateLog, cube_effect_attached_text, cube_state_changed_text, is_cube_effect,
};
use replicant_proto::EnemyId;

fn cube(ordinal: u16) -> EnemyId {
    EnemyId {
        group: CUBE_GROUP,
        ordinal,
    }
}

#[test]
fn a_cube_logs_only_when_its_motion_changes() {
    let mut log = CubeStateLog::default();
    assert!(log.observe("host", [(cube(3), 0x10, Some(4))]).is_empty());
    assert!(log.observe("host", [(cube(3), 0x10, Some(4))]).is_empty());
    assert_eq!(
        log.observe("host", [(cube(3), 0x10, Some(6))]),
        vec![cube_state_changed_text(
            "host",
            cube(3),
            0x10,
            Some(4),
            Some(6)
        )]
    );
}

#[test]
fn only_cubes_are_logged() {
    let mut log = CubeStateLog::default();
    let boss = EnemyId {
        group: 100,
        ordinal: 0,
    };
    log.observe("guest", [(boss, 0x20, Some(1))]);
    assert!(log.observe("guest", [(boss, 0x20, Some(2))]).is_empty());
}

#[test]
fn the_lines_name_the_cube_the_motion_and_the_attached_aura() {
    assert_eq!(
        cube_state_changed_text("guest", cube(7), 0xAB, None, Some(3)),
        "cube probe (guest): 1001:7 (actor 0xab) motion none -> 3"
    );
    assert!(is_cube_effect("efE_shahri_auraBlue"));
    assert!(!is_cube_effect("efP_underAura01"));
    assert_eq!(
        cube_effect_attached_text("efE_shahri_auraRed", 0xAB),
        "cube probe: efE_shahri_auraRed attached to actor 0xab by the game's own state"
    );
}
