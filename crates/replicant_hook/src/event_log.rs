mod global;

pub use global::{
    clear_persistent_event, event_log_enabled, post_event, set_event_log_enabled,
    set_persistent_event, take_visible_events,
};
