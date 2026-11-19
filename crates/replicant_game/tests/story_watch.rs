use replicant_game::event_log::EventLog;
use replicant_game::story_watch::{
    GUEST_WORD_STALE_AFTER_MS, GuestWatch, GuestWord, PlaceStep, RemovalChange,
    STORY_REMOVAL_SETTLES_MS, StoryRemoval, WATCH_BODY_LEFT_ALONE_BELOW,
    WATCH_BODY_TELEPORT_EVERY_MS, WATCH_BODY_TELEPORTED_ABOVE, WatchEnd, WatchFacts, WatchStep,
    ended_line, guest_is_watching_text, place_step, started_line, watching_host_text,
};

fn facts(story_has_taken_her: bool) -> WatchFacts<'static> {
    WatchFacts {
        setting_on: true,
        guest_in_session: true,
        story_has_taken_her,
        host_leads_a_stretch: false,
        pack: "A_DESERT_TEMPLE_01",
    }
}

#[test]
fn a_removal_counts_once_it_has_stood_for_the_settle_time() {
    let mut removal = StoryRemoval::new();
    removal.story_called(true, 1_000);
    assert_eq!(removal.step(1_000 + STORY_REMOVAL_SETTLES_MS - 1), None);
    assert!(!removal.settled_out());
    assert_eq!(
        removal.step(1_000 + STORY_REMOVAL_SETTLES_MS),
        Some(RemovalChange::TakenOut)
    );
    assert!(removal.settled_out());
    assert_eq!(removal.step(10_000), None);
}

#[test]
fn a_give_back_followed_at_once_by_the_removal_never_shows() {
    let mut removal = StoryRemoval::new();
    removal.story_called(true, 0);
    assert_eq!(
        removal.step(STORY_REMOVAL_SETTLES_MS),
        Some(RemovalChange::TakenOut)
    );
    removal.story_called(false, 5_000);
    assert_eq!(removal.step(5_100), None);
    removal.story_called(true, 5_200);
    assert_eq!(removal.step(5_200 + STORY_REMOVAL_SETTLES_MS), None);
    assert!(removal.settled_out());
}

#[test]
fn the_story_bringing_her_back_ends_the_removal() {
    let mut removal = StoryRemoval::new();
    removal.story_called(true, 0);
    removal.step(STORY_REMOVAL_SETTLES_MS);
    removal.story_called(false, 10_000);
    assert_eq!(
        removal.step(10_000 + STORY_REMOVAL_SETTLES_MS),
        Some(RemovalChange::BroughtBack)
    );
    assert!(!removal.settled_out());
}

#[test]
fn forgetting_the_removal_puts_her_back() {
    let mut removal = StoryRemoval::new();
    removal.story_called(true, 0);
    removal.step(STORY_REMOVAL_SETTLES_MS);
    removal.forget();
    assert!(!removal.settled_out());
    assert_eq!(removal.step(100_000), None);
}

#[test]
fn the_guest_starts_and_ends_watching_with_the_story() {
    let mut watch = GuestWatch::new();
    assert_eq!(watch.step(&facts(false)), WatchStep::Idle);
    assert_eq!(watch.step(&facts(true)), WatchStep::Started);
    assert!(watch.watching());
    assert_eq!(watch.step(&facts(true)), WatchStep::Continues);
    assert_eq!(
        watch.step(&facts(false)),
        WatchStep::Ended(WatchEnd::StoryBroughtHerBack)
    );
    assert!(!watch.watching());
}

#[test]
fn the_setting_off_never_watches_and_ends_a_watch() {
    let mut watch = GuestWatch::new();
    let off = WatchFacts {
        setting_on: false,
        ..facts(true)
    };
    assert_eq!(watch.step(&off), WatchStep::Idle);
    watch.step(&facts(true));
    assert_eq!(watch.step(&off), WatchStep::Ended(WatchEnd::SettingOff));
}

#[test]
fn leaving_the_session_ends_a_watch() {
    let mut watch = GuestWatch::new();
    watch.step(&facts(true));
    let alone = WatchFacts {
        guest_in_session: false,
        ..facts(true)
    };
    assert_eq!(
        watch.step(&alone),
        WatchStep::Ended(WatchEnd::LeftTheSession)
    );
}

#[test]
fn a_pack_where_the_guest_plays_is_never_watched() {
    let mut watch = GuestWatch::new();
    let duel = WatchFacts {
        pack: "B_STONE_FIELD_02",
        ..facts(true)
    };
    assert_eq!(watch.step(&duel), WatchStep::Idle);
    watch.step(&facts(true));
    assert_eq!(
        watch.step(&duel),
        WatchStep::Ended(WatchEnd::PackIsOneWhereTheGuestPlays)
    );
}

#[test]
fn a_small_error_is_left_a_middling_one_written_a_large_one_teleported() {
    assert_eq!(
        place_step(WATCH_BODY_LEFT_ALONE_BELOW - 0.01, 0),
        PlaceStep::Leave
    );
    assert_eq!(place_step(0.5, 0), PlaceStep::Write);
    assert_eq!(
        place_step(
            WATCH_BODY_TELEPORTED_ABOVE + 1.0,
            WATCH_BODY_TELEPORT_EVERY_MS
        ),
        PlaceStep::Teleport
    );
}

#[test]
fn the_teleport_waits_for_its_interval() {
    assert_eq!(
        place_step(
            WATCH_BODY_TELEPORTED_ABOVE + 1.0,
            WATCH_BODY_TELEPORT_EVERY_MS - 1
        ),
        PlaceStep::Leave
    );
}

#[test]
fn the_host_trusts_the_guests_word_until_it_goes_stale() {
    let mut word = GuestWord::new();
    assert!(!word.guest_watches(0));
    word.heard(true, 1_000);
    assert!(word.guest_watches(1_000 + GUEST_WORD_STALE_AFTER_MS - 1));
    assert!(!word.guest_watches(1_000 + GUEST_WORD_STALE_AFTER_MS));
    word.heard(true, 9_000);
    word.heard(false, 9_100);
    assert!(!word.guest_watches(9_200));
    word.heard(true, 9_300);
    word.forget();
    assert!(!word.guest_watches(9_400));
}

#[test]
fn the_lines_name_the_ids_and_the_persistent_texts_are_short() {
    assert!(started_line("Jay", "A_DESERT_TEMPLE_01").contains("-2"));
    assert!(ended_line(WatchEnd::StoryBroughtHerBack).contains("-2"));
    assert_eq!(watching_host_text("Jay"), "Watching Jay");
    assert_eq!(guest_is_watching_text("Sam"), "Sam is watching");
}

#[test]
fn the_watch_texts_fit_the_persistent_line() {
    let mut log = EventLog::new();
    assert!(log.set_persistent(&watching_host_text("Jay")));
    assert!(log.set_persistent(&guest_is_watching_text("Sam")));
    assert_eq!(log.persistent(), Some("Sam is watching"));
}
