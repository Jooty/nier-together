use replicant_game::readiness::{
    JOINING_IGNORED_AFTER_MS, LocalGameReadiness, PartnerWatch, PlayChecks, PlayReads,
    ReadinessChangeLog, SILENT_PARTNER_FALLBACK_MS, classify, joining_label, waiting_on,
};
use replicant_proto::GameReadiness;

const ALL_READY: PlayChecks = PlayChecks {
    phase_known: true,
    player_actor_spawned: true,
    vitals_readable: true,
};

#[test]
fn a_game_with_everything_readable_is_in_play() {
    assert_eq!(classify(ALL_READY), GameReadiness::InPlay);
}

#[test]
fn a_missing_phase_actor_or_vitals_means_loading() {
    for checks in [
        PlayChecks {
            phase_known: false,
            ..ALL_READY
        },
        PlayChecks {
            player_actor_spawned: false,
            ..ALL_READY
        },
        PlayChecks {
            vitals_readable: false,
            ..ALL_READY
        },
        PlayChecks::default(),
    ] {
        assert_eq!(classify(checks), GameReadiness::Loading);
    }
}

#[test]
fn an_unlinked_game_is_not_linked_and_ignores_reports() {
    let mut local = LocalGameReadiness::new();
    assert_eq!(local.current(), GameReadiness::NotLinked);
    local.game_reported(GameReadiness::InPlay);
    assert_eq!(local.current(), GameReadiness::NotLinked);
}

#[test]
fn a_freshly_linked_game_is_loading_until_it_reports_in_play() {
    let mut local = LocalGameReadiness::new();
    local.game_linked();
    assert_eq!(local.current(), GameReadiness::Loading);
    local.game_reported(GameReadiness::InPlay);
    assert_eq!(local.current(), GameReadiness::InPlay);
    local.game_reported(GameReadiness::Loading);
    assert_eq!(local.current(), GameReadiness::Loading);
}

#[test]
fn a_game_cannot_report_itself_as_not_linked() {
    let mut local = LocalGameReadiness::new();
    local.game_linked();
    local.game_reported(GameReadiness::NotLinked);
    assert_eq!(local.current(), GameReadiness::Loading);
}

#[test]
fn a_game_that_closes_goes_back_to_not_linked_and_loading_on_return() {
    let mut local = LocalGameReadiness::new();
    local.game_linked();
    local.game_reported(GameReadiness::InPlay);
    local.game_unlinked();
    assert_eq!(local.current(), GameReadiness::NotLinked);
    local.game_linked();
    assert_eq!(local.current(), GameReadiness::Loading);
}

#[test]
fn a_partner_who_just_connected_is_joining_not_in_play() {
    let mut partner = PartnerWatch::new();
    assert!(!partner.is_joining());
    partner.partner_connected(1_000);
    assert!(partner.is_joining());
    assert!(!partner.is_in_play());
    partner.partner_reported(GameReadiness::Loading);
    assert!(partner.is_joining());
}

#[test]
fn a_partner_reporting_in_play_is_no_longer_joining() {
    let mut partner = PartnerWatch::new();
    partner.partner_connected(0);
    partner.partner_reported(GameReadiness::InPlay);
    assert!(partner.is_in_play());
    assert!(!partner.is_joining());
}

#[test]
fn a_partner_whose_game_closes_goes_back_to_joining() {
    let mut partner = PartnerWatch::new();
    partner.partner_connected(0);
    partner.partner_reported(GameReadiness::InPlay);
    partner.partner_reported(GameReadiness::NotLinked);
    assert!(!partner.is_in_play());
    assert!(partner.is_joining());
}

#[test]
fn a_report_without_a_connected_partner_is_ignored() {
    let mut partner = PartnerWatch::new();
    partner.partner_reported(GameReadiness::InPlay);
    assert!(!partner.is_in_play());
    partner.partner_connected(0);
    partner.partner_gone();
    assert_eq!(partner.readiness(), GameReadiness::NotLinked);
    assert!(!partner.is_joining());
}

