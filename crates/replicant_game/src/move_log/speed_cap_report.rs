use crate::move_log::{
    SPEED_CAP_RATE_CHANNEL, SPEED_CAP_RATE_ID, SPEED_CAP_REPORT_WINDOW_AFTER_LOW_MS,
    state_name_for_rva,
};

#[derive(Debug, Default)]
pub struct SpeedCapWatch {
    last_low_ms: Option<u64>,
    last_second: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct SpeedCapSecond {
    pub phase: u32,
    pub swapped_body: bool,
    pub cap: Option<f32>,
    pub step_per_second: Option<f32>,
    pub ground_speed: Option<f32>,
    pub state_rva: Option<u32>,
}

impl SpeedCapWatch {
    pub fn due(&mut self, now_ms: u64, house_cap_seen: bool) -> bool {
        if house_cap_seen {
            self.last_low_ms = Some(now_ms);
        }
        let indoors = self
            .last_low_ms
            .is_some_and(|low| now_ms.saturating_sub(low) <= SPEED_CAP_REPORT_WINDOW_AFTER_LOW_MS);
        let second = now_ms / 1000;
        if !indoors || self.last_second == Some(second) {
            return false;
        }
        self.last_second = Some(second);
        true
    }
}

pub fn format_speed_cap_second(second: &SpeedCapSecond) -> String {
    let number = |value: Option<f32>| value.map_or_else(|| "?".to_owned(), |v| format!("{v:.2}"));
    let state = second.state_rva.map_or("none", state_name_for_rva);
    let body = if second.swapped_body {
        "swapped body"
    } else {
        "own body"
    };
    format!(
        "indoor walk: {body}, phase {}: {SPEED_CAP_RATE_ID}:{SPEED_CAP_RATE_CHANNEL} {}, step {}/s at ground speed {}, state {state}",
        second.phase,
        number(second.cap),
        number(second.step_per_second),
        number(second.ground_speed),
    )
}
