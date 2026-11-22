use std::sync::atomic::{AtomicBool, Ordering};

use automata_game::mode::SessionMode;
use automata_game::outfit_look::looks_allowed;
use automata_proto::PlayerState;
use nt_hook::diagnostics::log_line;
use nt_hook::net::{NetState, Player};

use crate::net::link_state::NetShared;
use crate::net::partner_watch::PartnerWatch;

static UNKNOWN_MODE_LOGGED: AtomicBool = AtomicBool::new(false);

pub trait SessionModeExt {
    fn session_mode(&self) -> SessionMode;

    /// The question every campaign-only module asks: the partner, and only in the campaign.
    fn campaign_partner(&self) -> Option<&Player<PlayerState, PartnerWatch>>;

    /// The link is up and the session is the campaign: the other copy takes part in the story.
    fn campaign_link_is_up(&self) -> bool;

    /// The player whose body the companion slot carries: the partner in the campaign, the
    /// lowest other slot in free roam.
    fn companion_player(&self) -> Option<&Player<PlayerState, PartnerWatch>>;

    /// Free roam always, the campaign only while the host's switch for any outfit and accessory is on.
    fn looks_allowed(&self) -> bool;

    fn rules_taken_line(&self) -> String;

    /// Every other player but a late joiner still waiting to be taken in.
    fn counted_others(&self) -> impl Iterator<Item = &Player<PlayerState, PartnerWatch>>;

    /// Connected to somebody who is counted.
    fn counts_as_connected(&self) -> bool;
}

impl SessionModeExt for NetShared {
    fn session_mode(&self) -> SessionMode {
        if !SessionMode::is_a_known_mode(self.rules.mode)
            && !UNKNOWN_MODE_LOGGED.swap(true, Ordering::Relaxed)
        {
            log_line(&format!(
                "session: mode {} is not known to this game, treated as free roam",
                self.rules.mode
            ));
        }
        SessionMode::from_rules(self.rules)
    }

    fn campaign_partner(&self) -> Option<&Player<PlayerState, PartnerWatch>> {
        self.session_mode()
            .story_rules_apply()
            .then(|| self.partner())
            .flatten()
            .filter(|partner| !partner.watch.held_back)
    }

    fn campaign_link_is_up(&self) -> bool {
        self.counts_as_connected() && self.session_mode().story_rules_apply()
    }

    fn companion_player(&self) -> Option<&Player<PlayerState, PartnerWatch>> {
        if self.session_mode().story_rules_apply() {
            self.campaign_partner()
        } else {
            self.counted_others().next()
        }
    }

    fn counted_others(&self) -> impl Iterator<Item = &Player<PlayerState, PartnerWatch>> {
        self.others().filter(|player| !player.watch.held_back)
    }

    fn counts_as_connected(&self) -> bool {
        self.state == NetState::Connected && self.counted_others().next().is_some()
    }

    fn looks_allowed(&self) -> bool {
        looks_allowed(self.session_mode(), self.extras.any_look_allowed)
    }

    fn rules_taken_line(&self) -> String {
        format!(
            "session: rules taken, mode {}, any outfit and accessory allowed: {}",
            self.session_mode().label(),
            if self.looks_allowed() { "yes" } else { "no" }
        )
    }
}