#[test]
fn a_joining_partner_counts_for_the_zone_rule_until_the_timeout() {
    let mut partner = PartnerWatch::new();
    assert!(!partner.counts_for_zone_rule(0));
    partner.partner_connected(5_000);
    assert!(partner.counts_for_zone_rule(5_000 + JOINING_IGNORED_AFTER_MS - 1));
    assert!(!partner.counts_for_zone_rule(5_000 + JOINING_IGNORED_AFTER_MS));
}

#[test]
fn an_in_play_partner_always_counts_for_the_zone_rule() {
    let mut partner = PartnerWatch::new();
    partner.partner_connected(0);
    partner.partner_reported(GameReadiness::InPlay);
    assert!(partner.counts_for_zone_rule(JOINING_IGNORED_AFTER_MS * 10));
}

#[test]
fn the_joining_label_names_what_the_partner_is_doing() {
    assert!(joining_label(GameReadiness::NotLinked).contains("not open"));
    assert!(joining_label(GameReadiness::Loading).contains("loading"));
}

#[test]
fn a_partner_that_never_reports_is_assumed_in_play_after_the_fallback() {
    let mut partner = PartnerWatch::new();
    partner.partner_connected(1_000);
    assert!(!partner.assume_in_play_if_silent(1_000 + SILENT_PARTNER_FALLBACK_MS - 1));
    assert!(partner.assume_in_play_if_silent(1_000 + SILENT_PARTNER_FALLBACK_MS));
    assert!(partner.is_in_play());
}

#[test]
fn a_partner_that_reported_loading_is_never_assumed_in_play() {
    let mut partner = PartnerWatch::new();
    partner.partner_connected(0);
    partner.partner_reported(GameReadiness::Loading);
    assert!(!partner.assume_in_play_if_silent(SILENT_PARTNER_FALLBACK_MS * 10));
    assert!(partner.is_joining());
}

#[test]
fn a_state_from_the_partner_proves_it_is_in_play() {
    let mut partner = PartnerWatch::new();
    assert!(!partner.partner_sent_state());
    partner.partner_connected(0);
    partner.partner_reported(GameReadiness::Loading);
    assert!(partner.partner_sent_state());
    assert!(partner.is_in_play());
    assert!(!partner.partner_sent_state());
}

#[test]
fn a_partner_whose_game_closes_is_flagged_closed_once() {
    let mut partner = PartnerWatch::new();
    partner.partner_connected(0);
    assert!(!partner.partner_reported(GameReadiness::NotLinked));
    assert!(!partner.is_game_closed());
    partner.partner_reported(GameReadiness::InPlay);
    assert!(partner.partner_reported(GameReadiness::NotLinked));
    assert!(partner.is_game_closed());
    assert!(!partner.partner_reported(GameReadiness::NotLinked));
    assert!(partner.is_game_closed());
}

#[test]
fn a_closed_partner_does_not_hold_up_zone_changes_and_reopening_clears_it() {
    let mut partner = PartnerWatch::new();
    partner.partner_connected(0);
    partner.partner_reported(GameReadiness::InPlay);
    partner.partner_reported(GameReadiness::NotLinked);
    assert!(!partner.counts_for_zone_rule(1_000));
    partner.partner_reported(GameReadiness::Loading);
    assert!(!partner.is_game_closed());
    assert!(partner.counts_for_zone_rule(1_000));
}

#[test]
fn a_closed_partner_sending_its_position_again_is_no_longer_closed() {
    let mut partner = PartnerWatch::new();
    partner.partner_connected(0);
    partner.partner_reported(GameReadiness::InPlay);
    partner.partner_reported(GameReadiness::NotLinked);
    assert!(partner.partner_sent_state());
    assert!(!partner.is_game_closed());
}

