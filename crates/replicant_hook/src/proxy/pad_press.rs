use std::sync::atomic::{AtomicBool, AtomicU16, AtomicU64, Ordering};
use std::sync::{Mutex, PoisonError};

use replicant_game::scale_up_prompt::{
    AnswerHider, Gesture, PAD_READING_STALE_AFTER_MS, XINPUT_STATE_BYTES, pad_gesture,
};

use crate::net::net_clock_ms;

const PLAYER_ONE_INDEX: u32 = 0;

static LATEST_BUTTONS: AtomicU16 = AtomicU16::new(0);
static LATEST_READ_AT_MS: AtomicU64 = AtomicU64::new(0);
static PROMPT_IS_UP: AtomicBool = AtomicBool::new(false);
static HIDER: Mutex<AnswerHider> = Mutex::new(AnswerHider::new());

pub(crate) fn set_prompt_up(up: bool) {
    PROMPT_IS_UP.store(up, Ordering::Relaxed);
}

pub(crate) fn current_gesture() -> Gesture {
    let age_ms = net_clock_ms().saturating_sub(LATEST_READ_AT_MS.load(Ordering::Relaxed));
    if age_ms > PAD_READING_STALE_AFTER_MS {
        return Gesture::Idle;
    }
    pad_gesture(LATEST_BUTTONS.load(Ordering::Relaxed))
}

pub(super) fn observe_and_hide_answer_buttons(user_index: u32, state_address: usize) {
    if user_index != PLAYER_ONE_INDEX || state_address == 0 {
        return;
    }
    #[expect(
        unsafe_code,
        reason = "views the XINPUT_STATE the game asked for as bytes"
    )]
    // SAFETY: the game passed a writable XINPUT_STATE, which is XINPUT_STATE_BYTES long.
    let state =
        unsafe { std::slice::from_raw_parts_mut(state_address as *mut u8, XINPUT_STATE_BYTES) };
    let raw = HIDER
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .hide_in_xinput_state(PROMPT_IS_UP.load(Ordering::Relaxed), state);
    LATEST_BUTTONS.store(raw, Ordering::Relaxed);
    LATEST_READ_AT_MS.store(net_clock_ms(), Ordering::Relaxed);
}
