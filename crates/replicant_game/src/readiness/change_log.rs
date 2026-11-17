use replicant_proto::GameReadiness;

use super::{PlayChecks, REPORT_STALE_MS};

const STILL_LOADING_AFTER_MS: u64 = 10_000;
const STILL_LOADING_REPEAT_MS: u64 = 5_000;

#[derive(Debug, Default)]
pub struct ReadinessChangeLog {
    last: Option<(GameReadiness, u64)>,
    last_still_loading_at_ms: Option<u64>,
}

impl ReadinessChangeLog {
    pub fn observe(
        &mut self,
        now_ms: u64,
        current: GameReadiness,
        waiting_on: &str,
        reads: &str,
        last_send: &str,
    ) -> Option<String> {
        let detail = format!("waiting on {waiting_on}; reads: {reads}; send: {last_send}");
        match self.last {
            Some((previous, since_ms)) if previous == current => {
                let in_state_ms = now_ms.saturating_sub(since_ms);
                let repeat_due = self
                    .last_still_loading_at_ms
                    .is_none_or(|at| now_ms.saturating_sub(at) >= STILL_LOADING_REPEAT_MS);
                if current == GameReadiness::Loading
                    && in_state_ms > STILL_LOADING_AFTER_MS
                    && repeat_due
                {
                    self.last_still_loading_at_ms = Some(now_ms);
                    return Some(format!(
                        "readiness: this game still {current:?} after {in_state_ms} ms, {detail}"
                    ));
                }
                None
            }
            Some((previous, since_ms)) => {
                self.last = Some((current, now_ms));
                self.last_still_loading_at_ms = None;
                Some(format!(
                    "readiness: this game {current:?}, was {previous:?} for {} ms, {detail}",
                    now_ms.saturating_sub(since_ms)
                ))
            }
            None => {
                self.last = Some((current, now_ms));
                self.last_still_loading_at_ms = None;
                Some(format!(
                    "readiness: this game {current:?}, first report, {detail}"
                ))
            }
        }
    }

    pub fn reset(&mut self) {
        self.last = None;
        self.last_still_loading_at_ms = None;
    }
}

pub fn waiting_on(checks: PlayChecks, report_age_ms: Option<u64>) -> String {
    let Some(age_ms) = report_age_ms else {
        return "the game's first report".to_owned();
    };
    if age_ms > REPORT_STALE_MS {
        return format!("no report from the game for {age_ms} ms");
    }
    let missing: Vec<&str> = [
        (checks.phase_known, "the phase"),
        (checks.player_actor_spawned, "the player actor"),
        (checks.vitals_readable, "the vitals"),
    ]
    .into_iter()
    .filter_map(|(ready, name)| (!ready).then_some(name))
    .collect();
    if missing.is_empty() {
        "nothing".to_owned()
    } else {
        missing.join(", ")
    }
}
