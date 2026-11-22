use std::path::Path;

pub const INPUT_DEVICE_FOLDERS: [&str; 3] =
    ["/sys/class/input", "/dev/input/by-id", "/dev/input/by-path"];
const JOYSTICK_NODE_PREFIX: &str = "js";
const JOYSTICK_LINK_SUFFIX: &str = "-joystick";

pub fn linux_controller_present(folders: &[&Path]) -> bool {
    folders
        .iter()
        .filter_map(|folder| std::fs::read_dir(folder).ok())
        .flatten()
        .filter_map(Result::ok)
        .any(|entry| is_controller_name(&entry.file_name().to_string_lossy()))
}

fn is_controller_name(name: &str) -> bool {
    let is_joystick_node = name
        .strip_prefix(JOYSTICK_NODE_PREFIX)
        .is_some_and(|number| {
            !number.is_empty() && number.bytes().all(|byte| byte.is_ascii_digit())
        });
    is_joystick_node || name.ends_with(JOYSTICK_LINK_SUFFIX)
}
