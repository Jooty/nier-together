use replicant_game::dialogue::{
    STORY_TALK_MENU_CLOSE_MOST_ASKS_PER_TALK, STORY_TALK_MENU_CLOSE_RETRY_AFTER_MS,
    StoryTalkMenuClose,
};

#[test]
fn run_1114s_the_guest_pause_menu_is_closed_when_the_hosts_story_talk_starts() {
    let mut rule = StoryTalkMenuClose::new();
    assert!(rule.should_close_the_pause_menu(Some(1323), true, false, 1_000));
}

#[test]
fn nothing_is_closed_without_a_story_talk_or_a_menu_or_with_an_own_window_open() {
    let mut rule = StoryTalkMenuClose::new();
    assert!(!rule.should_close_the_pause_menu(None, true, false, 1_000));
    assert!(!rule.should_close_the_pause_menu(Some(1323), false, false, 1_000));
    assert!(!rule.should_close_the_pause_menu(Some(1323), true, true, 1_000));
}

#[test]
fn a_menu_that_stays_open_is_asked_again_a_few_times_then_left() {
    let mut rule = StoryTalkMenuClose::new();
    let mut now = 1_000;
    let mut asks = 0;
    for _ in 0..10 {
        if rule.should_close_the_pause_menu(Some(1323), true, false, now) {
            asks += 1;
        }
        assert!(!rule.should_close_the_pause_menu(Some(1323), true, false, now + 10));
        now += STORY_TALK_MENU_CLOSE_RETRY_AFTER_MS;
    }
    assert_eq!(asks, STORY_TALK_MENU_CLOSE_MOST_ASKS_PER_TALK);
}

#[test]
fn the_next_story_talk_gets_its_own_asks() {
    let mut rule = StoryTalkMenuClose::new();
    assert!(rule.should_close_the_pause_menu(Some(1323), true, false, 1_000));
    assert!(rule.should_close_the_pause_menu(Some(1324), true, false, 1_100));
}
