use automata_game::address::{ENEMY_STATE_START_FUNCTION, ENEMY_TARGET_SETTER_FUNCTION};
use automata_game::enemies::EnemyId;
use automata_game::enemy_lead::{
    BOSS_CLASS_HEALTH_FROM, BossStartLog, ENEMY_IS_HELD_ONCE_IT_HAS_LIVED_HERE_MS,
    ENEMY_PLACE_IS_LOGGED_EVERY_MS, ENEMY_POSES_IN_ONE_LEAD_MESSAGE, EnemyLead, EnemyPose,
    HOLD_LIFTS_AFTER_THE_HOSTS_SILENCE_MS, HookedStarts, LeadFollower, LeadTarget,
    PLACE_ERROR_SNAPPED, PlaceLog, PlaceStep, PlayerPlaces, QueuedStart, StartTally,
    facing_to_write, guest_target_handle, lead_parts, lead_target_of, place_line, place_step,
    played_start_line, sent_start_line, target_wanted_on_the_host,
};
use automata_game::exe::ExeVariant;

const HOST_HERO: u32 = 0x0001_0203;
const PARTNER_BODY: u32 = 0x0004_0506;
const GUEST_HERO: u32 = 0x0007_0809;

fn id(serial: u32) -> EnemyId {
    EnemyId::Generated {
        generator: 0x00AB_CDEF,
        serial,
    }
}

fn pose(serial: u32, starts: u32) -> EnemyPose {
    EnemyPose {
        id: id(serial),
        position: [10.0, 0.5, -4.0],
        facing: 1.0,
        state: [0x12, 3, 0, 0],
        starts,
        target: LeadTarget::HostPlayer,
    }
}

fn lead(poses: &[EnemyPose]) -> EnemyLead {
    EnemyLead {
        poses: poses.to_vec(),
        starts: Vec::new(),
    }
}

fn played_state(follower: &mut LeadFollower, id: EnemyId, now_ms: u64) -> Option<[i32; 4]> {
    follower.start_to_play(id, now_ms).map(|start| start.state)
}

fn queued(serial: u32, number: u32, state_id: i32) -> QueuedStart {
    QueuedStart {
        id: id(serial),
        number,
        state: [state_id, 0, 0, 0],
        ago_ms: 40,
    }
}

fn follower_with_a_settled_body(now_ms: u64) -> LeadFollower {
    let mut follower = LeadFollower::default();
    follower.note_body(id(1), 0x77, now_ms);
    follower
}

#[test]
fn the_state_start_and_target_setter_are_the_ones_read_from_both_exes() {
    assert_eq!(ENEMY_STATE_START_FUNCTION.rva(ExeVariant::Main), 0x63_7970);
    assert_eq!(
        ENEMY_STATE_START_FUNCTION.rva(ExeVariant::Compat),
        0x63_0820
    );
    assert_eq!(
        ENEMY_TARGET_SETTER_FUNCTION.rva(ExeVariant::Main),
        0x63_ED20
    );
    assert_eq!(
        ENEMY_TARGET_SETTER_FUNCTION.rva(ExeVariant::Compat),
        0x63_7BA0
    );
}

#[test]
fn a_round_of_poses_is_cut_into_messages_that_each_pass_the_check() {
    let poses: Vec<EnemyPose> = (0..20).map(|serial| pose(serial, 0)).collect();
    let parts = lead_parts(&poses, &[]);
    assert_eq!(parts.len(), 3);
    assert!(parts.iter().all(EnemyLead::is_valid));
    assert_eq!(parts[0].poses.len(), ENEMY_POSES_IN_ONE_LEAD_MESSAGE);
    assert_eq!(parts.iter().map(|part| part.poses.len()).sum::<usize>(), 20);
}

#[test]
fn a_pose_with_a_place_off_the_scale_is_not_sent() {
    let mut bad = pose(1, 0);
    bad.position[1] = f32::NAN;
    assert!(!bad.is_valid());
    assert!(lead_parts(&[bad], &[]).is_empty());
    let mut far = pose(2, 0);
    far.facing = 100.0;
    assert!(!lead(&[far]).is_valid());
}

#[test]
fn a_message_with_too_many_poses_fails_the_check() {
    let poses: Vec<EnemyPose> = (0..=ENEMY_POSES_IN_ONE_LEAD_MESSAGE as u32)
        .map(|serial| pose(serial, 0))
        .collect();
    assert!(!lead(&poses).is_valid());
}

