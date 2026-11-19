use core::ffi::c_void;
use core::ptr;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, AtomicPtr, AtomicU32, Ordering};

use automata_game::address::{INPUT_UPDATE_TAIL_FUNCTION, PAD_SNAPSHOT_THE_GAME_READS};
use automata_game::pad_press::{
    BACK_BUTTON_PAD_MASK, INPUT_UPDATE_TAIL_BYTES_AT_OFFSET_SIX, INPUT_UPDATE_TAIL_FIRST_BYTES,
    INPUT_UPDATE_TAIL_LEN, INPUT_UPDATE_TAIL_RETURN_OFFSET, PAD_UPDATES_A_PRESS_TAKES,
    pad_words_after_press,
};
use automata_game::partner::code_starts_with;
use nt_hook::diagnostics::log_line;

use crate::game_calls::GameCalls;

const RETURN_INSTRUCTION: u8 = 0xC3;
const BYTES_BEFORE_THE_FIVE_AT_OFFSET_SIX: usize = 6;

static ORIGINAL_INPUT_UPDATE_TAIL: AtomicPtr<c_void> = AtomicPtr::new(ptr::null_mut());
static PRESS_CALLS: OnceLock<GameCalls> = OnceLock::new();
static PAD_UPDATES_LEFT: AtomicU32 = AtomicU32::new(0);
static HOOKED: AtomicBool = AtomicBool::new(false);
static FIRST_WRITE_LOGGED: AtomicBool = AtomicBool::new(false);

type InputUpdateTail = unsafe extern "system" fn(u32) -> u32;

/// Game thread: asks for one press of the back button on the pad snapshot; false when not hooked.
pub fn press_back_on_the_pad() -> bool {
    if !HOOKED.load(Ordering::Acquire) {
        return false;
    }
    PAD_UPDATES_LEFT.store(PAD_UPDATES_A_PRESS_TAKES, Ordering::Release);
    true
}

/// Hooks the last call of the game's input update, which runs after the pad snapshot is filled.
pub fn install_pad_press(calls: GameCalls) -> bool {
    let start_matches = calls
        .read_bytes(INPUT_UPDATE_TAIL_FUNCTION, INPUT_UPDATE_TAIL_LEN)
        .is_ok_and(|found| tail_bytes_are_the_noted_ones(&found));
    if !start_matches || PRESS_CALLS.set(calls).is_err() {
        log_line(
            "pad press: the input update's last call is not the noted code, nothing hooked; a menu the host's pause opened here is not closed for the player",
        );
        return false;
    }
    #[expect(
        unsafe_code,
        reason = "hooks the last call of the game's input update; the replacement has the signature of the original"
    )]
    // SAFETY: the address passed the byte checks for this function of the verified build; the
    // replacement takes the same one integer argument and calls the original first.
    let hooked = unsafe {
        nt_hook::detour::hook_address(
            "input update's last call (back press on the pad snapshot, whole function)",
            calls.address_value(INPUT_UPDATE_TAIL_FUNCTION) as usize,
            input_update_tail_detour as *const () as *mut c_void,
            &ORIGINAL_INPUT_UPDATE_TAIL,
        )
    };
    HOOKED.store(hooked, Ordering::Release);
    hooked
}

fn tail_bytes_are_the_noted_ones(found: &[u8]) -> bool {
    code_starts_with(found, &INPUT_UPDATE_TAIL_FIRST_BYTES)
        && found.get(BYTES_BEFORE_THE_FIVE_AT_OFFSET_SIX..BYTES_BEFORE_THE_FIVE_AT_OFFSET_SIX + 5)
            == Some(&INPUT_UPDATE_TAIL_BYTES_AT_OFFSET_SIX[..])
        && found.get(INPUT_UPDATE_TAIL_RETURN_OFFSET) == Some(&RETURN_INSTRUCTION)
}

#[expect(
    unsafe_code,
    reason = "calls the original through the trampoline MinHook built"
)]
extern "system" fn input_update_tail_detour(argument: u32) -> u32 {
    let original = ORIGINAL_INPUT_UPDATE_TAIL.load(Ordering::Acquire);
    if original.is_null() {
        return 1;
    }
    // SAFETY: the trampoline continues into the original function with the argument received.
    let answer =
        unsafe { core::mem::transmute::<*mut c_void, InputUpdateTail>(original)(argument) };
    let left = PAD_UPDATES_LEFT.load(Ordering::Acquire);
    if left > 0 {
        PAD_UPDATES_LEFT.store(left - 1, Ordering::Release);
        write_the_press(left);
    }
    answer
}

fn write_the_press(updates_left: u32) {
    let Some(calls) = PRESS_CALLS.get() else {
        return;
    };
    let address = calls.address_value(PAD_SNAPSHOT_THE_GAME_READS);
    let Ok(words) = calls.read_u32s_at::<3>(address) else {
        return;
    };
    let Some(after) = pad_words_after_press(updates_left, BACK_BUTTON_PAD_MASK, words) else {
        return;
    };
    let written = calls.write_u32s_at(address, &after).is_ok();
    if !FIRST_WRITE_LOGGED.swap(true, Ordering::Relaxed) {
        log_line(&format!(
            "pad press: the first back press is written on the pad snapshot the game reads (held, pressed, released {after:#X?}, written {written})"
        ));
    }
}
