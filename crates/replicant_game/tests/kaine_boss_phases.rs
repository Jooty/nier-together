use replicant_game::downed::{
    DOWNED_HEALTH, DownedEffect, DownedRules, REVIVE_HOLD_SECONDS, StepInput, WatchSide,
};
use replicant_game::kaine::{
    BOSS_FIGHT_KAINE_MODE, BossDrive, BossDriveEvent, BossDriveFacts, BossPhase, BossPhaseFacts,
    BossPhaseWatch, FIGHT_START_JOIN_WINDOW_MS, FIGHT_WON_SHARE_MARGIN_PERCENT, GuestSeat,
    HeldBackWhy, OWN_LIFE_RESTORE_WAITS_FOR_PLAY_MS, OwnLifeKeeper, OwnLifeStep,
    boss_fight_watch_side, boss_phase_text,
};
use replicant_proto::CombatEvent;

const NIER_MAX: i32 = 400;
const GUEST_OWN_HP: i32 = 740;
const GUEST_OWN_MAX: i32 = 900;
const BOSS_MAX: u32 = 4000;

fn live(guest: GuestSeat, now_ms: u64) -> BossDriveFacts {
    BossDriveFacts {
        in_boss_fight_pack: true,
        her_mode: Some(BOSS_FIGHT_KAINE_MODE),
        her_active: true,
        guest,
        game_poses_her: false,
        now_ms,
    }
}

fn phase_facts(life_percent: u32, event_scene: bool) -> BossPhaseFacts {
    BossPhaseFacts {
        life_percent,
        event_scene,
    }
}

#[test]
fn the_phases_follow_her_life_with_the_event_scene_only_at_the_down() {
    let mut watch = BossPhaseWatch::new();
    assert!(watch.step(phase_facts(100, false)).is_none());
    assert!(watch.step(phase_facts(91, false)).is_none());
    assert_eq!(
        watch.step(phase_facts(89, false)).map(|change| change.to),
        Some(BossPhase::SecondForm)
    );
    assert!(watch.step(phase_facts(61, false)).is_none());
    assert_eq!(
        watch.step(phase_facts(58, false)).map(|change| change.to),
        Some(BossPhase::Down)
    );
    assert!(watch.step(phase_facts(58, true)).is_none());
    assert!(watch.step(phase_facts(60, true)).is_none());
    assert_eq!(
        watch.step(phase_facts(60, false)).map(|change| change.to),
        Some(BossPhase::ThirdForm)
    );
    assert!(watch.step(phase_facts(35, false)).is_none());
    let won = watch.step(phase_facts(20, false)).expect("the end");
    assert_eq!(won.to, BossPhase::Won);
    assert!(boss_phase_text(won).contains("20%"));
    assert!(watch.step(phase_facts(0, false)).is_none());
}

#[test]
fn the_down_never_counts_as_the_third_form_before_the_scene_has_played() {
    let mut watch = BossPhaseWatch::new();
    watch.step(phase_facts(80, false));
    watch.step(phase_facts(55, false));
    for _ in 0..5 {
        assert!(watch.step(phase_facts(55, false)).is_none());
    }
    assert_eq!(watch.phase(), BossPhase::Down);
}

#[test]
fn the_drive_is_paused_by_the_down_scene_and_resumes_after_it_until_the_mode_ends() {
    let mut drive = BossDrive::new();
    assert_eq!(
        drive.step(&live(GuestSeat::Playing, 0)).last(),
        Some(&BossDriveEvent::Engaged)
    );
    let scene = BossDriveFacts {
        game_poses_her: true,
        ..live(GuestSeat::Playing, 40_000)
    };
    assert_eq!(
        drive.step(&scene),
        vec![BossDriveEvent::HeldBackToTheGame {
            why: HeldBackWhy::GamePosesHer
        }]
    );
    assert!(!drive.is_driving());
    assert!(drive.step(&scene).is_empty());
    assert_eq!(
        drive.step(&live(GuestSeat::Playing, 50_000)),
        vec![BossDriveEvent::Engaged]
    );
    let ally_again = BossDriveFacts {
        her_mode: Some(0),
        ..live(GuestSeat::Playing, 90_000)
    };
    assert_eq!(
        drive.step(&ally_again),
        vec![BossDriveEvent::FightEnded {
            was_driving: true,
            guest_had_left: false
        }]
    );
    assert!(!drive.boss_fight_is_live());
    assert!(!drive.guest_may_play_her());
}

