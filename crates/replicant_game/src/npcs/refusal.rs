use crate::npcs::NPC_TALK_COMMAND_IDS;

/// A talk start on a villager the partner is talking to is refused, unless a story step just
/// popped makes the villager talk: both games run the step, so each gets its own copy of the talk.
pub fn refuses_talk_start(
    command_id: i32,
    partner_is_talking_to_it: bool,
    a_story_step_makes_it_talk: bool,
) -> bool {
    partner_is_talking_to_it
        && !a_story_step_makes_it_talk
        && NPC_TALK_COMMAND_IDS.contains(&command_id)
}

pub fn talk_busy_notice(partner_name: &str) -> String {
    format!("{partner_name} is talking to them")
}
