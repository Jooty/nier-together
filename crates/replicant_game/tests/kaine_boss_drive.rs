use replicant_game::GameMemory;
use replicant_game::enemies::{partner_actors, partner_actors_for_enemy_scan};
use replicant_game::kaine::{
    BOSS_FIGHT_KAINE_MODE, BOSS_FIGHT_PACK, BossDrive, BossDriveEvent, BossDriveFacts,
    BossDriveMisses, FIGHT_START_JOIN_WINDOW_MS, GUEST_MOTION_KEYWORDS_TO_HER_MOTIONS, GuestSeat,
    HeldBackWhy, HerMotionChoice, her_motion_for, scaled_frame,
};
use replicant_game::offsets::PARTNER_KAINE_ACTOR_RVA;

use crate::fake_memory::FakeMemory;

const HER_PACK_MOTION_NAMES: [&str; 54] = [
    "kaine010_01_stay^lo_01",
    "kaine010_01_walk^lo_01",
    "kaine010_01_dash^lo_01",
    "kaine010_01_dash^e_01",
    "kaine010_01_turn_01",
    "kaine010_at_sword^c01_01",
    "kaine010_at_sword^c02_01",
    "kaine010_at_sword^c03_01",
    "kaine010_at_sword^c04_01",
    "kaine010_at_sword^c05_01",
    "kaine010_at_sword^c06_01",
    "kaine010_at_sword^t01_01",
    "kaine010_at_sword^t02_01",
    "kaine010_at_sword^t03_01",
    "kaine010_at_magic^c01_01",
    "kaine010_at_magic^c02_01",
    "kaine010_at_magic^c03^s_01",
    "kaine010_at_magic^c03^lo_01",
    "kaine010_at_magic^c03^e_01",
    "kaine010_01_jump^s_01",
    "kaine010_01_jump^01_01",
    "kaine010_01_jump^02_01",
    "kaine010_01_jump^e_01",
    "kaine010_02_land_01",
    "kaine010_01_staggerrun_01",
    "kaine010_01_fall^s_01",
    "kaine010_01_fall^lo_01",
    "kaine010_dm_damages^r_01",
    "kaine010_dm_damagem^r_01",
    "kaine010_dm_damagel^f_01",
    "kaine010_dm_damagel^b_01",
    "kaine010_dm_damagel^f^standup_01",
    "kaine010_dm_damagel^b^standup_01",
    "kaine010_dm_blow^f_01",
    "kaine010_dm_blow^f^damage_01",
    "kaine010_dm_blow^f^standup_01",
    "kaine010_dm_blow^f^recover_01",
    "kaine010_dm_gblow^f_01",
    "kaine010_dm_gblow^f^damage_01",
    "kaine010_dm_gblow^f^standup_01",
    "kaine010_dm_gblow^f^recover_01",
    "kaine010_df_guardstay^lo_01",
    "kaine010_df_guardwalk^f^lo_01",
    "kaine010_df_guardwalk^l^lo_01",
    "kaine010_df_guard_01",
    "kaine010_df_guard_02",
    "kaine010_df_guard_03",
    "kaine010_df_guard_04",
    "kaine010_df_guard_05",
    "kaine010_df_break_01",
    "kaine010_01_escape^f_01",
    "kaine010_01_escape^b_01",
    "kaine010_01_escape^l_01",
    "kaine010_01_escape^r_01",
];

