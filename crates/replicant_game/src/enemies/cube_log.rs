use std::collections::HashMap;

use replicant_proto::EnemyId;

pub const CUBE_GROUP: u16 = 1001;
pub const CUBE_STATE_LINES_PER_TICK: usize = 8;
pub const CUBE_EFFECT_NAME_PREFIX: &str = "efE_shahri_";
pub const CUBE_EFFECT_NAME_READ_LEN: usize = 48;
pub const CUBE_GLOW_EFFECT_NAME_PREFIX: &str = "efE_shahri_aura";

#[derive(Debug, Default)]
pub struct CubeStateLog {
    last: HashMap<EnemyId, Option<i32>>,
}

impl CubeStateLog {
    pub fn observe(
        &mut self,
        side: &str,
        cubes: impl IntoIterator<Item = (EnemyId, u64, Option<i32>)>,
    ) -> Vec<String> {
        let mut lines = Vec::new();
        for (id, actor, motion) in cubes {
            if id.group != CUBE_GROUP {
                continue;
            }
            let before = self.last.insert(id, motion);
            if before.is_some_and(|before| before != motion)
                && lines.len() < CUBE_STATE_LINES_PER_TICK
            {
                lines.push(cube_state_changed_text(
                    side,
                    id,
                    actor,
                    before.flatten(),
                    motion,
                ));
            }
        }
        lines
    }
}

pub fn is_cube_effect(name: &str) -> bool {
    name.starts_with(CUBE_EFFECT_NAME_PREFIX)
}

pub fn cube_glow_is_relayed(attaching_actor: u64, name: &str, local_is_host: bool) -> bool {
    local_is_host && attaching_actor != 0 && name.starts_with(CUBE_GLOW_EFFECT_NAME_PREFIX)
}

pub fn cube_effect_attached_text(name: &str, actor: u64) -> String {
    format!("cube probe: {name} attached to actor {actor:#x} by the game's own state")
}

pub fn cube_state_changed_text(
    side: &str,
    id: EnemyId,
    actor: u64,
    from: Option<i32>,
    to: Option<i32>,
) -> String {
    let show = |motion: Option<i32>| motion.map_or_else(|| "none".to_owned(), |m| m.to_string());
    format!(
        "cube probe ({side}): {}:{} (actor {actor:#x}) motion {} -> {}",
        id.group,
        id.ordinal,
        show(from),
        show(to)
    )
}
