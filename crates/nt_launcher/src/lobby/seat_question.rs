use std::io;

use nt_launcher_core::{StableSeatRead, StableSeatStep, seat_error_words};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuestionStep {
    Waiting,
    Ask,
    NoCompletedSave(String),
    Copy(Vec<u8>),
    CopyFailed(String),
    GuestStartsOwnGame,
}

#[derive(Debug, Clone, PartialEq)]
enum QuestionStage {
    CheckingSave(Option<StableSeatRead>),
    Asked,
    CopyingSave(Option<StableSeatRead>),
}

/// The host's decision about giving a late guest without a save a copy of the host's own.
#[derive(Debug, Clone, PartialEq)]
pub struct SeatQuestion {
    member_slot: u8,
    guest_name: String,
    host_slot: u8,
    seat_start_mode: Option<u8>,
    stage: QuestionStage,
}

impl SeatQuestion {
    pub fn new(
        member_slot: u8,
        guest_name: &str,
        host_slot: u8,
        seat_start_mode: Option<u8>,
    ) -> Self {
        Self {
            member_slot,
            guest_name: guest_name.trim().to_owned(),
            host_slot,
            seat_start_mode,
            stage: QuestionStage::CheckingSave(None),
        }
    }

    pub fn member_slot(&self) -> u8 {
        self.member_slot
    }

    pub fn guest_name(&self) -> &str {
        &self.guest_name
    }

    pub fn seat_start_mode(&self) -> Option<u8> {
        self.seat_start_mode
    }

    pub fn is_asked(&self) -> bool {
        self.stage == QuestionStage::Asked
    }

    pub fn needs_the_save_file(&self) -> bool {
        matches!(
            self.stage,
            QuestionStage::CheckingSave(_) | QuestionStage::CopyingSave(_)
        )
    }

    pub fn read_file(&mut self, file: io::Result<Vec<u8>>, now_seconds: f64) -> QuestionStep {
        let host_slot = self.host_slot;
        let (reading, copying) = match &mut self.stage {
            QuestionStage::CheckingSave(reading) => (reading, false),
            QuestionStage::CopyingSave(reading) => (reading, true),
            QuestionStage::Asked => return QuestionStep::Waiting,
        };
        let step = reading
            .get_or_insert_with(|| StableSeatRead::new(host_slot, now_seconds))
            .step(file, now_seconds);
        match step {
            StableSeatStep::TryAgain => QuestionStep::Waiting,
            StableSeatStep::Seat(seat) if copying => QuestionStep::Copy(seat),
            StableSeatStep::Seat(_) => {
                self.stage = QuestionStage::Asked;
                QuestionStep::Ask
            }
            StableSeatStep::NoFile => no_save_step(copying, "there is no co-op save file yet"),
            StableSeatStep::Unusable(error) => no_save_step(copying, seat_error_words(error)),
            StableSeatStep::KeptChanging => no_save_step(
                copying,
                "the save file kept changing, so no whole copy could be read",
            ),
        }
    }

    pub fn answered(&mut self, yes: bool) -> QuestionStep {
        if self.stage != QuestionStage::Asked {
            return QuestionStep::Waiting;
        }
        if yes {
            self.stage = QuestionStage::CopyingSave(None);
            QuestionStep::Waiting
        } else {
            QuestionStep::GuestStartsOwnGame
        }
    }
}

fn no_save_step(copying: bool, reason: &str) -> QuestionStep {
    if copying {
        QuestionStep::CopyFailed(reason.to_owned())
    } else {
        QuestionStep::NoCompletedSave(reason.to_owned())
    }
}
