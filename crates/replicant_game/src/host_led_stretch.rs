mod lines;
mod state;
mod stretches;

pub use lines::{ended_line, started_line};
pub use state::{HostLedStretchState, StretchChange, StretchEnd};
pub use stretches::{HOST_LED_STRETCHES, HostLedStretch, stretch_starting_at};

pub const HOST_LED_STRETCH_ENDS_ITSELF_AFTER_MS: u64 = 15 * 60 * 1_000;
