use std::sync::{Mutex, PoisonError};

use replicant_game::kaine::{BossPhaseFacts, BossPhaseWatch, boss_phase_text};
use replicant_game::marker::read_world_flags;

use crate::build_gate::VerifiedBuild;
use crate::diagnostics::log_line;
use crate::memory::InProcessMemory;

static WATCH: Mutex<BossPhaseWatch> = Mutex::new(BossPhaseWatch::new());

pub(super) fn tick(build: &VerifiedBuild, fight_is_live: bool) {
    let mut watch = WATCH.lock().unwrap_or_else(PoisonError::into_inner);
    if !fight_is_live {
        watch.reset();
        return;
    }
    let Some((hp, max_hp)) = crate::enemies::boss_kaine_life(build).filter(|(_, max)| *max > 0)
    else {
        return;
    };
    let memory = InProcessMemory::new(build);
    let life_percent = u32::try_from(hp.max(0)).unwrap_or(0).saturating_mul(100)
        / u32::try_from(max_hp).unwrap_or(1);
    let event_scene = read_world_flags(&memory).is_ok_and(|flags| flags.event_scene);
    if let Some(change) = watch.step(BossPhaseFacts {
        life_percent,
        event_scene,
    }) {
        log_line(&boss_phase_text(change));
    }
}
