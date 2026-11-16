use replicant_game::dialogue::{
    LINE_PRESS_GIVES_UP_AFTER_MS, LINE_PRESS_OPENS_THE_TALK_AFTER_MS, LinePress, PressStep,
    TALK_PLAYER_STATE_TYPING, TALK_PLAYER_STATE_WAITS_AT_A_PAGE_BREAK,
    TALK_PLAYER_STATE_WAITS_FOR_A_PRESS, TALK_UPDATE_INPUT_CONFIRM_OFFSET,
    TALK_UPDATE_INPUT_ENABLED_OFFSET, TalkWindowReading,
};

const DREAM_FIRST_TALK: u32 = 1422;
const DREAM_SECOND_TALK: u32 = 1423;
const FIRST_TALK_ADDRESS: u64 = 0x1_4400_1000;
const SECOND_TALK_ADDRESS: u64 = 0x1_4400_1200;
const HOST_ON_LINE_ONE_MS: u64 = 6_500_176;
const HOST_ON_THE_SECOND_TALK_MS: u64 = 6_502_451;
const PLAYER_STATE_WAITING_FOR_THE_VOICE: i32 = 2;

fn window(talk_address: u64, line: u16, line_count: u16, player_state: i32) -> TalkWindowReading {
    TalkWindowReading {
        open: true,
        talk_address,
        line,
        step: 0,
        line_count,
        player_state,
    }
}

#[test]
fn the_confirm_press_lives_in_the_update_input_the_games_own_button_test_reads() {
    // Talk player state 4 (0x4E0E10) tests pad+0x3e (enabled) and pad+0x24 bit 1, pad = input+4.
    assert_eq!(TALK_UPDATE_INPUT_ENABLED_OFFSET, 0x42);
    assert_eq!(TALK_UPDATE_INPUT_CONFIRM_OFFSET, 0x28);
    assert_eq!(TALK_PLAYER_STATE_TYPING, 3);
    assert_eq!(TALK_PLAYER_STATE_WAITS_FOR_A_PRESS, 4);
    // State 5 (0x4E10F0) waits at a page break and reads the same confirm.
    assert_eq!(TALK_PLAYER_STATE_WAITS_AT_A_PAGE_BREAK, 5);
}

#[test]
fn a_page_break_the_host_passed_is_pressed_too_and_counted_as_a_step() {
    let mut press = LinePress::new();
    press.want(
        DREAM_SECOND_TALK,
        0,
        1,
        true,
        SECOND_TALK_ADDRESS,
        HOST_ON_THE_SECOND_TALK_MS,
    );
    let at_the_break = window(
        SECOND_TALK_ADDRESS,
        0,
        2,
        TALK_PLAYER_STATE_WAITS_AT_A_PAGE_BREAK,
    );
    assert_eq!(
        press.step(&at_the_break, HOST_ON_THE_SECOND_TALK_MS + 16),
        Some(PressStep::Press),
        "run 1115j 01:48:22.764: the guest's window sat in player state 5 on line 0 of talk 1423 and was never pressed"
    );
    let past_it = TalkWindowReading {
        step: 1,
        ..window(SECOND_TALK_ADDRESS, 0, 2, TALK_PLAYER_STATE_TYPING)
    };
    assert_eq!(
        press.step(&past_it, HOST_ON_THE_SECOND_TALK_MS + 33),
        Some(PressStep::Reached {
            talk_id: DREAM_SECOND_TALK,
            line: 0,
            presses: 1,
        })
    );
}

#[test]
fn a_step_behind_on_the_last_line_is_still_pressed_and_a_step_ahead_is_not() {
    let mut press = LinePress::new();
    press.want(
        DREAM_FIRST_TALK,
        1,
        1,
        true,
        FIRST_TALK_ADDRESS,
        HOST_ON_LINE_ONE_MS,
    );
    let last_line_first_page = window(
        FIRST_TALK_ADDRESS,
        1,
        2,
        TALK_PLAYER_STATE_WAITS_FOR_A_PRESS,
    );
    assert_eq!(
        press.step(&last_line_first_page, HOST_ON_LINE_ONE_MS + 16),
        Some(PressStep::Press)
    );
    let mut same_line_later_step = press;
    same_line_later_step.want(
        DREAM_FIRST_TALK,
        1,
        0,
        true,
        FIRST_TALK_ADDRESS,
        HOST_ON_LINE_ONE_MS + 20,
    );
    let on_page_two = TalkWindowReading {
        step: 1,
        ..last_line_first_page
    };
    assert_eq!(
        same_line_later_step.step(&on_page_two, HOST_ON_LINE_ONE_MS + 33),
        Some(PressStep::Reached {
            talk_id: DREAM_FIRST_TALK,
            line: 1,
            presses: 0,
        })
    );
}

