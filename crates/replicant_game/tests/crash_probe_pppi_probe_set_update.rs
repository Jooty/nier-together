use std::collections::HashMap;

use nt_hook::crash_report::{CrashFacts, CrashRegisters, ModuleSpot};
use replicant_game::crash_probe::pppi_probe_set_update_lines;

const EXE_BASE: u64 = 0x1_4000_0000;
const FAULT_RVA: u64 = 0x72F5FA;
const COMPILED_SET: u64 = 0x2000_0000;
const WORKSPACE: u64 = 0x3000_0000;
const STACK: u64 = 0x4000_0000;
const MANAGER: u64 = 0x5000_0000;
const SYSTEM: u64 = MANAGER - 0x428;
const MAP_HEAD: u64 = 0x6000_0000;

fn facts(rva: u64, registers: Option<CrashRegisters>) -> CrashFacts {
    CrashFacts {
        code: 0xC000_0005,
        address: EXE_BASE + rva,
        thread_id: 1120,
        game_thread_id: Some(972),
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
        rax: u64::MAX,
        r8: COMPILED_SET,
        r9: WORKSPACE,
        rsp: STACK,
        ..CrashRegisters::default()
    }
}

fn memory(head_value: u64) -> HashMap<u64, u64> {
    HashMap::from([
        (COMPILED_SET, 0xFFFF_FFFF_FFFF_FFFF),
        (COMPILED_SET + 0x8, 0xFFFF_FFFF_FFFF_FFFF),
        (COMPILED_SET + 0x10, u64::MAX),
        (COMPILED_SET + 0x18, 7),
        (WORKSPACE + 0x10D8, 0xA619_F229),
        (STACK + 0x88, MANAGER + 0x18),
        (MANAGER + 0x8, MAP_HEAD),
        (MANAGER + 0x10, 12),
        (MAP_HEAD + 0x20, head_value),
        (SYSTEM + 0x3B0, 0),
        (SYSTEM + 0x3E0, 1),
        (SYSTEM + 0xD2B0, 0x7000_0000),
        (SYSTEM + 0xD2C8, 9),
    ])
}

fn lines_for(facts: &CrashFacts, memory: &HashMap<u64, u64>) -> Vec<String> {
    pppi_probe_set_update_lines(facts, &|address| memory.get(&address).copied())
}

#[test]
fn a_crash_in_the_probe_set_update_names_the_compiled_set_its_source_and_the_owner() {
    let lines = lines_for(&facts(FAULT_RVA, Some(registers())), &memory(0x1234));
    assert_eq!(lines.len(), 2);
    assert!(lines[0].contains("compiled probe set 0x20000000"));
    assert!(lines[0].contains("its source probe set pointer 0xFFFFFFFFFFFFFFFF"));
    assert!(lines[0].contains("source probe count unreadable"));
    assert!(lines[1].contains(&format!("manager 0x{MANAGER:X} in system 0x{SYSTEM:X}")));
    assert!(lines[1].contains("the lookup missed: false"));
    assert!(lines[1].contains("worker stop byte 0x1"));
    assert!(lines[1].contains("map count 0xC"));
    assert!(lines[1].contains("magic intact"));
}

#[test]
fn a_lookup_that_returned_the_maps_end_node_is_said_so() {
    let lines = lines_for(&facts(FAULT_RVA, Some(registers())), &memory(COMPILED_SET));
    assert!(lines[1].contains("the lookup missed: true"));
}

#[test]
fn a_crash_outside_the_probe_set_update_adds_nothing() {
    assert!(lines_for(&facts(0x50000, Some(registers())), &memory(0)).is_empty());
}

#[test]
fn a_crash_without_registers_adds_nothing() {
    assert!(lines_for(&facts(FAULT_RVA, None), &memory(0)).is_empty());
}

#[test]
fn unreadable_memory_is_said_so_and_never_stops_the_report() {
    let lines = lines_for(&facts(FAULT_RVA, Some(registers())), &HashMap::new());
    assert_eq!(lines.len(), 2);
    assert!(lines[0].contains("its source probe set pointer unreadable"));
    assert!(lines[1].contains("manager unknown"));
    assert!(lines[1].contains("magic unreadable (corrupted or freed)"));
}