#[test]
fn each_change_of_the_report_logs_once_with_its_age_and_reason() {
    let mut log = ReadinessChangeLog::default();
    let first = log
        .observe(
            1_000,
            GameReadiness::Loading,
            "the player actor",
            "r1",
            "sent",
        )
        .expect("the first report logs");
    assert!(first.contains("Loading"), "{first}");
    assert!(first.contains("first report"), "{first}");
    assert!(first.contains("waiting on the player actor"), "{first}");
    assert!(first.contains("reads: r1"), "{first}");
    assert!(first.contains("send: sent"), "{first}");
    assert_eq!(
        log.observe(1_500, GameReadiness::Loading, "x", "r", "sent"),
        None
    );
    assert_eq!(
        log.observe(2_900, GameReadiness::Loading, "x", "r", "sent"),
        None
    );
    let second = log
        .observe(4_500, GameReadiness::InPlay, "nothing", "r", "sent")
        .expect("a change logs");
    assert!(second.contains("InPlay"), "{second}");
    assert!(second.contains("was Loading for 3500 ms"), "{second}");
    assert!(second.contains("waiting on nothing"), "{second}");
    let third = log
        .observe(5_000, GameReadiness::Loading, "no report", "r", "refused")
        .expect("going back logs");
    assert!(third.contains("was InPlay for 500 ms"), "{third}");
    assert!(third.contains("send: refused"), "{third}");
}

#[test]
fn a_reset_log_logs_the_next_report_as_a_first() {
    let mut log = ReadinessChangeLog::default();
    log.observe(0, GameReadiness::Loading, "x", "r", "s");
    log.reset();
    let line = log
        .observe(10, GameReadiness::Loading, "x", "r", "s")
        .expect("logs");
    assert!(line.contains("first report"), "{line}");
}

#[test]
fn a_copy_that_stays_loading_logs_every_five_seconds_after_ten() {
    let mut log = ReadinessChangeLog::default();
    log.observe(0, GameReadiness::Loading, "x", "r", "s");
    let mut lines = Vec::new();
    for now_ms in (250..=26_000).step_by(250) {
        lines.extend(log.observe(now_ms, GameReadiness::Loading, "x", "r", "s"));
    }
    assert_eq!(lines.len(), 4, "{lines:?}");
    assert!(
        lines[0].contains("still Loading after 10250 ms"),
        "{lines:?}"
    );
    assert!(lines[1].contains("after 15250 ms"), "{lines:?}");
}

#[test]
fn a_failing_check_is_named_while_loading_and_in_play_logs_once() {
    let ready = PlayReads {
        zone: Some("Play_Env_06".to_owned()),
        phase_known: true,
        player_actor_spawned: true,
        position_readable: true,
        max_health: Some(100),
    };
    let zero_health = PlayReads {
        max_health: Some(0),
        ..ready.clone()
    };
    let no_actor = PlayReads {
        player_actor_spawned: false,
        position_readable: false,
        ..ready.clone()
    };
    let mut log = ReadinessChangeLog::default();
    let mut lines = Vec::new();
    for (now_ms, reads) in [
        (0, &no_actor),
        (250, &no_actor),
        (500, &zero_health),
        (750, &zero_health),
        (1_000, &ready),
        (1_250, &ready),
    ] {
        let checks = reads.checks();
        lines.extend(log.observe(
            now_ms,
            classify(checks),
            &waiting_on(checks, Some(100)),
            &reads.describe(),
            "sent",
        ));
    }
    assert_eq!(lines.len(), 2, "{lines:?}");
    assert!(lines[0].contains("Loading"), "{lines:?}");
    assert!(
        lines[0].contains("waiting on the player actor, the vitals"),
        "{lines:?}"
    );
    assert!(lines[0].contains("actor=false"), "{lines:?}");
    assert!(lines[1].contains("InPlay"), "{lines:?}");
    assert!(lines[1].contains("was Loading for 1000 ms"), "{lines:?}");
    assert!(lines[1].contains("max_health=100"), "{lines:?}");
    assert!(zero_health.describe().contains("max_health=0"));
    assert_eq!(waiting_on(zero_health.checks(), Some(0)), "the vitals");
}

#[test]
fn the_wait_names_each_missing_check_or_a_stale_report() {
    assert_eq!(waiting_on(ALL_READY, Some(100)), "nothing");
    let none = PlayChecks::default();
    assert_eq!(
        waiting_on(none, Some(100)),
        "the phase, the player actor, the vitals"
    );
    let only_vitals = PlayChecks {
        vitals_readable: false,
        ..ALL_READY
    };
    assert_eq!(waiting_on(only_vitals, Some(0)), "the vitals");
    assert_eq!(
        waiting_on(ALL_READY, Some(2_500)),
        "no report from the game for 2500 ms"
    );
    assert_eq!(waiting_on(ALL_READY, None), "the game's first report");
}
