mod body;
mod guest_side;
mod host_side;
mod removal_natives;
mod setting;
mod watched_camera;

pub(crate) use guest_side::{follow_the_host, this_game_is_watching, tick_story_watch};
pub(crate) use host_side::guest_is_watched;
pub(crate) use removal_natives::{forget_removal, install};
pub(crate) use setting::set_guest_watches;
pub(crate) use watched_camera::{
    fresh_host_room_status, host_camera_native_ran, install_status_hook,
};
