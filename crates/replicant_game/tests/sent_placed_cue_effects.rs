use replicant_game::effects::{
    LiveEffects, MAX_LASTING_SENT_EFFECTS, PUPPET_EFFECT_MAX_LIFETIME_MS, SentPlacedCueEffect,
    SentPlacedCueEffects,
};
use replicant_proto::EnemyId;

const WHIRLWIND_EFFECT_ID: u32 = 213;

fn boss() -> EnemyId {
    EnemyId {
        group: 800,
        ordinal: 0,
    }
}

fn placed(handle: u32) -> SentPlacedCueEffect {
    SentPlacedCueEffect {
        handle,
        enemy: boss(),
        id: WHIRLWIND_EFFECT_ID,
    }
}

#[test]
fn a_placed_effect_is_reported_once_when_the_host_game_ends_it() {
    let mut sent = SentPlacedCueEffects::default();
    sent.note_sent(placed(1));
    sent.note_sent(placed(2));
    assert!(sent.take_ended(|_| false).is_empty());
    assert_eq!(sent.take_ended(|handle| handle == 2), vec![placed(2)]);
    assert!(sent.take_ended(|handle| handle == 2).is_empty());
    assert_eq!(sent.take_all(), vec![placed(1)]);
    assert!(sent.is_empty());
}

#[test]
fn the_oldest_placed_effect_is_handed_back_when_the_list_is_full() {
    let mut sent = SentPlacedCueEffects::default();
    for handle in 0..u32::try_from(MAX_LASTING_SENT_EFFECTS).unwrap() {
        assert_eq!(sent.note_sent(placed(handle)), None);
    }
    assert_eq!(sent.note_sent(placed(9999)), Some(placed(0)));
}

#[test]
fn a_replayed_placed_effect_ends_on_the_hosts_stop_or_by_the_time_limit() {
    let mut replayed = LiveEffects::new();
    replayed.track(100, 7, 0);
    replayed.track(101, 8, 0);
    assert_eq!(replayed.take_by_sender(7), Some(100));
    assert_eq!(replayed.take_by_sender(7), None);
    assert!(
        replayed
            .take_expired(PUPPET_EFFECT_MAX_LIFETIME_MS - 1)
            .is_empty()
    );
    assert_eq!(
        replayed.take_expired(PUPPET_EFFECT_MAX_LIFETIME_MS),
        vec![(101, 8)]
    );
}
