use replicant_game::Replicant;
use replicant_game::story_jump::{jump_stat_level, levels_to_raise};
use replicant_proto::StoryJumpPoint;

use crate::build_gate::VerifiedBuild;
use crate::diagnostics::log_line;
use crate::memory::InProcessMemory;
use crate::prologue_levels::level_up_once;

pub(super) fn raise_stat_level(build: &VerifiedBuild, point: StoryJumpPoint) {
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
