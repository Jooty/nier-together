use replicant_game::scale_up_prompt::{
    AnswerHider, AskEffect, Gesture, HOLD_TO_ANSWER_MS, IGNORE_PRESSES_AFTER_SHOWN_MS,
    KEYBOARD_NOT_NOW_VIRTUAL_KEY, KEYBOARD_SCALE_UP_VIRTUAL_KEY, PAD_ANSWER_BUTTONS,
    PAD_NOT_NOW_BUTTON, PAD_SCALE_UP_BUTTON, PressSample, ScaleUpPrompt, XINPUT_STATE_BYTES,
    keyboard_gesture, pad_gesture,
};
use replicant_proto::{ScaleUpAnswer, ScaleUpAsk, scale_up_no_save_yet_line};

const GUEST_SLOT: u8 = 1;
const GUEST_NAME: &str = "Guesty";
const FRAME_MS: u64 = 16;

fn open_ask() -> ScaleUpAsk {
    ScaleUpAsk::Open {
        slot: GUEST_SLOT,
        name: GUEST_NAME.to_owned(),
    }
}

fn withdrawn_ask() -> ScaleUpAsk {
    ScaleUpAsk::Withdrawn { slot: GUEST_SLOT }
}

fn keyboard(gesture: Gesture) -> PressSample {
    PressSample {
        keyboard: gesture,
        ..PressSample::default()
    }
}

fn pad(gesture: Gesture) -> PressSample {
    PressSample {
        pad: gesture,
        ..PressSample::default()
    }
}

fn click(yes: bool) -> PressSample {
    PressSample {
        click: Some(yes),
        ..PressSample::default()
    }
}

struct Table {
    prompt: ScaleUpPrompt,
    now_ms: u64,
    answers: Vec<ScaleUpAnswer>,
}

impl Table {
    fn asked() -> Self {
        let mut prompt = ScaleUpPrompt::new();
        prompt.on_ask(open_ask());
        Self {
            prompt,
            now_ms: 10_000,
            answers: Vec::new(),
        }
    }

    fn frames(&mut self, safe: bool, input: PressSample, millis: u64) {
        let end = self.now_ms + millis;
        while self.now_ms < end {
            let step = self.prompt.step(safe, self.now_ms, input);
            self.answers.extend(step.answer);
            self.now_ms += FRAME_MS;
        }
    }

    fn quiet_until_presses_count(&mut self) {
        self.frames(
            true,
            PressSample::default(),
            IGNORE_PRESSES_AFTER_SHOWN_MS + FRAME_MS,
        );
    }
}

#[test]
fn an_ask_is_shown_at_a_safe_moment_and_not_before_the_first_frame() {
    let mut table = Table::asked();
    assert!(table.prompt.is_open());
    assert!(table.prompt.shown().is_none());
    table.frames(true, PressSample::default(), FRAME_MS);
    let shown = table.prompt.shown().expect("shown");
    assert_eq!((shown.slot, shown.name.as_str()), (GUEST_SLOT, GUEST_NAME));
}

#[test]
fn a_load_a_scene_or_a_fight_takes_the_prompt_away_and_it_stays_open_for_later() {
    let mut table = Table::asked();
    table.quiet_until_presses_count();
    assert!(table.prompt.shown().is_some());
    table.frames(false, PressSample::default(), 5_000);
    assert!(table.prompt.shown().is_none());
    assert!(table.prompt.is_open());
    assert!(!table.prompt.wants_input_masked());
    table.frames(true, PressSample::default(), FRAME_MS);
    assert!(table.prompt.shown().is_some());
    assert!(table.answers.is_empty());
}

#[test]
fn nothing_answers_while_the_game_is_not_at_a_safe_moment() {
    let mut table = Table::asked();
    table.frames(false, keyboard(Gesture::Yes), 5_000);
    table.frames(false, click(true), FRAME_MS);
    assert!(table.answers.is_empty());
    assert!(table.prompt.is_open());
}