#[test]
fn the_hold_lifts_one_second_after_the_hosts_stream_for_an_enemy_stops() {
    let mut follower = follower_with_a_settled_body(0);
    let heard_at = ENEMY_IS_HELD_ONCE_IT_HAS_LIVED_HERE_MS;
    follower.hear(heard_at, &lead(&[pose(1, 0)]));
    assert!(follower.holds(id(1), heard_at));
    assert!(follower.holds(id(1), heard_at + HOLD_LIFTS_AFTER_THE_HOSTS_SILENCE_MS - 1));
    assert!(!follower.holds(id(1), heard_at + HOLD_LIFTS_AFTER_THE_HOSTS_SILENCE_MS));
    assert_eq!(
        follower.fresh_pose(id(1), heard_at + HOLD_LIFTS_AFTER_THE_HOSTS_SILENCE_MS),
        None
    );
}

#[test]
fn a_body_new_on_this_copy_is_not_held_until_it_has_lived_here_a_while() {
    let mut follower = LeadFollower::default();
    follower.note_body(id(1), 0x77, 1_000);
    follower.hear(1_000, &lead(&[pose(1, 0)]));
    assert!(!follower.holds(id(1), 1_000));
    follower.hear(2_000, &lead(&[pose(1, 0)]));
    assert!(follower.holds(id(1), 1_000 + ENEMY_IS_HELD_ONCE_IT_HAS_LIVED_HERE_MS));
}

#[test]
fn a_new_handle_for_the_same_enemy_starts_its_time_here_again() {
    let mut follower = follower_with_a_settled_body(0);
    follower.hear(5_000, &lead(&[pose(1, 0)]));
    assert!(follower.holds(id(1), 5_000));
    follower.note_body(id(1), 0x78, 5_000);
    assert!(!follower.holds(id(1), 5_000));
}

#[test]
fn an_enemy_the_host_never_told_is_never_held() {
    let follower = follower_with_a_settled_body(0);
    assert!(!follower.holds(id(1), 10_000));
}

#[test]
fn each_host_start_is_played_once_and_the_first_pose_plays_the_hosts_state() {
    let mut follower = follower_with_a_settled_body(0);
    let now = ENEMY_IS_HELD_ONCE_IT_HAS_LIVED_HERE_MS;
    follower.hear(now, &lead(&[pose(1, 4)]));
    assert_eq!(
        played_state(&mut follower, id(1), now),
        Some([0x12, 3, 0, 0])
    );
    assert_eq!(follower.start_to_play(id(1), now), None);
    follower.hear(now + 100, &lead(&[pose(1, 4)]));
    assert_eq!(follower.start_to_play(id(1), now + 100), None);
    follower.hear(now + 200, &lead(&[pose(1, 5)]));
    assert_eq!(
        played_state(&mut follower, id(1), now + 200),
        Some([0x12, 3, 0, 0])
    );
}

#[test]
fn no_start_is_played_once_the_hold_has_lifted() {
    let mut follower = follower_with_a_settled_body(0);
    let now = ENEMY_IS_HELD_ONCE_IT_HAS_LIVED_HERE_MS;
    follower.hear(now, &lead(&[pose(1, 4)]));
    assert_eq!(
        follower.start_to_play(id(1), now + HOLD_LIFTS_AFTER_THE_HOSTS_SILENCE_MS),
        None
    );
}

#[test]
fn bodies_this_copy_no_longer_holds_are_forgotten_but_fresh_poses_kept() {
    let mut follower = follower_with_a_settled_body(0);
    follower.hear(5_000, &lead(&[pose(1, 0), pose(2, 0)]));
    follower.keep_only(&[], 5_100);
    assert!(!follower.holds(id(1), 5_100));
    assert!(follower.fresh_pose(id(2), 5_100).is_some());
    follower.keep_only(&[], 5_000 + HOLD_LIFTS_AFTER_THE_HOSTS_SILENCE_MS);
    assert_eq!(follower.heard_at_ms(id(2)), None);
}

#[test]
fn a_near_enemy_is_left_alone_a_far_one_is_put_on_the_hosts_place() {
    assert_eq!(
        place_step([1.0, 0.0, 1.0], [1.01, 0.0, 1.0]),
        PlaceStep::Leave
    );
    let host = [PLACE_ERROR_SNAPPED + 1.0, 0.0, 0.0];
    assert_eq!(place_step([0.0, 0.0, 0.0], host), PlaceStep::MoveTo(host));
}

#[test]
fn an_enemy_between_is_eased_toward_the_hosts_place() {
    let PlaceStep::MoveTo(moved) = place_step([0.0, 0.0, 0.0], [2.0, 0.0, 0.0]) else {
        panic!("an error of two units is eased");
    };
    assert!(moved[0] > 0.0 && moved[0] < 2.0);
    assert!(moved[1].abs() < f32::EPSILON && moved[2].abs() < f32::EPSILON);
}

