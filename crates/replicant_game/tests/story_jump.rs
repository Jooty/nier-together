use replicant_game::PhaseEntry;
use replicant_game::host_led_stretch::stretch_the_story_is_inside;
use replicant_game::prologue_levels::PROLOGUE_PHASES;
use replicant_game::story_jump::{
    ENTRY_SCRIPT_WAIT_GIVE_UP_MS, ITEM_AUDIT_AFTER_RESTORE_POINT_MS, JumpStep,
    LEVEL_CHECK_SETTLE_MS, LevelCheck, LevelWatcher, PLACEMENT_READ_BACK_AFTER_MS,
    PLACEMENTS_REPEATED_AFTER_AN_ENTRY_SCRIPT, PROLOGUE_END_PLAYER_LEVEL,
    RESTORE_POINT_WAITS_AFTER_PHASE_CHANGE_MS, ReadBackStep, StoryJumpRefusal, StoryJumpView,
    arrival_is_due, arrival_spot, check_level, check_story_jump, destination,
    guest_reloads_in_place, item_audit_is_due, item_shortfalls, jump_resets_prologue_level,
    jump_stat_level, jump_waits_for_pause_menu, level_check_is_due, levels_to_raise, loadout_steps,
    read_back_step, restore_point_is_due, shared_items_the_jump_gives, unreadable_items,
};
use replicant_proto::StoryJumpPoint;

fn ready_host() -> StoryJumpView {
    StoryJumpView {
        host: true,
        map_manager_active: true,
        event_scene: false,
        talk_window_open: false,
        ui_dialog_state: 0,
        pause_menu_open: false,
    }
}

#[test]
fn a_host_in_the_field_may_jump() {
    assert_eq!(check_story_jump(&ready_host()), Ok(()));
}

#[test]
fn a_guest_is_refused() {
    let view = StoryJumpView {
        host: false,
        ..ready_host()
    };
    assert_eq!(check_story_jump(&view), Err(StoryJumpRefusal::NotHost));
}

#[test]
fn a_host_outside_the_field_is_refused() {
    let loading = StoryJumpView {
        map_manager_active: false,
        ..ready_host()
    };
    assert_eq!(
        check_story_jump(&loading),
        Err(StoryJumpRefusal::NotInTheField)
    );
}

#[test]
fn an_event_scene_refuses_the_jump() {
    let scene = StoryJumpView {
        event_scene: true,
        ..ready_host()
    };
    assert_eq!(check_story_jump(&scene), Err(StoryJumpRefusal::EventScene));
}

#[test]
fn an_open_talk_refuses_the_jump() {
    let talk = StoryJumpView {
        talk_window_open: true,
        ..ready_host()
    };
    assert_eq!(check_story_jump(&talk), Err(StoryJumpRefusal::TalkOpen));
}

#[test]
fn an_open_dialog_refuses_the_jump_and_names_its_state() {
    let dialog = StoryJumpView {
        ui_dialog_state: 4,
        ..ready_host()
    };
    let refusal = check_story_jump(&dialog).unwrap_err();
    assert_eq!(refusal, StoryJumpRefusal::DialogOpen(4));
    assert!(refusal.to_string().contains("0x4"));
}

#[test]
fn the_pause_menu_does_not_refuse_the_jump_but_makes_it_wait() {
    let paused = StoryJumpView {
        pause_menu_open: true,
        ..ready_host()
    };
    assert_eq!(check_story_jump(&paused), Ok(()));
    assert!(jump_waits_for_pause_menu(&paused));
    assert!(!jump_waits_for_pause_menu(&ready_host()));
}

#[test]
fn after_the_prologue_sets_the_end_flags_and_the_loadout() {
    let steps = loadout_steps(StoryJumpPoint::VillageAfterPrologue);
    for expected in [
        JumpStep::SetFlag { id: 71 },
        JumpStep::SetFlag { id: 153 },
        JumpStep::SetItem { id: 1000, count: 1 },
        JumpStep::EquipWeapon { id: 1000 },
        JumpStep::MpFullMode { on: false },
    ] {
        assert!(steps.contains(&expected), "{expected:?}");
    }
}

#[test]
fn after_the_prologue_clears_the_resume_flags() {
    let steps = loadout_steps(StoryJumpPoint::VillageAfterPrologue);
    assert!(steps.contains(&JumpStep::ClearFlag { id: 1403 }));
    assert!(steps.contains(&JumpStep::ClearFlag { id: 149 }));
}

#[test]
fn after_the_prologue_lands_on_a_known_phase_and_a_valid_point() {
    let (name, point) = destination(StoryJumpPoint::VillageAfterPrologue);
    assert!(PhaseEntry::find(name).is_some());
    assert!((0..=replicant_game::offsets::MAX_PHASE_POINT).contains(&point));
}

#[test]
fn the_popola_talk_adds_its_flags_and_keeps_the_loadout() {
    let steps = loadout_steps(StoryJumpPoint::LibraryPopolaTalk);
    for expected in [
        JumpStep::SetFlag { id: 71 },
        JumpStep::SetFlag { id: 153 },
        JumpStep::SetFlag { id: 154 },
        JumpStep::SetFlag { id: 1299 },
        JumpStep::SetFlag { id: 1401 },
        JumpStep::ClearFlag { id: 1403 },
        JumpStep::ClearFlag { id: 149 },
        JumpStep::EquipWeapon { id: 1000 },
        JumpStep::MpFullMode { on: false },
    ] {
        assert!(steps.contains(&expected), "{expected:?}");
    }
    assert!(!steps.contains(&JumpStep::SetFlag { id: 156 }));
}

#[test]
fn the_popola_talk_lands_in_the_library_on_a_valid_point() {
    let (name, point) = destination(StoryJumpPoint::LibraryPopolaTalk);
    assert_eq!(name, "A_CENTER_LIBRARY_01");
    assert!(PhaseEntry::find(name).is_some());
    assert!((0..=replicant_game::offsets::MAX_PHASE_POINT).contains(&point));
}

#[test]
fn before_the_next_day_carries_the_popola_day_flags_and_the_map() {
    let steps = loadout_steps(StoryJumpPoint::VillageBeforeNextDay);
    for expected in [
        JumpStep::SetFlag { id: 71 },
        JumpStep::SetFlag { id: 154 },
        JumpStep::SetFlag { id: 156 },
        JumpStep::SetFlag { id: 163 },
        JumpStep::SetFlag { id: 164 },
        JumpStep::SetFlag { id: 166 },
        JumpStep::SetFlag { id: 1300 },
        JumpStep::ClearFlag { id: 1299 },
        JumpStep::ClearFlag { id: 1401 },
        JumpStep::ClearFlag { id: 1403 },
        JumpStep::SetItem { id: 723, count: 1 },
        JumpStep::EquipWeapon { id: 1000 },
    ] {
        assert!(steps.contains(&expected), "{expected:?}");
    }
    assert!(!steps.contains(&JumpStep::SetFlag { id: 1299 }));
    assert!(!steps.contains(&JumpStep::SetFlag { id: 1401 }));
}

#[test]
fn before_the_next_day_lands_in_the_village_outside_the_library() {
    let (name, point) = destination(StoryJumpPoint::VillageBeforeNextDay);
    assert_eq!(name, "A_CENTER_VILLAGE_01");
    assert_eq!(point, 4);
    assert!(PhaseEntry::find(name).is_some());
    assert!((0..=replicant_game::offsets::MAX_PHASE_POINT).contains(&point));
}

#[test]
fn the_next_day_jump_gives_the_map_item_and_its_tutorial() {
    let steps = loadout_steps(StoryJumpPoint::VillageBeforeNextDay);
    assert!(steps.contains(&JumpStep::SetItem { id: 723, count: 1 }));
    assert!(steps.contains(&JumpStep::SetTutorial { id: 2010 }));
}

#[test]
fn the_jumps_before_the_map_do_not_give_it() {
    for point in [
        StoryJumpPoint::VillageAfterPrologue,
        StoryJumpPoint::LibraryPopolaTalk,
    ] {
        let steps = loadout_steps(point);
        assert!(!steps.contains(&JumpStep::SetItem { id: 723, count: 1 }));
        assert!(!steps.contains(&JumpStep::SetTutorial { id: 2010 }));
    }
}

