use replicant_game::enemies::{
    BOSS_MOTION_LINES_PER_TICK, BossMotionLog, boss_motion_changed_text,
};
use replicant_game::puppet::LayerMotion;
use replicant_proto::EnemyId;

fn boss() -> EnemyId {
    EnemyId {
        group: 100,
        ordinal: 0,
    }
}

fn motion(index: i32, frame: f32) -> Option<LayerMotion> {
    Some(LayerMotion {
        index,
        frame,
        speed: 1.0,
        looping: true,
    })
}

#[test]
fn the_first_sighting_of_the_body_is_silent_and_a_change_is_logged_once() {
    let mut log = BossMotionLog::default();
    assert!(log.observe("host", [(boss(), motion(5, 0.0))]).is_empty());
    assert!(log.observe("host", [(boss(), motion(5, 9.0))]).is_empty());
    assert_eq!(
        log.observe("host", [(boss(), motion(1, 0.5))]),
        vec![boss_motion_changed_text(
            "host",
            boss(),
            Some(5),
            motion(1, 0.5)
        )]
    );
    assert!(log.observe("host", [(boss(), motion(1, 1.5))]).is_empty());
}

#[test]
fn the_l44_body_going_from_the_stay_pose_to_an_attack_start_is_one_line() {
    let mut log = BossMotionLog::default();
    log.observe("guest", [(boss(), motion(1, 1.9))]);
    let lines = log.observe("guest", [(boss(), motion(8, 0.0))]);
    assert_eq!(
        lines,
        vec!["boss body (guest): 100:0 motion 1 -> 8, at frame 0.0 speed 1.00 looping"]
    );
}

#[test]
fn an_unreadable_motion_is_logged_as_none() {
    let mut log = BossMotionLog::default();
    log.observe("host", [(boss(), motion(1, 0.0))]);
    assert_eq!(
        log.observe("host", [(boss(), None)]),
        vec!["boss body (host): 100:0 motion 1 -> none"]
    );
}

#[test]
fn a_tick_logs_at_most_the_line_budget() {
    let mut log = BossMotionLog::default();
    let many: Vec<EnemyId> = (0..10)
        .map(|ordinal| EnemyId {
            group: 100,
            ordinal,
        })
        .collect();
    log.observe("host", many.iter().map(|id| (*id, motion(1, 0.0))));
    let lines = log.observe("host", many.iter().map(|id| (*id, motion(2, 0.0))));
    assert_eq!(lines.len(), BOSS_MOTION_LINES_PER_TICK);
}
