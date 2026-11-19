use std::sync::{LazyLock, Mutex, PoisonError};

use replicant_game::Replicant;
use replicant_game::phase_request::LoadedPartSetsWatch;

use crate::build_gate::VerifiedBuild;
use crate::diagnostics::log_line;
use crate::memory::InProcessMemory;

static WATCH: LazyLock<Mutex<LoadedPartSetsWatch>> = LazyLock::new(Mutex::default);

pub fn tick_part_sets_log(build: &VerifiedBuild) {
    let game = Replicant::new(InProcessMemory::new(build));
    let Ok(sets) = game.loaded_part_sets() else {
        return;
    };
    let line = WATCH
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .observe(&sets);
    if let Some(line) = line {
        log_line(&line);
    }
}