#[test]
fn the_shrine_entrance_jump_lands_in_the_shrine_with_the_map_and_counters() {
    let steps = loadout_steps(StoryJumpPoint::ShrineEntrance);
    for expected in [
        JumpStep::SetItem { id: 723, count: 1 },
        JumpStep::SetTutorial { id: 2010 },
        JumpStep::SetFlag { id: 167 },
        JumpStep::SetFlag { id: 171 },
        JumpStep::SetFlag { id: 77 },
        JumpStep::SetFlag { id: 1303 },
        JumpStep::SetSnowFlag { id: 1052 },
        JumpStep::SetSnowFlag { id: 1069 },
        JumpStep::ClearFlag { id: 1300 },
        JumpStep::ClearFlag { id: 1443 },
        JumpStep::SetGameCounter {
            id: 100,
            value: 255,
        },
        JumpStep::SetGameCounter {
            id: 101,
            value: 255,
        },
    ] {
        assert!(steps.contains(&expected), "{expected:?}");
    }
    for set in [190, 191] {
        assert!(steps.contains(&JumpStep::SetFlag { id: set }), "{set}");
    }
    for id in 1401..=1410 {
        assert!(steps.contains(&JumpStep::ClearFlag { id }), "{id}");
    }
    for unset in [75, 192, 202, 205] {
        assert!(!steps.contains(&JumpStep::SetFlag { id: unset }), "{unset}");
    }
    let (name, point) = destination(StoryJumpPoint::ShrineEntrance);
    assert_eq!(name, "A_STONE_TEMPLE_01");
    assert_eq!(point, 1);
    assert!(PhaseEntry::find(name).is_some());
}

#[test]
fn the_shrine_top_jump_lands_in_the_stone_field_with_the_shrine_done() {
    let steps = loadout_steps(StoryJumpPoint::ShrineTop);
    for expected in [
        JumpStep::SetFlag { id: 190 },
        JumpStep::SetFlag { id: 191 },
        JumpStep::SetFlag { id: 75 },
        JumpStep::SetFlag { id: 192 },
        JumpStep::SetFlag { id: 138 },
        JumpStep::SetFlag { id: 193 },
        JumpStep::SetFlag { id: 1432 },
        JumpStep::SetFlag { id: 205 },
        JumpStep::SetItem { id: 305, count: 0 },
        JumpStep::SetItem { id: 723, count: 1 },
    ] {
        assert!(steps.contains(&expected), "{expected:?}");
    }
    for id in 1401..=1410 {
        assert!(steps.contains(&JumpStep::ClearFlag { id }), "{id}");
        assert!(!steps.contains(&JumpStep::SetFlag { id }), "{id}");
    }
    assert!(!steps.contains(&JumpStep::SetFlag { id: 194 }));
    assert!(!steps.contains(&JumpStep::SetSnowFlag { id: 1053 }));
    let (name, point) = destination(StoryJumpPoint::ShrineTop);
    assert_eq!(name, "A_STONE_FIELD_01");
    assert_eq!(point, 3);
    assert!(PhaseEntry::find(name).is_some());
}

#[test]
fn the_shrine_boss_jump_lands_in_the_arena_with_the_boss_scene_still_to_play() {
    let top = loadout_steps(StoryJumpPoint::ShrineTop);
    let steps = loadout_steps(StoryJumpPoint::ShrineBoss2);
    for step in &top {
        assert!(steps.contains(step), "{step:?}");
    }
    for expected in [
        JumpStep::SetFlag { id: 194 },
        JumpStep::SetFlag { id: 206 },
        JumpStep::SetSnowFlag { id: 1053 },
    ] {
        assert!(steps.contains(&expected), "{expected:?}");
    }
    for id in 1401..=1410 {
        assert!(steps.contains(&JumpStep::ClearFlag { id }), "{id}");
        assert!(!steps.contains(&JumpStep::SetFlag { id }), "{id}");
    }
    for id in [127, 1432, 1433, 1434] {
        assert!(steps.contains(&JumpStep::SetFlag { id }), "{id}");
    }
    assert!(steps.contains(&JumpStep::ClearFlag { id: 1303 }));
    let last_1303 = steps.iter().rposition(|step| {
        matches!(
            step,
            JumpStep::SetFlag { id: 1303 } | JumpStep::ClearFlag { id: 1303 }
        )
    });
    assert_eq!(
        last_1303.map(|index| steps[index]),
        Some(JumpStep::ClearFlag { id: 1303 })
    );
    for id in [195, 196, 204, 1435] {
        assert!(!steps.contains(&JumpStep::SetFlag { id }), "{id}");
    }
    for expected in [
        JumpStep::SetHaveWhiteBook { on: true },
        JumpStep::LearnMagic { id: 3 },
        JumpStep::SetMagicSlot { slot: 1, magic: 3 },
        JumpStep::SetTutorial { id: 2002 },
    ] {
        assert!(steps.contains(&expected), "{expected:?}");
    }
    let (name, point) = destination(StoryJumpPoint::ShrineBoss2);
    assert_eq!(name, "A_STONE_FIELD_02");
    assert_eq!(point, 1);
    assert!(PhaseEntry::find(name).is_some());
}

#[test]
fn every_jump_point_has_the_stat_level_of_its_row() {
    for (id, level) in [
        (30, 1),
        (40, 1),
        (60, 1),
        (70, 3),
        (80, 4),
        (100, 6),
        (120, 12),
        (180, 20),
        (190, 22),
        (200, 26),
        (210, 26),
        (240, 28),
        (250, 29),
        (280, 32),
        (290, 34),
        (310, 34),
        (340, 36),
        (350, 36),
        (370, 36),
        (380, 38),
        (400, 40),
        (410, 42),
        (420, 42),
        (430, 42),
    ] {
        let point = StoryJumpPoint::from_id(id).expect("the id is a point");
        assert_eq!(jump_stat_level(point), level, "R-{id:03}");
    }
}

#[test]
fn the_stat_levels_never_fall_along_the_id_order() {
    let levels: Vec<i32> = StoryJumpPoint::in_id_order().map(jump_stat_level).collect();
    assert!(levels.windows(2).all(|pair| pair[0] <= pair[1]));
}

#[test]
fn a_jump_raises_the_level_by_the_difference() {
    assert_eq!(levels_to_raise(1, 12), 11);
    assert_eq!(levels_to_raise(0, 40), 40);
}

#[test]
fn a_jump_never_lowers_the_level() {
    assert_eq!(levels_to_raise(50, 12), 0);
    assert_eq!(levels_to_raise(12, 12), 0);
}

#[test]
fn the_raise_stops_at_level_98() {
    assert_eq!(levels_to_raise(90, 200), 8);
    assert_eq!(levels_to_raise(98, 200), 0);
    assert_eq!(levels_to_raise(99, 200), 0);
}

#[test]
fn a_level_at_or_above_the_row_level_holds_and_a_lower_one_is_raised_again() {
    assert_eq!(check_level(12, Some(12)), LevelCheck::Matches);
    assert_eq!(check_level(12, Some(30)), LevelCheck::Matches);
    assert_eq!(
        check_level(12, Some(1)),
        LevelCheck::ApplyAgain {
            expected: 12,
            actual: 1
        }
    );
    assert_eq!(check_level(12, None), LevelCheck::Unreadable);
}

#[test]
fn the_watcher_reports_only_changes() {
    let mut watcher = LevelWatcher::new();
    assert_eq!(watcher.observe(21), Some((None, 21)));
    assert_eq!(watcher.observe(21), None);
    assert_eq!(watcher.observe(0), Some((Some(21), 0)));
}

#[test]
fn the_level_check_waits_for_the_new_phase_to_settle() {
    let settled = LEVEL_CHECK_SETTLE_MS;
    assert!(!level_check_is_due("A", "A", true, settled));
    assert!(!level_check_is_due("A", "B", false, settled));
    assert!(!level_check_is_due("A", "B", true, settled - 1));
    assert!(level_check_is_due("A", "B", true, settled));
}

