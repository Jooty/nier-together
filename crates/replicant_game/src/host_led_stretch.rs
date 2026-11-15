mod lines;
mod state;
mod stretches;

pub use lines::{
    armed_line, disarmed_line, door_refused_line, ended_line, first_step_refused_line,
    host_run_not_replayed_line, started_line, window_size_mirrored_line, window_size_sent_line,
};
pub use state::{HostLedStretchState, StretchChange, StretchEnd, StretchPlace};
pub use stretches::{
    HOST_LED_STRETCHES, HostLedStretch, stretch_armed_by, stretch_starting_at,
    stretch_the_guest_waits_to_start,
};

pub const HOST_LED_STRETCH_ENDS_ITSELF_AFTER_MS: u64 = 15 * 60 * 1_000;
pub const HOST_LED_LONG_STRETCH_ENDS_ITSELF_AFTER_MS: u64 = 3 * 60 * 60 * 1_000;
pub const GUEST_REFUSED_DOOR_CALLBACK_PREFIX: &str = "Open_Door";
pub const GUEST_DOOR_REFUSED_TEXT: &str = "The door will not open for you";
pub const GUEST_WAITS_FOR_THE_HOSTS_TALK_TEXT: &str = "The host starts this talk";