#[test]
fn a_window_behind_the_hosts_line_is_pressed_once_per_update_until_it_is_there() {
    let mut press = LinePress::new();
    press.want(
        DREAM_FIRST_TALK,
        1,
        0,
        true,
        FIRST_TALK_ADDRESS,
        HOST_ON_LINE_ONE_MS,
    );
    let waiting = window(
        FIRST_TALK_ADDRESS,
        0,
        2,
        TALK_PLAYER_STATE_WAITS_FOR_A_PRESS,
    );
    assert_eq!(
        press.step(&waiting, HOST_ON_LINE_ONE_MS + 16),
        Some(PressStep::Press),
        "run 1115j 01:48:20.176: the host went to line 1 of talk 1422 while the guest's window waited on line 0"
    );
    let arrived = window(FIRST_TALK_ADDRESS, 1, 2, TALK_PLAYER_STATE_TYPING);
    assert_eq!(
        press.step(&arrived, HOST_ON_LINE_ONE_MS + 33),
        Some(PressStep::Reached {
            talk_id: DREAM_FIRST_TALK,
            line: 1,
            presses: 1,
        })
    );
    assert_eq!(press.wanted(), None);
    assert_eq!(press.step(&arrived, HOST_ON_LINE_ONE_MS + 50), None);
}

#[test]
fn a_press_while_the_text_types_finishes_it_and_the_next_update_presses_again() {
    let mut press = LinePress::new();
    press.want(
        DREAM_FIRST_TALK,
        1,
        0,
        true,
        FIRST_TALK_ADDRESS,
        HOST_ON_LINE_ONE_MS,
    );
    let typing = window(FIRST_TALK_ADDRESS, 0, 2, TALK_PLAYER_STATE_TYPING);
    assert_eq!(
        press.step(&typing, HOST_ON_LINE_ONE_MS + 16),
        Some(PressStep::Press)
    );
    let waiting = window(
        FIRST_TALK_ADDRESS,
        0,
        2,
        TALK_PLAYER_STATE_WAITS_FOR_A_PRESS,
    );
    assert_eq!(
        press.step(&waiting, HOST_ON_LINE_ONE_MS + 33),
        Some(PressStep::Press)
    );
    let arrived = window(FIRST_TALK_ADDRESS, 1, 2, TALK_PLAYER_STATE_TYPING);
    assert_eq!(
        press.step(&arrived, HOST_ON_LINE_ONE_MS + 50),
        Some(PressStep::Reached {
            talk_id: DREAM_FIRST_TALK,
            line: 1,
            presses: 2,
        })
    );
}

#[test]
fn a_player_still_waiting_for_the_voice_takes_no_press() {
    let mut press = LinePress::new();
    press.want(
        DREAM_FIRST_TALK,
        1,
        0,
        true,
        FIRST_TALK_ADDRESS,
        HOST_ON_LINE_ONE_MS,
    );
    let voice = window(FIRST_TALK_ADDRESS, 0, 2, PLAYER_STATE_WAITING_FOR_THE_VOICE);
    assert_eq!(press.step(&voice, HOST_ON_LINE_ONE_MS + 16), None);
    assert_eq!(press.wanted(), Some((DREAM_FIRST_TALK, 1, 0)));
}

#[test]
fn the_same_wish_twice_keeps_the_press_count_and_a_new_line_starts_over() {
    let mut press = LinePress::new();
    press.want(
        DREAM_FIRST_TALK,
        1,
        0,
        true,
        FIRST_TALK_ADDRESS,
        HOST_ON_LINE_ONE_MS,
    );
    let waiting = window(
        FIRST_TALK_ADDRESS,
        0,
        2,
        TALK_PLAYER_STATE_WAITS_FOR_A_PRESS,
    );
    press.step(&waiting, HOST_ON_LINE_ONE_MS + 16);
    press.want(
        DREAM_FIRST_TALK,
        1,
        0,
        true,
        FIRST_TALK_ADDRESS,
        HOST_ON_LINE_ONE_MS + 20,
    );
    let arrived = window(FIRST_TALK_ADDRESS, 1, 2, TALK_PLAYER_STATE_TYPING);
    assert_eq!(
        press.step(&arrived, HOST_ON_LINE_ONE_MS + 33),
        Some(PressStep::Reached {
            talk_id: DREAM_FIRST_TALK,
            line: 1,
            presses: 1,
        })
    );
}

