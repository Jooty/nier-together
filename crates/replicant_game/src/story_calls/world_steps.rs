use crate::story_calls::{SCENE_STEPS_THAT_CHANGE_THE_WORLD, name_hash};

pub fn is_world_change_step(pack_hash: u64, function_hash: u64) -> bool {
    SCENE_STEPS_THAT_CHANGE_THE_WORLD
        .iter()
        .any(|(pack, function)| {
            name_hash(pack) == pack_hash && name_hash(function) == function_hash
        })
}

pub fn is_world_change_step_name(function: &str) -> bool {
    SCENE_STEPS_THAT_CHANGE_THE_WORLD
        .iter()
        .any(|(_, step)| *step == function)
}
