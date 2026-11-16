use crate::story_calls::is_event_id_name;

/// While the guest is not taken along, which of the host's story runs still run on the guest's
/// game: a story event's own steps do (they put the scene's actors and talks in this game's
/// world), the doors and checkpoint flags do not (they would move the watching guest's body).
pub fn host_run_replays_on_the_watching_guest(function: &str) -> bool {
    is_event_id_name(function)
}
