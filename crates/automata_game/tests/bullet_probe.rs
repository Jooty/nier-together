use automata_game::address::BULLET_SHOT_SETUP_FUNCTION;
use automata_game::bullet_probe::{BulletShot, PLAYER_BODY_OBJECT_ID};
use automata_game::enemy_lead::BOSS_CLASS_HEALTH_FROM;
use automata_game::exe::ExeVariant;

fn shot_by(owner_object: u32, owner_max_health: i32) -> BulletShot {
    BulletShot {
        owner_object,
        owner_handle: 0x0100_0100,
        owner_health: owner_max_health,
        owner_max_health,
        shot_kind: 9,
        behavior_kind: 8,
        attack_group: 0x7C_0000,
        attack_index: 0x2EE,
        position: [-451.29, -59.52, 853.82],
    }
}

#[test]
fn the_shot_setup_function_is_the_one_found_in_both_exes() {
    assert_eq!(BULLET_SHOT_SETUP_FUNCTION.rva(ExeVariant::Main), 0x60F310);
    assert_eq!(BULLET_SHOT_SETUP_FUNCTION.rva(ExeVariant::Compat), 0x6082D0);
}

#[test]
fn an_owner_with_boss_health_counts_as_boss_class() {
    assert!(shot_by(0x2_1100, BOSS_CLASS_HEALTH_FROM).is_by_a_boss_class_owner());
    assert!(!shot_by(0x2_1100, BOSS_CLASS_HEALTH_FROM - 1).is_by_a_boss_class_owner());
}

#[test]
fn a_player_body_never_counts_as_boss_class() {
    assert!(!shot_by(PLAYER_BODY_OBJECT_ID, 90_000).is_by_a_boss_class_owner());
}

#[test]
fn the_line_names_the_owner_the_kinds_and_the_place() {
    assert_eq!(
        shot_by(0x2_1100, 43_846).line(),
        "bullets: spawn by object 0x21100 (handle 0x1000100, health 43846/43846): shot kind 9, behavior kind 8, attack 0x7C0000/0x2EE, at -451.29 -59.52 853.82"
    );
}
