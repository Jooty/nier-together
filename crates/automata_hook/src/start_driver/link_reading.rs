use automata_game::load_gate::{PartnerLoad, StoryPlace};
use automata_game::travel::JumpRole;
use nt_hook::net::net_clock_ms;

use crate::net::{
    NetShared, SessionModeExt, handle, load_role_of, partner_load_of, partner_place_of,
};

pub(crate) struct LinkReading {
    pub role: JumpRole,
    pub load_role: JumpRole,
    pub partner: PartnerLoad,
    pub partner_place: StoryPlace,
    pub campaign_link_up: bool,
    pub partner_present: bool,
    pub partner_watching: bool,
    pub partner_free: bool,
}

pub(crate) fn link_reading() -> LinkReading {
    let Some(net) = handle() else {
        return alone_reading();
    };
    net.update(|shared| counted_reading(shared))
}

fn alone_reading() -> LinkReading {
    LinkReading {
        role: JumpRole::Solo,
        load_role: JumpRole::Solo,
        partner: PartnerLoad::default(),
        partner_place: StoryPlace::default(),
        campaign_link_up: false,
        partner_present: false,
        partner_watching: false,
        partner_free: false,
    }
}

fn counted_reading(shared: &NetShared) -> LinkReading {
    let latest = shared
        .companion_player()
        .and_then(|partner| partner.latest.as_ref());
    LinkReading {
        role: JumpRole::of_session(
            shared.session_mode(),
            shared.counts_as_connected(),
            shared.local_slot,
        ),
        load_role: load_role_of(shared),
        partner: partner_load_of(shared, net_clock_ms()),
        partner_place: partner_place_of(shared),
        campaign_link_up: shared.campaign_link_is_up(),
        partner_present: shared.campaign_partner().is_some(),
        partner_watching: latest.is_some_and(|latest| latest.state.watching),
        partner_free: latest.is_some_and(|latest| latest.state.free),
    }
}
