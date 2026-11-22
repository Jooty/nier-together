use std::collections::{BTreeMap, HashSet};

use crate::memory::GameMemory;
use crate::offsets::{
    ACTOR_CLASS_ID_OFFSET, ACTOR_KIND_MASK_OFFSET, ACTOR_TRANSFORM_BLOCK_BYTES,
    ACTOR_TRANSFORM_BLOCK_OFFSET, MAX_ENEMY_ACTORS_SCANNED_PER_GROUP, PUPPET_ACTOR_GROUP,
};
use crate::puppet::{PuppetPose, actor_is_live};

pub const FIGHT_ACTOR_PROBE_INTERVAL_MS: u64 = 5000;
pub const FIGHT_ACTOR_PROBE_NEW_KEY_GAP_MS: u64 = 1000;
pub const FIGHT_7_BOSS_GROUP: u16 = 100;
pub const FIGHT_7_BOSS_CLASS_ID: i32 = 0x51;
pub const FIGHT_11_BOSS_GROUP: u16 = 800;
pub const FIGHT_11_LIBRARY_BOSS_GROUP: u16 = 901;
pub const ACTOR_MODEL_SLOTS_OFFSET: u64 = 0x8DC0;
pub const ACTOR_MODEL_SLOT_COUNT: u64 = 8;
pub const FIGHT_ACTOR_PROBE_GROUP_COUNT: u16 = 0x400;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbedFight {
    Seven,
    Eleven,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProbedActor {
    pub group: u16,
    pub actor: u64,
    pub class_id: i32,
    pub kind_mask: u32,
    pub model_slots_filled: u32,
    pub position: [f32; 3],
}

#[derive(Debug, Default)]
pub struct FightActorProbe {
    seen_keys: HashSet<(u16, i32, u32)>,
    last_dump_ms: Option<u64>,
}

impl ProbedFight {
    pub fn name(self) -> &'static str {
        match self {
            Self::Seven => "fight 7 (100:0, class 0x51)",
            Self::Eleven => "fight 11 (800:0 or 901:0)",
        }
    }
}

impl FightActorProbe {
    pub fn observe(&mut self, side: &str, now_ms: u64, actors: &[ProbedActor]) -> Vec<String> {
        let has_new_key = actors
            .iter()
            .any(|actor| !self.seen_keys.contains(&key_of(actor)));
        let due = self.last_dump_ms.is_none_or(|last| {
            let gap = now_ms.saturating_sub(last);
            gap >= FIGHT_ACTOR_PROBE_INTERVAL_MS
                || (has_new_key && gap >= FIGHT_ACTOR_PROBE_NEW_KEY_GAP_MS)
        });
        if !due {
            return Vec::new();
        }
        self.last_dump_ms = Some(now_ms);
        self.seen_keys.extend(actors.iter().map(key_of));
        actor_dump_lines(side, actors)
    }
}

pub fn probed_fight(group: u16, class_id: i32) -> Option<ProbedFight> {
    match (group, class_id) {
        (FIGHT_7_BOSS_GROUP, FIGHT_7_BOSS_CLASS_ID) => Some(ProbedFight::Seven),
        (FIGHT_11_BOSS_GROUP | FIGHT_11_LIBRARY_BOSS_GROUP, _) => Some(ProbedFight::Eleven),
        _ => None,
    }
}

pub fn actor_dump_lines(side: &str, actors: &[ProbedActor]) -> Vec<String> {
    let mut by_key: BTreeMap<(u16, i32, u32), Vec<&ProbedActor>> = BTreeMap::new();
    for actor in actors {
        by_key.entry(key_of(actor)).or_default().push(actor);
    }
    let mut lines = vec![format!(
        "fight actors ({side}): {} live actors in {} kinds",
        actors.len(),
        by_key.len()
    )];
    for ((group, class_id, kind_mask), members) in by_key {
        let first = members[0];
        let most_slots = members
            .iter()
            .map(|member| member.model_slots_filled)
            .max()
            .unwrap_or(0);
        lines.push(format!(
            "fight actors ({side}): group {group} class {class_id:#x} kind bits {kind_mask:#x} count {} first {:#x} at ({:.1}, {:.1}, {:.1}) model slots filled up to {most_slots}",
            members.len(),
            first.actor,
            first.position[0],
            first.position[1],
            first.position[2]
        ));
    }
    lines
}

pub fn raw_group_count_line(side: &str, group_counts: &[i32], actors: &[ProbedActor]) -> String {
    let mut groups = Vec::new();
    for (group, raw) in (0..FIGHT_ACTOR_PROBE_GROUP_COUNT).zip(group_counts) {
        if *raw <= 0 || i32::from(group) == PUPPET_ACTOR_GROUP {
            continue;
        }
        let live = actors.iter().filter(|actor| actor.group == group).count();
        let cut = if *raw > MAX_ENEMY_ACTORS_SCANNED_PER_GROUP {
            " (scan cap hit)"
        } else {
            ""
        };
        groups.push(format!("{group}: manager {raw}, live scanned {live}{cut}"));
    }
    format!(
        "fight actors ({side}): raw group counts: {}",
        groups.join("; ")
    )
}

pub fn read_live_actors<M: GameMemory>(
    memory: &M,
    group_counts: &[i32],
    mut actor_in_group: impl FnMut(u16, i32) -> u64,
) -> Vec<ProbedActor> {
    let mut actors = Vec::new();
    for (group, count) in (0..FIGHT_ACTOR_PROBE_GROUP_COUNT).zip(group_counts) {
        if i32::from(group) == PUPPET_ACTOR_GROUP {
            continue;
        }
        for index in 0..(*count).clamp(0, MAX_ENEMY_ACTORS_SCANNED_PER_GROUP) {
            let actor = actor_in_group(group, index);
            if actor == 0 || !actor_is_live(memory, actor).unwrap_or(false) {
                continue;
            }
            actors.push(read_actor(memory, group, actor));
        }
    }
    actors
}

fn key_of(actor: &ProbedActor) -> (u16, i32, u32) {
    (actor.group, actor.class_id, actor.kind_mask)
}

fn read_actor<M: GameMemory>(memory: &M, group: u16, actor: u64) -> ProbedActor {
    let block = memory
        .read_array::<ACTOR_TRANSFORM_BLOCK_BYTES>(actor + ACTOR_TRANSFORM_BLOCK_OFFSET)
        .ok();
    let position = block.map_or([0.0; 3], |block| {
        let pose = PuppetPose::from_transform_block(&block);
        [pose.position.x, pose.position.y, pose.position.z]
    });
    let model_slots_filled = (0..ACTOR_MODEL_SLOT_COUNT)
        .filter(|slot| {
            memory
                .read_u64(actor + ACTOR_MODEL_SLOTS_OFFSET + slot * 8)
                .is_ok_and(|pointer| pointer != 0)
        })
        .count();
    ProbedActor {
        group,
        actor,
        class_id: memory.read_i32(actor + ACTOR_CLASS_ID_OFFSET).unwrap_or(-1),
        kind_mask: memory
            .read_i32(actor + ACTOR_KIND_MASK_OFFSET)
            .unwrap_or(0)
            .cast_unsigned(),
        model_slots_filled: u32::try_from(model_slots_filled).unwrap_or(u32::MAX),
        position,
    }
}
