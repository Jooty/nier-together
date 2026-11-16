use std::sync::PoisonError;

use replicant_game::Replicant;
use replicant_game::level_up::{LAST_LEVEL_THE_GAME_ALLOWS, LevelUpPlan, level_up_plan};
use replicant_game::prologue_levels::LocalLevel;

use crate::build_gate::VerifiedBuild;
use crate::diagnostics::log_line;
use crate::memory::InProcessMemory;
use crate::prologue_levels::tick::{
    SWAPPED_LEVEL, add_experience_through_game, wears_the_swapped_body,
};

pub fn level_up_once(build: &VerifiedBuild) -> Result<(), String> {
    let game = Replicant::new(InProcessMemory::new(build));
    let level = game.level().map_err(|error| error.to_string())?;
    let experience = game.experience().map_err(|error| error.to_string())?;
    let needed = game
        .experience_to_leave_level(level)
        .map_err(|error| error.to_string())?;
    let gain = match level_up_plan(level, experience, needed) {
        LevelUpPlan::Gain { experience } => experience,
        LevelUpPlan::AtTheLevelCap => {
            return Err(format!(
                "level up: already at level {level}, the last level the game allows is {LAST_LEVEL_THE_GAME_ALLOWS}"
            ));
        }
        LevelUpPlan::NoLevelTable => {
            return Err("level up: the player param has no level table".to_owned());
        }
    };
    let max_health_before = game.max_health().unwrap_or_default();
    add_experience_through_game(build, gain);
    let after = LocalLevel {
        level: game.level().unwrap_or(level),
        experience: game.experience().unwrap_or(experience),
    };
    let max_health_after = game.max_health().unwrap_or_default();
    let on_the_swapped_body = wears_the_swapped_body();
    if on_the_swapped_body {
        SWAPPED_LEVEL
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .raised_by_the_trainer(after);
    }
    log_line(&format!(
        "level up: level {level} (exp {experience}) -> {} (exp {}), +{gain} experience through the game, max hp {max_health_before} -> {max_health_after}, swapped body held at it: {on_the_swapped_body}",
        after.level, after.experience
    ));
    if after.level > level {
        Ok(())
    } else {
        Err(format!(
            "level up: the level stayed {level} after +{gain} experience"
        ))
    }
}
