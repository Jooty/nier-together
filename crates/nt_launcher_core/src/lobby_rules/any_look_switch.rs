use nt_wire::SessionRules;

use crate::lobby_rules::current_mode;
use crate::profile::GameProfile;

pub const ANY_LOOK_LABEL: &str = "Any outfit and accessory";
pub const ANY_LOOK_NOTE: &str = "Outfits and accessories are meant to be unlocked by playing. Turn this on to let both players wear any of them in this co-op campaign.";

/// The host's switch for any outfit and accessory is for a game that has one, in its campaign only.
pub fn any_look_switch_offered(profile: &GameProfile, rules: SessionRules) -> bool {
    profile.any_look_switch && current_mode(profile, rules).is_none_or(|mode| mode.is_campaign())
}