#[test]
fn a_retry_that_finds_the_guest_still_loading_lets_it_take_her_over_within_the_first_seconds() {
    let mut drive = BossDrive::new();
    drive.step(&live(GuestSeat::Playing, 0));
    let loading = BossDriveFacts {
        in_boss_fight_pack: false,
        her_mode: None,
        ..live(GuestSeat::Absent, 30_000)
    };
    assert!(matches!(
        drive.step(&loading).as_slice(),
        [BossDriveEvent::FightEnded { .. }]
    ));
    let events = drive.step(&live(GuestSeat::Absent, 31_000));
    assert!(drive.guest_may_play_her());
    assert!(!drive.is_driving());
    assert!(matches!(events[0], BossDriveEvent::FightBegan { .. }));
    assert_eq!(
        drive.step(&live(GuestSeat::Playing, 31_000 + 8_000)),
        vec![BossDriveEvent::Engaged]
    );
    assert!(drive.is_driving());
}

#[test]
fn a_guest_that_arrives_after_the_first_seconds_watches_until_the_fight_ends() {
    let mut drive = BossDrive::new();
    drive.step(&live(GuestSeat::Absent, 5_000));
    let late = 5_000 + FIGHT_START_JOIN_WINDOW_MS;
    assert_eq!(
        drive.step(&live(GuestSeat::Absent, late)),
        vec![BossDriveEvent::JoinWindowClosed {
            guest: GuestSeat::Absent
        }]
    );
    assert!(
        drive
            .step(&live(GuestSeat::Playing, late + 1_000))
            .is_empty()
    );
    assert!(!drive.is_driving());
    assert!(!drive.guest_may_play_her());
}

#[test]
fn a_playing_guest_who_drops_mid_fight_is_the_leave_and_not_the_join_window() {
    let mut drive = BossDrive::new();
    drive.step(&live(GuestSeat::Playing, 0));
    assert_eq!(
        drive.step(&live(GuestSeat::Absent, 2_000)),
        vec![BossDriveEvent::GuestLeft]
    );
}

#[test]
fn the_guests_own_life_comes_back_after_a_won_fight_once_it_is_in_play() {
    let mut keeper = OwnLifeKeeper::new();
    keeper.fight_began(GUEST_OWN_HP);
    keeper.boss_life_seen(2400, BOSS_MAX);
    keeper.boss_life_seen(800, BOSS_MAX);
    assert!(keeper.fight_over(100_000, false));
    assert_eq!(
        keeper.step(100_500, false, 180, GUEST_OWN_MAX),
        OwnLifeStep::Leave
    );
    assert_eq!(
        keeper.step(103_000, true, 180, GUEST_OWN_MAX),
        OwnLifeStep::Restore { to: GUEST_OWN_HP }
    );
    assert_eq!(
        keeper.step(104_000, true, GUEST_OWN_HP, GUEST_OWN_MAX),
        OwnLifeStep::Leave
    );
}

#[test]
fn a_defeat_or_a_fight_left_midway_leaves_the_games_own_life_alone() {
    let mut keeper = OwnLifeKeeper::new();
    keeper.fight_began(GUEST_OWN_HP);
    keeper.boss_life_seen(2400, BOSS_MAX);
    assert!(!keeper.fight_over(50_000, false));
    assert_eq!(
        keeper.step(51_000, true, 900, GUEST_OWN_MAX),
        OwnLifeStep::Leave
    );
    assert_eq!(keeper.remembered(), None);
}

#[test]
fn a_dropped_link_gives_the_guest_its_own_life_back_whatever_hers_was() {
    let mut keeper = OwnLifeKeeper::new();
    keeper.fight_began(GUEST_OWN_HP);
    keeper.boss_life_seen(2400, BOSS_MAX);
    assert!(keeper.fight_over(50_000, true));
    assert_eq!(
        keeper.step(50_200, true, 540, GUEST_OWN_MAX),
        OwnLifeStep::Restore { to: GUEST_OWN_HP }
    );
}

