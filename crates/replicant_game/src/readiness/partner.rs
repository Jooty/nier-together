use replicant_proto::GameReadiness;

use crate::readiness::{JOINING_IGNORED_AFTER_MS, SILENT_PARTNER_FALLBACK_MS};

pub const PARTNER_GAME_CLOSED_LABEL: &str = "disconnected: their game closed";
pub const PARTNER_WAITING_FOR_A_SAFE_MOMENT_LABEL: &str =
    "joining: waiting for a safe moment to take them in";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PartnerWatch {
    readiness: GameReadiness,
    connected_at_ms: Option<u64>,
    heard_from_partner: bool,
    game_closed: bool,
    joined_while_host_played: bool,
    waiting_for_safe_moment: bool,
}

impl PartnerWatch {
    pub const fn new() -> Self {
        Self {
            readiness: GameReadiness::NotLinked,
            connected_at_ms: None,
            heard_from_partner: false,
            game_closed: false,
            joined_while_host_played: false,
            waiting_for_safe_moment: false,
        }
    }

    pub fn partner_connected(&mut self, now_ms: u64) {
        self.readiness = GameReadiness::NotLinked;
        self.connected_at_ms = Some(now_ms);
        self.heard_from_partner = false;
        self.game_closed = false;
        self.joined_while_host_played = false;
        self.waiting_for_safe_moment = false;
    }

    pub fn partner_connected_while_host_plays(&mut self, now_ms: u64) {
        self.partner_connected(now_ms);
        self.joined_while_host_played = true;
        self.waiting_for_safe_moment = true;
    }

    pub fn joined_while_host_played(self) -> bool {
        self.connected_at_ms.is_some() && self.joined_while_host_played
    }

    pub fn is_waiting_for_safe_moment(self) -> bool {
        self.connected_at_ms.is_some() && self.waiting_for_safe_moment
    }

    pub fn take_in_if_safe(&mut self, host_is_safe: bool) -> bool {
        if !self.is_waiting_for_safe_moment() || !self.readiness.is_in_play() || !host_is_safe {
            return false;
        }
        self.waiting_for_safe_moment = false;
        true
    }

    pub fn partner_sent_state(&mut self) -> bool {
        if self.connected_at_ms.is_none() || self.readiness.is_in_play() {
            return false;
        }
        self.readiness = GameReadiness::InPlay;
        self.heard_from_partner = true;
        self.game_closed = false;
        !self.waiting_for_safe_moment
    }

    pub fn assume_in_play_if_silent(&mut self, now_ms: u64) -> bool {
        let Some(connected_at) = self.connected_at_ms else {
            return false;
        };
        if self.heard_from_partner
            || now_ms.saturating_sub(connected_at) < SILENT_PARTNER_FALLBACK_MS
        {
            return false;
        }
        self.readiness = GameReadiness::InPlay;
        self.heard_from_partner = true;
        !self.waiting_for_safe_moment
    }

    pub fn partner_gone(&mut self) {
        *self = Self::new();
    }

    pub fn partner_reported(&mut self, readiness: GameReadiness) -> bool {
        if self.connected_at_ms.is_none() {
            return false;
        }
        let just_closed = readiness == GameReadiness::NotLinked
            && self.readiness != GameReadiness::NotLinked
            && !self.game_closed;
        self.game_closed =
            readiness == GameReadiness::NotLinked && (self.game_closed || just_closed);
        self.readiness = readiness;
        self.heard_from_partner = true;
        just_closed
    }

    pub fn is_game_closed(self) -> bool {
        self.connected_at_ms.is_some() && self.game_closed
    }

    pub fn readiness(self) -> GameReadiness {
        self.readiness
    }

    pub fn is_in_play(self) -> bool {
        self.connected_at_ms.is_some()
            && self.readiness.is_in_play()
            && !self.waiting_for_safe_moment
    }

    pub fn is_joining(self) -> bool {
        self.connected_at_ms.is_some() && !self.is_in_play()
    }

    pub fn counts_for_the_host_rules(self) -> bool {
        if self.joined_while_host_played() {
            return self.is_in_play();
        }
        !self.is_game_closed()
    }

    pub fn counts_for_zone_rule(self, now_ms: u64) -> bool {
        let Some(connected_at) = self.connected_at_ms else {
            return false;
        };
        if self.game_closed {
            return false;
        }
        self.readiness.is_in_play()
            || now_ms.saturating_sub(connected_at) < JOINING_IGNORED_AFTER_MS
    }
}

pub fn joining_label(readiness: GameReadiness) -> &'static str {
    match readiness {
        GameReadiness::NotLinked => "joining: their game is not open yet",
        GameReadiness::Loading => "joining: their game is on the title screen or loading",
        GameReadiness::InPlay => "in play",
    }
}
