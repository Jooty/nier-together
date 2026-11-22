use crate::crash_report::{
    ACCESS_VIOLATION_CODE, BREAKPOINT_CODE, FAIL_FAST_CODE, HEAP_CORRUPTION_CODE,
    ILLEGAL_INSTRUCTION_CODE, IN_PAGE_ERROR_CODE, INTEGER_DIVIDE_BY_ZERO_CODE,
    PRIVILEGED_INSTRUCTION_CODE, STACK_OVERFLOW_CODE,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModuleSpot {
    pub name: String,
    pub base: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StackFrame {
    pub address: u64,
    pub module: ModuleSpot,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CrashRegisters {
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rbp: u64,
    pub rsp: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
}

pub type CrashProbe = fn(&CrashFacts, &dyn Fn(u64) -> Option<u64>) -> Vec<String>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CrashFacts {
    pub code: u32,
    pub address: u64,
    pub thread_id: u32,
    pub game_thread_id: Option<u32>,
    pub registers: Option<CrashRegisters>,
    pub module: Option<ModuleSpot>,
    pub in_our_module: bool,
    pub access: Option<(u64, u64)>,
    pub last_chance: bool,
    pub stack: Vec<StackFrame>,
}

pub fn is_fatal_code(code: u32) -> bool {
    matches!(
        code,
        ACCESS_VIOLATION_CODE
            | IN_PAGE_ERROR_CODE
            | ILLEGAL_INSTRUCTION_CODE
            | STACK_OVERFLOW_CODE
            | HEAP_CORRUPTION_CODE
            | FAIL_FAST_CODE
            | INTEGER_DIVIDE_BY_ZERO_CODE
            | PRIVILEGED_INSTRUCTION_CODE
            | BREAKPOINT_CODE
    )
}

pub fn is_return_address_candidate(word: u64, module_bases: &[u64], module_span: u64) -> bool {
    module_bases
        .iter()
        .any(|base| word >= *base && word - *base < module_span)
}

pub fn early_line(code: u32, address: u64, thread_id: u32) -> String {
    format!("crash: fatal exception code 0x{code:08X} at 0x{address:016X} on thread {thread_id}")
}

pub fn access_violation_kind(kind: u64) -> &'static str {
    match kind {
        0 => "read",
        1 => "write",
        8 => "execute",
        _ => "other",
    }
}

pub fn last_lines(text: &str, count: usize) -> Vec<&str> {
    let lines: Vec<&str> = text.lines().collect();
    lines[lines.len().saturating_sub(count)..].to_vec()
}

pub fn thread_line(thread_id: u32, game_thread_id: Option<u32>) -> String {
    match game_thread_id {
        Some(game) if game == thread_id => format!("crash: thread {thread_id} is the game thread"),
        Some(game) => {
            format!("crash: thread {thread_id} is not the game thread (the game thread is {game})")
        }
        None => format!("crash: thread {thread_id}, the game thread is not known yet"),
    }
}

pub fn report_text(facts: &CrashFacts, log_tail: &[&str]) -> String {
    report_text_with(facts, &[], log_tail)
}

pub fn report_text_with(facts: &CrashFacts, probe_lines: &[String], log_tail: &[&str]) -> String {
    let stage = if facts.last_chance {
        "unhandled exception"
    } else if facts.in_our_module {
        "exception in our own module"
    } else {
        "first-chance fatal exception in another module"
    };
    let mut text = format!(
        "crash: {stage} code 0x{:08X} at 0x{:016X} on thread {}\n",
        facts.code, facts.address, facts.thread_id
    );
    match &facts.module {
        Some(module) => text.push_str(&format!(
            "crash: module {} base 0x{:016X} rva 0x{:X}{}\n",
            module.name,
            module.base,
            facts.address.wrapping_sub(module.base),
            if facts.in_our_module {
                " (our hook)"
            } else {
                ""
            }
        )),
        None => text.push_str("crash: address is in no loaded module\n"),
    }
    if let Some((kind, target)) = facts.access {
        text.push_str(&format!(
            "crash: {} of 0x{target:016X}\n",
            access_violation_kind(kind)
        ));
    }
    text.push_str(&thread_line(facts.thread_id, facts.game_thread_id));
    text.push('\n');
    for line in probe_lines {
        text.push_str(line);
        text.push('\n');
    }
    for (index, frame) in facts.stack.iter().enumerate() {
        text.push_str(&format!(
            "crash: stack {index} 0x{:016X} {} rva 0x{:X}\n",
            frame.address,
            frame.module.name,
            frame.address.wrapping_sub(frame.module.base)
        ));
    }
    text.push_str(&format!(
        "crash: last {} log lines follow\n",
        log_tail.len()
    ));
    for line in log_tail {
        text.push_str("crash:   ");
        text.push_str(line);
        text.push('\n');
    }
    text
}
