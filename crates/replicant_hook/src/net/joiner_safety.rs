use std::sync::{Mutex, PoisonError};

use replicant_game::Replicant;
use replicant_game::enemies::read_boss_bar_open;
use replicant_game::readiness::{JoinerWaitLog, SafetyReads};
use replicant_game::stay_together::local_phase_index;

use crate::build_gate::VerifiedBuild;
use crate::cutscene_skip::{control_slots_free_now, event_scene_running};
use crate::diagnostics::log_line;
use crate::memory::InProcessMemory;
use crate::net::net_clock_ms;
use crate::stay_together::game_says_free_to_act;

static WAIT_LOG: Mutex<JoinerWaitLog> = Mutex::new(JoinerWaitLog::new());

pub(super) fn host_is_between_things(build: &VerifiedBuild, a_joiner_waits: bool) -> bool {
    let game = Replicant::new(InProcessMemory::new(build));
    let reads = SafetyReads {
        free_to_act: game_says_free_to_act(build),
        in_event_scene: event_scene_running(build),
        control_slots_free: control_slots_free_now(build),
        boss_gauge_open: read_boss_bar_open(game.memory()).unwrap_or(false),
        fight_window_open: local_phase_index(&game).is_some_and(crate::fight_sync::covers_phase),
    };
    let block = reads.first_block();
    let mut log = WAIT_LOG.lock().unwrap_or_else(PoisonError::into_inner);
    if a_joiner_waits {
        if let Some(line) = log.observe(block, net_clock_ms()) {
            log_line(&line);
        }
    } else {
        log.reset();
    }
    block.is_none()
}
