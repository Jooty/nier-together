use core::ffi::c_void;
use std::sync::{Mutex, PoisonError};

use replicant_game::GameMemory;
use replicant_game::dialogue::{
    CONTROL_MASK_SET_SLOT_ZERO_RVA, GAME_STATE_RVA, LinePress, LineSteps, PressStep,
    TALK_DATA_LINE_COUNT_OFFSET, TALK_UPDATE_INPUT_BUTTON_PRESSED_BIT, TALK_UPDATE_INPUT_BYTES,
    TALK_UPDATE_INPUT_CONFIRM_OFFSET, TALK_UPDATE_INPUT_ENABLED_OFFSET, TALK_WINDOW_CLOSE_RVA,
    TALK_WINDOW_CURRENT_LINE_OFFSET, TALK_WINDOW_CURRENT_TALK_OFFSET,
    TALK_WINDOW_MASK_BEFORE_OPEN_OFFSET, TALK_WINDOW_OPEN_RVA, TALK_WINDOW_PAGE_OFFSET,
    TALK_WINDOW_PLAYER_OFFSET, TALK_WINDOW_RVA, TalkWindowReading,
};
use replicant_proto::DialogueLine;

use crate::build_gate::VerifiedBuild;
use crate::diagnostics::log_line;
use crate::memory::InProcessMemory;
use crate::net::net_clock_ms;

static LINE_PRESS: Mutex<LinePress> = Mutex::new(LinePress::new());
static LINE_STEPS: Mutex<LineSteps> = Mutex::new(LineSteps::new());
static TALK_TO_CHECK: Mutex<Option<DialogueLine>> = Mutex::new(None);
static TALK_TO_OPEN: Mutex<Option<DialogueLine>> = Mutex::new(None);

type OpenTalk = unsafe extern "system" fn(*mut c_void, i32);
type SetMaskSlot = unsafe extern "system" fn(*mut c_void, u32);
type CloseTalk = unsafe extern "system" fn(*mut c_void, u8);

/// Opens this game's talk window on the partner's talk without taking this player's controls,
/// then asks for the partner's line. Runs on the game thread. False when the game refused.
#[expect(
    unsafe_code,
    reason = "calls TalkWindow::Open and the control mask setter by address"
)]
pub(super) fn open_mirror(build: &VerifiedBuild, line: DialogueLine) -> bool {
    let memory = InProcessMemory::new(build);
    let base = memory.module_base();
    let window = base + u64::from(TALK_WINDOW_RVA);
    let Ok(talk_id) = i32::try_from(line.talk_id) else {
        return false;
    };
    // SAFETY: TalkWindow::Open(window, talkId) in the verified build, called on the game thread
    // like `_StartTalk` does.
    unsafe {
        core::mem::transmute::<usize, OpenTalk>(address(base, TALK_WINDOW_OPEN_RVA))(
            window as *mut c_void,
            talk_id,
        );
    }
    if memory.read_i32(window).is_ok_and(|state| state == 0) {
        return false;
    }
    // Open ORs 0x20 into mask slot 0 (pausing this player's systems); put back what it saved.
    if let Ok(mask_before) = memory.read_i32(window + TALK_WINDOW_MASK_BEFORE_OPEN_OFFSET) {
        // SAFETY: the slot 0 setter (game state, value) in the verified build, game thread.
        unsafe {
            core::mem::transmute::<usize, SetMaskSlot>(address(
                base,
                CONTROL_MASK_SET_SLOT_ZERO_RVA,
            ))(
                (base + u64::from(GAME_STATE_RVA)) as *mut c_void,
                mask_before.cast_unsigned(),
            );
        }
    }
    if line.line > 0 {
        want_line(&memory, line, true);
    }
    true
}

/// The talk window as the game's own update is about to see it; also counts the line's steps,
/// so it is read once per frame, from the window's update.
pub(super) fn read_talk_window(memory: &InProcessMemory) -> TalkWindowReading {
    let window = memory.module_base() + u64::from(TALK_WINDOW_RVA);
    let open = memory.read_i32(window).is_ok_and(|state| state != 0);
    let talk_address = memory
        .read_u64(window + TALK_WINDOW_CURRENT_TALK_OFFSET)
        .unwrap_or(0);
    let line_count = if talk_address == 0 {
        0
    } else {
        memory
            .read_i32(talk_address + TALK_DATA_LINE_COUNT_OFFSET)
            .ok()
            .and_then(|count| u16::try_from(count).ok())
            .unwrap_or(0)
    };
    let small = |offset: u64| {
        memory
            .read_i32(window + offset)
            .ok()
            .and_then(|value| u16::try_from(value).ok())
            .unwrap_or(0)
    };
    let line = small(TALK_WINDOW_CURRENT_LINE_OFFSET);
    let page = small(TALK_WINDOW_PAGE_OFFSET);
    let player_state = memory
        .read_i32(window + TALK_WINDOW_PLAYER_OFFSET)
        .unwrap_or(0);
    let mut steps = LINE_STEPS.lock().unwrap_or_else(PoisonError::into_inner);
    let step = if open {
        steps.observe(talk_address, line, page, player_state)
    } else {
        steps.forget();
        0
    };
    TalkWindowReading {
        open,
        talk_address,
        line,
        step,
        line_count,
        player_state,
    }
}

