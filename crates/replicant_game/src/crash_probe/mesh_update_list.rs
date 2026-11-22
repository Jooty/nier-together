use nt_hook::crash_report::{CrashFacts, CrashRegisters};

use super::readout::{ReadWord, address_text, field, hex_text};

pub const MESH_UPDATE_FUNCTION_START_RVA: u64 = 0x48C10;
pub const MESH_UPDATE_FUNCTION_END_RVA: u64 = 0x496FA;
const MESH_UPDATE_STATE_INSIDE_COMPONENT_BYTES: u64 = 0xF0;
const RELEASED_RESOURCE_POINTER_OFFSET: u64 = 0x8;
const LIST_SENTINEL_ITEM_OFFSET: u64 = 0x72A8;
const LIST_FIRST_CELL_OFFSET: u64 = 0x72B0;
const ASYNC_UPDATE_BYTE_OFFSET: u64 = 0x7545;
const CELL_NEXT_OFFSET: u64 = 8;
const NODE_ACTIVE_OFFSET: u64 = 0x30;
const NODE_CALLBACK_OFFSET: u64 = 0x58;
const NODE_NAME_HASH_OFFSET: u64 = 0x68;
const SMALLEST_REAL_ADDRESS: u64 = 0x1_0000;

pub fn mesh_update_list_lines(facts: &CrashFacts, read: ReadWord<'_>) -> Vec<String> {
    let (Some(registers), Some(exe)) = (facts.registers, facts.module.as_ref()) else {
        return Vec::new();
    };
    let rva = facts.address.wrapping_sub(exe.base);
    if !(MESH_UPDATE_FUNCTION_START_RVA..MESH_UPDATE_FUNCTION_END_RVA).contains(&rva) {
        return Vec::new();
    }
    let state = registers.rbx;
    let component = state.wrapping_sub(MESH_UPDATE_STATE_INSIDE_COMPONENT_BYTES);
    let released = field(read, state, RELEASED_RESOURCE_POINTER_OFFSET);
    let sentinel = field(read, state, LIST_SENTINEL_ITEM_OFFSET);
    let first_cell = field(read, state, LIST_FIRST_CELL_OFFSET);
    let async_byte = field(read, state, ASYNC_UPDATE_BYTE_OFFSET).map(|word| word & 0xFF);
    let vtable = read(component);
    let mut lines = vec![format!(
        "crash: mesh update list: faulted at exe rva 0x{rva:X}; update state 0x{state:X} (component 0x{component:X}, vtable {}), \
         resource pointer {} (0 means the model state was released), async update byte {}, list sentinel item {}, first cell {}",
        address_text(vtable, exe.base),
        hex_text(released),
        hex_text(async_byte),
        hex_text(sentinel),
        hex_text(first_cell),
    )];
    lines.push(first_cell_line(first_cell, sentinel, read));
    lines.push(node_line(&registers, exe.base, read));
    lines
}

fn first_cell_line(first_cell: Option<u64>, sentinel: Option<u64>, read: ReadWord<'_>) -> String {
    let Some(cell) = first_cell.filter(|cell| *cell >= SMALLEST_REAL_ADDRESS) else {
        return String::from("crash: mesh update list: the first cell is not a real address");
    };
    let item = read(cell);
    let next = read(cell + CELL_NEXT_OFFSET);
    format!(
        "crash: mesh update list: first cell 0x{cell:X} holds item {} (sentinel {}) and next {}",
        hex_text(item),
        hex_text(sentinel),
        hex_text(next)
    )
}

fn node_line(registers: &CrashRegisters, exe_base: u64, read: ReadWord<'_>) -> String {
    let node = registers.rdi;
    let callback = field(read, node, NODE_CALLBACK_OFFSET);
    let active = field(read, node, NODE_ACTIVE_OFFSET).map(|word| word & 0xFFFF_FFFF);
    let hash = field(read, node, NODE_NAME_HASH_OFFSET).map(|word| word & 0xFFFF_FFFF);
    format!(
        "crash: mesh update list: bad cell 0x{:X} (the next link of the cell after node 0x{node:X}), node active {}, \
         name hash {}, callback {}",
        registers.r14,
        hex_text(active),
        hex_text(hash),
        address_text(callback, exe_base)
    )
}
