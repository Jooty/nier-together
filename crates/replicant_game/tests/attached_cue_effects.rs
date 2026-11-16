use replicant_game::effects::{
    AttachedCueEffect, AttachedCueEffects, MAX_ATTACHED_CUE_EFFECTS,
    REMOVE_ATTACHED_ANY_BONE_AT_OR_BELOW,
};
use replicant_proto::EnemyId;

const HOST_EMIL: u64 = 0x7_8A10_0000;
const EMIL_MAGIC_ARRAY_EFFECT_ID: u32 = 458;
const EMIL_CANE_EFFECT_ID: u32 = 459;

fn emil() -> EnemyId {
    EnemyId {
        group: 0xFFFF,
        ordinal: 0xFFFE,
    }
}

fn magic_array_on(bone: i16) -> AttachedCueEffect {
    AttachedCueEffect {
        actor: HOST_EMIL,
        enemy: emil(),
        id: EMIL_MAGIC_ARRAY_EFFECT_ID,
        bone,
    }
}

#[test]
fn a_removal_of_a_sent_attach_is_sent_wherever_the_game_removes_it() {
    // Emil's 0xB30 removes 458 with bone -1 (0x14017B8E0); before the fix only a removal seen
    // inside the AI step crossed, now any removal of an attach that crossed does.
    let mut sent = AttachedCueEffects::default();
    sent.note_attached(magic_array_on(-1));
    assert_eq!(sent.take_removed(HOST_EMIL, EMIL_CANE_EFFECT_ID, -1), None);
    assert_eq!(
        sent.take_removed(HOST_EMIL, EMIL_MAGIC_ARRAY_EFFECT_ID, -1),
        Some(emil())
    );
    assert_eq!(
        sent.take_removed(HOST_EMIL, EMIL_MAGIC_ARRAY_EFFECT_ID, -1),
        None
    );
}

#[test]
fn a_removal_never_sent_as_an_attach_is_not_sent() {
    let mut sent = AttachedCueEffects::default();
    assert_eq!(
        sent.take_removed(HOST_EMIL, EMIL_MAGIC_ARRAY_EFFECT_ID, -1),
        None
    );
    sent.note_attached(magic_array_on(3));
    assert_eq!(
        sent.take_removed(HOST_EMIL, EMIL_MAGIC_ARRAY_EFFECT_ID, 4),
        None
    );
    assert_eq!(
        sent.take_removed(0x1234, EMIL_MAGIC_ARRAY_EFFECT_ID, 3),
        None
    );
}

#[test]
fn a_removal_for_any_bone_takes_every_bone_of_that_effect() {
    let mut sent = AttachedCueEffects::default();
    sent.note_attached(magic_array_on(1));
    sent.note_attached(magic_array_on(2));
    assert_eq!(
        sent.take_removed(
            HOST_EMIL,
            EMIL_MAGIC_ARRAY_EFFECT_ID,
            REMOVE_ATTACHED_ANY_BONE_AT_OR_BELOW
        ),
        Some(emil())
    );
    assert!(sent.is_empty());
}

#[test]
fn an_actor_that_is_destroyed_or_no_longer_shared_hands_back_its_attaches() {
    // L34 run1 host 08:35:11.833: "the variant companion is no longer sent", so Emil's actor no
    // longer stands for the companion's id and the magic array's removal must be sent.
    let mut sent = AttachedCueEffects::default();
    sent.note_attached(magic_array_on(-1));
    assert!(sent.take_gone(|_, _| true).is_empty());
    let gone = sent.take_gone(|_, _| false);
    assert_eq!(gone, vec![magic_array_on(-1)]);
    assert!(sent.is_empty());
}

#[test]
fn the_same_attach_twice_is_kept_once_and_a_full_list_drops_the_oldest() {
    let mut sent = AttachedCueEffects::default();
    sent.note_attached(magic_array_on(-1));
    sent.note_attached(magic_array_on(-1));
    for bone in 0..MAX_ATTACHED_CUE_EFFECTS as i16 {
        sent.note_attached(magic_array_on(bone));
    }
    assert_eq!(
        sent.take_removed(HOST_EMIL, EMIL_MAGIC_ARRAY_EFFECT_ID, -1),
        None
    );
    assert_eq!(
        sent.take_removed(HOST_EMIL, EMIL_MAGIC_ARRAY_EFFECT_ID, 0),
        Some(emil())
    );
}
