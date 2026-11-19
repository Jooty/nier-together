use replicant_game::kaine::{
    MOST_DISTINCT_MISSED_MOTION_NAMES_LOGGED, MissedMotionNames, is_player_motion_name,
    missed_motion_text, swapped_body_motion_twin,
};

const MANOR_PUPPET_LINES: [&str; 3] = [
    "18:31:52.216 puppet 0x8d6a02d0: motion 248 (NIER013_bioRun_lp) from frame 2.2, blend 0.10 s",
    "18:31:52.498 puppet 0x8d6a02d0: motion 1036 (NIER013_walk_ed_1) from frame 1.2, blend 0.10 s",
    "18:31:49.578 puppet 0x8d6a02d0: motion 14 (NIER010_01_DASH^S_02) from frame 1.1, blend 0.10 s",
];

fn motion_name_of(line: &str) -> &str {
    let open = line.find('(').expect("the line names a motion");
    let close = line.find(')').expect("the line closes the name");
    &line[open + 1..close]
}

#[test]
fn the_run_the_game_plays_in_the_manor_has_a_twin_in_her_pack() {
    let name = motion_name_of(MANOR_PUPPET_LINES[0]);
    assert_eq!(name, "NIER013_bioRun_lp");
    assert_eq!(
        swapped_body_motion_twin(name).as_deref(),
        Some("KAINE015_run_lp")
    );
}

#[test]
fn the_end_of_the_manor_run_has_a_twin_too() {
    assert_eq!(
        swapped_body_motion_twin("NIER013_bioRun_ed").as_deref(),
        Some("KAINE015_run_ed")
    );
}

#[test]
fn a_motion_her_pack_has_gets_no_twin() {
    let name = motion_name_of(MANOR_PUPPET_LINES[1]);
    assert_eq!(name, "NIER013_walk_ed_1");
    assert_eq!(swapped_body_motion_twin(name), None);
    assert_eq!(swapped_body_motion_twin("KAINE015_sws_walk_st"), None);
}

#[test]
fn the_indoor_jog_has_her_run_loop_as_its_twin() {
    assert_eq!(
        swapped_body_motion_twin("NIER013_jog_lp").as_deref(),
        Some("KAINE015_run_lp")
    );
}

#[test]
fn the_young_brothers_indoor_jog_has_her_run_loop_as_its_twin() {
    // run-1116a guest run2, 08:27:34.232: "kaine motions: the game asked her body for
    // NIER025_jog_lp, her pack has none and no twin is set, nothing plays"
    assert!(is_player_motion_name("NIER025_jog_lp"));
    assert_eq!(
        swapped_body_motion_twin("NIER025_jog_lp").as_deref(),
        Some("KAINE015_run_lp")
    );
}

#[test]
fn the_floor_change_walk_has_her_run_loop_as_its_twin() {
    // 08:27:39.353, the tick From_1F_to_2F_02 ran _SetUniqueActorScriptActionSetMotion(-1,
    // '_01_walk^lo_01'): "her pack has none and no twin is set, nothing plays", so she stayed on
    // the landing inside the trap for the way back
    assert_eq!(
        swapped_body_motion_twin("_01_walk^lo_01").as_deref(),
        Some("KAINE015_run_lp")
    );
}

#[test]
fn the_twin_is_found_for_a_name_asked_without_the_pack_prefix() {
    assert_eq!(
        swapped_body_motion_twin("jog_lp").as_deref(),
        Some("KAINE015_run_lp")
    );
    assert_eq!(
        swapped_body_motion_twin("bioRun_lp").as_deref(),
        Some("KAINE015_run_lp")
    );
}

#[test]
fn the_twin_is_found_whatever_the_case() {
    assert_eq!(
        swapped_body_motion_twin("nier013_BIORUN_LP").as_deref(),
        Some("KAINE015_run_lp")
    );
}

#[test]
fn a_name_of_another_pack_or_an_empty_one_has_no_twin() {
    assert_eq!(swapped_body_motion_twin("EM_jog_lp"), None);
    assert_eq!(swapped_body_motion_twin(""), None);
}