const L43_FIGHT_PUPPET_LINES: [&str; 10] = [
    "22:50:32.061 puppet 0xe4485bd0: motion 374 (KAINE015_sws_idle_lp) from frame 1.2, blend 0.10 s",
    "22:52:19.721 puppet 0x850609d0: motion 265 (KAINE015_sws_atkS_1) from frame 1.4, blend 0.10 s",
    "22:53:21.842 puppet 0x850609d0: motion 391 (KAINE015_sws_run_lp) from frame 1.5, blend 0.10 s",
    "22:53:33.779 puppet 0x850609d0: motion 300 (KAINE015_sws_dodge_f) from frame 2.0, blend 0.10 s",
    "22:53:43.362 puppet 0x850609d0: motion 194 (KAINE015_magSpearCharge_trg_st) from frame 0.1, blend 0.10 s",
    "22:55:49.326 puppet 0x850609d0: motion 52 (KAINE015_dmgS_r) from frame 0.3, blend 0.10 s",
    "22:55:54.361 puppet 0x850609d0: motion 243 (KAINE015_springup_f) from frame 2.3, blend 0.10 s",
    "22:58:04.548 puppet 0x850609d0: motion 399 (KAINE015_sws_walk_lp) from frame 2.4, blend 0.10 s",
    "22:58:25.420 puppet 0x850609d0: motion 371 (KAINE015_sws_guard_lp) from frame 1.5, blend 0.10 s",
    "22:59:17.356 puppet 0x850609d0: motion 386 (KAINE015_sws_runJump) from frame 1.9, blend 0.10 s",
];

const L43_GUEST_MOTION_NAMES: [&str; 45] = [
    "KAINE015_idle_lp",
    "KAINE015_sws_idle_st",
    "KAINE015_sws_idle_lp",
    "KAINE015_sws_walk_st",
    "KAINE015_sws_walk_lp",
    "KAINE015_sws_walk_ed",
    "KAINE015_sws_run_st",
    "KAINE015_sws_run_lp",
    "KAINE015_sws_run_ed",
    "KAINE015_sws_runJump_st",
    "KAINE015_sws_runJump",
    "KAINE015_sws_runJump2nd",
    "KAINE015_sws_dodge_f",
    "KAINE015_sws_dodge_b",
    "KAINE015_sws_dodge_l",
    "KAINE015_sws_dashFromF_st_f",
    "KAINE015_sws_dash_ed_f",
    "KAINE015_sws_atkS",
    "KAINE015_sws_atkS_1",
    "KAINE015_sws_atkS_3",
    "KAINE015_sws_atkS_4",
    "KAINE015_sws_atkS_5",
    "KAINE015_sws_atkSDash",
    "KAINE015_sws_atkSCharge_st",
    "KAINE015_sws_atkSCharge",
    "KAINE015_sws_atkSAir",
    "KAINE015_sws_atkSAir_1",
    "KAINE015_sws_atkSAir_2",
    "KAINE015_sws_atkH",
    "KAINE015_sws_atkHDash",
    "KAINE015_sws_atkHCharge_st",
    "KAINE015_sws_guard_st",
    "KAINE015_sws_guard_lp",
    "KAINE015_sws_guard_ed",
    "KAINE015_springup_f",
    "KAINE015_springup_b",
    "KAINE015_magSpearCharge_trg_st",
    "KAINE015_magSpearCharge_trg_lp",
    "KAINE015_dmgS_r",
    "KAINE015_dmgBlowWall_st_f",
    "KAINE015_dmgBlow_st_f",
    "KAINE015_dmgBlow_st_b",
    "KAINE015_dmgBlow_lp_b",
    "KAINE015_dmgBlow_ed_f",
    "KAINE015_dmgBlow_ed_b",
];

fn motion_name_of(line: &str) -> &str {
    let open = line.find('(').expect("the line names a motion");
    let close = line.find(')').expect("the line closes the name");
    &line[open + 1..close]
}

fn her_pack_index(wanted: &str) -> Option<i32> {
    HER_PACK_MOTION_NAMES
        .iter()
        .position(|name| *name == wanted)
        .map(|index| i32::try_from(index).expect("a small index"))
}

fn wanted_for(guest_name: &str) -> Option<&'static str> {
    match her_motion_for(guest_name, her_pack_index) {
        HerMotionChoice::Play { wanted, .. } => Some(wanted),
        _ => None,
    }
}

fn facts(guest: GuestSeat) -> BossDriveFacts {
    BossDriveFacts {
        in_boss_fight_pack: true,
        her_mode: Some(BOSS_FIGHT_KAINE_MODE),
        her_active: true,
        guest,
        game_poses_her: false,
        now_ms: 0,
    }
}