#[test]
fn only_the_arena_floor_b2_after_boss_fight_6_and_before_the_ride_jumps_place_the_players() {
    for point in [
        StoryJumpPoint::VillageAfterPrologue,
        StoryJumpPoint::LibraryPopolaTalk,
        StoryJumpPoint::VillageBeforeNextDay,
        StoryJumpPoint::ShrineEntrance,
        StoryJumpPoint::ShrineTop,
        StoryJumpPoint::AerieBoss3,
        StoryJumpPoint::JunkHeapBoss5,
        StoryJumpPoint::AerieBoss6,
    ] {
        assert_eq!(arrival_spot(point, true), None);
        assert_eq!(arrival_spot(point, false), None);
    }
    let host = arrival_spot(StoryJumpPoint::ShrineBoss2, true).expect("host spot");
    let guest = arrival_spot(StoryJumpPoint::ShrineBoss2, false).expect("guest spot");
    assert!((host.x + 2.2).abs() < 0.001 && (host.y - 146.8).abs() < 0.001);
    assert!((host.z + 8.6).abs() < 0.001);
    assert!((guest.x - host.x - 2.0).abs() < 0.001);
    assert_eq!((guest.y, guest.z), (host.y, host.z));
}

#[test]
fn the_floor_b2_jump_places_the_host_where_the_lift_lands_and_the_guest_beside_it() {
    let host = arrival_spot(StoryJumpPoint::JunkHeapB2, true).expect("host spot");
    let guest = arrival_spot(StoryJumpPoint::JunkHeapB2, false).expect("guest spot");
    assert_eq!((host.x, host.y, host.z), (-76.5, 4.9, -124.0));
    assert!((guest.x + 77.1).abs() < 0.001);
    assert_eq!((guest.y, guest.z), (host.y, host.z));
}

#[test]
fn the_floor_b2_jump_replays_the_lift_ride_writes_after_the_first_upgrade_state() {
    let steps = loadout_steps(StoryJumpPoint::JunkHeapB2);
    let upgraded = steps
        .iter()
        .position(|step| *step == JumpStep::SetFlag { id: 1314 })
        .expect("upgrade flag");
    let cleared = steps
        .iter()
        .position(|step| *step == JumpStep::ClearFlag { id: 1314 })
        .expect("upgrade flag cleared");
    let lift = steps
        .iter()
        .position(|step| *step == JumpStep::SetFlag { id: 302 })
        .expect("lift flag");
    assert!(upgraded < cleared && cleared < lift);
}

#[test]
fn the_boss_fight_3_jump_keeps_the_whole_arena_loadout_and_ends_with_the_mp_step() {
    let arena = loadout_steps(StoryJumpPoint::ShrineBoss2);
    let fight = loadout_steps(StoryJumpPoint::AerieBoss3);
    let arena_without_mp = &arena[..arena.len() - 1];
    assert_eq!(&fight[..arena_without_mp.len()], arena_without_mp);
    assert_eq!(fight.last(), Some(&JumpStep::MpFullMode { on: false }));
    assert_eq!(
        fight
            .iter()
            .filter(|step| matches!(step, JumpStep::MpFullMode { .. }))
            .count(),
        1
    );
}

#[test]
fn the_boss_fight_3_jump_leaves_the_flags_that_start_the_fight_on_arrival() {
    let (flags, counters, snow) = final_story_state(loadout_steps(StoryJumpPoint::AerieBoss3));
    for id in [
        200, 201, 202, 217, 218, 219, 1361, 130, 195, 196, 198, 204, 220, 221, 222, 224, 225, 239,
        70, 79, 240, 241, 242, 255, 1442, 1451, 243, 1402,
    ] {
        assert_eq!(flags.get(&id), Some(&true), "{id}");
    }
    for id in [
        244, 245, 246, 247, 248, 249, 374, 387, 1304, 1305, 1401, 1403, 1407, 1408, 1409, 1433,
        1434,
    ] {
        assert_ne!(flags.get(&id), Some(&true), "{id}");
    }
    assert_eq!(counters.get(&100), Some(&255));
    assert_eq!(counters.get(&101), Some(&255));
    assert!(!snow.contains(&18));
    assert!(snow.contains(&1085));
}

#[test]
fn the_boss_fight_3_jump_gives_the_spells_and_tutorials_the_story_gave_before_it() {
    let steps = loadout_steps(StoryJumpPoint::AerieBoss3);
    for expected in [
        JumpStep::LearnMagic { id: 3 },
        JumpStep::LearnMagic { id: 6 },
        JumpStep::SetMagicSlot { slot: 0, magic: 6 },
        JumpStep::SetTutorial { id: 2005 },
        JumpStep::SetTutorial { id: 2020 },
        JumpStep::SetTutorial { id: 2079 },
        JumpStep::SetTutorial { id: 2014 },
    ] {
        assert!(steps.contains(&expected), "{expected:?}");
    }
    assert!(!steps.contains(&JumpStep::LearnMagic { id: 5 }));
}

#[test]
fn the_boss_fight_3_jump_lands_at_the_first_start_point_of_its_area() {
    let (name, point) = destination(StoryJumpPoint::AerieBoss3);
    assert_eq!(name, "A_CLIFF_VILLAGE_01");
    assert_eq!(point, 1);
    assert!(PhaseEntry::find(name).is_some());
}

#[test]
fn an_arrival_in_the_same_phase_is_due_once_the_map_was_left_and_has_settled() {
    let settled = LEVEL_CHECK_SETTLE_MS;
    assert!(!arrival_is_due("A", "A", false, true, settled));
    assert!(arrival_is_due("A", "A", true, true, settled));
    assert!(!arrival_is_due("A", "A", true, false, settled));
    assert!(!arrival_is_due("A", "A", true, true, settled - 1));
    assert!(arrival_is_due("A", "B", false, true, settled));
}

fn final_story_state(
    steps: Vec<JumpStep>,
) -> (
    std::collections::HashMap<u16, bool>,
    std::collections::HashMap<u8, u8>,
    std::collections::HashSet<u16>,
) {
    let mut flags = std::collections::HashMap::new();
    let mut counters = std::collections::HashMap::new();
    let mut snow = std::collections::HashSet::new();
    for step in steps {
        match step {
            JumpStep::SetFlag { id } => {
                flags.insert(id, true);
            }
            JumpStep::ClearFlag { id } => {
                flags.insert(id, false);
            }
            JumpStep::SetGameCounter { id, value } => {
                counters.insert(id, value);
            }
            JumpStep::SetSnowFlag { id } => {
                snow.insert(id);
            }
            _ => {}
        }
    }
    (flags, counters, snow)
}

#[test]
fn the_junk_heap_jump_keeps_the_whole_boss_fight_3_loadout_and_ends_with_the_mp_step() {
    let fight = loadout_steps(StoryJumpPoint::AerieBoss3);
    let junk_heap = loadout_steps(StoryJumpPoint::JunkHeapB2);
    let fight_without_mp = &fight[..fight.len() - 1];
    assert_eq!(&junk_heap[..fight_without_mp.len()], fight_without_mp);
    assert_eq!(junk_heap.last(), Some(&JumpStep::MpFullMode { on: false }));
    assert_eq!(
        junk_heap
            .iter()
            .filter(|step| matches!(step, JumpStep::MpFullMode { .. }))
            .count(),
        1
    );
}

#[test]
fn the_junk_heap_jump_leaves_the_story_flags_a_played_run_has_on_floor_b2() {
    let (flags, counters, snow) = final_story_state(loadout_steps(StoryJumpPoint::JunkHeapB2));
    for id in [
        70, 79, 129, 240, 241, 242, 243, 244, 245, 246, 247, 248, 249, 253, 255, 256, 281, 282,
        283, 284, 285, 286, 287, 288, 290, 291, 292, 301, 302,
    ] {
        assert_eq!(flags.get(&id), Some(&true), "{id}");
    }
    for id in [
        1401, 1402, 1403, 1404, 1405, 1406, 1407, 1408, 1409, 1432, 1433, 1434, 1442, 1451, 137,
        252, 1317, 1314,
    ] {
        assert_ne!(flags.get(&id), Some(&true), "{id}");
    }
    assert_eq!(counters.get(&100), Some(&255));
    assert_eq!(counters.get(&101), Some(&255));
    assert!(!snow.contains(&18));
    assert!(snow.contains(&1085));
    assert!(snow.contains(&1051));
    assert!(snow.contains(&5));
}

