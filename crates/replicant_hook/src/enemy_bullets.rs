//! Host enemies' bullets: sent by the host, drawn on the guest as effects or through the game's
//! sprite batch (visual only).

mod bullet_start;
mod script_commands;
mod send;
mod shot_cues;
mod shot_sounds;
mod show;
mod sprite_draw;
mod tick;

pub(crate) use bullet_start::bullet_start_in_flight;
pub use bullet_start::install as install_bullet_starts;
pub(crate) use script_commands::command_in_flight;
pub use script_commands::install as install_script_commands;
pub(crate) use shot_cues::{note_effect_made, note_sound_made};
pub(crate) use shot_sounds::play_shot_sound;
pub use sprite_draw::install;
pub use tick::tick;

const ENEMY_BULLET_SUMMARY_INTERVAL_MS: u64 = 5000;
