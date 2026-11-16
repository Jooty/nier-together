use replicant_game::dialogue::{MirrorWindowSize, layout_size_text};

const PLAIN_SIZE: i32 = 0;
const FULL_PAGE_SIZE: i32 = 9;
const FULL_PAGE_LAYOUT: i32 = 0xB;
const PLAIN_LAYOUT: i32 = 1;

#[test]
fn run_1116a_the_mirror_opens_the_hosts_dream_talk_at_the_hosts_full_page_size() {
    // Run 1116a: the guest's own EID_1500_a0005_040_Select01 set size 0, the host's
    // EID_1500_a0005_010_01_Start set 9 before `_SetTalk(1422)`; the guest's mirror opened 1422
    // at 08:29:46.876 with size 0, a plain window.
    let mut sizes = MirrorWindowSize::new();
    sizes.note_own_size(PLAIN_SIZE);
    sizes.note_host_size(FULL_PAGE_SIZE);
    assert_eq!(sizes.size_for_the_mirror(), Some(FULL_PAGE_SIZE));
    assert_eq!(sizes.size_for_the_mirror(), None);
    assert_eq!(sizes.size_to_give_back(), Some(PLAIN_SIZE));
    assert_eq!(sizes.size_to_give_back(), None);
}

#[test]
fn a_size_the_guests_own_script_set_meanwhile_is_kept_after_the_mirror() {
    let mut sizes = MirrorWindowSize::new();
    sizes.note_host_size(FULL_PAGE_SIZE);
    assert_eq!(sizes.size_for_the_mirror(), Some(FULL_PAGE_SIZE));
    sizes.note_own_size(FULL_PAGE_SIZE);
    assert_eq!(sizes.size_to_give_back(), None);
}

#[test]
fn the_window_is_left_alone_while_no_host_size_is_known_or_both_sizes_match() {
    let mut sizes = MirrorWindowSize::new();
    assert_eq!(sizes.size_for_the_mirror(), None);
    sizes.note_host_size(PLAIN_SIZE);
    assert_eq!(sizes.size_for_the_mirror(), None);
    assert_eq!(sizes.size_to_give_back(), None);
}

#[test]
fn the_stored_layouts_eleven_to_thirteen_are_the_full_page() {
    assert!(layout_size_text(FULL_PAGE_LAYOUT).ends_with("full page"));
    assert!(layout_size_text(PLAIN_LAYOUT).ends_with("plain window"));
}