#[test]
fn the_junk_heap_jump_leaves_out_the_side_quest_flags_and_counters() {
    let steps = loadout_steps(StoryJumpPoint::JunkHeapB2);
    for step in &steps {
        match step {
            JumpStep::SetFlag { id } | JumpStep::ClearFlag { id } => {
                assert!(
                    !matches!(*id, 331 | 332 | 333 | 335) && !(1500..1700).contains(id),
                    "{id}"
                );
            }
            JumpStep::SetGameCounter { id, .. } => assert!(*id >= 100, "{id}"),
            _ => {}
        }
    }
}

#[test]
fn the_junk_heap_jump_gives_the_items_the_story_gave_and_the_upgraded_weapon() {
    let steps = loadout_steps(StoryJumpPoint::JunkHeapB2);
    for expected in [
        JumpStep::SetItem { id: 1009, count: 1 },
        JumpStep::SetItem { id: 1109, count: 1 },
        JumpStep::EquipWeapon { id: 1109 },
        JumpStep::SetItem { id: 515, count: 1 },
        JumpStep::SetItem { id: 531, count: 1 },
        JumpStep::SetItem { id: 715, count: 1 },
        JumpStep::SetItem { id: 716, count: 1 },
        JumpStep::SetItem { id: 250, count: 1 },
        JumpStep::LearnMagic { id: 5 },
    ] {
        assert!(steps.contains(&expected), "{expected:?}");
    }
    for id in [2004, 2021, 2022, 2055] {
        assert!(steps.contains(&JumpStep::SetTutorial { id }), "{id}");
    }
    let weapon_given = steps
        .iter()
        .position(|step| *step == JumpStep::SetItem { id: 1009, count: 1 })
        .expect("weapon given");
    let weapon_upgraded = steps
        .iter()
        .position(|step| *step == JumpStep::SetItem { id: 1109, count: 1 })
        .expect("weapon upgraded");
    assert!(weapon_given < weapon_upgraded);
}

#[test]
fn the_junk_heap_jump_lands_on_the_first_start_point_of_the_second_robot_floor() {
    let (name, point) = destination(StoryJumpPoint::JunkHeapB2);
    assert_eq!(name, "A_MOUNT_ROBOT_02");
    assert_eq!(point, 1);
    assert!(PhaseEntry::find(name).is_some());
}

#[test]
fn the_boss_fight_5_jump_keeps_the_floor_b2_loadout_and_ends_with_the_mp_step() {
    let floor = loadout_steps(StoryJumpPoint::JunkHeapB2);
    let fight = loadout_steps(StoryJumpPoint::JunkHeapBoss5);
    let floor_without_mp = &floor[..floor.len() - 1];
    assert_eq!(&fight[..floor_without_mp.len()], floor_without_mp);
    assert_eq!(fight.last(), Some(&JumpStep::MpFullMode { on: false }));
    assert_eq!(
        fight
            .iter()
            .filter(|step| matches!(step, JumpStep::MpFullMode { .. }))
            .count(),
        1
    );
}

#[test]
fn the_boss_fight_5_jump_leaves_the_flags_and_tutorials_the_bomb_and_truck_stretch_sets() {
    let steps = loadout_steps(StoryJumpPoint::JunkHeapBoss5);
    let (flags, _, snow) = final_story_state(steps.clone());
    for id in [302, 303, 304, 305, 313, 314, 315] {
        assert_eq!(flags.get(&id), Some(&true), "{id}");
    }
    for id in [1401, 1402, 1403, 1404, 1405, 1406, 1407, 1314] {
        assert_ne!(flags.get(&id), Some(&true), "{id}");
    }
    assert!(snow.contains(&5));
    for id in [2050, 2052] {
        assert!(steps.contains(&JumpStep::SetTutorial { id }), "{id}");
    }
    assert!(!steps.contains(&JumpStep::SetTutorial { id: 2019 }));
}

#[test]
fn the_boss_fight_5_jump_arrives_at_the_first_start_point_of_the_last_robot_floor() {
    let (name, point) = destination(StoryJumpPoint::JunkHeapBoss5);
    assert_eq!(name, "A_MOUNT_ROBOT_10");
    assert_eq!(point, 1);
    assert!(PhaseEntry::find(name).is_some());
}

#[test]
fn the_after_boss_fight_6_jump_keeps_the_boss_fight_5_loadout_and_ends_with_the_mp_step() {
    let earlier = loadout_steps(StoryJumpPoint::JunkHeapBoss5);
    let after = loadout_steps(StoryJumpPoint::AerieAfterBoss6);
    let earlier_without_mp = &earlier[..earlier.len() - 1];
    assert_eq!(&after[..earlier_without_mp.len()], earlier_without_mp);
    assert_eq!(after.last(), Some(&JumpStep::MpFullMode { on: false }));
    assert_eq!(
        after
            .iter()
            .filter(|step| matches!(step, JumpStep::MpFullMode { .. }))
            .count(),
        1
    );
}

#[test]
fn the_after_boss_fight_6_jump_leaves_the_flags_a_played_run_has_after_the_ending_scene() {
    let (flags, counters, snow) = final_story_state(loadout_steps(StoryJumpPoint::AerieAfterBoss6));
    for id in [
        102, 306, 307, 308, 309, 132, 310, 311, 374, 375, 376, 377, 378, 379, 380, 381, 382, 383,
        384, 385, 386, 387, 125, 128, 248, 249, 1409,
    ] {
        assert_eq!(flags.get(&id), Some(&true), "{id}");
    }
    for id in [
        1307, 1308, 1309, 1310, 1401, 1402, 1403, 1404, 1405, 1406, 1407, 1408, 1410, 1411,
    ] {
        assert_ne!(flags.get(&id), Some(&true), "{id}");
    }
    assert_eq!(counters.get(&100), Some(&255));
    assert_eq!(counters.get(&101), Some(&255));
    assert!(!snow.contains(&18));
    assert!(snow.contains(&1056));
    assert!(!snow.contains(&1061));
    assert!(!counters.contains_key(&200));
}

#[test]
fn the_after_boss_fight_6_jump_gives_the_spells_and_tutorials_of_both_fight_endings() {
    let steps = loadout_steps(StoryJumpPoint::AerieAfterBoss6);
    for expected in [
        JumpStep::LearnMagic { id: 8 },
        JumpStep::LearnMagic { id: 4 },
        JumpStep::SetTutorial { id: 2007 },
        JumpStep::SetTutorial { id: 2003 },
    ] {
        assert!(steps.contains(&expected), "{expected:?}");
    }
}

#[test]
fn the_after_boss_fight_6_jump_lands_in_the_area_of_the_fight() {
    let (name, point) = destination(StoryJumpPoint::AerieAfterBoss6);
    assert_eq!(name, "A_CLIFF_VILLAGE_01");
    assert_eq!(point, 1);
    assert!(PhaseEntry::find(name).is_some());
}

#[test]
fn the_after_boss_fight_6_jump_places_the_host_where_the_ending_scene_leaves_it_and_the_guest_beside_it()
 {
    let host = arrival_spot(StoryJumpPoint::AerieAfterBoss6, true).expect("host spot");
    let guest = arrival_spot(StoryJumpPoint::AerieAfterBoss6, false).expect("guest spot");
    assert_eq!((host.x, host.y, host.z), (26.25, 30.61, 107.67));
    assert!((guest.x - host.x - 1.5).abs() < 0.001);
    assert_eq!((guest.y, guest.z), (host.y, host.z));
}

