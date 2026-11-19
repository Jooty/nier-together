use std::time::Instant;

use replicant_game::readiness::{PlayReads, REPORT_STALE_MS, waiting_on};
use replicant_proto::{GameReadiness, PlayerSettings, SessionOptions};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ReplicantLinkState {
    pub options: SessionOptions,
    pub pending_settings: Option<PlayerSettings>,
    pub local_readiness: GameReadiness,
    pub local_readiness_at: Option<Instant>,
    pub local_reads: PlayReads,
}

impl ReplicantLinkState {
    pub fn effective_local_readiness(&self) -> GameReadiness {
        let fresh = self
            .local_readiness_at
            .is_some_and(|at| at.elapsed().as_millis() as u64 <= REPORT_STALE_MS);
        if fresh {
            self.local_readiness
        } else {
            GameReadiness::Loading
        }
    }

    pub fn local_readiness_wait(&self) -> String {
        let age_ms = self
            .local_readiness_at
            .map(|at| at.elapsed().as_millis() as u64);
        waiting_on(self.local_reads.checks(), age_ms)
    }
}
