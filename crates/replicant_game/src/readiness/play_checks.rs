use replicant_proto::GameReadiness;

use crate::stay_together::local_phase_index;
use crate::{GameMemory, Replicant};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PlayChecks {
    pub phase_known: bool,
    pub player_actor_spawned: bool,
    pub vitals_readable: bool,
}

pub fn classify(checks: PlayChecks) -> GameReadiness {
    if checks.phase_known && checks.player_actor_spawned && checks.vitals_readable {
        GameReadiness::InPlay
    } else {
        GameReadiness::Loading
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PlayReads {
    pub zone: Option<String>,
    pub phase_known: bool,
    pub player_actor_spawned: bool,
    pub position_readable: bool,
    pub max_health: Option<i32>,
}

impl PlayReads {
    pub fn checks(&self) -> PlayChecks {
        PlayChecks {
            phase_known: self.phase_known,
            player_actor_spawned: self.player_actor_spawned,
            vitals_readable: self.position_readable && self.max_health.is_some_and(|max| max > 0),
        }
    }

    pub fn describe(&self) -> String {
        let zone = self.zone.as_deref().unwrap_or("unreadable");
        let max_health = self
            .max_health
            .map_or_else(|| "unreadable".to_owned(), |max| max.to_string());
        format!(
            "zone={zone} phase_known={} actor={} position={} max_health={max_health}",
            self.phase_known, self.player_actor_spawned, self.position_readable
        )
    }
}

pub fn read_play_reads<M: GameMemory>(game: &Replicant<M>) -> PlayReads {
    PlayReads {
        zone: game.current_phase().ok(),
        phase_known: local_phase_index(game).is_some(),
        player_actor_spawned: game.playable_actor().is_ok(),
        position_readable: game.position().is_ok(),
        max_health: game.max_health().ok(),
    }
}

pub fn read_play_checks<M: GameMemory>(game: &Replicant<M>) -> PlayChecks {
    read_play_reads(game).checks()
}