#[test]
fn the_boss_fight_6_jump_keeps_the_boss_fight_5_loadout_and_ends_with_the_mp_step() {
    let earlier = loadout_steps(StoryJumpPoint::JunkHeapBoss5);
    let fight = loadout_steps(StoryJumpPoint::AerieBoss6);
    let earlier_without_mp = &earlier[..earlier.len() - 1];
    assert_eq!(&fight[..earlier_without_mp.len()], earlier_without_mp);
    assert_eq!(fight.last(), Some(&JumpStep::MpFullMode { on: false }));
    assert_eq!(
        fight
            .iter()
            .filter(|step| matches!(step, JumpStep::MpFullMode { .. }))
            .count(),
        1
    );
}

#[test]
fn the_after_boss_fight_6_jump_is_the_boss_fight_6_loadout_plus_the_fight() {
    let fight = loadout_steps(StoryJumpPoint::AerieBoss6);
    let after = loadout_steps(StoryJumpPoint::AerieAfterBoss6);
    let fight_without_mp = &fight[..fight.len() - 1];
    assert_eq!(&after[..fight_without_mp.len()], fight_without_mp);
    assert_eq!(after.len(), fight.len() + 50);
}

#[test]
fn the_boss_fight_6_jump_leaves_the_flags_that_make_the_area_start_run_the_approach_to_the_fight() {
    let (flags, counters, snow) = final_story_state(loadout_steps(StoryJumpPoint::AerieBoss6));
    for id in [102, 306, 307, 308, 309, 310, 311, 132, 374, 248, 249] {
        assert_eq!(flags.get(&id), Some(&true), "{id}");
    }
    for id in [
        375, 376, 377, 378, 379, 380, 381, 382, 383, 384, 385, 386, 387, 1307, 1308, 1309, 1310,
        1401, 1402, 1403, 1404, 1405, 1406, 1407, 1408, 1409, 1410, 1411, 125,
    ] {
        assert_ne!(flags.get(&id), Some(&true), "{id}");
    }
    assert_eq!(counters.get(&100), Some(&255));
    assert_eq!(counters.get(&101), Some(&255));
    assert!(!snow.contains(&18));
    assert!(snow.contains(&1056));
    assert!(!counters.contains_key(&200));
}

#[test]
fn the_boss_fight_6_jump_gives_the_spell_of_boss_fight_5_but_not_the_one_after_boss_fight_6() {
    let steps = loadout_steps(StoryJumpPoint::AerieBoss6);
    assert!(steps.contains(&JumpStep::LearnMagic { id: 8 }));
    assert!(steps.contains(&JumpStep::SetTutorial { id: 2007 }));
    assert!(!steps.contains(&JumpStep::LearnMagic { id: 4 }));
    assert!(!steps.contains(&JumpStep::SetTutorial { id: 2003 }));
}

#[test]
fn the_boss_fight_6_jump_enters_the_area_at_the_point_the_field_exit_uses() {
    let (name, point) = destination(StoryJumpPoint::AerieBoss6);
    assert_eq!(name, "A_CLIFF_VILLAGE_01");
    assert_eq!(point, 1);
    assert!(PhaseEntry::find(name).is_some());
}

#[test]
fn the_after_the_facade_jump_is_the_after_boss_fight_6_loadout_plus_the_walk_the_town_and_the_exit_steps()
 {
    let after = loadout_steps(StoryJumpPoint::AerieAfterBoss6);
    let after_facade = loadout_steps(StoryJumpPoint::DesertAfterFacade);
    let after_without_mp = &after[..after.len() - 1];
    assert_eq!(&after_facade[..after_without_mp.len()], after_without_mp);
    assert_eq!(after_facade.len(), after.len() + 12 + 21);
    assert_eq!(
        after_facade.last(),
        Some(&JumpStep::MpFullMode { on: false })
    );
    assert_eq!(
        after_facade
            .iter()
            .filter(|step| matches!(step, JumpStep::MpFullMode { .. }))
            .count(),
        1
    );
}

#[test]
fn the_after_the_facade_jump_leaves_the_flags_the_run_had_when_it_left_the_town() {
    let (flags, counters, snow) =
        final_story_state(loadout_steps(StoryJumpPoint::DesertAfterFacade));
    for id in [
        87, 387, 389, 405, 406, 407, 408, 409, 410, 413, 414, 415, 416, 417, 418, 419, 420, 421,
        422, 431, 432, 433, 1325, 1364,
    ] {
        assert_eq!(flags.get(&id), Some(&true), "{id}");
    }
    for id in [1323, 1401, 1402, 1403, 1409] {
        assert_ne!(flags.get(&id), Some(&true), "{id}");
    }
    assert_eq!(counters.get(&100), Some(&255));
    assert_eq!(counters.get(&101), Some(&255));
    assert!(!snow.contains(&18));
    assert!(!counters.contains_key(&200));
    assert!(snow.contains(&1056));
    assert!(snow.contains(&1084));
}

#[test]
fn the_after_the_facade_jump_enters_the_field_at_the_point_the_run_left_the_town_for() {
    let (name, point) = destination(StoryJumpPoint::DesertAfterFacade);
    assert_eq!(name, "A_DESERT_FIELD_01");
    assert_eq!(point, 3);
    assert!(PhaseEntry::find(name).is_some());
}

#[test]
fn the_after_the_facade_jump_places_nobody() {
    assert!(arrival_spot(StoryJumpPoint::DesertAfterFacade, true).is_none());
    assert!(arrival_spot(StoryJumpPoint::DesertAfterFacade, false).is_none());
}

#[test]
fn the_temple_entrance_jump_is_the_after_the_facade_loadout_plus_the_walk_to_the_temple() {
    let before = loadout_steps(StoryJumpPoint::DesertAfterFacade);
    let temple = loadout_steps(StoryJumpPoint::TempleEntrance);
    assert_eq!(&temple[..before.len() - 1], &before[..before.len() - 1]);
    assert_eq!(temple.len(), before.len() + 6);
    assert_eq!(temple.last(), Some(&JumpStep::MpFullMode { on: false }));
    assert_eq!(
        temple
            .iter()
            .filter(|step| matches!(step, JumpStep::MpFullMode { .. }))
            .count(),
        1
    );
}

#[test]
fn the_temple_entrance_jump_adds_the_flags_the_run_set_before_the_entrance_scene() {
    let (flags, counters, snow) = final_story_state(loadout_steps(StoryJumpPoint::TempleEntrance));
    for id in [457, 458, 459, 460, 1436, 434, 1325, 420, 433] {
        assert_eq!(flags.get(&id), Some(&true), "{id}");
    }
    for id in [89, 435, 1401, 1402, 1403] {
        assert_ne!(flags.get(&id), Some(&true), "{id}");
    }
    assert_eq!(counters.get(&100), Some(&255));
    assert_eq!(counters.get(&101), Some(&255));
    assert!(!snow.contains(&18));
    assert!(snow.contains(&1084));
}

#[test]
fn the_temple_entrance_jump_enters_the_temple_at_point_one_and_places_nobody() {
    let (name, point) = destination(StoryJumpPoint::TempleEntrance);
    assert_eq!(name, "A_DESERT_TEMPLE_01");
    assert_eq!(point, 1);
    assert!(PhaseEntry::find(name).is_some());
    assert!(arrival_spot(StoryJumpPoint::TempleEntrance, true).is_none());
    assert!(arrival_spot(StoryJumpPoint::TempleEntrance, false).is_none());
}

#[test]
fn the_temple_before_boss_jump_is_the_entrance_loadout_plus_the_trial_rooms() {
    let before = loadout_steps(StoryJumpPoint::TempleEntrance);
    let boss = loadout_steps(StoryJumpPoint::TempleBoss7);
    assert_eq!(&boss[..before.len() - 1], &before[..before.len() - 1]);
    assert_eq!(boss.len(), before.len() + 14);
    assert_eq!(boss.last(), Some(&JumpStep::MpFullMode { on: false }));
    assert_eq!(
        boss.iter()
            .filter(|step| matches!(step, JumpStep::MpFullMode { .. }))
            .count(),
        1
    );
}