#[test]
fn holding_the_keyboard_chord_for_yes_answers_yes_once() {
    let mut table = Table::asked();
    table.quiet_until_presses_count();
    table.frames(true, keyboard(Gesture::Yes), HOLD_TO_ANSWER_MS * 3);
    assert_eq!(
        table.answers,
        vec![ScaleUpAnswer {
            slot: GUEST_SLOT,
            yes: true
        }]
    );
    assert!(!table.prompt.is_open());
}

#[test]
fn holding_the_pad_chord_for_no_answers_no_once() {
    let mut table = Table::asked();
    table.quiet_until_presses_count();
    table.frames(true, pad(Gesture::No), HOLD_TO_ANSWER_MS * 3);
    assert_eq!(
        table.answers,
        vec![ScaleUpAnswer {
            slot: GUEST_SLOT,
            yes: false
        }]
    );
}

#[test]
fn a_click_answers_at_once_for_the_button_clicked() {
    let mut table = Table::asked();
    table.quiet_until_presses_count();
    table.frames(true, click(false), FRAME_MS);
    assert_eq!(
        table.answers,
        vec![ScaleUpAnswer {
            slot: GUEST_SLOT,
            yes: false
        }]
    );
}

#[test]
fn a_press_too_short_to_be_deliberate_answers_nothing() {
    let mut table = Table::asked();
    table.quiet_until_presses_count();
    table.frames(
        true,
        keyboard(Gesture::Yes),
        HOLD_TO_ANSWER_MS - 2 * FRAME_MS,
    );
    table.frames(true, PressSample::default(), FRAME_MS);
    table.frames(
        true,
        keyboard(Gesture::Yes),
        HOLD_TO_ANSWER_MS - 2 * FRAME_MS,
    );
    assert!(table.answers.is_empty());
    assert!(table.prompt.is_open());
}

#[test]
fn changing_the_held_answer_starts_the_hold_again() {
    let mut table = Table::asked();
    table.quiet_until_presses_count();
    table.frames(
        true,
        keyboard(Gesture::Yes),
        HOLD_TO_ANSWER_MS - 2 * FRAME_MS,
    );
    table.frames(
        true,
        keyboard(Gesture::No),
        HOLD_TO_ANSWER_MS - 2 * FRAME_MS,
    );
    assert!(table.answers.is_empty());
}

#[test]
fn a_keyboard_yes_with_a_pad_no_at_once_answers_nothing() {
    let mut table = Table::asked();
    table.quiet_until_presses_count();
    let both = PressSample {
        keyboard: Gesture::Yes,
        pad: Gesture::No,
        click: None,
    };
    table.frames(true, both, HOLD_TO_ANSWER_MS * 3);
    assert!(table.answers.is_empty());
}

#[test]
fn a_chord_already_held_when_the_prompt_appears_must_be_let_go_first() {
    let mut table = Table::asked();
    table.frames(
        true,
        keyboard(Gesture::Yes),
        IGNORE_PRESSES_AFTER_SHOWN_MS + 3_000,
    );
    assert!(table.answers.is_empty());
    table.frames(true, PressSample::default(), FRAME_MS);
    table.frames(true, keyboard(Gesture::Yes), HOLD_TO_ANSWER_MS * 2);
    assert_eq!(table.answers.len(), 1);
}

#[test]
fn a_chord_held_through_a_scene_does_not_answer_when_the_prompt_comes_back() {
    let mut table = Table::asked();
    table.quiet_until_presses_count();
    table.frames(false, keyboard(Gesture::Yes), 3_000);
    table.frames(
        true,
        keyboard(Gesture::Yes),
        IGNORE_PRESSES_AFTER_SHOWN_MS + 3_000,
    );
    assert!(table.answers.is_empty());
}

#[test]
fn a_click_in_the_first_moments_after_the_prompt_appears_does_nothing() {
    let mut table = Table::asked();
    table.frames(true, click(true), IGNORE_PRESSES_AFTER_SHOWN_MS / 2);
    assert!(table.answers.is_empty());
    assert!(table.prompt.is_open());
}