#[test]
fn a_place_that_is_not_a_number_is_left_alone() {
    assert_eq!(
        place_step([0.0, 0.0, 0.0], [f32::NAN, 0.0, 0.0]),
        PlaceStep::Leave
    );
}

#[test]
fn facing_eases_along_the_shorter_turn_and_stays_in_one_turn() {
    assert_eq!(facing_to_write(1.0, 1.01), None);
    let eased = facing_to_write(3.0, -3.0).expect("a turn of about 0.28 radians is eased");
    assert!(
        eased > 3.0 || eased < -3.0,
        "eased the short way across pi: {eased}"
    );
    assert!((-core::f32::consts::PI..core::f32::consts::PI).contains(&eased));
}

#[test]
fn the_first_tally_counts_nothing_and_sends_what_was_read() {
    let mut tally = StartTally::default();
    assert_eq!(tally.tally(id(1), [5, 1, 0, 0], None), (0, [5, 1, 0, 0]));
}

#[test]
fn a_hooked_start_counts_with_its_own_arguments() {
    let mut tally = StartTally::default();
    let first = HookedStarts {
        count: 7,
        last: [5, 0, 0, 0],
        recent: Default::default(),
    };
    tally.tally(id(1), [5, 2, 0, 0], Some(first));
    let second = HookedStarts {
        count: 9,
        last: [0x20, 1, 2, 3],
        recent: Default::default(),
    };
    assert_eq!(
        tally.tally(id(1), [0x20, 4, 0, 0], Some(second)),
        (2, [0x20, 1, 2, 3])
    );
    assert_eq!(
        tally.tally(id(1), [0x20, 5, 0, 0], Some(second)),
        (2, [0x20, 1, 2, 3])
    );
}

#[test]
fn a_changed_state_id_without_a_hooked_start_counts_once() {
    let mut tally = StartTally::default();
    tally.tally(id(1), [5, 0, 0, 0], None);
    assert_eq!(tally.tally(id(1), [5, 9, 0, 0], None), (0, [5, 0, 0, 0]));
    assert_eq!(tally.tally(id(1), [6, 1, 0, 0], None), (1, [6, 1, 0, 0]));
    assert_eq!(tally.tally(id(1), [6, 2, 0, 0], None), (1, [6, 1, 0, 0]));
}

#[test]
fn a_tally_kept_for_an_enemy_that_left_starts_again_from_nothing() {
    let mut tally = StartTally::default();
    tally.tally(id(1), [5, 0, 0, 0], None);
    tally.tally(id(1), [6, 0, 0, 0], None);
    tally.keep_only(&[]);
    assert_eq!(tally.tally(id(1), [7, 0, 0, 0], None), (0, [7, 0, 0, 0]));
}

#[test]
fn place_lines_come_once_per_two_second_step_of_the_wall_clock() {
    let mut log = PlaceLog::default();
    let step = ENEMY_PLACE_IS_LOGGED_EVERY_MS;
    assert!(log.round_is_due(10 * step + 1_900));
    assert!(!log.round_is_due(10 * step + 1_999));
    assert!(log.round_is_due(11 * step));
    assert!(!log.round_is_due(11 * step + 50));
    assert!(log.take_line(0).allowed);
}

#[test]
fn a_place_line_names_the_enemy_by_the_id_both_copies_share() {
    let line = place_line(
        id(3),
        [1.0, -2.5, 3.25],
        [0x12, 1, 0, 0],
        "following the host",
    );
    assert_eq!(
        line,
        "enemies: place of generator 0xABCDEF spawn 3 at 1.00 -2.50 3.25, state 0x12 1 0 0, following the host"
    );
}

#[test]
fn the_host_keeps_the_target_until_the_other_player_is_clearly_nearer() {
    let players = PlayerPlaces {
        host_player: Some([10.0, 0.0, 0.0]),
        guest_player: Some([-8.0, 0.0, 0.0]),
    };
    assert_eq!(
        target_wanted_on_the_host([0.0, 0.0, 0.0], players, LeadTarget::HostPlayer),
        Some(LeadTarget::HostPlayer)
    );
    let nearer = PlayerPlaces {
        guest_player: Some([-5.0, 0.0, 0.0]),
        ..players
    };
    assert_eq!(
        target_wanted_on_the_host([0.0, 0.0, 0.0], nearer, LeadTarget::HostPlayer),
        Some(LeadTarget::GuestPlayer)
    );
    assert_eq!(
        target_wanted_on_the_host([0.0, 0.0, 0.0], nearer, LeadTarget::GuestPlayer),
        Some(LeadTarget::GuestPlayer)
    );
}