#[test]
fn the_temple_before_boss_jump_sets_the_room_flags_and_not_the_fight_flags() {
    let (flags, _, _) = final_story_state(loadout_steps(StoryJumpPoint::TempleBoss7));
    for id in [89, 435, 436, 437, 439, 448, 449, 450, 451, 452, 453, 454] {
        assert_eq!(flags.get(&id), Some(&true), "{id}");
    }
    for id in [1325, 1436] {
        assert_eq!(flags.get(&id), Some(&false), "{id}");
    }
    for id in [440, 441, 442, 443, 444, 445, 455] {
        assert_ne!(flags.get(&id), Some(&true), "{id}");
    }
}

#[test]
fn the_temple_before_boss_jump_enters_the_second_temple_area_at_point_one() {
    let (name, point) = destination(StoryJumpPoint::TempleBoss7);
    assert_eq!(name, "A_DESERT_TEMPLE_02");
    assert_eq!(point, 1);
    assert!(PhaseEntry::find(name).is_some());
    assert!(arrival_spot(StoryJumpPoint::TempleBoss7, true).is_none());
    assert!(arrival_spot(StoryJumpPoint::TempleBoss7, false).is_none());
}

#[test]
fn the_village_jump_adds_the_fight_flags_the_last_talk_and_the_walk_home() {
    let steps = loadout_steps(StoryJumpPoint::VillageAfterTemple);
    let (flags, counters, snow) = final_story_state(steps.clone());
    for id in [440, 441, 442, 443, 444, 445, 446, 455, 133, 448, 454] {
        assert_eq!(flags.get(&id), Some(&true), "{id}");
    }
    for id in [1326, 1365, 470, 471] {
        assert_ne!(flags.get(&id), Some(&true), "{id}");
    }
    assert_eq!(counters.get(&100), Some(&3));
    assert_eq!(counters.get(&101), Some(&3));
    assert!(snow.contains(&1041));
    assert!(steps.contains(&JumpStep::SetItem { id: 1006, count: 1 }));
    assert!(steps.contains(&JumpStep::LearnMagic { id: 9 }));
    assert!(steps.contains(&JumpStep::SetTutorial { id: 2008 }));
    assert_eq!(steps.last(), Some(&JumpStep::MpFullMode { on: false }));
}

#[test]
fn the_village_jump_enters_the_village_at_point_two_and_places_nobody() {
    let (name, point) = destination(StoryJumpPoint::VillageAfterTemple);
    assert_eq!(name, "A_CENTER_VILLAGE_01");
    assert_eq!(point, 2);
    assert!(PhaseEntry::find(name).is_some());
    assert!(arrival_spot(StoryJumpPoint::VillageAfterTemple, true).is_none());
    assert!(arrival_spot(StoryJumpPoint::VillageAfterTemple, false).is_none());
}

#[test]
fn the_seafront_jump_carries_the_red_bag_and_the_flags_the_run_set_by_then() {
    let steps = loadout_steps(StoryJumpPoint::SouthPlainsRedBag);
    let (flags, counters, snow) = final_story_state(steps.clone());
    for id in [
        1365, 470, 471, 472, 473, 254, 93, 134, 475, 476, 558, 559, 560, 477, 1366,
    ] {
        assert_eq!(flags.get(&id), Some(&true), "{id}");
    }
    for id in [1355, 1356] {
        assert_ne!(flags.get(&id), Some(&true), "{id}");
    }
    for id in [11, 12, 14, 15, 16, 18, 26] {
        assert!(snow.contains(&id), "{id}");
    }
    for id in [19, 20, 21, 22, 24, 700] {
        assert!(!snow.contains(&id), "{id}");
    }
    assert_eq!(counters.get(&200), Some(&1));
    assert_eq!(counters.get(&100), Some(&3));
    assert_eq!(counters.get(&101), Some(&3));
    assert!(steps.contains(&JumpStep::SetItem { id: 325, count: 1 }));
    assert_eq!(steps.last(), Some(&JumpStep::MpFullMode { on: false }));
}

#[test]
fn the_seafront_jump_enters_the_southern_plains_at_point_three_and_places_nobody() {
    let (name, point) = destination(StoryJumpPoint::SouthPlainsRedBag);
    assert_eq!(name, "A_SOUTH_FIELD_01");
    assert_eq!(point, 3);
    assert!(PhaseEntry::find(name).is_some());
    assert!(arrival_spot(StoryJumpPoint::SouthPlainsRedBag, true).is_none());
    assert!(arrival_spot(StoryJumpPoint::SouthPlainsRedBag, false).is_none());
}

#[test]
fn the_manor_jump_has_given_the_bag_away_and_has_not_played_the_entrance_scene() {
    let steps = loadout_steps(StoryJumpPoint::ManorFront);
    let (flags, counters, snow) = final_story_state(steps.clone());
    for id in [499, 500, 501, 1311, 348, 477] {
        assert_eq!(flags.get(&id), Some(&true), "{id}");
    }
    for id in [502, 513, 131] {
        assert_ne!(flags.get(&id), Some(&true), "{id}");
    }
    for id in [18, 19, 20, 21, 22, 24, 26] {
        assert!(snow.contains(&id), "{id}");
    }
    assert_eq!(counters.get(&100), Some(&3));
    assert_eq!(counters.get(&101), Some(&3));
    let last_bag_step = steps
        .iter()
        .rev()
        .find(|step| matches!(step, JumpStep::SetItem { id: 325, .. }));
    assert_eq!(
        last_bag_step,
        Some(&JumpStep::SetItem { id: 325, count: 0 })
    );
    assert_eq!(steps.last(), Some(&JumpStep::MpFullMode { on: false }));
}

#[test]
fn the_manor_jump_enters_the_southern_plains_at_point_two_and_places_nobody() {
    let (name, point) = destination(StoryJumpPoint::ManorFront);
    assert_eq!(name, "A_SOUTH_FIELD_01");
    assert_eq!(point, 2);
    assert!(PhaseEntry::find(name).is_some());
    assert!(arrival_spot(StoryJumpPoint::ManorFront, true).is_none());
    assert!(arrival_spot(StoryJumpPoint::ManorFront, false).is_none());
}

#[test]
fn the_forest_jump_is_the_village_loadout_plus_the_flags_set_before_entering() {
    let steps = loadout_steps(StoryJumpPoint::ForestArrival);
    let (flags, counters, snow) = final_story_state(steps.clone());
    for id in [1365, 470, 471, 472, 473, 254, 1355, 1356] {
        assert_eq!(flags.get(&id), Some(&true), "{id}");
    }
    for id in [93, 134, 475, 476, 477, 1366] {
        assert_ne!(flags.get(&id), Some(&true), "{id}");
    }
    assert_eq!(counters.get(&200), Some(&1));
    assert_eq!(counters.get(&100), Some(&255));
    assert_eq!(counters.get(&101), Some(&255));
    assert!(!snow.contains(&18));
    assert!(!steps.contains(&JumpStep::SetItem { id: 325, count: 1 }));
    assert_eq!(steps.last(), Some(&JumpStep::MpFullMode { on: false }));
}

#[test]
fn the_forest_jump_enters_the_forest_at_point_one_and_places_nobody() {
    let (name, point) = destination(StoryJumpPoint::ForestArrival);
    assert_eq!(name, "C_FOREST_FIELD_01");
    assert_eq!(point, 1);
    assert!(PhaseEntry::find(name).is_some());
    assert!(arrival_spot(StoryJumpPoint::ForestArrival, true).is_none());
    assert!(arrival_spot(StoryJumpPoint::ForestArrival, false).is_none());
}

#[test]
fn the_emil_jump_is_the_manor_front_plus_the_scene_the_keys_and_emils_flag() {
    let steps = loadout_steps(StoryJumpPoint::ManorEmilFollows);
    let (flags, counters, snow) = final_story_state(steps.clone());
    for id in [
        499, 502, 503, 104, 522, 504, 505, 526, 516, 506, 507, 517, 508, 518, 523, 524, 509, 136,
        515,
    ] {
        assert_eq!(flags.get(&id), Some(&true), "{id}");
    }
    for id in [1311, 510, 511, 512, 513, 131, 106, 1433, 1434, 1435] {
        assert_ne!(flags.get(&id), Some(&true), "{id}");
    }
    assert!(snow.contains(&1050));
    assert_eq!(counters.get(&100), Some(&255));
    assert_eq!(counters.get(&101), Some(&255));
    for id in [717, 256, 257, 258, 259] {
        assert!(steps.contains(&JumpStep::SetItem { id, count: 1 }), "{id}");
    }
    assert_eq!(steps.last(), Some(&JumpStep::MpFullMode { on: false }));
}

