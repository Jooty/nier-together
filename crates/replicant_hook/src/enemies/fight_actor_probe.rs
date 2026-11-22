use std::sync::{LazyLock, Mutex, PoisonError};
use std::time::Instant;

use replicant_game::enemies::{
    EnemySighting, FightActorProbe, ProbedFight, probed_fight, raw_group_count_line,
    read_group_counts, read_live_actors,
};
use replicant_game::npcs::motion_layers_text;
use replicant_proto::EnemyId;

use crate::boss_messages::is_boss;
use crate::build_gate::VerifiedBuild;
use crate::diagnostics::log_line;
use crate::enemies::enemy_id_text;
use crate::memory::InProcessMemory;
use crate::world_objects::actor_in_group;

static PROBE: LazyLock<Mutex<FightActorProbe>> = LazyLock::new(Mutex::default);
static EPOCH: LazyLock<Instant> = LazyLock::new(Instant::now);

pub(super) fn log_fight_actors(
    build: &VerifiedBuild,
    side: &str,
    seen: &[(EnemyId, EnemySighting)],
) {
    let Some((fight, id, body)) = probed_boss(seen) else {
        return;
    };
    let memory = InProcessMemory::new(build);
    let Ok(counts) = read_group_counts(&memory) else {
        return;
    };
    let actors = read_live_actors(&memory, &counts, |group, index| {
        actor_in_group(build, group, index)
    });
    let now_ms = u64::try_from(EPOCH.elapsed().as_millis()).unwrap_or(u64::MAX);
    let lines = PROBE
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .observe(side, now_ms, &actors);
    if lines.is_empty() {
        return;
    }
    log_line(&format!(
        "fight actors ({side}): {} body {} {}",
        fight.name(),
        enemy_id_text(id),
        motion_layers_text(&memory, body.actor)
    ));
    for line in lines {
        log_line(&line);
    }
    log_line(&raw_group_count_line(side, &counts, &actors));
}

fn probed_boss(
    seen: &[(EnemyId, EnemySighting)],
) -> Option<(ProbedFight, EnemyId, &EnemySighting)> {
    seen.iter().find_map(|(id, sighting)| {
        if !is_boss(*id) {
            return None;
        }
        probed_fight(id.group, sighting.class_id).map(|fight| (fight, *id, sighting))
    })
}