#[test]
fn a_prompt_back_after_a_scene_ignores_presses_for_the_same_moments_again() {
    let mut table = Table::asked();
    table.quiet_until_presses_count();
    table.frames(false, PressSample::default(), 2_000);
    table.frames(true, click(true), FRAME_MS);
    table.frames(true, click(true), IGNORE_PRESSES_AFTER_SHOWN_MS / 2);
    assert!(table.answers.is_empty());
}

#[test]
fn the_buttons_are_hidden_from_the_game_exactly_while_the_prompt_shows() {
    let mut table = Table::asked();
    assert!(!table.prompt.wants_input_masked());
    table.quiet_until_presses_count();
    assert!(table.prompt.wants_input_masked());
    table.frames(true, keyboard(Gesture::Yes), HOLD_TO_ANSWER_MS * 2);
    assert_eq!(table.answers.len(), 1);
    assert!(!table.prompt.wants_input_masked());
}

#[test]
fn a_withdrawn_ask_takes_the_prompt_away_and_no_answer_follows() {
    let mut table = Table::asked();
    table.quiet_until_presses_count();
    table.prompt.on_ask(withdrawn_ask());
    assert!(!table.prompt.is_open());
    assert!(table.prompt.shown().is_none());
    table.frames(true, keyboard(Gesture::Yes), HOLD_TO_ANSWER_MS * 3);
    table.frames(true, click(true), FRAME_MS);
    assert!(table.answers.is_empty());
}

#[test]
fn a_withdrawal_for_another_guest_leaves_the_prompt_alone() {
    let mut table = Table::asked();
    table.prompt.on_ask(ScaleUpAsk::Withdrawn { slot: 2 });
    assert!(table.prompt.is_open());
}

#[test]
fn the_same_ask_sent_again_is_one_prompt_and_one_answer() {
    let mut table = Table::asked();
    table.quiet_until_presses_count();
    table.prompt.on_ask(open_ask());
    table.frames(true, click(true), FRAME_MS);
    table.prompt.on_ask(open_ask());
    table.frames(
        true,
        PressSample::default(),
        IGNORE_PRESSES_AFTER_SHOWN_MS * 2,
    );
    assert_eq!(table.answers.len(), 1);
    assert!(table.prompt.shown().is_none());
}

#[test]
fn the_same_ask_sent_again_while_shown_keeps_the_hold_going() {
    let mut table = Table::asked();
    table.quiet_until_presses_count();
    table.frames(true, keyboard(Gesture::Yes), HOLD_TO_ANSWER_MS / 2);
    table.prompt.on_ask(open_ask());
    table.frames(true, keyboard(Gesture::Yes), HOLD_TO_ANSWER_MS);
    assert_eq!(table.answers.len(), 1);
}

#[test]
fn a_guest_who_leaves_and_comes_back_is_asked_about_again() {
    let mut table = Table::asked();
    table.quiet_until_presses_count();
    table.frames(true, click(true), FRAME_MS);
    assert_eq!(table.answers.len(), 1);
    table.prompt.on_ask(withdrawn_ask());
    table.prompt.on_ask(open_ask());
    table.frames(
        true,
        PressSample::default(),
        IGNORE_PRESSES_AFTER_SHOWN_MS + FRAME_MS,
    );
    assert!(table.prompt.shown().is_some());
    table.frames(true, click(false), FRAME_MS);
    assert_eq!(table.answers.len(), 2);
}

#[test]
fn a_relink_drops_the_prompt_and_the_launchers_resent_ask_shows_it_once_more() {
    let mut table = Table::asked();
    table.quiet_until_presses_count();
    table.prompt.clear();
    assert!(!table.prompt.is_open());
    assert!(table.prompt.shown().is_none());
    table.prompt.on_ask(open_ask());
    table.frames(
        true,
        PressSample::default(),
        IGNORE_PRESSES_AFTER_SHOWN_MS + FRAME_MS,
    );
    assert!(table.prompt.shown().is_some());
    table.frames(true, click(true), FRAME_MS);
    assert_eq!(table.answers.len(), 1);
}

