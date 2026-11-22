use replicant_game::event_log::EventLog;
use replicant_game::story_watch::{
    AERIE_PACK_PREFIX, GUEST_WORD_STALE_AFTER_MS, GuestWatch, GuestWord, PlaceStep, RemovalChange,
    STORY_REMOVAL_SETTLES_MS, StoryRemoval, WATCH_BODY_LEFT_ALONE_BELOW,
    WATCH_BODY_TELEPORT_EVERY_MS, WATCH_BODY_TELEPORTED_ABOVE, WatchEnd, WatchFacts, WatchStep,
    ended_line, guest_is_watching_text, place_step, started_line, watching_host_text,
};

fn facts(story_has_taken_her: bool) -> WatchFacts<'static> {
    WatchFacts {
        setting_on: true,
        guest_in_session: true,
        story_has_taken_her,
        taken_at_the_aerie_exit: false,
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

#[test]
fn after_the_aerie_exit_removal_the_guest_watches_only_inside_the_aerie() {
    let mut watch = GuestWatch::new();
    let in_the_aerie = WatchFacts {
        taken_at_the_aerie_exit: true,
        pack: "A_CLIFF_VILLAGE_01",
        ..facts(true)
    };
    let outside = WatchFacts {
        pack: "A_DESERT_TEMPLE_01",
        ..in_the_aerie
    };
    assert!(in_the_aerie.pack.starts_with(AERIE_PACK_PREFIX));
    assert_eq!(watch.step(&outside), WatchStep::Idle);
    assert_eq!(watch.step(&in_the_aerie), WatchStep::Started);
    assert_eq!(
        watch.step(&outside),
        WatchStep::Ended(WatchEnd::LeftTheAerieAfterTheExitRemoval)
    );
    assert_eq!(watch.step(&in_the_aerie), WatchStep::Started);
}

#[test]
fn another_removal_keeps_the_guest_watching_outside_the_aerie() {
    let mut watch = GuestWatch::new();
    assert_eq!(watch.step(&facts(true)), WatchStep::Started);
    assert_eq!(watch.step(&facts(true)), WatchStep::Continues);
}

#[test]
fn the_aerie_exit_flag_lasts_until_the_story_calls_again() {
    let mut removal = StoryRemoval::new();
    removal.aerie_exit_removed_her(0);
    assert!(!removal.taken_at_the_aerie_exit());
    removal.step(STORY_REMOVAL_SETTLES_MS);
    assert!(removal.taken_at_the_aerie_exit());
    removal.story_called(false, 10_000);
    removal.step(10_000 + STORY_REMOVAL_SETTLES_MS);
    assert!(!removal.taken_at_the_aerie_exit());
    removal.story_called(true, 20_000);
    removal.step(20_000 + STORY_REMOVAL_SETTLES_MS);
    assert!(removal.settled_out());
    assert!(!removal.taken_at_the_aerie_exit());
}

#[test]
fn a_later_removal_keeps_the_aerie_exit_mark_and_only_an_activate_clears_it() {
    let mut removal = StoryRemoval::new();
    removal.aerie_exit_removed_her(0);
    removal.step(STORY_REMOVAL_SETTLES_MS);
    removal.story_called(true, 10_000);
    removal.step(10_000 + STORY_REMOVAL_SETTLES_MS);
    assert!(removal.taken_at_the_aerie_exit());
    removal.story_called(false, 20_000);
    removal.step(20_000 + STORY_REMOVAL_SETTLES_MS);
    assert!(!removal.taken_at_the_aerie_exit());
}

#[test]
fn after_a_load_the_flag_and_an_inactive_companion_bring_the_aerie_exit_mark_back() {
    let mut removal = StoryRemoval::new();
    assert!(!removal.restore_aerie_exit_from_game_state(false, true));
    assert!(!removal.restore_aerie_exit_from_game_state(true, false));
    assert!(!removal.taken_at_the_aerie_exit());
    assert!(removal.restore_aerie_exit_from_game_state(true, true));
    assert!(removal.settled_out());
    assert!(removal.taken_at_the_aerie_exit());
    assert!(!removal.restore_aerie_exit_from_game_state(true, true));
}

#[test]
fn a_story_call_seen_before_the_load_check_wins_over_the_game_state() {
    let mut removal = StoryRemoval::new();
    removal.story_called(false, 0);
    assert!(!removal.restore_aerie_exit_from_game_state(true, true));
    let mut removal = StoryRemoval::new();
    removal.restore_aerie_exit_from_game_state(true, true);
    removal.story_called(false, 5_000);
    removal.step(5_000 + STORY_REMOVAL_SETTLES_MS);
    assert!(!removal.settled_out());
    assert!(!removal.taken_at_the_aerie_exit());
}
