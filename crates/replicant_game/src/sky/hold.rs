use replicant_proto::SkyState;

use crate::sky::{APPLY_RETRY_MS, SKY_APPLY_LIMIT_MS, SkyReading, ZONE_SETTLE_MS};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct SkyApply {
    pub weather: Option<(u8, u8)>,
    pub env: Option<(i32, f32)>,
    pub wind: Option<(i32, f32)>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SkyOutcome {
    pub phase: u16,
    pub timed_out: bool,
}

#[derive(Debug, Default)]
pub struct SkyHold {
    target: Option<SkyState>,
    fresh: bool,
    applied_before_resources_ready: bool,
    last_attempt_ms: Option<u64>,
    started_ms: Option<u64>,
    outcome: Option<SkyOutcome>,
    last_received: Option<SkyState>,
    zone_loads: Option<u32>,
    quiet_until_ms: u64,
}

impl SkyApply {
    pub fn is_empty(&self) -> bool {
        self.weather.is_none() && self.env.is_none() && self.wind.is_none()
    }
}

impl SkyHold {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn receive(&mut self, state: SkyState) {
        self.target = Some(state);
        self.last_received = Some(state);
        self.fresh = true;
        self.applied_before_resources_ready = false;
        self.last_attempt_ms = None;
        self.started_ms = None;
        self.outcome = None;
    }

    /// Counts a change of the guest's phase load count as a zone load: after the settle time the host's last sky for
    /// this phase is applied once more. Returns whether a sky is armed again, so the caller re-applies its record too.
    pub fn note_zone_loads(&mut self, now_ms: u64, guest_phase: u16, zone_loads: u32) -> bool {
        let previous = self.zone_loads.replace(zone_loads);
        if previous.is_none_or(|count| count == zone_loads) {
            return false;
        }
        self.quiet_until_ms = now_ms + ZONE_SETTLE_MS;
        let Some(state) = self
            .target
            .or(self.last_received)
            .filter(|state| state.phase == guest_phase)
        else {
            return false;
        };
        self.target = Some(state);
        self.fresh = true;
        self.applied_before_resources_ready = false;
        self.last_attempt_ms = None;
        self.started_ms = None;
        self.outcome = None;
        true
    }

    pub fn waits_for_zone_settle(&self, now_ms: u64) -> bool {
        now_ms < self.quiet_until_ms
    }

    pub fn take_outcome(&mut self) -> Option<SkyOutcome> {
        self.outcome.take()
    }

    pub fn note_area_set_outside_the_hold(&mut self) {
        if self.target.is_some() {
            self.applied_before_resources_ready = true;
        }
    }

    pub fn step(&mut self, now_ms: u64, guest_phase: u16, reading: &SkyReading) -> SkyApply {
        let Some(target) = self.target else {
            return SkyApply::default();
        };
        if target.phase != guest_phase
            || !reading.is_settled()
            || self.waits_for_zone_settle(now_ms)
        {
            return SkyApply::default();
        }
        let started_ms = *self.started_ms.get_or_insert(now_ms);
        let blend = if self.fresh {
            target.env_blend_seconds
        } else {
            0.0
        };
        let env_mismatch = reading.env_area != target.env_area;
        let resources_arrived = self.applied_before_resources_ready && reading.sky_resources_ready;
        let apply = SkyApply {
            weather: (reading.weather != target.weather
                || reading.weather_offset != target.weather_offset)
                .then_some((target.weather, target.weather_offset)),
            env: (env_mismatch || resources_arrived)
                .then_some((target.env_area, if env_mismatch { blend } else { 0.0 })),
            wind: (target.wind_area >= 0 && reading.wind_area != target.wind_area)
                .then_some((target.wind_area, blend)),
        };
        let timed_out = now_ms.saturating_sub(started_ms) >= SKY_APPLY_LIMIT_MS;
        if apply.is_empty() {
            self.fresh = false;
            self.last_attempt_ms = None;
            if !self.applied_before_resources_ready || timed_out {
                self.finish(target.phase, self.applied_before_resources_ready);
            }
            return apply;
        }
        if timed_out {
            self.finish(target.phase, true);
            return SkyApply::default();
        }
        if !self.fresh
            && self
                .last_attempt_ms
                .is_some_and(|last| now_ms.saturating_sub(last) < APPLY_RETRY_MS)
        {
            return SkyApply::default();
        }
        self.last_attempt_ms = Some(now_ms);
        self.fresh = false;
        if apply.env.is_some() {
            self.applied_before_resources_ready = env_mismatch && !reading.sky_resources_ready;
        }
        apply
    }

    fn finish(&mut self, phase: u16, timed_out: bool) {
        self.target = None;
        self.outcome = Some(SkyOutcome { phase, timed_out });
    }
}
