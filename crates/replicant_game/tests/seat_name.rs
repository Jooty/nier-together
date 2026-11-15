use replicant_game::story_progress::{
    SEAT_NAME_MAX_CHARS, check_seat, fix_slot_checksum, seat_with_player_name, slot_player_name,
};

const SLOT_BYTES: usize = 0x9260;
const NAME_OFFSET: usize = 0x2C;
const NAME_BYTES: usize = 0x20;
const CHECKSUM_OFFSET: usize = 0x9250;

fn host_slot() -> Vec<u8> {
    let mut slot = vec![0_u8; SLOT_BYTES];
    slot[..4].copy_from_slice(&200_u32.to_le_bytes());
    slot[4] = b'P';
    slot[NAME_OFFSET..NAME_OFFSET + 4].copy_from_slice(b"Host");
    for (index, byte) in slot[0x100..0x900].iter_mut().enumerate() {
        *byte = index as u8;
    }
    assert!(fix_slot_checksum(&mut slot));
    slot
}

fn name_in(slot: &[u8]) -> String {
    let block = slot_player_name(slot).unwrap();
    let end = block.iter().position(|&byte| byte == 0).unwrap();
    String::from_utf8(block[..end].to_vec()).unwrap()
}

#[test]
fn the_guests_name_is_written_into_the_name_block() {
    let renamed = seat_with_player_name(&host_slot(), "Mira").unwrap();
    assert_eq!(name_in(&renamed), "Mira");
    assert!(
        slot_player_name(&renamed).unwrap()[4..]
            .iter()
            .all(|&byte| byte == 0)
    );
}

#[test]
fn the_renamed_seat_has_a_valid_checksum() {
    let renamed = seat_with_player_name(&host_slot(), "Mira").unwrap();
    assert_eq!(check_seat(&renamed), Ok(()));
}

#[test]
fn every_byte_outside_the_name_and_checksum_is_the_hosts() {
    let host = host_slot();
    let renamed = seat_with_player_name(&host, "Mira").unwrap();
    for (index, (before, after)) in host.iter().zip(&renamed).enumerate() {
        let in_name = (NAME_OFFSET..NAME_OFFSET + NAME_BYTES).contains(&index);
        let in_checksum = (CHECKSUM_OFFSET..CHECKSUM_OFFSET + 4).contains(&index);
        if !in_name && !in_checksum {
            assert_eq!(before, after, "byte {index:#x}");
        }
    }
}

#[test]
fn a_long_name_is_cut_to_the_name_limit() {
    let renamed = seat_with_player_name(&host_slot(), "Abcdefghijklmnop").unwrap();
    assert_eq!(name_in(&renamed), "Abcdefghij");
    assert_eq!(SEAT_NAME_MAX_CHARS, 10);
}

#[test]
fn a_name_that_does_not_fit_the_block_in_bytes_is_cut_on_a_letter() {
    let renamed = seat_with_player_name(
        &host_slot(),
        "\u{1F600}\u{1F600}\u{1F600}\u{1F600}\u{1F600}\u{1F600}\u{1F600}\u{1F600}\u{1F600}",
    )
    .unwrap();
    assert_eq!(name_in(&renamed).chars().count(), 7);
    assert_eq!(check_seat(&renamed), Ok(()));
}

#[test]
fn an_empty_name_or_a_damaged_seat_is_refused() {
    assert!(seat_with_player_name(&host_slot(), "   ").is_none());
    assert!(seat_with_player_name(&host_slot()[1..], "Mira").is_none());
}
