use std::panic;
use std::sync::Mutex;

use crate::crash_report::{CrashFacts, CrashProbe};

use super::handlers::read_word;

static PROBES: Mutex<Vec<CrashProbe>> = Mutex::new(Vec::new());

pub fn register_probe(probe: CrashProbe) {
    if let Ok(mut probes) = PROBES.lock() {
        probes.push(probe);
    }
}

pub(super) fn probe_lines(facts: &CrashFacts) -> Vec<String> {
    let Ok(probes) = PROBES.try_lock() else {
        return Vec::new();
    };
    let mut lines = Vec::new();
    for probe in probes.iter() {
        if let Ok(found) = panic::catch_unwind(|| probe(facts, &read_word)) {
            lines.extend(found);
        }
    }
    lines
}
