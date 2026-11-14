use replicant_game::rule_rooms::{RulePlayer, RulePop, break_of_a_watching_guest};
use replicant_game::story_watch::watched_body_moves_on_keys;

const TEMPLE: &str = "A_DESERT_TEMPLE_01";

#[test]
fn a_watching_guest_takes_no_run_break_in_room_three() {
    let pop = break_of_a_watching_guest(TEMPLE, "Run_has_Done", RulePlayer::Guest, true);
    assert!(matches!(pop, Some(RulePop::DropWhileWatching { .. })));
    assert!(pop.unwrap().drops());
}

#[test]
fn a_watching_guest_takes_no_move_break_in_room_two() {
    let pop = break_of_a_watching_guest(TEMPLE, "Move_has_Done", RulePlayer::Guest, true);
    assert!(matches!(pop, Some(RulePop::DropWhileWatching { .. })));
}

#[test]
fn a_guest_that_plays_is_bound_by_the_rule() {
    assert!(break_of_a_watching_guest(TEMPLE, "Run_has_Done", RulePlayer::Guest, false).is_none());
}

#[test]
fn the_host_is_bound_whatever_the_guest_does() {
    assert!(break_of_a_watching_guest(TEMPLE, "Run_has_Done", RulePlayer::Host, true).is_none());
}

#[test]
fn a_watching_guest_still_runs_every_call_that_is_not_a_break() {
    assert!(
        break_of_a_watching_guest(TEMPLE, "Set_Law_Game_03", RulePlayer::Guest, true).is_none()
    );
    assert!(
        break_of_a_watching_guest(TEMPLE, "Clear_Law_Game_03", RulePlayer::Guest, true).is_none()
    );
    assert!(
        break_of_a_watching_guest(TEMPLE, "Some_Other_Call", RulePlayer::Guest, true).is_none()
    );
}

#[test]
fn a_watched_body_running_on_a_held_key_is_a_leak() {
    assert!(watched_body_moves_on_keys(
        Some(4.7),
        Some([false, true, false, false])
    ));
}

#[test]
fn a_watched_body_moved_by_the_placement_alone_is_no_leak() {
    assert!(!watched_body_moves_on_keys(Some(4.7), Some([false; 4])));
    assert!(!watched_body_moves_on_keys(Some(0.0), Some([true; 4])));
    assert!(!watched_body_moves_on_keys(None, None));
}
