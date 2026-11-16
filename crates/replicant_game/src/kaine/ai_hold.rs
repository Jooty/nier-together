/// One tick of the companion as the AI hold sees it, read at her update.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HerAiHoldReading {
    pub kept_off: bool,
    pub her_script_mode: bool,
    pub event_scene: bool,
}

/// Whether her per-frame update is skipped this tick: while the rule keeps her off, every tick she
/// is active outside her script mode and outside an event scene would be an AI tick (sensing,
/// attacks, following). In script mode or a scene the game drives her, so the update runs.
pub fn her_ai_is_held(reading: HerAiHoldReading) -> bool {
    reading.kept_off && !reading.her_script_mode && !reading.event_scene
}

pub fn format_held_ticks(held: u32) -> String {
    if held == 0 {
        "her update ran as the game's".to_owned()
    } else {
        format!("her update was held {held} ticks (active with her AI off)")
    }
}
