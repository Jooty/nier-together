use replicant_game::enemies::{
    CUBE_GROUP, CubeStateLog, cube_effect_attached_text, cube_glow_is_relayed,
    cube_state_changed_text, is_cube_effect,
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

#[test]
fn the_host_relays_both_glow_colours_from_the_run_but_not_the_smoke() {
    let run_attaches = [
        ("efE_shahri_auraRed", 0x8197_b640_u64),
        ("efE_shahri_auraRed", 0xe467_1a60),
        ("efE_shahri_auraBlue", 0xe40c_0230),
        ("efE_shahri_auraBlue", 0xe3eb_3bc0),
        ("efE_shahri_smoke01", 0xe489_f420),
    ];
    let relayed: Vec<bool> = run_attaches
        .iter()
        .map(|(name, actor)| cube_glow_is_relayed(*actor, name, true))
        .collect();
    assert_eq!(relayed, vec![true, true, true, true, false]);
}

#[test]
fn the_guest_and_a_spawn_outside_an_attach_relay_no_glow() {
    assert!(!cube_glow_is_relayed(
        0x8197_b640,
        "efE_shahri_auraRed",
        false
    ));
    assert!(!cube_glow_is_relayed(0, "efE_shahri_auraRed", true));
}
