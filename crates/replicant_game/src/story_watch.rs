mod guest_watch;
mod host_word;
mod input_leak;
mod lines;
mod place;
mod removal;

pub use guest_watch::{GuestWatch, WatchEnd, WatchFacts, WatchStep};
pub use host_word::GuestWord;
pub use input_leak::{WATCHED_BODY_MOVES_ON_KEYS_ABOVE_SPEED, watched_body_moves_on_keys};
pub use lines::{
    camera_line, ended_line, guest_is_watching_text, started_line, stretch_started_line,
    watch_hold_line, watch_let_go_line, watching_host_text,
};
pub use place::{PlaceStep, place_step};
pub use removal::{RemovalChange, StoryRemoval};

pub const ACTIVATE_UNIQUE_ACTOR_NATIVE_RVA: u32 = 0x6E_3AA0;
pub const DISACTIVATE_UNIQUE_ACTOR_NATIVE_RVA: u32 = 0x6E_8770;
pub const COMPANION_SCRIPT_ID: i32 = -2;
pub const STORY_REMOVAL_SETTLES_MS: u64 = 1_500;
pub const GUEST_WORD_STALE_AFTER_MS: u64 = 6_000;
pub const GUEST_WORD_RESENT_EVERY_MS: u64 = 2_000;
pub const WATCH_BODY_LEFT_ALONE_BELOW: f32 = 0.05;
pub const WATCH_BODY_TELEPORTED_ABOVE: f32 = 1.5;
pub const WATCH_BODY_TELEPORT_EVERY_MS: u64 = 500;
pub const WATCH_LINE_EVERY_MS: u64 = 1_000;
pub const PACKS_WHERE_THE_GUEST_PLAYS_THOUGH_THE_STORY_TAKES_HER: [&str; 1] = ["B_STONE_FIELD_02"];
