use crate::coop_pause::MENU_PLAYER_INPUT_BIT;

/// Holds the breaker's body still through a return the way the game's own `_PauseOn` does: the
/// player-input bit in control mask slot 0, where movement reads it (a hook on the mask getter
/// never reaches movement). Only a slot 0 value this hold wrote is ever taken back.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BreakerBodyHold {
    written: Option<u32>,
}

impl BreakerBodyHold {
    pub const fn new() -> Self {
        Self { written: None }
    }

    pub fn is_holding(&self) -> bool {
        self.written.is_some()
    }

    /// The slot 0 value to write now, if any, given what slot 0 holds and whether the return
    /// still holds the body.
    pub fn slot_zero_to_write(&mut self, slot_zero: u32, hold: bool) -> Option<u32> {
        if self.written.is_some_and(|written| written != slot_zero) {
            // The game wrote slot 0 since (a script pause, a menu): that value is the game's.
            self.written = None;
        }
        if hold {
            if slot_zero & MENU_PLAYER_INPUT_BIT != 0 {
                return None;
            }
            let held = slot_zero | MENU_PLAYER_INPUT_BIT;
            self.written = Some(held);
            return Some(held);
        }
        let written = self.written.take()?;
        Some(written & !MENU_PLAYER_INPUT_BIT)
    }
}

pub fn body_held_line(slot_zero: u32, held: u32) -> String {
    format!(
        "rule room: the breaker's body is held still until the fade in ends, as the game's own pause holds it (control mask slot 0 {slot_zero:#x} -> {held:#x})"
    )
}

pub fn body_let_go_line(slot_zero: u32, released: u32) -> String {
    format!(
        "rule room: the breaker's body is let go, the fade in has ended (control mask slot 0 {slot_zero:#x} -> {released:#x})"
    )
}
