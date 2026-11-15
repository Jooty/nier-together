use crate::story_progress::{
    PLAYER_NAME_BLOCK_OFFSET, PLAYER_NAME_BYTES, check_seat, fix_slot_checksum,
};

pub const SEAT_NAME_MAX_CHARS: usize = 10;

pub fn seat_with_player_name(seat: &[u8], name: &str) -> Option<Vec<u8>> {
    check_seat(seat).ok()?;
    let cut = cut_name(name);
    if cut.is_empty() {
        return None;
    }
    let mut renamed = seat.to_vec();
    let block =
        &mut renamed[PLAYER_NAME_BLOCK_OFFSET..PLAYER_NAME_BLOCK_OFFSET + PLAYER_NAME_BYTES];
    block.fill(0);
    block[..cut.len()].copy_from_slice(cut.as_bytes());
    fix_slot_checksum(&mut renamed).then_some(renamed)
}

fn cut_name(name: &str) -> String {
    let mut cut = String::new();
    for letter in name.trim().chars().take(SEAT_NAME_MAX_CHARS) {
        // The last byte of the block stays 0 as the terminator.
        if cut.len() + letter.len_utf8() >= PLAYER_NAME_BYTES {
            break;
        }
        cut.push(letter);
    }
    cut
}