#[test]
fn the_emil_jump_enters_the_manor_at_point_one_and_places_nobody() {
    let (name, point) = destination(StoryJumpPoint::ManorEmilFollows);
    assert_eq!(name, "A_SOUTH_MANSION_01");
    assert_eq!(point, 1);
    assert!(PhaseEntry::find(name).is_some());
    assert!(arrival_spot(StoryJumpPoint::ManorEmilFollows, true).is_none());
    assert!(arrival_spot(StoryJumpPoint::ManorEmilFollows, false).is_none());
}

#[test]
fn the_boss_fight_8_jump_keeps_emil_and_leaves_the_fight_flags_unset() {
    let steps = loadout_steps(StoryJumpPoint::ManorBoss8);
    let (flags, _, _) = final_story_state(steps.clone());
    for id in [509, 136, 515, 1433, 1434, 1435] {
        assert_eq!(flags.get(&id), Some(&true), "{id}");
    }
    for id in [510, 511, 512, 106, 131] {
        assert_ne!(flags.get(&id), Some(&true), "{id}");
    }
    assert_eq!(steps.last(), Some(&JumpStep::MpFullMode { on: false }));
}

#[test]
fn the_boss_fight_8_jump_enters_the_second_manor_area_at_point_one_and_places_nobody() {
    let (name, point) = destination(StoryJumpPoint::ManorBoss8);
    assert_eq!(name, "A_SOUTH_MANSION_02");
    assert_eq!(point, 1);
    assert!(PhaseEntry::find(name).is_some());
    assert!(arrival_spot(StoryJumpPoint::ManorBoss8, true).is_none());
    assert!(arrival_spot(StoryJumpPoint::ManorBoss8, false).is_none());
}

#[test]
fn the_boss_fight_9_jump_is_the_village_with_the_manor_fight_done_and_the_cutscene_flag_on() {
    let steps = loadout_steps(StoryJumpPoint::VillageBoss9);
    let (flags, counters, snow) = final_story_state(steps.clone());
    for id in [
        509, 510, 511, 512, 513, 131, 106, 533, 534, 535, 536, 537, 538, 539, 540, 1367,
    ] {
        assert_eq!(flags.get(&id), Some(&true), "{id}");
    }
    for id in [136, 1433, 1434, 1435, 541, 542, 543, 1327, 1328] {
        assert_ne!(flags.get(&id), Some(&true), "{id}");
    }
    assert!(snow.contains(&1065));
    assert_eq!(counters.get(&100), Some(&3));
    assert_eq!(counters.get(&101), Some(&3));
    assert!(steps.contains(&JumpStep::SetItem { id: 267, count: 1 }));
    assert_eq!(steps.last(), Some(&JumpStep::MpFullMode { on: false }));
}

#[test]
fn the_boss_fight_9_jump_lands_inside_no_host_led_stretch() {
    let (flags, _, _) = final_story_state(loadout_steps(StoryJumpPoint::VillageBoss9));
    let flag_on = |id: u16| flags.get(&id) == Some(&true);
    assert!(stretch_the_story_is_inside("A_CENTER_VILLAGE_01", flag_on).is_none());
    assert!(stretch_the_story_is_inside("A_SOUTH_MANSION_02", flag_on).is_none());
}

#[test]
fn the_boss_fight_9_jump_enters_the_village_at_point_four_and_places_nobody() {
    let (name, point) = destination(StoryJumpPoint::VillageBoss9);
    assert_eq!(name, "A_CENTER_VILLAGE_01");
    assert_eq!(point, 4);
    assert!(PhaseEntry::find(name).is_some());
    assert!(arrival_spot(StoryJumpPoint::VillageBoss9, true).is_none());
    assert!(arrival_spot(StoryJumpPoint::VillageBoss9, false).is_none());
}

#[test]
fn the_boss_fight_10_jump_is_the_library_with_the_village_chain_done_and_no_fight_flag_on() {
    let steps = loadout_steps(StoryJumpPoint::LibraryBoss10);
    let (flags, _, _) = final_story_state(steps.clone());
    for id in [533, 540, 541, 542, 543, 544, 545, 546, 547, 548, 549] {
        assert_eq!(flags.get(&id), Some(&true), "{id}");
    }
    for id in [550, 551, 552, 553, 554, 555, 556, 1298, 1330] {
        assert_ne!(flags.get(&id), Some(&true), "{id}");
    }
    for id in [7, 10] {
        assert!(steps.contains(&JumpStep::LearnMagic { id }), "{id}");
    }
    assert_eq!(steps.last(), Some(&JumpStep::MpFullMode { on: false }));
}

#[test]
fn the_boss_fight_10_jump_enters_the_library_at_point_one_and_places_nobody() {
    let (name, point) = destination(StoryJumpPoint::LibraryBoss10);
    assert_eq!(name, "A_CENTER_LIBRARY_01");
    assert_eq!(point, 1);
    assert!(PhaseEntry::find(name).is_some());
    assert!(arrival_spot(StoryJumpPoint::LibraryBoss10, true).is_none());
    assert!(arrival_spot(StoryJumpPoint::LibraryBoss10, false).is_none());
}

#[test]
fn the_boss_fight_11_jump_has_the_first_library_fight_won_and_the_intro_chain_left_to_the_stage_start()
 {
    let steps = loadout_steps(StoryJumpPoint::LibraryBoss11);
    let (flags, _, _) = final_story_state(steps.clone());
    for id in [541, 549, 550, 551, 552] {
        assert_eq!(flags.get(&id), Some(&true), "{id}");
    }
    for id in [553, 554, 555, 556] {
        assert_ne!(flags.get(&id), Some(&true), "{id}");
    }
    assert_eq!(steps.last(), Some(&JumpStep::MpFullMode { on: false }));
}

#[test]
fn the_boss_fight_11_jump_enters_the_library_at_point_one_and_places_nobody() {
    let (name, point) = destination(StoryJumpPoint::LibraryBoss11);
    assert_eq!(name, "A_CENTER_LIBRARY_01");
    assert_eq!(point, 1);
    assert!(arrival_spot(StoryJumpPoint::LibraryBoss11, true).is_none());
    assert!(arrival_spot(StoryJumpPoint::LibraryBoss11, false).is_none());
}

#[test]
fn the_part_2_start_jump_has_every_story_change_the_l39_host_sent_from_the_end_of_fight_11_to_the_first_scene_end()
 {
    let (flags, counters, snow) = final_story_state(loadout_steps(StoryJumpPoint::Part2Start));
    let flag_on = |id: u16| flags.get(&id) == Some(&true);
    for id in [541, 549, 550, 551, 552, 553, 554, 555, 556, 72, 626, 627] {
        assert!(flag_on(id), "{id}");
    }
    assert_eq!(flags.get(&125), Some(&false));
    assert_eq!(counters.get(&71), Some(&13));
    assert_eq!(counters.get(&74), Some(&7));
    assert!(snow.contains(&1071));
}

#[test]
fn the_part_2_start_jump_leaves_out_what_the_village_scripts_write_after_the_first_scene() {
    let (flags, _, _) = final_story_state(loadout_steps(StoryJumpPoint::Part2Start));
    for id in [628, 630, 840, 1346, 845] {
        assert_ne!(flags.get(&id), Some(&true), "{id}");
    }
}

#[test]
fn the_part_2_start_jump_ends_with_the_mp_step_and_changes_nothing_of_the_boss_fight_11_jump_before_it()
 {
    let before = loadout_steps(StoryJumpPoint::LibraryBoss11);
    let steps = loadout_steps(StoryJumpPoint::Part2Start);
    assert_eq!(steps.last(), Some(&JumpStep::MpFullMode { on: false }));
    assert_eq!(steps[..before.len() - 1], before[..before.len() - 1]);
}