#[test]
fn a_target_that_is_neither_player_is_left_to_the_game() {
    let players = PlayerPlaces {
        host_player: Some([1.0, 0.0, 0.0]),
        guest_player: Some([2.0, 0.0, 0.0]),
    };
    for current in [LeadTarget::Nobody, LeadTarget::SomethingElse] {
        assert_eq!(
            target_wanted_on_the_host([0.0, 0.0, 0.0], players, current),
            None
        );
    }
}

#[test]
fn without_the_guests_body_the_host_keeps_its_own_hero_as_the_target() {
    let players = PlayerPlaces {
        host_player: Some([100.0, 0.0, 0.0]),
        guest_player: None,
    };
    assert_eq!(
        target_wanted_on_the_host([0.0, 0.0, 0.0], players, LeadTarget::GuestPlayer),
        Some(LeadTarget::HostPlayer)
    );
}

#[test]
fn a_handle_is_named_by_the_player_it_belongs_to() {
    let named = |handle| lead_target_of(handle, Some(HOST_HERO), Some(PARTNER_BODY));
    assert_eq!(named(0), LeadTarget::Nobody);
    assert_eq!(named(u32::MAX), LeadTarget::Nobody);
    assert_eq!(named(HOST_HERO), LeadTarget::HostPlayer);
    assert_eq!(named(PARTNER_BODY), LeadTarget::GuestPlayer);
    assert_eq!(named(0x0099_0000), LeadTarget::SomethingElse);
}

#[test]
fn on_the_guest_the_hosts_player_is_the_partner_body_and_the_guests_is_its_own_hero() {
    let mapped = |target| guest_target_handle(target, Some(GUEST_HERO), Some(PARTNER_BODY));
    assert_eq!(mapped(LeadTarget::HostPlayer), Some(PARTNER_BODY));
    assert_eq!(mapped(LeadTarget::GuestPlayer), Some(GUEST_HERO));
    assert_eq!(mapped(LeadTarget::Nobody), None);
    assert_eq!(mapped(LeadTarget::SomethingElse), None);
}

#[test]
fn a_defeat_state_of_the_host_is_left_to_the_defeat_it_tells() {
    let mut follower = follower_with_a_settled_body(0);
    let now = ENEMY_IS_HELD_ONCE_IT_HAS_LIVED_HERE_MS;
    let mut defeat = pose(1, 9);
    defeat.state = [0x8E, 0, 0, 0];
    follower.hear(now, &lead(&[defeat]));
    assert_eq!(follower.start_to_play(id(1), now), None);
}

#[test]
fn every_start_inside_one_send_is_queued_in_order_and_defeat_states_are_left_out_on_the_guest() {
    let mut tally = StartTally::default();
    let before = HookedStarts::first([0x10, 0, 0, 0], 1_000);
    tally.tally(id(1), [0x10, 0, 0, 0], Some(before));
    let after = before
        .with_start([0x8E, 1, 0, 0], 1_020)
        .with_start([0xAC, 2, 0, 0], 1_050)
        .with_start([0x30, 3, 0, 0], 1_080);
    let (starts, sent) = tally.tally(id(1), [0x30, 3, 0, 0], Some(after));
    assert_eq!((starts, sent), (3, [0x30, 3, 0, 0]));
    let queued = tally.take_queued(id(1), 1_100);
    assert_eq!(
        queued.iter().map(|start| start.number).collect::<Vec<_>>(),
        [1, 2, 3]
    );
    assert_eq!(
        queued
            .iter()
            .map(|start| start.state[0])
            .collect::<Vec<_>>(),
        [0x8E, 0xAC, 0x30]
    );
    assert_eq!(
        queued.iter().map(|start| start.ago_ms).collect::<Vec<_>>(),
        [80, 50, 20]
    );
    assert!(tally.take_queued(id(1), 1_200).is_empty());

    let mut follower = follower_with_a_settled_body(0);
    let now = ENEMY_IS_HELD_ONCE_IT_HAS_LIVED_HERE_MS;
    let mut message = lead(&[pose(1, 3)]);
    message.starts = queued;
    follower.hear(now, &message);
    let mut played = Vec::new();
    while let Some(start) = follower.start_to_play(id(1), now) {
        played.push((start.number, start.state[0]));
    }
    assert_eq!(played, [(Some(3), 0x30)]);
    assert_eq!(follower.start_to_play(id(1), now), None);
}

