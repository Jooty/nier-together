use nt_launcher_core::ControllerCheck;

pub struct SystemControllers;

impl ControllerCheck for SystemControllers {
    #[cfg(windows)]
    fn controller_connected(&self) -> bool {
        crate::controllers::xinput::xinput_controller_connected()
    }

    #[cfg(not(windows))]
    fn controller_connected(&self) -> bool {
        let folders: Vec<&std::path::Path> = crate::controllers::INPUT_DEVICE_FOLDERS
            .iter()
            .map(std::path::Path::new)
            .collect();
        crate::controllers::linux_controller_present(&folders)
    }
}
