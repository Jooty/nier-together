//! The defeat screen led by the host, applied in the game's own defeat screen update.

mod state;
mod update_hook;

pub use state::tick;
pub use update_hook::install;
