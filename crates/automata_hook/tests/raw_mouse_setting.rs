use automata_hook::raw_mouse_setting::choose_raw_mouse;

#[test]
fn nothing_given_means_raw_movement_is_off() {
    assert!(!choose_raw_mouse(None, None).on);
}

#[test]
fn the_config_file_can_turn_it_on_or_leave_it_off() {
    assert!(choose_raw_mouse(None, Some("raw_mouse = true")).on);
    assert!(choose_raw_mouse(None, Some("raw_mouse = \"on\"")).on);
    assert!(!choose_raw_mouse(None, Some("raw_mouse = false")).on);
}

#[test]
fn the_environment_wins_over_the_config_file() {
    let choice = choose_raw_mouse(Some("on".to_owned()), Some("raw_mouse = \"off\""));
    assert!(choice.on);
    let choice = choose_raw_mouse(Some("off".to_owned()), Some("raw_mouse = true"));
    assert!(!choice.on);
}

#[test]
fn numbers_and_words_for_a_flag_are_understood() {
    assert!(!choose_raw_mouse(Some("0".to_owned()), None).on);
    assert!(choose_raw_mouse(Some("1".to_owned()), None).on);
    assert!(!choose_raw_mouse(Some("false".to_owned()), None).on);
}

#[test]
fn an_unknown_value_keeps_it_off_and_says_so() {
    let choice = choose_raw_mouse(Some("maybe".to_owned()), None);
    assert!(!choice.on);
    assert!(choice.log_line.contains("maybe"));
}

#[test]
fn an_unreadable_config_changes_nothing() {
    assert!(!choose_raw_mouse(None, Some("this is [not toml")).on);
}