#[test]
fn the_restore_is_clamped_to_the_maximum_and_dropped_when_play_never_comes() {
    let mut keeper = OwnLifeKeeper::new();
    keeper.fight_began(1200);
    keeper.boss_life_seen(100, BOSS_MAX);
    keeper.fight_over(0, false);
    assert_eq!(
        keeper.step(1_000, true, 300, GUEST_OWN_MAX),
        OwnLifeStep::Restore { to: GUEST_OWN_MAX }
    );
    keeper.fight_began(500);
    keeper.boss_life_seen(100, BOSS_MAX);
    keeper.fight_over(0, false);
    assert_eq!(
        keeper.step(OWN_LIFE_RESTORE_WAITS_FOR_PLAY_MS, true, 300, GUEST_OWN_MAX),
        OwnLifeStep::Leave
    );
    assert_eq!(keeper.remembered(), None);
}

#[test]
fn a_boss_just_above_the_end_share_still_counts_as_won() {
    let mut keeper = OwnLifeKeeper::new();
    keeper.fight_began(GUEST_OWN_HP);
    let share = 20 + FIGHT_WON_SHARE_MARGIN_PERCENT;
    keeper.boss_life_seen(BOSS_MAX * share / 100, BOSS_MAX);
    assert!(keeper.fight_over(0, false));
}

#[test]
fn a_second_fight_starting_before_the_restore_keeps_the_first_own_life() {
    let mut keeper = OwnLifeKeeper::new();
    keeper.fight_began(GUEST_OWN_HP);
    keeper.boss_life_seen(100, BOSS_MAX);
    keeper.fight_over(0, false);
    keeper.fight_began(180);
    assert_eq!(keeper.remembered(), Some(GUEST_OWN_HP));
}

fn down_step(health: i32, last_was_up: bool) -> StepInput {
    StepInput {
        dt_seconds: 0.4,
        session_active: true,
        local_health: health,
        lethal_hit_seen: last_was_up && health == DOWNED_HEALTH,
        partner_distance: Some(0.0),
    }
}

#[test]
fn nobody_is_a_watcher_outside_the_pvp_fight() {
    assert_eq!(boss_fight_watch_side(false, false), None);
    assert_eq!(boss_fight_watch_side(true, false), Some(WatchSide::Partner));
    assert_eq!(
        boss_fight_watch_side(false, true),
        Some(WatchSide::ThisGame)
    );
}

#[test]
fn the_hosts_death_in_the_fight_is_the_normal_defeat_for_both_with_no_revive_by_her() {
    let mut host = DownedRules::new();
    host.set_guest(false);
    host.set_watch(boss_fight_watch_side(true, false).expect("host side"));
    host.step(down_step(NIER_MAX, false));
    let effects = host.step(down_step(DOWNED_HEALTH, true));
    assert!(host.is_game_over());
    assert!(effects.contains(&DownedEffect::Send(CombatEvent::GameOver)));
    assert!(effects.contains(&DownedEffect::ForceKill));

    let mut guest = DownedRules::new();
    guest.set_guest(true);
    guest.set_watch(boss_fight_watch_side(false, true).expect("guest side"));
    guest.step(down_step(GUEST_OWN_HP, false));
    guest.on_partner_event(CombatEvent::GameOver);
    assert!(guest.is_game_over());
}

#[test]
fn the_guest_in_the_fight_never_revives_the_host_and_ends_the_game_the_moment_he_goes_down() {
    let mut guest = DownedRules::new();
    guest.set_guest(true);
    guest.set_watch(WatchSide::ThisGame);
    guest.step(down_step(GUEST_OWN_HP, false));
    let effects = guest.on_partner_event(CombatEvent::Downed);
    assert!(guest.is_game_over());
    assert!(effects.contains(&DownedEffect::Send(CombatEvent::GameOver)));
    let mut sent_revive = false;
    for _ in 0..((REVIVE_HOLD_SECONDS / 0.4) as usize + 5) {
        let effects = guest.step(down_step(DOWNED_HEALTH, false));
        sent_revive |= effects.contains(&DownedEffect::Send(CombatEvent::Revived));
    }
    assert!(!sent_revive);
    assert!(guest.is_game_over());
}