#[test]
fn queued_starts_that_are_not_defeat_states_are_all_played() {
    let mut follower = follower_with_a_settled_body(0);
    let now = ENEMY_IS_HELD_ONCE_IT_HAS_LIVED_HERE_MS;
    let mut message = lead(&[pose(1, 3)]);
    message.starts = vec![queued(1, 1, 0x20), queued(1, 2, 0x21), queued(1, 3, 0x22)];
    follower.hear(now, &message);
    let mut played = Vec::new();
    while let Some(start) = follower.start_to_play(id(1), now) {
        played.push(start.state[0]);
    }
    assert_eq!(played, [0x20, 0x21, 0x22]);
    assert_eq!(follower.start_to_play(id(1), now), None);
}

#[test]
fn a_queued_start_heard_twice_or_out_of_date_is_played_once() {
    let mut follower = follower_with_a_settled_body(0);
    let now = ENEMY_IS_HELD_ONCE_IT_HAS_LIVED_HERE_MS;
    let mut first = lead(&[pose(1, 2)]);
    first.starts = vec![queued(1, 1, 0x20), queued(1, 2, 0x21)];
    follower.hear(now, &first);
    let mut overlap = lead(&[pose(1, 3)]);
    overlap.starts = vec![queued(1, 2, 0x21), queued(1, 3, 0x22)];
    follower.hear(now + 100, &overlap);
    let mut played = Vec::new();
    while let Some(start) = follower.start_to_play(id(1), now + 100) {
        played.push(start.state[0]);
    }
    assert_eq!(played, [0x20, 0x21, 0x22]);
}

#[test]
fn starts_heard_before_the_hold_begins_are_dropped_and_the_latest_pose_plays_instead() {
    let mut follower = LeadFollower::default();
    follower.note_body(id(1), 0x77, 1_000);
    let mut message = lead(&[pose(1, 3)]);
    message.starts = vec![queued(1, 2, 0x21), queued(1, 3, 0x22)];
    follower.hear(1_000, &message);
    assert!(follower.start_to_play(id(1), 1_000).is_none());
    follower.hear(2_600, &lead(&[pose(1, 3)]));
    let start = follower
        .start_to_play(id(1), 2_600)
        .expect("the latest pose plays");
    assert_eq!((start.number, start.state), (None, [0x12, 3, 0, 0]));
}

#[test]
fn the_host_remembers_at_most_the_most_recent_starts_of_an_enemy() {
    let mut tally = StartTally::default();
    let mut hooked = HookedStarts::first([1, 0, 0, 0], 10);
    tally.tally(id(1), [1, 0, 0, 0], Some(hooked));
    for state_id in 2..=10 {
        hooked = hooked.with_start([state_id, 0, 0, 0], 10 * i64::from(state_id) as u64);
    }
    tally.tally(id(1), [10, 0, 0, 0], Some(hooked));
    let numbers: Vec<u32> = tally
        .take_queued(id(1), 200)
        .iter()
        .map(|start| start.number)
        .collect();
    assert_eq!(numbers, [6, 7, 8, 9]);
}

#[test]
fn starts_wait_for_a_paused_host_and_a_lead_cuts_them_by_enemy() {
    let mut tally = StartTally::default();
    let before = HookedStarts::first([1, 0, 0, 0], 10);
    tally.tally(id(1), [1, 0, 0, 0], Some(before));
    tally.tally(
        id(1),
        [2, 0, 0, 0],
        Some(before.with_start([2, 0, 0, 0], 20)),
    );
    let waiting = tally.take_queued(id(1), 30);
    assert_eq!(waiting.len(), 1);
    let parts = lead_parts(&[pose(1, 1)], &waiting);
    assert_eq!(parts.len(), 2);
    assert_eq!(parts[1].starts, waiting);
    assert!(parts.iter().all(EnemyLead::is_valid));
    assert_eq!(lead_parts(&[pose(2, 0)], &waiting).len(), 1);
}

#[test]
fn only_enemies_that_showed_boss_health_get_start_lines() {
    let mut log = BossStartLog::default();
    log.note_health(id(1), BOSS_CLASS_HEALTH_FROM);
    log.note_health(id(2), BOSS_CLASS_HEALTH_FROM - 1);
    assert!(log.is_boss(id(1)));
    assert!(!log.is_boss(id(2)));
    let sent = sent_start_line(id(1), 7, [0x8E, 1, 2, 3], 40);
    assert!(sent.contains("start 7") && sent.contains("0x8E 1 2 3") && sent.contains("40 ms"));
    let played = played_start_line(id(1), None, [0x20, 0, 0, 0], 0, true);
    assert!(played.contains("latest pose") && played.contains("started: true"));
}
