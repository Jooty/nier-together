use replicant_game::effects::{
    CUBE_GLOW_REPLAY_EXPIRES_AFTER_MS, CUBE_GLOW_RESEND_INTERVAL_MS, ReplayedCubeGlow,
    ReplayedCubeGlows, SentCubeGlow, SentCubeGlows,
};
use replicant_proto::{AttachedEffect, EnemyId};

const HOST_CUBE: u64 = 0x7_8A10_0000;
const GUEST_COPY: u64 = 0x7_9B20_0000;
const AURA_RED_EFFECT_ID: u32 = 1301;

fn cube() -> EnemyId {
    EnemyId {
        group: 1001,
        ordinal: 7,
    }
}

fn red_glow(last_sent_ms: u64) -> SentCubeGlow {
    SentCubeGlow {
        actor: HOST_CUBE,
        enemy: cube(),
        id: AURA_RED_EFFECT_ID,
        attached: AttachedEffect {
            bone: -1,
            unique: true,
            colour: [1.0; 4],
            style: [1.0; 4],
            mode: 0,
            flags: 0,
            extra: 0,
        },
        last_sent_ms,
    }
}

fn replayed(refreshed_ms: u64) -> ReplayedCubeGlow {
    ReplayedCubeGlow {
        actor: GUEST_COPY,
        enemy: cube(),
        id: AURA_RED_EFFECT_ID,
        bone: -1,
        refreshed_ms,
    }
}

#[test]
fn an_attached_glow_is_sent_again_every_two_seconds_until_it_is_removed() {
    let mut sent = SentCubeGlows::default();
    sent.note_attached(red_glow(1000));
    assert!(
        sent.take_due_resends(1000 + CUBE_GLOW_RESEND_INTERVAL_MS - 1)
            .is_empty()
    );
    assert_eq!(
        sent.take_due_resends(1000 + CUBE_GLOW_RESEND_INTERVAL_MS)
            .len(),
        1
    );
    assert!(
        sent.take_due_resends(1000 + CUBE_GLOW_RESEND_INTERVAL_MS + 1)
            .is_empty()
    );
    assert_eq!(
        sent.take_removed(HOST_CUBE, AURA_RED_EFFECT_ID, -1),
        Some(cube())
    );
    assert!(sent.take_due_resends(100_000).is_empty());
}

#[test]
fn a_glow_whose_first_send_was_refused_is_resent_at_once() {
    let mut sent = SentCubeGlows::default();
    sent.note_attached(red_glow(0));
    assert_eq!(sent.take_due_resends(CUBE_GLOW_RESEND_INTERVAL_MS).len(), 1);
}

#[test]
fn attaching_the_same_glow_twice_keeps_one() {
    let mut sent = SentCubeGlows::default();
    sent.note_attached(red_glow(0));
    sent.note_attached(red_glow(500));
    assert_eq!(sent.len(), 1);
}

#[test]
fn a_cube_that_died_has_its_glow_taken_for_a_removal() {
    let mut sent = SentCubeGlows::default();
    sent.note_attached(red_glow(0));
    let gone = sent.take_gone(|_, _| false);
    assert_eq!(gone.len(), 1);
    assert!(sent.is_empty());
}

#[test]
fn a_resend_refreshes_the_glow_the_guest_already_has() {
    let mut have = ReplayedCubeGlows::default();
    assert!(!have.refresh(GUEST_COPY, AURA_RED_EFFECT_ID, -1, 0));
    have.note_attached(replayed(0));
    assert!(have.refresh(GUEST_COPY, AURA_RED_EFFECT_ID, -1, 2000));
    assert!(
        have.take_expired(2000 + CUBE_GLOW_REPLAY_EXPIRES_AFTER_MS - 1)
            .is_empty()
    );
}

#[test]
fn a_glow_nobody_refreshes_expires_and_a_removal_forgets_it() {
    let mut have = ReplayedCubeGlows::default();
    have.note_attached(replayed(0));
    assert_eq!(
        have.take_expired(CUBE_GLOW_REPLAY_EXPIRES_AFTER_MS).len(),
        1
    );
    have.note_attached(replayed(0));
    assert!(have.take_removed(GUEST_COPY, AURA_RED_EFFECT_ID, -1));
    assert!(have.is_empty());
}

#[test]
fn a_glow_on_a_copy_that_is_replaced_is_taken_gone() {
    let mut have = ReplayedCubeGlows::default();
    have.note_attached(replayed(0));
    assert_eq!(have.take_gone(|_, _| false).len(), 1);
}
