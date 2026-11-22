use crate::mode::SessionMode;

/// Free roam always allows a look; the campaign allows it only when the host's switch is on.
pub fn looks_allowed(mode: SessionMode, host_allows_any_look: bool) -> bool {
    !mode.story_rules_apply() || host_allows_any_look
}
