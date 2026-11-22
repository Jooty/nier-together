use crate::free_roam_scenes::UpdateHold;

pub fn held_action_line(
    hold: UpdateHold,
    id: u32,
    state: u32,
    play_asked: u8,
    actions_held: usize,
) -> String {
    match hold {
        UpdateHold::AreaStart => format!(
            "free roam: an event action's area update is held (action id {id:#X}, state {state:#X}, play asked {play_asked}, area hit not read); first hold of this action, {actions_held} actions held so far, logged once per action"
        ),
        UpdateHold::DeniedScene => format!(
            "free roam: scene denied, it never starts here (action id {id:#X}, state {state:#X}, play asked {play_asked}); first denial of this action, {actions_held} actions held so far, logged once per action"
        ),
    }
}

pub fn capped_hap_line(
    action_id: u32,
    index: u32,
    action_count: u32,
    haps_capped: usize,
) -> String {
    format!(
        "free roam: a hap is stopped before its nearness gate, nothing behind the gate runs (gate action id {action_id:#X}, index {index} of {action_count}); first stop of this gate, {haps_capped} gates stopped so far, logged once per gate"
    )
}
