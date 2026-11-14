use crate::story_watch::{
    WATCH_BODY_LEFT_ALONE_BELOW, WATCH_BODY_TELEPORT_EVERY_MS, WATCH_BODY_TELEPORTED_ABOVE,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaceStep {
    Leave,
    Write,
    Teleport,
}

/// How the watching guest's hidden body is put at the host's body: a plain write for a small
/// error, the game's own move for a large one, at most once per teleport interval.
pub fn place_step(error: f32, since_last_teleport_ms: u64) -> PlaceStep {
    if error < WATCH_BODY_LEFT_ALONE_BELOW {
        return PlaceStep::Leave;
    }
    if error <= WATCH_BODY_TELEPORTED_ABOVE {
        return PlaceStep::Write;
    }
    if since_last_teleport_ms >= WATCH_BODY_TELEPORT_EVERY_MS {
        return PlaceStep::Teleport;
    }
    PlaceStep::Leave
}
