use nt_launcher::app::Launcher;
use nt_launcher::config::LauncherConfig;
use nt_launcher::controllers::linux_controller_present;
use nt_launcher::game_link::{write_keyboard_mouse, write_settings};
use nt_launcher_core::{
    AUTOMATA, ControllerCheck, KeyboardMouseFixes, PlayerSettings, REPLICANT, SessionOptions,
};

use crate::scratch::fresh_dir;

struct Controllers(bool);

impl ControllerCheck for Controllers {
    fn controller_connected(&self) -> bool {
        self.0
    }
}

fn automata_launcher(controller: bool, name: &str) -> Launcher {
    let mut launcher = Launcher::with_config(LauncherConfig::default());
    launcher.config_file = Some(fresh_dir(name).join("launcher.toml"));
    launcher.profile = &AUTOMATA;
    launcher.controller_check = Box::new(Controllers(controller));
    launcher
}

fn fixes(on: bool) -> KeyboardMouseFixes {
    KeyboardMouseFixes { on, asked: true }
}

#[test]
fn no_controller_and_never_asked_means_the_lobby_prompts() {
    assert!(
        automata_launcher(
            false,
            "km_no_controller_and_never_asked_means_the_lobby_prompts"
        )
        .keyboard_mouse_offer_due()
    );
}

#[test]
fn a_controller_means_the_lobby_does_not_prompt() {
    assert!(
        !automata_launcher(true, "km_a_controller_means_the_lobby_does_not_prompt")
            .keyboard_mouse_offer_due()
    );
}

#[test]
fn replicant_never_prompts() {
    let mut launcher = automata_launcher(false, "km_replicant_never_prompts");
    launcher.profile = &REPLICANT;
    assert!(!launcher.keyboard_mouse_offer_due());
}

#[test]
fn an_answer_stops_the_prompt_and_is_remembered_in_the_config() {
    let mut launcher = automata_launcher(
        false,
        "km_an_answer_stops_the_prompt_and_is_remembered_in_the_config",
    );
    launcher.answer_keyboard_mouse_offer(true);
    assert!(!launcher.keyboard_mouse_offer_due());
    assert_eq!(launcher.config.keyboard_mouse, fixes(true));
    let saved = launcher.config_file.as_deref().unwrap();
    assert_eq!(LauncherConfig::load(saved).keyboard_mouse, fixes(true));
}

#[test]
fn the_settings_switch_stores_the_same_value_as_the_prompt() {
    let mut from_prompt = automata_launcher(false, "km_switch_from_prompt");
    from_prompt.answer_keyboard_mouse_offer(true);
    let mut from_settings = automata_launcher(false, "km_switch_from_settings");
    from_settings.set_keyboard_mouse_fixes(true);
    assert_eq!(
        from_prompt.config.keyboard_mouse,
        from_settings.config.keyboard_mouse
    );
}

#[test]
fn a_controller_found_later_does_not_undo_the_answer() {
    let mut launcher = automata_launcher(
        false,
        "km_a_controller_found_later_does_not_undo_the_answer",
    );
    launcher.answer_keyboard_mouse_offer(true);
    launcher.controller_check = Box::new(Controllers(true));
    launcher.forget_controller_check();
    assert!(!launcher.keyboard_mouse_offer_due());
    assert!(launcher.config.keyboard_mouse.on);
}

#[test]
fn the_controller_is_checked_again_after_leaving_the_lobby() {
    let mut launcher = automata_launcher(
        true,
        "km_the_controller_is_checked_again_after_leaving_the_lobby",
    );
    assert!(!launcher.keyboard_mouse_offer_due());
    launcher.controller_check = Box::new(Controllers(false));
    assert!(!launcher.keyboard_mouse_offer_due());
    launcher.forget_controller_check();
    assert!(launcher.keyboard_mouse_offer_due());
}

#[test]
fn the_choice_survives_a_save_and_load() {
    let dir = fresh_dir("keyboard_mouse_config");
    let path = dir.join("launcher.toml");
    let config = LauncherConfig {
        keyboard_mouse: fixes(true),
        ..LauncherConfig::default()
    };
    config.save(&path).unwrap();
    assert_eq!(LauncherConfig::load(&path).keyboard_mouse, fixes(true));
}

#[test]
fn an_unanswered_choice_adds_nothing_to_the_config_file() {
    assert!(
        !LauncherConfig::default()
            .render()
            .contains("keyboard_mouse")
    );
}

#[test]
fn the_game_file_gets_raw_mouse_true_when_on() {
    let dir = fresh_dir("keyboard_mouse_on");
    write_keyboard_mouse(&AUTOMATA, &dir, fixes(true)).unwrap();
    let text = std::fs::read_to_string(dir.join(AUTOMATA.link_file_name)).unwrap();
    assert!(text.contains("raw_mouse = true"));
}

#[test]
fn turning_it_off_removes_the_key_so_the_game_default_off_applies() {
    let dir = fresh_dir("keyboard_mouse_off");
    write_keyboard_mouse(&AUTOMATA, &dir, fixes(true)).unwrap();
    write_keyboard_mouse(&AUTOMATA, &dir, fixes(false)).unwrap();
    let text = std::fs::read_to_string(dir.join(AUTOMATA.link_file_name)).unwrap();
    assert!(!text.contains("raw_mouse"));
}

#[test]
fn the_other_game_settings_keep_the_raw_mouse_key() {
    let dir = fresh_dir("keyboard_mouse_kept");
    write_keyboard_mouse(&AUTOMATA, &dir, fixes(true)).unwrap();
    write_settings(
        &AUTOMATA,
        &dir,
        &PlayerSettings::default(),
        &SessionOptions::default(),
    )
    .unwrap();
    let text = std::fs::read_to_string(dir.join(AUTOMATA.link_file_name)).unwrap();
    assert!(text.contains("raw_mouse = true"));
}

#[test]
fn a_joystick_node_means_a_controller() {
    let dir = fresh_dir("controllers_js");
    std::fs::write(dir.join("js0"), "").unwrap();
    assert!(linux_controller_present(&[dir.as_path()]));
}

#[test]
fn a_joystick_link_by_id_means_a_controller() {
    let dir = fresh_dir("controllers_by_id");
    std::fs::write(dir.join("usb-Sony_Controller-event-joystick"), "").unwrap();
    assert!(linux_controller_present(&[dir.as_path()]));
}

#[test]
fn keyboards_mice_and_other_nodes_are_not_controllers() {
    let dir = fresh_dir("controllers_none");
    for name in [
        "usb-Logitech_Mouse-event-mouse",
        "usb-Corsair_Keyboard-event-kbd",
        "event3",
        "js",
        "jsx",
        "mouse0",
    ] {
        std::fs::write(dir.join(name), "").unwrap();
    }
    assert!(!linux_controller_present(&[dir.as_path()]));
}

#[test]
fn a_missing_folder_means_no_controller() {
    let dir = fresh_dir("controllers_missing").join("not-there");
    assert!(!linux_controller_present(&[dir.as_path()]));
}