fn before_the_fight(guest: GuestSeat) -> BossDriveFacts {
    BossDriveFacts {
        her_mode: Some(0),
        ..facts(guest)
    }
}

#[test]
fn the_fight_is_live_once_her_mode_is_2_in_the_fights_pack_and_not_before() {
    let mut drive = BossDrive::new();
    assert!(drive.step(&before_the_fight(GuestSeat::Playing)).is_empty());
    assert!(!drive.boss_fight_is_live());
    let events = drive.step(&facts(GuestSeat::Playing));
    assert_eq!(
        events[0],
        BossDriveEvent::FightBegan {
            guest: GuestSeat::Playing
        }
    );
    assert!(drive.boss_fight_is_live());
}

#[test]
fn another_pack_is_never_the_fight_whatever_her_mode_says() {
    let mut drive = BossDrive::new();
    let elsewhere = BossDriveFacts {
        in_boss_fight_pack: false,
        ..facts(GuestSeat::Playing)
    };
    assert!(drive.step(&elsewhere).is_empty());
    assert!(!drive.boss_fight_is_live());
    assert_eq!(BOSS_FIGHT_PACK, "A_CLIFF_VILLAGE_01");
}

#[test]
fn she_is_driven_while_the_guest_plays_her_and_the_game_is_not_posing_her() {
    let mut drive = BossDrive::new();
    let events = drive.step(&facts(GuestSeat::Playing));
    assert_eq!(events.last(), Some(&BossDriveEvent::Engaged));
    assert!(drive.is_driving());
    assert!(drive.guest_may_play_her());
    assert!(drive.step(&facts(GuestSeat::Playing)).is_empty());
}

#[test]
fn she_is_not_driven_while_she_is_inactive_as_in_the_l43_ticks_before_the_script_turned_her_on() {
    let mut drive = BossDrive::new();
    let inactive = BossDriveFacts {
        her_active: false,
        ..facts(GuestSeat::Playing)
    };
    let events = drive.step(&inactive);
    assert!(!drive.is_driving());
    assert_eq!(
        events.last(),
        Some(&BossDriveEvent::HeldBackToTheGame {
            why: HeldBackWhy::NotActive
        })
    );
    let events = drive.step(&facts(GuestSeat::Playing));
    assert_eq!(events, vec![BossDriveEvent::Engaged]);
}

#[test]
fn a_scene_hands_her_back_to_the_game_and_the_drive_resumes_after_it() {
    let mut drive = BossDrive::new();
    drive.step(&facts(GuestSeat::Playing));
    let scene = BossDriveFacts {
        game_poses_her: true,
        ..facts(GuestSeat::Playing)
    };
    assert_eq!(
        drive.step(&scene),
        vec![BossDriveEvent::HeldBackToTheGame {
            why: HeldBackWhy::GamePosesHer
        }]
    );
    assert!(!drive.is_driving());
    assert!(drive.guest_may_play_her());
    assert!(drive.step(&scene).is_empty());
    assert_eq!(
        drive.step(&facts(GuestSeat::Playing)),
        vec![BossDriveEvent::Engaged]
    );
}

#[test]
fn the_guest_leaving_gives_her_ai_back_and_the_fight_carries_on() {
    let mut drive = BossDrive::new();
    drive.step(&facts(GuestSeat::Playing));
    assert_eq!(
        drive.step(&facts(GuestSeat::Absent)),
        vec![BossDriveEvent::GuestLeft]
    );
    assert!(!drive.is_driving());
    assert!(drive.boss_fight_is_live());
    assert!(!drive.guest_may_play_her());
}

#[test]
fn a_guest_that_comes_back_mid_fight_does_not_drive_her_until_the_fight_is_over() {
    let mut drive = BossDrive::new();
    drive.step(&facts(GuestSeat::Playing));
    drive.step(&facts(GuestSeat::Absent));
    assert!(drive.step(&facts(GuestSeat::Playing)).is_empty());
    assert!(!drive.is_driving());
    assert!(!drive.guest_may_play_her());
    assert!(drive.boss_fight_is_live());
}