#[test]
fn only_the_players_own_pack_names_are_player_motion_names() {
    assert!(is_player_motion_name("NIER013_bioRun_lp"));
    assert!(is_player_motion_name("KAINE015_idle_lp"));
    assert!(!is_player_motion_name(motion_name_of(
        MANOR_PUPPET_LINES[2]
    )));
    assert!(!is_player_motion_name("EM_slime_walk"));
    assert!(!is_player_motion_name("idle_lp"));
}

#[test]
fn a_missed_name_is_logged_once_with_its_twin() {
    let mut missed = MissedMotionNames::new();
    let line = missed.observe("NIER013_bioRun_lp", Some("KAINE015_run_lp"));
    assert_eq!(
        line.as_deref(),
        Some(
            "kaine motions: the game asked her body for NIER013_bioRun_lp, her pack has none, it plays KAINE015_run_lp instead"
        )
    );
    assert_eq!(
        missed.observe("nier013_biorun_lp", Some("KAINE015_run_lp")),
        None
    );
}

#[test]
fn a_missed_name_with_no_twin_says_nothing_plays() {
    let mut missed = MissedMotionNames::new();
    let line = missed.observe("NIER013_wait_lp", None).expect("first miss");
    assert_eq!(line, missed_motion_text("NIER013_wait_lp", None));
    assert!(line.ends_with("no twin is set, nothing plays"));
}

#[test]
fn an_empty_name_is_never_logged_and_a_name_without_the_pack_prefix_is() {
    let mut missed = MissedMotionNames::new();
    assert_eq!(missed.observe("", None), None);
    assert!(missed.observe("jog_lp", None).is_some());
}

const NIER025_NAMES_MISSED_AT_THE_STORY_JUMP_LOAD: usize = 48;
const HER_MOTION_WORK: u64 = 0x1_4000_0000;
const ANOTHER_MOTION_WORK: u64 = 0x1_5000_0000;

#[test]
fn the_flood_of_another_packs_names_at_the_load_no_longer_hides_the_indoor_miss() {
    // run-1115j guest run2, 01:44:50.423 to 01:44:50.426: 48 distinct NIER025_* misses at the
    // story jump load used up the old cap of 48 before she ever walked indoors
    let mut missed = MissedMotionNames::new();
    for number in 0..NIER025_NAMES_MISSED_AT_THE_STORY_JUMP_LOAD {
        let name = format!("NIER025_swl_missing_{number}");
        assert!(missed.observe_on(HER_MOTION_WORK, &name, None).is_some());
    }
    assert!(
        missed
            .observe_on(HER_MOTION_WORK, "NIER025_jog_lp", None)
            .is_some()
    );
    assert_eq!(
        missed.last_missed_on(HER_MOTION_WORK),
        Some("NIER025_jog_lp")
    );
}

#[test]
fn the_last_miss_is_kept_per_motion_work_and_a_miss_with_a_twin_is_not_a_miss() {
    let mut missed = MissedMotionNames::new();
    missed.observe_on(HER_MOTION_WORK, "NIER020_sws_walk_lp", None);
    missed.observe_on(ANOTHER_MOTION_WORK, "EM_slime_walk", None);
    missed.observe_on(HER_MOTION_WORK, "NIER013_jog_lp", Some("KAINE015_run_lp"));
    assert_eq!(
        missed.last_missed_on(HER_MOTION_WORK),
        Some("NIER020_sws_walk_lp")
    );
    assert_eq!(
        missed.last_missed_on(ANOTHER_MOTION_WORK),
        Some("EM_slime_walk")
    );
    assert_eq!(missed.last_missed_on(0x9999), None);
    missed.observe_on(HER_MOTION_WORK, "NIER020_sws_walk_lp", None);
    assert_eq!(
        missed.last_missed_on(HER_MOTION_WORK),
        Some("NIER020_sws_walk_lp")
    );
}

#[test]
fn the_log_stops_after_the_most_distinct_names() {
    let mut missed = MissedMotionNames::new();
    for number in 0..MOST_DISTINCT_MISSED_MOTION_NAMES_LOGGED {
        let name = format!("NIER013_missing_{number}");
        assert!(missed.observe(&name, None).is_some());
    }
    assert_eq!(missed.observe("NIER013_one_more", None), None);
}