#[test]
fn the_press_on_the_last_line_takes_the_window_on_to_the_hosts_next_talk() {
    let mut press = LinePress::new();
    press.want(
        DREAM_SECOND_TALK,
        0,
        0,
        false,
        FIRST_TALK_ADDRESS,
        HOST_ON_THE_SECOND_TALK_MS,
    );
    let last_line = window(
        FIRST_TALK_ADDRESS,
        1,
        2,
        TALK_PLAYER_STATE_WAITS_FOR_A_PRESS,
    );
    assert_eq!(
        press.step(&last_line, HOST_ON_THE_SECOND_TALK_MS + 16),
        Some(PressStep::Press),
        "run 1115j 01:48:22.451: the host's window went from the last line of 1422 to 1423 on a press"
    );
    let next_talk = window(
        SECOND_TALK_ADDRESS,
        0,
        2,
        PLAYER_STATE_WAITING_FOR_THE_VOICE,
    );
    assert_eq!(
        press.step(&next_talk, HOST_ON_THE_SECOND_TALK_MS + 33),
        Some(PressStep::TalkChanged {
            talk_id: DREAM_SECOND_TALK,
            line: 0,
            presses: 1,
        })
    );
    assert_eq!(press.wanted(), None);
}

#[test]
fn a_press_never_leaves_the_talk_the_host_is_still_on() {
    let mut press = LinePress::new();
    press.want(
        DREAM_FIRST_TALK,
        5,
        0,
        true,
        FIRST_TALK_ADDRESS,
        HOST_ON_LINE_ONE_MS,
    );
    let last_line = window(
        FIRST_TALK_ADDRESS,
        1,
        2,
        TALK_PLAYER_STATE_WAITS_FOR_A_PRESS,
    );
    assert_eq!(press.step(&last_line, HOST_ON_LINE_ONE_MS + 16), None);
    assert_eq!(
        press.step(
            &last_line,
            HOST_ON_LINE_ONE_MS + LINE_PRESS_GIVES_UP_AFTER_MS
        ),
        Some(PressStep::GaveUp {
            talk_id: DREAM_FIRST_TALK,
            line: 5,
            presses: 0,
        })
    );
}

#[test]
fn a_switch_the_presses_do_not_reach_is_opened_in_the_window_instead() {
    let mut press = LinePress::new();
    press.want(
        DREAM_SECOND_TALK,
        0,
        0,
        false,
        FIRST_TALK_ADDRESS,
        HOST_ON_THE_SECOND_TALK_MS,
    );
    let stuck = window(FIRST_TALK_ADDRESS, 1, 2, PLAYER_STATE_WAITING_FOR_THE_VOICE);
    assert_eq!(press.step(&stuck, HOST_ON_THE_SECOND_TALK_MS + 16), None);
    assert_eq!(
        press.step(
            &stuck,
            HOST_ON_THE_SECOND_TALK_MS + LINE_PRESS_OPENS_THE_TALK_AFTER_MS
        ),
        Some(PressStep::OpenInstead {
            talk_id: DREAM_SECOND_TALK,
            line: 0,
            presses: 0,
        })
    );
    assert_eq!(press.wanted(), None);
}

#[test]
fn a_window_that_closes_drops_the_wish() {
    let mut press = LinePress::new();
    press.want(
        DREAM_FIRST_TALK,
        1,
        0,
        true,
        FIRST_TALK_ADDRESS,
        HOST_ON_LINE_ONE_MS,
    );
    let closed = TalkWindowReading {
        open: false,
        ..window(FIRST_TALK_ADDRESS, 0, 2, 0)
    };
    assert_eq!(
        press.step(&closed, HOST_ON_LINE_ONE_MS + 16),
        Some(PressStep::Closed {
            talk_id: DREAM_FIRST_TALK,
            line: 1,
        })
    );
    press.clear();
    assert_eq!(press.step(&closed, HOST_ON_LINE_ONE_MS + 33), None);
}

#[test]
fn a_stale_wish_gives_up_before_it_can_report_a_talk_change() {
    let mut press = LinePress::new();
    press.want(
        DREAM_SECOND_TALK,
        0,
        0,
        false,
        FIRST_TALK_ADDRESS,
        HOST_ON_THE_SECOND_TALK_MS,
    );
    let much_later = window(SECOND_TALK_ADDRESS, 0, 2, TALK_PLAYER_STATE_TYPING);
    assert_eq!(
        press.step(
            &much_later,
            HOST_ON_THE_SECOND_TALK_MS + LINE_PRESS_GIVES_UP_AFTER_MS
        ),
        Some(PressStep::GaveUp {
            talk_id: DREAM_SECOND_TALK,
            line: 0,
            presses: 0,
        })
    );
}
