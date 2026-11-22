mod choice;

pub use choice::{RawMouseChoice, choose_raw_mouse, raw_mouse_from_launcher};

pub const RAW_MOUSE_ENV: &str = "NIERTOGETHER_RAW_MOUSE";
pub const RAW_MOUSE_KEY: &str = "raw_mouse";
