use crate::rule_rooms::{RulePlayer, RulePop, RuleRoomCall, rule_room_call};

pub fn break_of_a_watching_guest(
    pack: &str,
    function: &str,
    player: RulePlayer,
    watching: bool,
) -> Option<RulePop> {
    if !watching || player != RulePlayer::Guest {
        return None;
    }
    match rule_room_call(pack, function)? {
        (room, RuleRoomCall::Break(act)) if act.is_a_players_act() => {
            Some(RulePop::DropWhileWatching { room, act })
        }
        _ => None,
    }
}
