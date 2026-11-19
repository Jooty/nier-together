use replicant_game::downed::{
    DOWNED_HEALTH, DownedEffect, DownedRules, DownedState, REVIVE_HOLD_SECONDS, StepInput,
    WatchSide,
};
use replicant_proto::CombatEvent;

const FULL_HEALTH: i32 = 400;
const TICK_SECONDS: f32 = 0.4;
const RIGHT_NEXT_TO_THE_PARTNER: Option<f32> = Some(0.0);

fn step(health: i32, distance: Option<f32>) -> StepInput {
    StepInput {
        dt_seconds: TICK_SECONDS,
        session_active: true,
        local_health: health,
        lethal_hit_seen: health == DOWNED_HEALTH,
        partner_distance: distance,
    }
}

fn rules_watching(watch: WatchSide) -> DownedRules {
    let mut rules = DownedRules::new();
    rules.set_watch(watch);
    rules.step(step(FULL_HEALTH, None));
    rules
}

fn has_kill(effects: &[DownedEffect]) -> bool {
    effects.contains(&DownedEffect::ForceKill)
}

fn has_game_over_word(effects: &[DownedEffect]) -> bool {
    effects.contains(&DownedEffect::Send(CombatEvent::GameOver))
}

#[test]
fn a_watching_player_never_revives_the_partner_in_play_by_standing_close() {
    let mut rules = rules_watching(WatchSide::ThisGame);
    rules.on_partner_event(CombatEvent::Downed);
    let mut sent_revive = false;
    for _ in 0..((REVIVE_HOLD_SECONDS / TICK_SECONDS) as usize + 5) {
        let effects = rules.step(step(FULL_HEALTH, RIGHT_NEXT_TO_THE_PARTNER));
        sent_revive |= effects.contains(&DownedEffect::Send(CombatEvent::Revived));
    }
    assert!(!sent_revive);
    assert_eq!(rules.revive_progress(), 0.0);
}

#[test]
fn a_watching_player_cannot_revive_by_the_button_either() {
    let mut rules = rules_watching(WatchSide::ThisGame);
    rules.on_partner_event(CombatEvent::Downed);
    assert!(rules.request_revive_partner().is_empty());
}

#[test]
fn the_host_ends_the_game_when_it_goes_down_while_the_guest_watches() {
    let mut rules = rules_watching(WatchSide::Partner);
    let effects = rules.step(step(DOWNED_HEALTH, None));
    assert!(rules.is_game_over());
    assert!(rules.ended_by_lone_player());
    assert!(has_game_over_word(&effects));
    assert!(has_kill(&effects));
}

#[test]
fn the_guest_ends_the_game_when_the_host_goes_down_while_it_watches() {
    let mut rules = rules_watching(WatchSide::ThisGame);
    rules.set_guest(true);
    let effects = rules.on_partner_event(CombatEvent::Downed);
    assert!(rules.is_game_over());
    assert!(rules.ended_by_lone_player());
    assert!(!has_kill(&effects));
}

#[test]
fn a_guest_in_play_ends_the_game_at_once_when_the_watching_host_cannot_revive() {
    let mut rules = rules_watching(WatchSide::Partner);
    rules.set_guest(true);
    let effects = rules.step(step(DOWNED_HEALTH, None));
    assert!(rules.is_game_over());
    assert!(has_game_over_word(&effects));
}

#[test]
fn a_watching_player_cannot_be_put_down() {
    let mut rules = rules_watching(WatchSide::ThisGame);
    assert!(rules.request_go_down().is_empty());
    let effects = rules.step(step(DOWNED_HEALTH, None));
    assert_eq!(rules.local(), DownedState::Up);
    assert!(!rules.is_game_over());
    assert!(!effects.contains(&DownedEffect::EnterDowned));
}

#[test]
fn a_stray_revive_word_does_not_lift_the_player_in_play_while_the_partner_watches() {
    let mut rules = rules_watching(WatchSide::Partner);
    rules.step(step(DOWNED_HEALTH, None));
    let effects = rules.on_partner_event(CombatEvent::Revived);
    assert!(rules.is_game_over());
    assert!(!effects.contains(&DownedEffect::LeaveDowned));
}

#[test]
fn without_a_watcher_a_single_down_still_waits_for_a_revive() {
    let mut rules = rules_watching(WatchSide::Nobody);
    let effects = rules.step(step(DOWNED_HEALTH, None));
    assert_eq!(rules.local(), DownedState::Downed);
    assert!(!rules.is_game_over());
    assert!(!has_kill(&effects));
}

#[test]
fn without_a_watcher_standing_close_still_revives() {
    let mut rules = rules_watching(WatchSide::Nobody);
    rules.on_partner_event(CombatEvent::Downed);
    let mut sent_revive = false;
    for _ in 0..((REVIVE_HOLD_SECONDS / TICK_SECONDS) as usize + 2) {
        let effects = rules.step(step(FULL_HEALTH, RIGHT_NEXT_TO_THE_PARTNER));
        sent_revive |= effects.contains(&DownedEffect::Send(CombatEvent::Revived));
    }
    assert!(sent_revive);
}

#[test]
fn the_watch_starting_after_the_player_in_play_is_already_down_ends_the_game() {
    let mut rules = rules_watching(WatchSide::Nobody);
    rules.step(step(DOWNED_HEALTH, None));
    rules.set_watch(WatchSide::Partner);
    rules.step(step(DOWNED_HEALTH, None));
    assert!(rules.is_game_over());
}