#[test]
fn a_second_guest_replaces_the_ask_for_the_first() {
    let mut table = Table::asked();
    table.prompt.on_ask(ScaleUpAsk::Open {
        slot: 2,
        name: "Other".to_owned(),
    });
    table.quiet_until_presses_count();
    table.frames(true, click(true), FRAME_MS);
    assert_eq!(table.answers, vec![ScaleUpAnswer { slot: 2, yes: true }]);
}

#[test]
fn no_save_yet_is_a_log_line_in_the_launchers_words_and_opens_no_prompt() {
    let mut prompt = ScaleUpPrompt::new();
    let effect = prompt.on_ask(ScaleUpAsk::NoSaveYet {
        slot: GUEST_SLOT,
        name: GUEST_NAME.to_owned(),
    });
    assert_eq!(
        effect,
        AskEffect::Line(scale_up_no_save_yet_line(GUEST_NAME))
    );
    assert!(!prompt.is_open());
}

#[test]
fn an_open_or_withdrawn_ask_adds_no_log_line() {
    let mut prompt = ScaleUpPrompt::new();
    assert_eq!(prompt.on_ask(open_ask()), AskEffect::Nothing);
    assert_eq!(prompt.on_ask(withdrawn_ask()), AskEffect::Nothing);
}

#[test]
fn one_key_or_one_button_is_one_answer_and_both_are_none() {
    assert_eq!(keyboard_gesture(true, false), Gesture::Yes);
    assert_eq!(keyboard_gesture(false, true), Gesture::No);
    assert_eq!(keyboard_gesture(true, true), Gesture::Mixed);
    assert_eq!(keyboard_gesture(false, false), Gesture::Idle);
    assert_eq!(pad_gesture(PAD_SCALE_UP_BUTTON), Gesture::Yes);
    assert_eq!(pad_gesture(PAD_NOT_NOW_BUTTON), Gesture::No);
    assert_eq!(pad_gesture(PAD_ANSWER_BUTTONS), Gesture::Mixed);
    assert_eq!(pad_gesture(!PAD_ANSWER_BUTTONS), Gesture::Idle);
}

#[test]
fn the_answer_keys_are_not_bound_in_the_games_default_key_config() {
    const DEFAULT_BOUND_VIRTUAL_KEYS: [u16; 28] = [
        0x20, 0x46, 0xDD, 0xBF, 0x51, 0x45, 0x11, 0x10, 0x50, 0x4D, 0x09, 0x31, 0x33, 0x32, 0x23,
        0x57, 0x53, 0x41, 0x44, 0x26, 0x28, 0x25, 0x27, 0x12, 0x5A, 0x43, 0x42, 0x4E,
    ];
    // defaults from param/xon/key_config, mapped back through the exe's g_VirtualKeyToKeyBit table
    for key in [KEYBOARD_SCALE_UP_VIRTUAL_KEY, KEYBOARD_NOT_NOW_VIRTUAL_KEY] {
        assert!(!DEFAULT_BOUND_VIRTUAL_KEYS.contains(&key));
        assert!((0x70..=0x87).contains(&key), "a function key");
    }
}

struct PadReads {
    hider: AnswerHider,
}

impl PadReads {
    fn new() -> Self {
        Self {
            hider: AnswerHider::new(),
        }
    }

    fn read(&mut self, prompt_up: bool, buttons: u16) -> [u8; XINPUT_STATE_BYTES] {
        let mut state = [0u8; XINPUT_STATE_BYTES];
        state[0..4].copy_from_slice(&7u32.to_le_bytes());
        state[4..6].copy_from_slice(&buttons.to_le_bytes());
        state[6] = 200;
        state[7] = 100;
        state[8..16].copy_from_slice(&[0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x78]);
        self.hider.hide_in_xinput_state(prompt_up, &mut state);
        state
    }
}

fn buttons_of(state: &[u8; XINPUT_STATE_BYTES]) -> u16 {
    u16::from_le_bytes([state[4], state[5]])
}

