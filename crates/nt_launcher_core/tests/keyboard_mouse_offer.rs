use nt_launcher_core::{
    AUTOMATA, ControllerCheck, KEYBOARD_MOUSE_FIXES_EXPLAINER_LINES,
    KEYBOARD_MOUSE_FIXES_HOVER_TEXT, KeyboardMouseFixes, REPLICANT, offers_keyboard_mouse_fixes,
};

struct Controllers(bool);

impl ControllerCheck for Controllers {
    fn controller_connected(&self) -> bool {
        self.0
    }
}

#[test]
fn no_controller_and_never_asked_means_the_offer_shows() {
    assert!(KeyboardMouseFixes::default().offer_due(&Controllers(false)));
}

#[test]
fn a_controller_means_no_offer() {
    assert!(!KeyboardMouseFixes::default().offer_due(&Controllers(true)));
}

#[test]
fn once_asked_the_offer_never_shows_again() {
    let mut fixes = KeyboardMouseFixes::default();
    fixes.answer_offer(false);
    assert!(!fixes.offer_due(&Controllers(false)));
}

#[test]
fn turning_on_from_the_offer_stores_on_and_asked() {
    let mut fixes = KeyboardMouseFixes::default();
    fixes.answer_offer(true);
    assert_eq!(
        fixes,
        KeyboardMouseFixes {
            on: true,
            asked: true
        }
    );
}

#[test]
fn not_now_keeps_the_fixes_off() {
    let mut fixes = KeyboardMouseFixes::default();
    fixes.answer_offer(false);
    assert_eq!(
        fixes,
        KeyboardMouseFixes {
            on: false,
            asked: true
        }
    );
}

#[test]
fn the_settings_switch_writes_the_value_the_offer_writes() {
    let mut from_offer = KeyboardMouseFixes::default();
    from_offer.answer_offer(true);
    let mut from_settings = KeyboardMouseFixes::default();
    from_settings.set_in_settings(true);
    assert_eq!(from_offer, from_settings);
}

#[test]
fn using_the_settings_switch_means_the_offer_is_not_needed() {
    let mut fixes = KeyboardMouseFixes::default();
    fixes.set_in_settings(false);
    assert!(!fixes.offer_due(&Controllers(false)));
}

#[test]
fn a_controller_plugged_in_later_does_not_undo_the_choice() {
    let mut fixes = KeyboardMouseFixes::default();
    fixes.answer_offer(true);
    assert!(!fixes.offer_due(&Controllers(true)));
    assert!(fixes.on);
}

#[test]
fn only_automata_offers_the_fixes() {
    assert!(offers_keyboard_mouse_fixes(&AUTOMATA));
    assert!(!offers_keyboard_mouse_fixes(&REPLICANT));
}

#[test]
fn the_text_has_no_em_dash() {
    for line in KEYBOARD_MOUSE_FIXES_EXPLAINER_LINES {
        assert!(!line.contains('\u{2014}'));
    }
    assert!(!KEYBOARD_MOUSE_FIXES_HOVER_TEXT.contains('\u{2014}'));
}
