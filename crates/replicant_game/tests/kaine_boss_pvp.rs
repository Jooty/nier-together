use replicant_game::kaine::{
    BOSS_FIGHT_GROUP, BOSS_HIT_DEFAULT_PERCENT_OF_NIER_MAX_HP, BossLifeFollow, BossLifeStep,
    BossNierHits, NATIVE_HIT_WINDOW_MS, PVP_DAMAGE_PERCENT, SENT_HIT_REPEAT_WINDOW_MS,
    SentHitVerdict, guest_life_for_boss_life, is_the_boss_kaine, percent_for_motion,
    pvp_scaled_damage,
};
use replicant_proto::EnemyId;

const BOSS_MAX: u32 = 4000;
const OWN_MAX: i32 = 1000;
const LIGHT_STEP: &str = "kaine010_at_sword^c02_01";
const HEAVY_STEP: &str = "kaine010_at_sword^t01_01";

fn life_after(follow: &mut BossLifeFollow, host_hp: u32, own_hp: i32) -> i32 {
    match follow.step(Some((host_hp, BOSS_MAX)), own_hp, OWN_MAX) {
        BossLifeStep::Set { to, .. } => to,
        BossLifeStep::Leave => own_hp,
    }
}

#[test]
fn her_enemy_id_is_the_one_in_her_group() {
    assert!(is_the_boss_kaine(EnemyId {
        group: BOSS_FIGHT_GROUP,
        ordinal: 0
    }));
    assert!(!is_the_boss_kaine(EnemyId::companion(0)));
    assert_eq!(BOSS_FIGHT_GROUP, 1022);
}

#[test]
fn the_guests_life_is_the_same_share_of_its_maximum() {
    assert_eq!(
        guest_life_for_boss_life(BOSS_MAX, BOSS_MAX, OWN_MAX),
        Some(1000)
    );
    assert_eq!(guest_life_for_boss_life(2400, BOSS_MAX, OWN_MAX), Some(600));
    assert_eq!(guest_life_for_boss_life(1, BOSS_MAX, OWN_MAX), Some(1));
}

#[test]
fn a_boss_at_zero_or_an_unreadable_one_asks_for_no_life() {
    assert_eq!(guest_life_for_boss_life(0, BOSS_MAX, OWN_MAX), None);
    assert_eq!(guest_life_for_boss_life(10, 0, OWN_MAX), None);
    assert_eq!(guest_life_for_boss_life(10, BOSS_MAX, 0), None);
    let mut follow = BossLifeFollow::new();
    assert_eq!(follow.step(None, 500, OWN_MAX), BossLifeStep::Leave);
}

#[test]
fn a_boss_life_above_her_maximum_never_lifts_the_guest_past_its_own() {
    assert_eq!(
        guest_life_for_boss_life(BOSS_MAX + 500, BOSS_MAX, OWN_MAX),
        Some(OWN_MAX)
    );
}

#[test]
fn the_guests_life_follows_the_fights_steps_and_the_reset_after_the_down() {
    let mut follow = BossLifeFollow::new();
    let mut own = OWN_MAX;
    let mut seen = Vec::new();
    for host_hp in [4000, 3600, 2400, 2400, 2399, 800] {
        own = life_after(&mut follow, host_hp, own);
        seen.push(own);
    }
    assert_eq!(seen, vec![1000, 900, 600, 600, 600, 200]);
}

#[test]
fn the_life_goes_back_up_when_the_host_resets_her() {
    let mut follow = BossLifeFollow::new();
    let down = life_after(&mut follow, 2400, 700);
    assert_eq!(down, 600);
    let reset = life_after(&mut follow, 3200, down);
    assert_eq!(reset, 800);
    assert_eq!(follow.last_set_to(), Some(800));
}

#[test]
fn nothing_is_written_when_the_life_already_matches() {
    let mut follow = BossLifeFollow::new();
    assert_eq!(
        follow.step(Some((2400, BOSS_MAX)), 600, OWN_MAX),
        BossLifeStep::Leave
    );
}

#[test]
fn a_sent_hit_with_no_native_one_is_applied_at_the_motions_strength() {
    let mut hits = BossNierHits::new();
    assert_eq!(
        hits.sent_hit(10_000, 1, LIGHT_STEP, 1000),
        SentHitVerdict::Apply {
            damage: 21,
            learned: false
        }
    );
    assert_eq!(
        hits.sent_hit(10_000, 2, HEAVY_STEP, 1000),
        SentHitVerdict::Apply {
            damage: 42,
            learned: false
        }
    );
    assert_eq!(
        percent_for_motion("kaine010_01_walk^lo_01"),
        BOSS_HIT_DEFAULT_PERCENT_OF_NIER_MAX_HP
    );
}

#[test]
fn a_native_hit_in_the_window_means_the_sent_one_is_not_counted_again() {
    let mut hits = BossNierHits::new();
    hits.native_hit(10_000, LIGHT_STEP, 77);
    assert_eq!(
        hits.sent_hit(10_000 + NATIVE_HIT_WINDOW_MS - 1, 3, LIGHT_STEP, 1000),
        SentHitVerdict::CountedNatively
    );
}

#[test]
fn a_native_hit_long_before_does_not_stop_a_sent_one_and_sets_the_strength() {
    let mut hits = BossNierHits::new();
    hits.native_hit(10_000, LIGHT_STEP, 77);
    assert_eq!(
        hits.sent_hit(10_000 + NATIVE_HIT_WINDOW_MS, 4, LIGHT_STEP, 1000),
        SentHitVerdict::Apply {
            damage: 26,
            learned: true
        }
    );
    assert_eq!(
        hits.sent_hit(10_000 + NATIVE_HIT_WINDOW_MS, 5, HEAVY_STEP, 1000),
        SentHitVerdict::Apply {
            damage: 42,
            learned: false
        }
    );
}

#[test]
fn one_swing_that_keeps_overlapping_is_applied_once() {
    let mut hits = BossNierHits::new();
    assert!(matches!(
        hits.sent_hit(5_000, 9, LIGHT_STEP, 1000),
        SentHitVerdict::Apply { .. }
    ));
    assert_eq!(
        hits.sent_hit(5_200, 9, LIGHT_STEP, 1000),
        SentHitVerdict::Repeat
    );
    assert!(matches!(
        hits.sent_hit(5_000 + SENT_HIT_REPEAT_WINDOW_MS, 9, LIGHT_STEP, 1000),
        SentHitVerdict::Apply { .. }
    ));
}

#[test]
fn a_hit_is_never_below_one_point() {
    let mut hits = BossNierHits::new();
    assert_eq!(
        hits.sent_hit(1_000, 1, LIGHT_STEP, 3),
        SentHitVerdict::Apply {
            damage: 1,
            learned: false
        }
    );
}

#[test]
fn pvp_hits_deal_the_pvp_share_of_the_normal_damage() {
    assert_eq!(PVP_DAMAGE_PERCENT, 35);
    assert_eq!(pvp_scaled_damage(100), 35);
    assert_eq!(pvp_scaled_damage(14), 4);
    assert_eq!(pvp_scaled_damage(1), 1);
    assert_eq!(pvp_scaled_damage(0), 0);
}