const LB_RB_A_X: u16 = 0x0100 | 0x0200 | 0x1000 | 0x4000;

#[test]
fn the_games_pad_bytes_lose_only_the_answer_buttons_and_keep_sticks_triggers_and_the_rest() {
    let mut pad = PadReads::new();
    let state = pad.read(true, LB_RB_A_X | PAD_ANSWER_BUTTONS);
    assert_eq!(buttons_of(&state), LB_RB_A_X);
    assert_eq!(&state[0..4], &7u32.to_le_bytes());
    assert_eq!(
        &state[6..16],
        &[200, 100, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x78]
    );
}

#[test]
fn the_first_press_of_an_answer_button_never_reaches_the_game() {
    let mut pad = PadReads::new();
    assert_eq!(buttons_of(&pad.read(true, 0)), 0);
    assert_eq!(buttons_of(&pad.read(true, PAD_SCALE_UP_BUTTON)), 0);
    assert_eq!(buttons_of(&pad.read(true, PAD_NOT_NOW_BUTTON)), 0);
}

#[test]
fn a_button_stays_hidden_after_the_answer_until_it_is_let_go() {
    let mut pad = PadReads::new();
    pad.read(true, PAD_SCALE_UP_BUTTON);
    assert_eq!(buttons_of(&pad.read(false, PAD_SCALE_UP_BUTTON)), 0);
    assert_eq!(
        buttons_of(&pad.read(false, PAD_SCALE_UP_BUTTON | 0x1000)),
        0x1000
    );
    assert_eq!(buttons_of(&pad.read(false, 0)), 0);
    assert_eq!(
        buttons_of(&pad.read(false, PAD_SCALE_UP_BUTTON)),
        PAD_SCALE_UP_BUTTON,
        "a later ordinary press is the game's"
    );
}

#[test]
fn a_button_held_when_a_load_scene_fight_or_withdrawal_takes_the_prompt_stays_hidden_until_up() {
    let mut pad = PadReads::new();
    pad.read(true, 0);
    assert_eq!(buttons_of(&pad.read(false, PAD_NOT_NOW_BUTTON)), 0);
    assert_eq!(buttons_of(&pad.read(false, PAD_NOT_NOW_BUTTON)), 0);
    pad.read(false, 0);
    assert_eq!(
        buttons_of(&pad.read(false, PAD_NOT_NOW_BUTTON)),
        PAD_NOT_NOW_BUTTON
    );
}

#[test]
fn letting_one_button_go_first_does_not_leak_the_other() {
    let mut pad = PadReads::new();
    pad.read(true, PAD_ANSWER_BUTTONS);
    assert_eq!(buttons_of(&pad.read(false, PAD_ANSWER_BUTTONS)), 0);
    assert_eq!(buttons_of(&pad.read(false, PAD_NOT_NOW_BUTTON)), 0);
    assert_eq!(buttons_of(&pad.read(false, 0)), 0);
}

#[test]
fn play_is_untouched_while_no_prompt_is_up_even_with_both_shoulders_and_the_sticks() {
    let mut pad = PadReads::new();
    let state = pad.read(false, LB_RB_A_X);
    assert_eq!(buttons_of(&state), LB_RB_A_X);
    assert_eq!(
        &state[6..16],
        &[200, 100, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x78]
    );
}

#[test]
fn a_short_buffer_is_left_alone() {
    let mut hider = AnswerHider::new();
    let mut short = [0xFFu8; 8];
    hider.hide_in_xinput_state(true, &mut short);
    assert_eq!(short, [0xFFu8; 8]);
}

#[test]
fn ordinary_play_presses_never_make_an_answer() {
    let mut table = Table::asked();
    table.quiet_until_presses_count();
    for buttons in [LB_RB_A_X, 0x2000, 0x0100 | 0x0200, 0x0010, 0x0020] {
        table.frames(true, pad(pad_gesture(buttons)), HOLD_TO_ANSWER_MS * 3);
    }
    assert!(table.answers.is_empty());
}
