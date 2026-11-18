mod campaign_load;
mod chapter_table_read;
mod driver;
mod driver_state;
mod link_reading;
mod session_jump_driver;

pub use driver::{install, start_index_of_record, status_line, tick};
pub(crate) use link_reading::link_reading;
pub use session_jump_driver::{ask_session_jump, run_session_jump};

const SCENE_NAME_READ_LIMIT: usize = 0x20;
const NOT_READ_SENTINEL: i32 = -99;
