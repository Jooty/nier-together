use std::sync::{Mutex, PoisonError};

use replicant_game::enemies::{
    BossGaugeReading, BossGaugeWatch, EnemySighting, read_boss_bar_handle, read_boss_bar_state,
};
use replicant_proto::EnemyId;

use crate::boss_messages::is_boss;
use crate::build_gate::VerifiedBuild;
use crate::diagnostics::log_line;
use crate::enemies::enemy_id_text;
use crate::memory::InProcessMemory;

static WATCH: Mutex<BossGaugeWatch> = Mutex::new(BossGaugeWatch::new());

pub(super) fn log_boss_gauge(build: &VerifiedBuild, side: &str, seen: &[(EnemyId, EnemySighting)]) {
    let mut watch = WATCH.lock().unwrap_or_else(PoisonError::into_inner);
    let boss = seen
        .iter()
        .find(|(id, _)| is_boss(*id))
        .map(|(id, _)| enemy_id_text(*id));
    if boss.is_none() {
        watch.forget();
        return;
    }
    let memory = InProcessMemory::new(build);
    let bound_actor = read_boss_bar_handle(&memory).and_then(|handle| handle.resolve(&memory));
    let bound_enemy = bound_actor.and_then(|actor| {
        seen.iter()
            .find(|(_, sighting)| sighting.actor == actor)
            .map(|(id, _)| enemy_id_text(*id))
    });
    let reading = BossGaugeReading {
        state: read_boss_bar_state(&memory),
        bound_enemy,
        boss,
    };
    if let Some(line) = watch.observe(side, &reading) {
        log_line(&line);
    }
}
