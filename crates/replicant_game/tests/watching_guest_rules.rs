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

#[test]
fn the_free_to_act_test_runs_without_the_watching_hold_bit() {
    use replicant_game::coop_pause::{MENU_PLAYER_INPUT_BIT, mask_for_the_free_to_act_test};
    let mask = MENU_PLAYER_INPUT_BIT | 0x1;
    assert_eq!(mask_for_the_free_to_act_test(mask, true, true), 0x1);
    assert_eq!(mask_for_the_free_to_act_test(mask, true, false), mask);
    assert_eq!(mask_for_the_free_to_act_test(mask, false, true), mask);
}

#[test]
fn the_watch_hold_writes_the_bit_once_and_takes_back_only_what_it_wrote() {
    use replicant_game::coop_pause::MENU_PLAYER_INPUT_BIT;
    use replicant_game::rule_rooms::BreakerBodyHold;
    let mut hold = BreakerBodyHold::new();
    assert_eq!(
        hold.slot_zero_to_write(0, true),
        Some(MENU_PLAYER_INPUT_BIT)
    );
    assert_eq!(hold.slot_zero_to_write(MENU_PLAYER_INPUT_BIT, true), None);
    assert_eq!(
        hold.slot_zero_to_write(MENU_PLAYER_INPUT_BIT, false),
        Some(0)
    );
    let mut hold = BreakerBodyHold::new();
    hold.slot_zero_to_write(0, true);
    assert_eq!(hold.slot_zero_to_write(0x481, false), None);
}

#[test]
fn the_watch_hold_puts_the_bit_back_when_the_game_clears_it() {
    use replicant_game::coop_pause::MENU_PLAYER_INPUT_BIT;
    use replicant_game::rule_rooms::BreakerBodyHold;
    let mut hold = BreakerBodyHold::new();
    hold.slot_zero_to_write(0, true);
    assert_eq!(
        hold.slot_zero_to_write(0, true),
        Some(MENU_PLAYER_INPUT_BIT)
    );
}