/// The press count inside the line this window is on, as counted at the last update.
pub(super) fn current_step(memory: &InProcessMemory) -> u16 {
    let window = memory.module_base() + u64::from(TALK_WINDOW_RVA);
    let small = |offset: u64| {
        memory
            .read_i32(window + offset)
            .ok()
            .and_then(|value| u16::try_from(value).ok())
            .unwrap_or(0)
    };
    let talk_address = memory
        .read_u64(window + TALK_WINDOW_CURRENT_TALK_OFFSET)
        .unwrap_or(0);
    let player_state = memory
        .read_i32(window + TALK_WINDOW_PLAYER_OFFSET)
        .unwrap_or(0);
    LINE_STEPS
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .observe(
            talk_address,
            small(TALK_WINDOW_CURRENT_LINE_OFFSET),
            small(TALK_WINDOW_PAGE_OFFSET),
            player_state,
        )
}

/// Asks for the partner's line and step: the window's update presses confirm, as the partner's
/// own presses did, until the window is there. `on_this_talk` when the window already holds that talk.
pub(super) fn want_line(memory: &InProcessMemory, line: DialogueLine, on_this_talk: bool) {
    let reading = read_talk_window(memory);
    let mut press = LINE_PRESS.lock().unwrap_or_else(PoisonError::into_inner);
    let before = press.wanted();
    press.want(
        line.talk_id,
        line.line,
        line.step,
        on_this_talk,
        reading.talk_address,
        net_clock_ms(),
    );
    if press.wanted() == before {
        return;
    }
    log_line(&format!(
        "dialogue: the partner is on line {} step {} of talk {}, this window (on line {} step {} of {} lines, player state {}) presses confirm in the game's own update until it is too{}",
        line.line,
        line.step,
        line.talk_id,
        reading.line,
        reading.step,
        reading.line_count,
        reading.player_state,
        if on_this_talk {
            ""
        } else {
            ", past the last line of the talk it holds"
        }
    ));
}

pub(super) fn clear_wanted_line() {
    LINE_PRESS
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .clear();
}

/// Runs on the game thread inside the window's update: writes the confirm press into `input` when
/// the window is behind the wanted line. True when a press was written.
pub(super) fn press_for_the_wanted_line(
    reading: &TalkWindowReading,
    input: &mut [u8; TALK_UPDATE_INPUT_BYTES],
) -> bool {
    let step = LINE_PRESS
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .step(reading, net_clock_ms());
    match step {
        Some(PressStep::Press) => {
            input[TALK_UPDATE_INPUT_ENABLED_OFFSET] = 1;
            input[TALK_UPDATE_INPUT_CONFIRM_OFFSET] = TALK_UPDATE_INPUT_BUTTON_PRESSED_BIT;
            true
        }
        Some(PressStep::Reached {
            talk_id,
            line,
            presses,
        }) => {
            log_line(&format!(
                "dialogue: this window reached line {line} step {} of talk {talk_id} after {presses} confirm press(es) through the game's own update",
                reading.step
            ));
            false
        }
        Some(PressStep::TalkChanged {
            talk_id,
            line,
            presses,
        }) => {
            *TALK_TO_CHECK.lock().unwrap_or_else(PoisonError::into_inner) = Some(DialogueLine {
                talk_id,
                line,
                step: 0,
            });
            log_line(&format!(
                "dialogue: this window went on to another talk after {presses} confirm press(es), checking it is talk {talk_id}"
            ));
            false
        }
        Some(PressStep::OpenInstead {
            talk_id,
            line,
            presses,
        }) => {
            *TALK_TO_OPEN.lock().unwrap_or_else(PoisonError::into_inner) = Some(DialogueLine {
                talk_id,
                line,
                step: 0,
            });
            log_line(&format!(
                "dialogue: {presses} confirm press(es) did not take this window on to talk {talk_id}, it is opened in the window instead"
            ));
            false
        }
        Some(PressStep::GaveUp {
            talk_id,
            line,
            presses,
        }) => {
            log_line(&format!(
                "dialogue: gave up pressing for line {line} of talk {talk_id} after {presses} press(es): the window is on line {} step {} of {} lines with player state {}",
                reading.line, reading.step, reading.line_count, reading.player_state
            ));
            false
        }
        Some(PressStep::Closed { talk_id, line }) => {
            log_line(&format!(
                "dialogue: the window closed before line {line} of talk {talk_id} was reached"
            ));
            false
        }
        None => false,
    }
}

pub(super) fn take_talk_to_check() -> Option<DialogueLine> {
    TALK_TO_CHECK
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .take()
}

pub(super) fn take_talk_to_open() -> Option<DialogueLine> {
    TALK_TO_OPEN
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .take()
}

#[expect(unsafe_code, reason = "calls TalkWindow::Close by address")]
pub(super) fn close_mirror(build: &VerifiedBuild) {
    clear_wanted_line();
    let base = build.module_base();
    // SAFETY: TalkWindow::Close(window, immediately) in the verified build, game thread.
    unsafe {
        core::mem::transmute::<usize, CloseTalk>(address(base, TALK_WINDOW_CLOSE_RVA))(
            (base + u64::from(TALK_WINDOW_RVA)) as *mut c_void,
            1,
        );
    }
}

fn address(base: u64, rva: u32) -> usize {
    (base + u64::from(rva)) as usize
}