#[test]
fn the_fight_ending_clears_the_leave_so_the_next_fight_can_be_driven() {
    let mut drive = BossDrive::new();
    drive.step(&facts(GuestSeat::Playing));
    drive.step(&facts(GuestSeat::Absent));
    assert_eq!(
        drive.step(&before_the_fight(GuestSeat::Playing)),
        vec![BossDriveEvent::FightEnded {
            was_driving: false,
            guest_had_left: true
        }]
    );
    assert!(!drive.boss_fight_is_live());
    assert_eq!(
        drive.step(&facts(GuestSeat::Playing)).last(),
        Some(&BossDriveEvent::Engaged)
    );
}

#[test]
fn with_no_guest_the_fight_is_the_games_and_she_is_never_driven() {
    let mut drive = BossDrive::new();
    let events = drive.step(&facts(GuestSeat::Absent));
    assert_eq!(
        events,
        vec![
            BossDriveEvent::FightBegan {
                guest: GuestSeat::Absent
            },
            BossDriveEvent::HeldBackToTheGame {
                why: HeldBackWhy::NoGuestYet
            }
        ]
    );
    assert!(!drive.is_driving());
    assert!(drive.boss_fight_is_live());
    let late = BossDriveFacts {
        now_ms: FIGHT_START_JOIN_WINDOW_MS,
        ..facts(GuestSeat::Absent)
    };
    assert_eq!(
        drive.step(&late),
        vec![BossDriveEvent::JoinWindowClosed {
            guest: GuestSeat::Absent
        }]
    );
    assert!(!drive.guest_may_play_her());
}

#[test]
fn a_guest_that_only_watches_past_the_first_seconds_never_drives_her() {
    let mut drive = BossDrive::new();
    drive.step(&facts(GuestSeat::Watching));
    let late = BossDriveFacts {
        now_ms: FIGHT_START_JOIN_WINDOW_MS,
        ..facts(GuestSeat::Watching)
    };
    assert_eq!(
        drive.step(&late),
        vec![BossDriveEvent::JoinWindowClosed {
            guest: GuestSeat::Watching
        }]
    );
    let playing_late = BossDriveFacts {
        now_ms: FIGHT_START_JOIN_WINDOW_MS + 1,
        ..facts(GuestSeat::Playing)
    };
    assert!(drive.step(&playing_late).is_empty());
    assert!(!drive.is_driving());
}

#[test]
fn the_fight_ending_while_driving_ends_the_drive() {
    let mut drive = BossDrive::new();
    drive.step(&facts(GuestSeat::Playing));
    assert_eq!(
        drive.step(&before_the_fight(GuestSeat::Playing)),
        vec![BossDriveEvent::FightEnded {
            was_driving: true,
            guest_had_left: false
        }]
    );
    assert!(!drive.is_driving());
    assert!(!drive.guest_may_play_her());
}

#[test]
fn the_enemy_scan_sees_her_only_while_she_is_the_boss() {
    let memory = FakeMemory::new();
    let her = memory.module_base() + u64::from(PARTNER_KAINE_ACTOR_RVA);
    assert!(partner_actors(&memory).contains(&her));
    assert!(partner_actors_for_enemy_scan(&memory, false).contains(&her));
    assert!(!partner_actors_for_enemy_scan(&memory, true).contains(&her));
    assert_eq!(
        partner_actors_for_enemy_scan(&memory, true).len() + 1,
        partner_actors(&memory).len()
    );
}

#[test]
fn every_motion_the_table_asks_of_her_pack_is_in_her_pack() {
    for (_, wanted) in GUEST_MOTION_KEYWORDS_TO_HER_MOTIONS {
        assert!(
            her_pack_index(wanted).is_some(),
            "{wanted} is not in her pack"
        );
    }
}

