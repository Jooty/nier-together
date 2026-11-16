use crate::callback_gate::event_base;
use crate::stay_together::{PARTNER_TIMEOUT_MS, distance_between};
use crate::talk_guard::{
    IMPORTANT_TALK_BASES, IMPORTANT_TALK_IDS, TALK_IDS_THAT_NEED_THE_PARTNER_IN_THE_HOUSE,
};
use crate::zone_rules::{Companion, Placement, ZoneRadius};

#[derive(Debug, Clone, PartialEq)]
pub struct PressAsk<'a> {
    pub connected: bool,
    pub partner_game_open: bool,
    pub local: Option<Placement>,
    pub partner: Option<&'a Companion>,
    pub radius: ZoneRadius,
    pub talk_id: Option<i32>,
    pub partner_side_view: Option<PartnerSideView>,
}

/// The partner's last [`replicant_proto::SideViewState`] report, with its age.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PartnerSideView {
    pub phase: u16,
    pub in_side_view: bool,
    pub age_ms: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PressVerdict {
    Allow,
    Wait {
        partner: String,
        distance_units: f32,
    },
    WaitForTheHouse {
        partner: String,
        distance_units: f32,
    },
}

impl PressVerdict {
    pub fn prompt_text(&self) -> Option<String> {
        match self {
            Self::Allow => None,
            Self::Wait { partner, .. } => Some(format!(
                "All players have to be there to start this ({partner} is missing)"
            )),
            Self::WaitForTheHouse { partner, .. } => Some(format!(
                "All players have to be inside the house to start this ({partner} is outside)"
            )),
        }
    }
}

pub fn is_important_talk(name: &str) -> bool {
    IMPORTANT_TALK_BASES
        .binary_search(&event_base(name))
        .is_ok()
}

pub fn important_base_of_talk_id(talk_id: i32) -> Option<&'static str> {
    let first = IMPORTANT_TALK_IDS.partition_point(|(id, _)| *id < talk_id);
    IMPORTANT_TALK_IDS
        .get(first)
        .filter(|(id, _)| *id == talk_id)
        .map(|(_, base)| *base)
}

pub fn talk_needs_the_partner_in_the_house(talk_id: i32) -> bool {
    TALK_IDS_THAT_NEED_THE_PARTNER_IN_THE_HOUSE.contains(&talk_id)
}

/// Whether the partner's game reports its side view on in this phase, freshly. The side view is
/// on from a house's entry step to its exit step, so a partner on the doorstep is outside.
pub fn partner_inside(local_phase: u16, side_view: Option<PartnerSideView>) -> bool {
    side_view.is_some_and(|view| {
        view.in_side_view && view.phase == local_phase && view.age_ms <= PARTNER_TIMEOUT_MS
    })
}

pub fn judge_press(ask: &PressAsk<'_>) -> PressVerdict {
    if !ask.connected || !ask.partner_game_open {
        return PressVerdict::Allow;
    }
    let (Some(local), Some(partner)) = (ask.local, ask.partner) else {
        return PressVerdict::Allow;
    };
    let Some(placement) = partner.placement else {
        return PressVerdict::Allow;
    };
    if partner.age_ms > PARTNER_TIMEOUT_MS || placement.phase != local.phase {
        return PressVerdict::Allow;
    }
    let distance = distance_between(local.position, placement.position);
    if let Some(distance) = distance
        && distance > ask.radius.units()
    {
        return PressVerdict::Wait {
            partner: partner.name.clone(),
            distance_units: distance,
        };
    }
    let needs_the_house = ask.talk_id.is_some_and(talk_needs_the_partner_in_the_house);
    if needs_the_house && !partner_inside(local.phase, ask.partner_side_view) {
        return PressVerdict::WaitForTheHouse {
            partner: partner.name.clone(),
            distance_units: distance.unwrap_or_default(),
        };
    }
    PressVerdict::Allow
}
