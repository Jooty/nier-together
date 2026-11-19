use std::fmt;

use crate::start::{PlayReadings, is_in_play};
use crate::travel::JumpRole;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionJumpOrigin {
    ThisPlayer,
    TheHostsOrder,
}

impl SessionJumpOrigin {
    pub fn label(self) -> &'static str {
        match self {
            Self::ThisPlayer => "asked here",
            Self::TheHostsOrder => "the host's order",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionJumpReadings {
    pub play: PlayReadings,
    pub role: JumpRole,
    pub not_free_reason: Option<String>,
    pub defeat_flow_active: bool,
    pub hero_is_down: bool,
    pub load_under_way: bool,
    pub jump_under_way: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionJumpRefusal {
    OnlyTheHostAsks,
    OnlyTheGuestFollows,
    JumpUnderWay,
    NotInPlay,
    LoadUnderWay,
    PlayerNotFree(String),
    DefeatFlowActive,
    HeroIsDown,
}

impl fmt::Display for SessionJumpRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OnlyTheHostAsks => write!(formatter, "only the host starts a jump in a session"),
            Self::OnlyTheGuestFollows => {
                write!(
                    formatter,
                    "an order of the host reached a game that is not the guest"
                )
            }
            Self::JumpUnderWay => write!(formatter, "another jump is under way"),
            Self::NotInPlay => write!(formatter, "the scene is not in play"),
            Self::LoadUnderWay => write!(formatter, "a load is under way"),
            Self::PlayerNotFree(reason) => write!(formatter, "the player is not free ({reason})"),
            Self::DefeatFlowActive => write!(formatter, "the defeat flow is running"),
            Self::HeroIsDown => write!(formatter, "the hero is down"),
        }
    }
}

pub fn decide_session_jump(
    origin: SessionJumpOrigin,
    readings: &SessionJumpReadings,
) -> Result<(), SessionJumpRefusal> {
    match (origin, readings.role) {
        (SessionJumpOrigin::ThisPlayer, JumpRole::Guest) => {
            return Err(SessionJumpRefusal::OnlyTheHostAsks);
        }
        (SessionJumpOrigin::TheHostsOrder, JumpRole::Solo | JumpRole::Host) => {
            return Err(SessionJumpRefusal::OnlyTheGuestFollows);
        }
        _ => {}
    }
    if readings.jump_under_way {
        return Err(SessionJumpRefusal::JumpUnderWay);
    }
    if readings.load_under_way {
        return Err(SessionJumpRefusal::LoadUnderWay);
    }
    if !is_in_play(&readings.play) {
        return Err(SessionJumpRefusal::NotInPlay);
    }
    if readings.defeat_flow_active {
        return Err(SessionJumpRefusal::DefeatFlowActive);
    }
    if readings.hero_is_down {
        return Err(SessionJumpRefusal::HeroIsDown);
    }
    match &readings.not_free_reason {
        Some(reason) => Err(SessionJumpRefusal::PlayerNotFree(reason.clone())),
        None => Ok(()),
    }
}