#[test]
fn the_l43_fight_lines_map_to_motions_her_pack_holds() {
    let wanted: Vec<_> = L43_FIGHT_PUPPET_LINES
        .iter()
        .map(|line| wanted_for(motion_name_of(line)))
        .collect();
    assert_eq!(
        wanted,
        vec![
            Some("kaine010_01_stay^lo_01"),
            Some("kaine010_at_sword^c02_01"),
            Some("kaine010_01_dash^lo_01"),
            Some("kaine010_01_escape^f_01"),
            Some("kaine010_at_magic^c01_01"),
            Some("kaine010_dm_damages^r_01"),
            Some("kaine010_dm_damagel^f^standup_01"),
            Some("kaine010_01_walk^lo_01"),
            Some("kaine010_df_guardstay^lo_01"),
            Some("kaine010_01_jump^01_01"),
        ]
    );
}

#[test]
fn the_sword_chain_walks_up_her_combo_steps() {
    let chain: Vec<_> = [
        "KAINE015_sws_atkS",
        "KAINE015_sws_atkS_1",
        "KAINE015_sws_atkS_3",
        "KAINE015_sws_atkS_4",
        "KAINE015_sws_atkS_5",
    ]
    .iter()
    .map(|name| wanted_for(name))
    .collect();
    assert_eq!(
        chain,
        vec![
            Some("kaine010_at_sword^c01_01"),
            Some("kaine010_at_sword^c02_01"),
            Some("kaine010_at_sword^c04_01"),
            Some("kaine010_at_sword^c05_01"),
            Some("kaine010_at_sword^c06_01"),
        ]
    );
}

#[test]
fn a_run_jump_is_a_jump_and_not_a_run() {
    assert_eq!(
        wanted_for("KAINE015_sws_runJump"),
        Some("kaine010_01_jump^01_01")
    );
    assert_eq!(
        wanted_for("KAINE015_sws_run_ed"),
        Some("kaine010_01_dash^e_01")
    );
}

#[test]
fn every_motion_the_guest_played_in_l43_has_a_twin_or_is_logged_once() {
    let mut misses = BossDriveMisses::new();
    let mut unmapped = Vec::new();
    for name in L43_GUEST_MOTION_NAMES {
        let choice = her_motion_for(name, her_pack_index);
        if !matches!(choice, HerMotionChoice::Play { .. }) {
            unmapped.push(name);
        }
        if misses.observe(name, choice).is_some() {
            assert!(unmapped.contains(&name));
        }
    }
    assert!(unmapped.is_empty(), "no twin: {unmapped:?}");
}

#[test]
fn a_motion_with_no_family_keeps_her_last_motion_and_is_named_in_the_log_once() {
    let mut misses = BossDriveMisses::new();
    let choice = her_motion_for("KAINE015_sws_fishingCast", her_pack_index);
    assert_eq!(choice, HerMotionChoice::NoFamily);
    let text = misses.observe("KAINE015_sws_fishingCast", choice);
    assert!(text.is_some_and(|text| text.contains("no twin in her pack")));
    assert_eq!(misses.observe("kaine015_sws_fishingcast", choice), None);
}

#[test]
fn a_family_her_pack_cannot_answer_is_a_miss_and_not_a_guess() {
    let choice = her_motion_for("KAINE015_sws_idle_lp", |_| None);
    assert_eq!(
        choice,
        HerMotionChoice::NotInHerPack {
            wanted: "kaine010_01_stay^lo_01"
        }
    );
}

#[test]
fn her_frame_is_the_guests_place_in_the_motion_scaled_to_her_length() {
    assert!((scaled_frame(30.0, 61, 31) - 15.0).abs() < 1e-4);
    assert!((scaled_frame(60.0, 61, 21) - 20.0).abs() < 1e-4);
    assert_eq!(scaled_frame(5.0, 0, 31), 0.0);
    assert_eq!(scaled_frame(f32::NAN, 61, 31), 0.0);
}
