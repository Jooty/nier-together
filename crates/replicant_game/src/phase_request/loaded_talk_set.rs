const TALK_SET_PREFIX: &str = "talk_";

pub fn loaded_talk_set_letter(set_name: &str) -> Option<u8> {
    let letter = set_name.strip_prefix(TALK_SET_PREFIX)?;
    match letter.as_bytes() {
        [single] if single.is_ascii_uppercase() => Some(*single),
        _ => None,
    }
}
