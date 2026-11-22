use std::io;

use nt_launcher_core::{
    STABLE_READ_GIVE_UP_SECONDS, SeatError, StableSeatRead, StableSeatStep,
    UNUSABLE_SAVE_SETTLE_SECONDS,
};
use replicant_game::story_progress::{
    SAVE_FILE_BYTES, SLOT_BYTES, SLOTS_FILE_OFFSET, fix_slot_checksum,
};

const HOST_SLOT: u8 = 2;
const FRAME_SECONDS: f64 = 0.016;

fn file_with_a_save(seed: u8) -> Vec<u8> {
    let mut file = vec![0_u8; SAVE_FILE_BYTES];
    let start = SLOTS_FILE_OFFSET + usize::from(HOST_SLOT) * SLOT_BYTES;
    let slot = &mut file[start..start + SLOT_BYTES];
    slot[..4].copy_from_slice(&200_u32.to_le_bytes());
    slot[4] = b'P';
    slot[0x300] = seed;
    assert!(fix_slot_checksum(slot));
    file
}

fn seat_of(file: &[u8]) -> Vec<u8> {
    let start = SLOTS_FILE_OFFSET + usize::from(HOST_SLOT) * SLOT_BYTES;
    file[start..start + SLOT_BYTES].to_vec()
}

#[test]
fn a_whole_save_is_used_only_after_it_reads_the_same_twice() {
    let file = file_with_a_save(1);
    let mut read = StableSeatRead::new(HOST_SLOT, 0.0);
    assert_eq!(
        read.step(Ok(file.clone()), 0.0),
        StableSeatStep::TryAgain,
        "one read proves nothing"
    );
    assert_eq!(
        read.step(Ok(file.clone()), FRAME_SECONDS),
        StableSeatStep::Seat(seat_of(&file))
    );
}

#[test]
fn a_half_written_save_is_never_used_and_the_finished_one_is() {
    let finished = file_with_a_save(7);
    let mut torn = finished.clone();
    let start = SLOTS_FILE_OFFSET + usize::from(HOST_SLOT) * SLOT_BYTES;
    torn[start + 0x300..start + 0x400].fill(0xEE);
    let truncated = finished[..SAVE_FILE_BYTES / 2].to_vec();
    let mut read = StableSeatRead::new(HOST_SLOT, 0.0);
    let mut now = 0.0;
    for partial in [truncated, torn.clone(), torn] {
        let step = read.step(Ok(partial), now);
        assert!(
            matches!(step, StableSeatStep::TryAgain),
            "a torn read gave {step:?}"
        );
        now += FRAME_SECONDS;
    }
    assert_eq!(
        read.step(Ok(finished.clone()), now),
        StableSeatStep::TryAgain,
        "the finished file has to read the same twice as well"
    );
    assert_eq!(
        read.step(Ok(finished.clone()), now + FRAME_SECONDS),
        StableSeatStep::Seat(seat_of(&finished))
    );
}

#[test]
fn a_save_that_changes_on_every_read_is_given_up_on() {
    let mut read = StableSeatRead::new(HOST_SLOT, 10.0);
    let mut now = 10.0;
    let mut seed = 1_u8;
    loop {
        let step = read.step(Ok(file_with_a_save(seed)), now);
        seed = seed.wrapping_add(1);
        now += FRAME_SECONDS;
        match step {
            StableSeatStep::TryAgain => assert!(now < 10.0 + STABLE_READ_GIVE_UP_SECONDS + 1.0),
            StableSeatStep::KeptChanging => break,
            other => panic!("a changing file gave {other:?}"),
        }
    }
}

#[test]
fn a_missing_file_has_no_save() {
    let mut read = StableSeatRead::new(HOST_SLOT, 0.0);
    let missing = io::Error::from(io::ErrorKind::NotFound);
    assert_eq!(read.step(Err(missing), 0.0), StableSeatStep::NoFile);
}

#[test]
fn a_file_that_cannot_be_opened_right_now_is_tried_again() {
    let mut read = StableSeatRead::new(HOST_SLOT, 0.0);
    let busy = io::Error::from(io::ErrorKind::PermissionDenied);
    assert_eq!(read.step(Err(busy), 0.0), StableSeatStep::TryAgain);
}

#[test]
fn an_empty_slot_stays_unusable_only_once_the_file_has_stopped_changing() {
    let mut file = file_with_a_save(1);
    let start = SLOTS_FILE_OFFSET + usize::from(HOST_SLOT) * SLOT_BYTES;
    file[start..start + SLOT_BYTES].fill(0);
    let mut read = StableSeatRead::new(HOST_SLOT, 0.0);
    assert_eq!(read.step(Ok(file.clone()), 0.0), StableSeatStep::TryAgain);
    assert_eq!(
        read.step(Ok(file.clone()), UNUSABLE_SAVE_SETTLE_SECONDS / 2.0),
        StableSeatStep::TryAgain,
        "a torn write can look empty for a moment"
    );
    let step = read.step(Ok(file), UNUSABLE_SAVE_SETTLE_SECONDS + 0.1);
    assert!(
        matches!(step, StableSeatStep::Unusable(SeatError::NotAGameSlot)),
        "{step:?}"
    );
}
