use std::collections::HashMap;

use nt_hook::crash_report::{CrashFacts, CrashRegisters, ModuleSpot};
use replicant_game::crash_probe::mesh_update_list_lines;

const EXE_BASE: u64 = 0x1_4000_0000;
const FAULT_RVA: u64 = 0x496BD;
const STATE: u64 = 0x2000_0000;
const FIRST_CELL: u64 = 0x3000_0000;
const NODE: u64 = 0x4000_0000;

fn facts(rva: u64, registers: Option<CrashRegisters>) -> CrashFacts {
    CrashFacts {
        code: 0xC000_0005,
        address: EXE_BASE + rva,
        thread_id: 972,
        game_thread_id: Some(40),
        registers,
        module: Some(ModuleSpot {
            name: "NieR Replicant ver.1.22474487139.exe".to_owned(),
            base: EXE_BASE,
        }),
        in_our_module: false,
        access: Some((0, u64::MAX)),
        last_chance: false,
        stack: Vec::new(),
    }
}

fn registers() -> CrashRegisters {
    CrashRegisters {
        rbx: STATE,
        rdi: NODE,
        r14: u64::MAX,
        ..CrashRegisters::default()
    }
}

fn memory() -> HashMap<u64, u64> {
    HashMap::from([
        (STATE + 0x8, 0),
        (STATE + 0x72A8, 0x5000_0000),
        (STATE + 0x72B0, FIRST_CELL),
        (STATE + 0x7545, 0x01),
        (STATE - 0xF0, EXE_BASE + 0xD4_D000),
        (FIRST_CELL, NODE),
        (FIRST_CELL + 8, u64::MAX),
        (NODE + 0x30, 1),
        (NODE + 0x58, EXE_BASE + 0x4AEA0),
        (NODE + 0x68, 0xDEAD_BEEF),
    ])
}

fn lines_for(facts: &CrashFacts, memory: &HashMap<u64, u64>) -> Vec<String> {
    mesh_update_list_lines(facts, &|address| memory.get(&address).copied())
}

#[test]
fn a_crash_in_the_mesh_update_names_the_list_the_node_and_the_callback() {
    let lines = lines_for(&facts(FAULT_RVA, Some(registers())), &memory());
    assert_eq!(lines.len(), 3);
    assert!(lines[0].contains("update state 0x20000000"));
    assert!(lines[0].contains("resource pointer 0x0 (0 means the model state was released)"));
    assert!(lines[0].contains("async update byte 0x1"));
    assert!(lines[0].contains("vtable exe rva 0xD4D000"));
    assert!(lines[1].contains("first cell 0x30000000 holds item 0x40000000"));
    assert!(lines[1].contains("next 0xFFFFFFFFFFFFFFFF"));
    assert!(lines[2].contains("bad cell 0xFFFFFFFFFFFFFFFF"));
    assert!(lines[2].contains("node 0x40000000"));
    assert!(lines[2].contains("name hash 0xDEADBEEF"));
    assert!(lines[2].contains("callback exe rva 0x4AEA0"));
}

#[test]
fn a_crash_outside_the_mesh_update_adds_nothing() {
    let lines = lines_for(&facts(0x50000, Some(registers())), &memory());
    assert!(lines.is_empty());
}

#[test]
fn a_crash_without_registers_adds_nothing() {
    assert!(lines_for(&facts(FAULT_RVA, None), &memory()).is_empty());
}

#[test]
fn unreadable_memory_is_said_so_and_never_stops_the_report() {
    let lines = lines_for(&facts(FAULT_RVA, Some(registers())), &HashMap::new());
    assert_eq!(lines.len(), 3);
    assert!(lines[0].contains("resource pointer unreadable"));
    assert!(lines[1].contains("not a real address"));
    assert!(lines[2].contains("callback unreadable"));
}
