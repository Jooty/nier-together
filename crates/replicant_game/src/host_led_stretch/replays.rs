use crate::story_calls::{is_event_id_name, is_world_change_step_name};

/// While the guest is not taken along, the host's story event steps and its world-changing steps
/// still run on the guest's game; doors and checkpoint flags do not (they would move its body).
pub fn host_run_replays_on_the_watching_guest(function: &str) -> bool {
    is_event_id_name(function) || is_world_change_step_name(function)
}
