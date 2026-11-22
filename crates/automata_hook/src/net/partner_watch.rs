use automata_game::character::Character;
use automata_game::load_gate::StoryPlace;
use automata_game::outfit_look::{HOST_SLOT, PlayerLook};

use crate::net::link_state::NetShared;
use crate::net::load_signal::HeardLoadSignal;

/// What this game has heard from one other player since they joined.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PartnerWatch {
    pub character: Option<Character>,
    /// The other game's hello or a state has arrived since the join.
    pub heard: bool,
    pub load: Option<HeardLoadSignal>,
    pub place: StoryPlace,
    pub look: Option<PlayerLook>,
    /// The player joined while this game was already in play.
    pub joined_late: bool,
    /// A late joiner stays unseen by every reader until a safe moment takes it in.
    pub held_back: bool,
    /// The held player's game has sent a body state.
    pub arrival_waiting: bool,
    pub first_state_heard_in_play: Option<bool>,
}

pub trait NetSharedExt {
    /// The partner's watch, or an empty one while no partner is in the session.
    fn partner_watch(&self) -> PartnerWatch;

    /// The watch of the player in one slot, or an empty one while the slot holds nobody.
    fn player_watch(&self, slot: u8) -> PartnerWatch;

    /// This game linked to the host after the host's game was already in play.
    fn host_was_in_play_when_first_heard(&self) -> bool;
}

impl NetSharedExt for NetShared {
    fn partner_watch(&self) -> PartnerWatch {
        self.partner()
            .map_or_else(PartnerWatch::default, |partner| partner.watch)
    }

    fn player_watch(&self, slot: u8) -> PartnerWatch {
        self.player(slot)
            .map_or_else(PartnerWatch::default, |player| player.watch)
    }

    fn host_was_in_play_when_first_heard(&self) -> bool {
        self.player_watch(HOST_SLOT).first_state_heard_in_play == Some(true)
    }
}
