use replicant_game::coop_pause::MENU_PLAYER_INPUT_BIT;
use replicant_game::rule_rooms::{
    BreakerBodyHold, BreakerReturn, RETURN_FADE_IN_MS, RETURN_FADE_OUT_MS, RULE_ROOMS, RulePlayer,
    body_held_line, body_let_go_line,
};

const ROOM_3: usize = 2;
const GAME_PAUSE_ON_4_SLOT_ZERO: u32 = 0x480;

fn hold_ticks(
    breaker: &BreakerReturn,
    body: &mut BreakerBodyHold,
    slot_zero: &mut u32,
    from_ms: u64,
    to_ms: u64,
) -> Vec<(u64, u32)> {
    let mut writes = Vec::new();
    let mut now_ms = from_ms;
    while now_ms <= to_ms {
        if let Some(written) = body.slot_zero_to_write(*slot_zero, breaker.input_blocked(now_ms)) {
            *slot_zero = written;
            writes.push((now_ms, written));
        }
        now_ms += 16;
    }
    writes
}

#[test]
fn the_run_of_1114o_the_body_ran_at_7_inside_each_return_and_is_now_held_through_it() {
    // Host breaks of room 3 (19:22:xx, ms of the minute) and a move line inside each return
    // that showed a held key and the run speed 7.00.
    let breaks_and_running_inside_ms = [(27_145, 29_231), (29_411, 31_258), (36_458, 38_283)];
    for (broke_at_ms, ran_at_ms) in breaks_and_running_inside_ms {
        let mut breaker = BreakerReturn::new();
        let mut body = BreakerBodyHold::new();
        let mut slot_zero = 0;
        assert!(breaker.begin(&RULE_ROOMS[ROOM_3], RulePlayer::Host, broke_at_ms));
        let let_go_at_ms = broke_at_ms + RETURN_FADE_OUT_MS + RETURN_FADE_IN_MS;
        let writes = hold_ticks(&breaker, &mut body, &mut slot_zero, broke_at_ms, ran_at_ms);
        assert_eq!(writes, vec![(broke_at_ms, MENU_PLAYER_INPUT_BIT)]);
        assert!(body.is_holding());
        assert!(ran_at_ms < let_go_at_ms);
        let writes = hold_ticks(
            &breaker,
            &mut body,
            &mut slot_zero,
            ran_at_ms,
            let_go_at_ms + 32,
        );
        assert_eq!(writes.len(), 1);
        assert!(writes[0].0 >= let_go_at_ms);
        assert_eq!(writes[0].1, 0);
        assert!(!body.is_holding());
    }
}

#[test]
fn a_pause_the_game_writes_during_the_hold_is_the_games_and_is_not_taken_back() {
    let mut body = BreakerBodyHold::new();
    assert_eq!(
        body.slot_zero_to_write(0, true),
        Some(MENU_PLAYER_INPUT_BIT)
    );
    assert_eq!(
        body.slot_zero_to_write(GAME_PAUSE_ON_4_SLOT_ZERO, true),
        None
    );
    assert_eq!(
        body.slot_zero_to_write(GAME_PAUSE_ON_4_SLOT_ZERO, false),
        None
    );
    assert!(!body.is_holding());
}

#[test]
fn a_game_pause_off_during_the_hold_does_not_free_the_body_before_the_fade_in_ends() {
    let mut body = BreakerBodyHold::new();
    assert_eq!(
        body.slot_zero_to_write(0, true),
        Some(MENU_PLAYER_INPUT_BIT)
    );
    assert_eq!(
        body.slot_zero_to_write(0, true),
        Some(MENU_PLAYER_INPUT_BIT)
    );
    assert_eq!(
        body.slot_zero_to_write(MENU_PLAYER_INPUT_BIT, false),
        Some(0)
    );
}

#[test]
fn a_body_the_game_already_holds_is_left_to_the_game() {
    let mut body = BreakerBodyHold::new();
    assert_eq!(
        body.slot_zero_to_write(GAME_PAUSE_ON_4_SLOT_ZERO, true),
        None
    );
    assert!(!body.is_holding());
    assert_eq!(
        body.slot_zero_to_write(GAME_PAUSE_ON_4_SLOT_ZERO, false),
        None
    );
}

#[test]
fn other_slot_zero_bits_stay_as_the_game_set_them() {
    let mut body = BreakerBodyHold::new();
    assert_eq!(body.slot_zero_to_write(0x20, true), Some(0xA0));
    assert_eq!(body.slot_zero_to_write(0xA0, false), Some(0x20));
}

#[test]
fn not_held_with_no_return_writes_nothing() {
    let mut body = BreakerBodyHold::new();
    assert_eq!(body.slot_zero_to_write(0, false), None);
}

#[test]
fn the_hold_lines_say_what_slot_0_held_and_holds_now() {
    assert_eq!(
        body_held_line(0, 0x80),
        "rule room: the breaker's body is held still until the fade in ends, as the game's own pause holds it (control mask slot 0 0x0 -> 0x80)"
    );
    assert_eq!(
        body_let_go_line(0x80, 0),
        "rule room: the breaker's body is let go, the fade in has ended (control mask slot 0 0x80 -> 0x0)"
    );
}
