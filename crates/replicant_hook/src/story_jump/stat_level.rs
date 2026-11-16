use replicant_game::Replicant;
use replicant_game::story_jump::{
    PLAYER_UNIQUE_ACTOR_ID, PROLOGUE_END_PLAYER_LEVEL, SET_UNIQUE_ACTOR_LEVEL_NATIVE_RVA,
    jump_resets_prologue_level, jump_stat_level, levels_to_raise,
};
use replicant_proto::StoryJumpPoint;

use crate::build_gate::VerifiedBuild;
use crate::diagnostics::log_line;
use crate::memory::InProcessMemory;
use crate::prologue_levels::level_up_once;
use crate::story_jump::natives::call_native;

pub(super) fn raise_stat_level(build: &VerifiedBuild, point: StoryJumpPoint) {
    reset_the_prologue_level(build);
    raise_stat_level_to(build, jump_stat_level(point));
}

pub(super) fn raise_stat_level_to(build: &VerifiedBuild, target: i32) {
    let read_level = || Replicant::new(InProcessMemory::new(build)).level().ok();
    let Some(was) = read_level() else {
        log_line("story jump: level unreadable, not raised");
        return;
    };
    let mut now = was;
    for _ in 0..levels_to_raise(was, target) {
        if level_up_once(build).is_err() {
            break;
        }
        match read_level() {
            Some(level) if level > now => now = level,
            _ => break,
        }
    }
    log_line(&format!("story jump: level {now} (was {was})"));
}

fn reset_the_prologue_level(build: &VerifiedBuild) {
    let game = Replicant::new(InProcessMemory::new(build));
    let Ok(phase) = game.current_phase() else {
        return;
    };
    if !jump_resets_prologue_level(&phase) {
        return;
    }
    let was = game.level().ok();
    call_native(
        build,
        SET_UNIQUE_ACTOR_LEVEL_NATIVE_RVA,
        &[PLAYER_UNIQUE_ACTOR_ID, PROLOGUE_END_PLAYER_LEVEL],
    );
    let was_text = was.map_or_else(|| "unknown".to_owned(), |level| level.to_string());
    log_line(&format!(
        "story jump: left {phase} early, level {was_text} reset to {PROLOGUE_END_PLAYER_LEVEL} as its end does"
    ));
}
