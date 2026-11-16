use replicant_proto::StoryJumpPoint;

use crate::level_up::LAST_LEVEL_THE_GAME_ALLOWS;

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
    }
}

pub fn levels_to_raise(current: i32, target: i32) -> i32 {
    (target.min(LAST_LEVEL_THE_GAME_ALLOWS) - current).max(0)
}
