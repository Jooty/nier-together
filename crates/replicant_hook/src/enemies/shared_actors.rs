use std::collections::HashMap;

use replicant_proto::EnemyId;

use crate::enemies::EnemySyncRole;
use crate::enemies::combat_view::with_view;

/// This frame's enemy sync role and the local actors that are shared enemies (the host's real
/// enemies, or the guest's copies), by actor address. Read-only.
#[derive(Debug, Clone, Default)]
pub struct SharedEnemyActors {
    pub role: Option<EnemySyncRole>,
    pub actors: HashMap<u64, EnemyId>,
}

pub fn shared_enemy_actors() -> SharedEnemyActors {
    with_view(|view| SharedEnemyActors {
        role: view.role,
        actors: view.enemies.clone(),
    })
}

/// This frame's role and the network id of `actor` when it is a shared enemy, boss or companion.
pub fn shared_actor_id(actor: u64) -> Option<(EnemySyncRole, EnemyId)> {
    with_view(|view| Some((view.role?, *view.enemies.get(&actor)?)))
}

/// The local actor that stands for shared actor `id` this frame.
pub fn shared_actor_of(id: EnemyId) -> Option<u64> {
    with_view(|view| {
        view.enemies
            .iter()
            .find(|(_, known)| **known == id)
            .map(|(actor, _)| *actor)
    })
}
