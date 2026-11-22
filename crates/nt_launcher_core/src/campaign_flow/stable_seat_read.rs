use std::io;

use crate::launcher_game::{SeatError, seat_bytes_of};

pub const STABLE_READ_GIVE_UP_SECONDS: f64 = 5.0;
pub const UNUSABLE_SAVE_SETTLE_SECONDS: f64 = 1.0;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StableSeatStep {
    TryAgain,
    Seat(Vec<u8>),
    NoFile,
    Unusable(SeatError),
    KeptChanging,
}

/// Reads of a save file the game may be writing: a seat counts only once the whole file reads
/// the same twice in a row and the seat passes its checks.
#[derive(Clone, Debug, PartialEq)]
pub struct StableSeatRead {
    slot: u8,
    began_at_seconds: f64,
    previous: Option<Vec<u8>>,
    unchanged_since_seconds: f64,
}

impl StableSeatRead {
    pub fn new(slot: u8, now_seconds: f64) -> Self {
        Self {
            slot,
            began_at_seconds: now_seconds,
            previous: None,
            unchanged_since_seconds: now_seconds,
        }
    }

    pub fn step(&mut self, file: io::Result<Vec<u8>>, now_seconds: f64) -> StableSeatStep {
        let bytes = match file {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return StableSeatStep::NoFile,
            Err(_) => {
                self.previous = None;
                return self.try_again_or_give_up(now_seconds);
            }
        };
        if self.previous.as_ref() != Some(&bytes) {
            self.previous = Some(bytes);
            self.unchanged_since_seconds = now_seconds;
            return self.try_again_or_give_up(now_seconds);
        }
        match seat_bytes_of(&bytes, self.slot) {
            Ok(seat) => StableSeatStep::Seat(seat),
            Err(error)
                if now_seconds - self.unchanged_since_seconds >= UNUSABLE_SAVE_SETTLE_SECONDS =>
            {
                StableSeatStep::Unusable(error)
            }
            Err(_) => self.try_again_or_give_up(now_seconds),
        }
    }

    fn try_again_or_give_up(&self, now_seconds: f64) -> StableSeatStep {
        if now_seconds - self.began_at_seconds >= STABLE_READ_GIVE_UP_SECONDS {
            StableSeatStep::KeptChanging
        } else {
            StableSeatStep::TryAgain
        }
    }
}
