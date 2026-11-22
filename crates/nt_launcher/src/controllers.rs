mod linux;
mod system;
#[cfg(windows)]
mod xinput;

pub use linux::{INPUT_DEVICE_FOLDERS, linux_controller_present};
pub use system::SystemControllers;