#[test]
fn the_part_2_start_jump_enters_the_part_2_village_at_point_one_and_places_nobody() {
    let (name, point) = destination(StoryJumpPoint::Part2Start);
    assert_eq!(name, "B_CENTER_VILLAGE_01");
    assert_eq!(point, 1);
    assert!(PhaseEntry::find(name).is_some());
    assert!(arrival_spot(StoryJumpPoint::Part2Start, true).is_none());
    assert!(arrival_spot(StoryJumpPoint::Part2Start, false).is_none());
}

#[test]
fn the_part_2_start_jump_lands_inside_no_host_led_stretch() {
    let (flags, _, _) = final_story_state(loadout_steps(StoryJumpPoint::Part2Start));
    let flag_on = |id: u16| flags.get(&id) == Some(&true);
    assert!(stretch_the_story_is_inside("B_CENTER_VILLAGE_01", flag_on).is_none());
}

#[test]
fn the_library_jumps_land_inside_no_host_led_stretch() {
    for point in [StoryJumpPoint::LibraryBoss10, StoryJumpPoint::LibraryBoss11] {
        let (flags, _, _) = final_story_state(loadout_steps(point));
        let flag_on = |id: u16| flags.get(&id) == Some(&true);
        assert!(stretch_the_story_is_inside("A_CENTER_LIBRARY_01", flag_on).is_none());
    }
}

#[test]
fn a_guest_already_in_the_hosts_jump_phase_reloads_it_in_place_as_the_host_did() {
    let (manor, _) = destination(StoryJumpPoint::ManorEmilFollows);
    assert!(
        guest_reloads_in_place(Some(manor), manor),
        "run 1115j 01:54:06: the host reloaded its phase in place and its stage start placed the new companion; the guest applied the flags and kept its old actors"
    );
    let (field, _) = destination(StoryJumpPoint::ManorFront);
    assert!(!guest_reloads_in_place(Some(field), manor));
    assert!(!guest_reloads_in_place(None, manor));
}

#[test]
fn the_restore_point_is_made_after_the_placement_read_back_and_the_level_check() {
    let (from, to) = ("A_TOKYO_FIELD_01", "A_CENTER_VILLAGE_01");
    let placed_at = LEVEL_CHECK_SETTLE_MS;
    assert!(arrival_is_due(from, to, true, true, placed_at));
    assert!(level_check_is_due(from, to, true, placed_at));
    assert!(!restore_point_is_due(from, to, true, true, placed_at));
    let read_back_at = placed_at + PLACEMENT_READ_BACK_AFTER_MS;
    assert!(RESTORE_POINT_WAITS_AFTER_PHASE_CHANGE_MS >= read_back_at);
    assert!(!restore_point_is_due(
        from,
        to,
        true,
        true,
        RESTORE_POINT_WAITS_AFTER_PHASE_CHANGE_MS - 1
    ));
    assert!(restore_point_is_due(
        from,
        to,
        true,
        true,
        RESTORE_POINT_WAITS_AFTER_PHASE_CHANGE_MS
    ));
}

#[test]
fn the_restore_point_waits_for_the_destination_to_load() {
    let late = RESTORE_POINT_WAITS_AFTER_PHASE_CHANGE_MS * 10;
    let here = "A_CENTER_VILLAGE_01";
    assert!(!restore_point_is_due(here, here, false, true, late));
    assert!(!restore_point_is_due(here, "B_TEST", true, false, late));
    assert!(restore_point_is_due(here, here, true, true, late));
}

#[test]
fn a_jump_from_the_prologue_resets_its_forced_level_before_raising() {
    for phase in PROLOGUE_PHASES {
        assert!(jump_resets_prologue_level(phase));
    }
    assert_eq!(PROLOGUE_END_PLAYER_LEVEL, 0);
    let target = jump_stat_level(StoryJumpPoint::VillageAfterPrologue);
    assert_eq!(levels_to_raise(PROLOGUE_END_PLAYER_LEVEL, target), target);
}

#[test]
fn a_jump_outside_the_prologue_never_resets_the_level() {
    let (village, _) = destination(StoryJumpPoint::VillageAfterPrologue);
    assert!(!jump_resets_prologue_level(village));
    assert!(!jump_resets_prologue_level(""));
}

#[test]
fn the_after_the_facade_jump_gives_one_shared_item_and_no_key_or_story_items() {
    assert_eq!(
        shared_items_the_jump_gives(StoryJumpPoint::DesertAfterFacade),
        vec![(250, 1)]
    );
    assert!(shared_items_the_jump_gives(StoryJumpPoint::VillageAfterPrologue).is_empty());
}

#[test]
fn a_shared_item_the_load_took_from_this_game_is_found_missing() {
    let wanted = shared_items_the_jump_gives(StoryJumpPoint::DesertAfterFacade);
    let lost = item_shortfalls(&wanted, |_| Some(0));
    assert_eq!(lost.len(), 1);
    assert_eq!(lost[0].item_id, 250);
    assert_eq!(lost[0].missing(), 1);
}

#[test]
fn a_shared_item_still_held_or_unreadable_is_left_alone() {
    let wanted = shared_items_the_jump_gives(StoryJumpPoint::DesertAfterFacade);
    assert!(item_shortfalls(&wanted, |_| Some(1)).is_empty());
    assert!(item_shortfalls(&wanted, |_| Some(5)).is_empty());
    assert!(item_shortfalls(&wanted, |_| None).is_empty());
}

#[test]
fn a_later_set_to_zero_removes_the_item_from_the_wanted_list() {
    let wanted = shared_items_the_jump_gives(StoryJumpPoint::VillageBoss9);
    for step in loadout_steps(StoryJumpPoint::VillageBoss9) {
        if let JumpStep::SetItem { id, count: 0 } = step {
            assert!(wanted.iter().all(|(held, _)| i32::from(*held) != id));
        }
    }
}

const ENTRY_SCRIPT_MOVE_UNITS: f32 = 4.5;

#[test]
fn a_placement_the_entry_script_moved_waits_for_control_then_is_placed_again() {
    let left = PLACEMENTS_REPEATED_AFTER_AN_ENTRY_SCRIPT;
    assert_eq!(
        read_back_step(
            ENTRY_SCRIPT_MOVE_UNITS,
            false,
            left,
            PLACEMENT_READ_BACK_AFTER_MS
        ),
        ReadBackStep::WaitForControl
    );
    assert_eq!(
        read_back_step(ENTRY_SCRIPT_MOVE_UNITS, true, left, 9_000),
        ReadBackStep::PlaceAgain
    );
}

#[test]
fn a_placement_within_reach_arrives_at_once_even_while_a_script_runs() {
    let left = PLACEMENTS_REPEATED_AFTER_AN_ENTRY_SCRIPT;
    assert_eq!(
        read_back_step(0.4, false, left, PLACEMENT_READ_BACK_AFTER_MS),
        ReadBackStep::Arrived
    );
}

#[test]
fn a_placement_still_off_after_the_repeat_is_reported_as_missed() {
    assert_eq!(
        read_back_step(ENTRY_SCRIPT_MOVE_UNITS, true, 0, 2_000),
        ReadBackStep::DidNotArrive
    );
    assert_eq!(
        read_back_step(
            ENTRY_SCRIPT_MOVE_UNITS,
            false,
            PLACEMENTS_REPEATED_AFTER_AN_ENTRY_SCRIPT,
            ENTRY_SCRIPT_WAIT_GIVE_UP_MS + 1
        ),
        ReadBackStep::DidNotArrive
    );
}

#[test]
fn an_item_whose_count_cannot_be_read_is_reported_instead_of_skipped() {
    let wanted = [(250, 1), (251, 1)];
    let unreadable = unreadable_items(&wanted, |id| (id == 251).then_some(1));
    assert_eq!(unreadable, vec![250]);
    assert!(unreadable_items(&wanted, |_| Some(0)).is_empty());
}

#[test]
fn the_items_are_checked_again_a_while_after_the_restore_point() {
    assert!(!item_audit_is_due(ITEM_AUDIT_AFTER_RESTORE_POINT_MS - 1));
    assert!(item_audit_is_due(ITEM_AUDIT_AFTER_RESTORE_POINT_MS));
}
