use crate::level_up::LAST_LEVEL_THE_GAME_ALLOWS;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LevelUpPlan {
    Gain { experience: i32 },
    AtTheLevelCap,
    NoLevelTable,
}

pub fn level_up_plan(
    level: i32,
    experience: i32,
    experience_to_leave_level: Option<i32>,
) -> LevelUpPlan {
    if level >= LAST_LEVEL_THE_GAME_ALLOWS {
        return LevelUpPlan::AtTheLevelCap;
    }
    match experience_to_leave_level {
        None => LevelUpPlan::NoLevelTable,
        Some(needed) => LevelUpPlan::Gain {
            experience: needed.saturating_sub(experience).max(0),
        },
    }
}
