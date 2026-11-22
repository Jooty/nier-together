use nt_hook::crash_report::{CrashFacts, CrashRegisters};

use super::readout::{ReadWord, field, hex_text};

pub const PPPI_UPDATE_FUNCTION_START_RVA: u64 = 0x72F540;
pub const PPPI_UPDATE_FUNCTION_END_RVA: u64 = 0x72F6B5;
const PPPI_UPDATE_FIRST_RVA_WITH_STACK_ARGUMENTS_READY: u64 = 0x72F54A;
const OUTPUT_WORKSPACE_STACK_OFFSET: u64 = 0x88;
const OUTPUT_WORKSPACE_INSIDE_MANAGER_BYTES: u64 = 0x18;
const MANAGER_INSIDE_SYSTEM_BYTES: u64 = 0x428;
const MANAGER_MAP_HEAD_OFFSET: u64 = 0x8;
const MANAGER_MAP_COUNT_OFFSET: u64 = 0x10;
const MAP_NODE_VALUE_OFFSET: u64 = 0x20;
const SYSTEM_JOB_PENDING_BYTE_OFFSET: u64 = 0x3B0;
const SYSTEM_STOP_BYTE_OFFSET: u64 = 0x3E0;
const SYSTEM_PROBE_ENTRIES_OFFSET: u64 = 0xD2B0;
const SYSTEM_PROBE_ENTRY_COUNT_OFFSET: u64 = 0xD2C8;
const COMPILED_SET_SOURCE_POINTER_OFFSET: u64 = 0x10;
const SOURCE_PROBE_COUNT_OFFSET: u64 = 0x14;
const WORKSPACE_MAGIC_OFFSET: u64 = 0x10D8;
const WORKSPACE_MAGIC: u64 = 0xA619_F229;
const COMPILED_SET_WORD_OFFSETS: [u64; 4] = [0x0, 0x8, 0x10, 0x18];

pub fn pppi_probe_set_update_lines(facts: &CrashFacts, read: ReadWord<'_>) -> Vec<String> {
    let (Some(registers), Some(exe)) = (facts.registers, facts.module.as_ref()) else {
        return Vec::new();
    };
    let rva = facts.address.wrapping_sub(exe.base);
    if !(PPPI_UPDATE_FUNCTION_START_RVA..PPPI_UPDATE_FUNCTION_END_RVA).contains(&rva) {
        return Vec::new();
    }
    vec![
        compiled_set_line(rva, &registers, read),
        owner_line(rva, &registers, read),
    ]
}

fn compiled_set_line(rva: u64, registers: &CrashRegisters, read: ReadWord<'_>) -> String {
    let compiled = registers.r8;
    let words: Vec<String> = COMPILED_SET_WORD_OFFSETS
        .iter()
        .map(|offset| hex_text(field(read, compiled, *offset)))
        .collect();
    let source = field(read, compiled, COMPILED_SET_SOURCE_POINTER_OFFSET);
    let source_probe_count = source
        .and_then(|pointer| field(read, pointer, SOURCE_PROBE_COUNT_OFFSET))
        .map(|word| word & 0xFFFF_FFFF);
    format!(
        "crash: pppi probe set update: faulted at exe rva 0x{rva:X}; compiled probe set 0x{compiled:X} words [{}], \
         its source probe set pointer {} (rax 0x{:X}), source probe count {}",
        words.join(", "),
        hex_text(source),
        registers.rax,
        hex_text(source_probe_count)
    )
}

fn owner_line(rva: u64, registers: &CrashRegisters, read: ReadWord<'_>) -> String {
    let workspace = registers.r9;
    let magic = field(read, workspace, WORKSPACE_MAGIC_OFFSET).map(|word| word & 0xFFFF_FFFF);
    let magic_text = match magic {
        Some(WORKSPACE_MAGIC) => String::from("intact"),
        other => format!("{} (corrupted or freed)", hex_text(other)),
    };
    let output_workspace = (rva >= PPPI_UPDATE_FIRST_RVA_WITH_STACK_ARGUMENTS_READY)
        .then(|| field(read, registers.rsp, OUTPUT_WORKSPACE_STACK_OFFSET))
        .flatten();
    let Some(manager) = output_workspace
        .map(|pointer| pointer.wrapping_sub(OUTPUT_WORKSPACE_INSIDE_MANAGER_BYTES))
        .filter(|manager| *manager >= MANAGER_INSIDE_SYSTEM_BYTES)
    else {
        return format!(
            "crash: pppi probe set update: manager unknown (stack argument unreadable), workspace 0x{workspace:X} magic {magic_text}"
        );
    };
    let system = manager - MANAGER_INSIDE_SYSTEM_BYTES;
    let head = field(read, manager, MANAGER_MAP_HEAD_OFFSET);
    let head_value = head.and_then(|node| field(read, node, MAP_NODE_VALUE_OFFSET));
    let lookup_missed = head_value.map(|value| value == registers.r8);
    format!(
        "crash: pppi probe set update: manager 0x{manager:X} in system 0x{system:X}, map head {} with value {} \
         (compiled set is the head's value, so the lookup missed: {}), map count {}, worker stop byte {}, \
         job pending byte {}, probe entries {} count {}, workspace 0x{workspace:X} magic {magic_text}",
        hex_text(head),
        hex_text(head_value),
        lookup_missed.map_or_else(|| String::from("unknown"), |missed| missed.to_string()),
        hex_text(field(read, manager, MANAGER_MAP_COUNT_OFFSET).map(|word| word & 0xFFFF_FFFF)),
        hex_text(field(read, system, SYSTEM_STOP_BYTE_OFFSET).map(|word| word & 0xFF)),
        hex_text(field(read, system, SYSTEM_JOB_PENDING_BYTE_OFFSET).map(|word| word & 0xFF)),
        hex_text(field(read, system, SYSTEM_PROBE_ENTRIES_OFFSET)),
        hex_text(
            field(read, system, SYSTEM_PROBE_ENTRY_COUNT_OFFSET).map(|word| word & 0xFFFF_FFFF)
        ),
    )
}
