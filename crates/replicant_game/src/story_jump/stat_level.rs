use replicant_proto::StoryJumpPoint;

use crate::level_up::LAST_LEVEL_THE_GAME_ALLOWS;
use crate::prologue_levels::PROLOGUE_PHASES;

pub const PROLOGUE_END_PLAYER_LEVEL: i32 = 0;

pub fn jump_stat_level(point: StoryJumpPoint) -> i32 {
    match point {
        StoryJumpPoint::VillageAfterPrologue
        | StoryJumpPoint::LibraryPopolaTalk
        | StoryJumpPoint::VillageBeforeNextDay => 1,
        StoryJumpPoint::ShrineEntrance => 3,
        StoryJumpPoint::ShrineTop => 4,
        StoryJumpPoint::ShrineBoss2 => 6,
        StoryJumpPoint::AerieBoss3 => 12,
        StoryJumpPoint::JunkHeapB2 => 20,
        StoryJumpPoint::JunkHeapBoss5 => 22,
        StoryJumpPoint::AerieBoss6 | StoryJumpPoint::AerieAfterBoss6 => 26,
        StoryJumpPoint::DesertAfterFacade => 28,
        StoryJumpPoint::TempleEntrance => 29,
        StoryJumpPoint::TempleBoss7 => 32,
        StoryJumpPoint::VillageAfterTemple | StoryJumpPoint::ForestArrival => 34,
        StoryJumpPoint::SouthPlainsRedBag
        | StoryJumpPoint::ManorFront
        | StoryJumpPoint::ManorEmilFollows => 36,
        StoryJumpPoint::ManorBoss8 => 38,
        StoryJumpPoint::VillageBoss9 => 40,
        StoryJumpPoint::LibraryBoss10 | StoryJumpPoint::LibraryBoss11 => 42,
    }
}

pub fn levels_to_raise(current: i32, target: i32) -> i32 {
    (target.min(LAST_LEVEL_THE_GAME_ALLOWS) - current).max(0)
}

/// The prologue's scripts force the stored level (21, then 29) and its end puts it back to 0 with
/// `_SetUniqueActorLevel(-1, 0)`. A jump taken in the prologue skips that end, so it does the same
/// reset before raising; anywhere else the level is the player's own and is never lowered.
pub fn jump_resets_prologue_level(phase_at_apply: &str) -> bool {
    PROLOGUE_PHASES.contains(&phase_at_apply)
}
