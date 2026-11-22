const EXE_SPAN_BYTES: u64 = 0x0400_0000;

pub(super) type ReadWord<'a> = &'a dyn Fn(u64) -> Option<u64>;

pub(super) fn field(read: ReadWord<'_>, base: u64, offset: u64) -> Option<u64> {
    read(base.checked_add(offset)?)
}

pub(super) fn hex_text(value: Option<u64>) -> String {
    value.map_or_else(|| String::from("unreadable"), |word| format!("0x{word:X}"))
}

pub(super) fn address_text(value: Option<u64>, exe_base: u64) -> String {
    match value {
        None => String::from("unreadable"),
        Some(0) => String::from("0"),
        Some(address) if address.wrapping_sub(exe_base) < EXE_SPAN_BYTES => {
            format!("exe rva 0x{:X}", address - exe_base)
        }
        Some(address) => format!("0x{address:X} (outside the game's exe)"),
    }
}
