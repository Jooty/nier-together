use automata_game::address::PHASE_EVENT_ACTION_UPDATE_FUNCTION;
use automata_game::exe::ExeVariant;
use automata_game::free_roam_scenes::{
    DENIED_SCENE_ACTION_IDS, FREE_ROAM_NEEDED_SCENE_ACTION_IDS, HAP_STEP_RESULT_OFFSET,
    HeldActions, PHASE_EVENT_ACTION_IDLE_STATE, PHASE_EVENT_ACTION_LOAD_REFUSED_STATE,
    PHASE_EVENT_ACTION_LOADED_STATE, PHASE_EVENT_ACTION_LOADING_STATE,
    PHASE_EVENT_ACTION_PRE_START_STATES, UpdateHold, hap_step_result_for_held_update,
    held_action_line, scene_is_denied, scene_is_free_roam_needed, update_hold,
};

const FADE_STATE: u32 = 0xB572_274C;
const START_WAIT_STATE: u32 = 0xDFC1_471F;
const STARTING_STATE: u32 = 0xA09A_3421;
const FINISHING_STATE: u32 = 0xECBC_CDF8;
const DONE_STATE: u32 = 0x26B8_0D1F;
const NOT_ASKED: u8 = 0;
const ASKED: u8 = 1;
const BLOCKING_FLAGS: u32 = 0x418;
const NON_BLOCKING_FLAGS: u32 = 0x10;
const BLOCKING_WITHOUT_RESULT_FLAGS: u32 = 0x0080_0008;

const FIRST_AREA_SCENE: u32 = 0x3F27_D712;
const CHAINED_AFTER_FIRST_AREA_SCENE: u32 = 0xAE34_28D5;
const LANDING_SCENE: u32 = 0xCEDF_793A;

const SCENES_FIRST_HANDED_BACK_FOR_A_HAP_THAT_RUNS_BEFORE_THEM: [u32; 49] = [
    0x0FC2DB38, 0x127EF5ED, 0x13BA6C0A, 0x15973503, 0x16002123, 0x16D730BD, 0x187B1028, 0x1F4627D0,
    0x2A186CB8, 0x2DA0A3B5, 0x2DF117FE, 0x2ECE0D3B, 0x33492B64, 0x35FAD482, 0x3D1B8F16, 0x40F306C0,
    0x46C80158, 0x51AFCF84, 0x55AB8FD0, 0x5A3386B6, 0x5A798BB0, 0x5A9C11C7, 0x61CC136C, 0x6CAF0544,
    0x79A557DF, 0x7A648611, 0x7DC0F895, 0x885A6D15, 0x91CAC295, 0x96C0A342, 0xA04F7AB5, 0xA5C9CFFF,
    0xA9703D94, 0xAEC4696B, 0xB1003A4E, 0xB34E2FE6, 0xB38C48D4, 0xB58233A5, 0xB8A361ED, 0xC205C81C,
    0xCD0DC410, 0xD63C740D, 0xD9279D5B, 0xEF5A44E2, 0xF03C3B85, 0xF695F59A, 0xF8FDC3A8, 0xF98DF8F5,
    0xF9DBA7C9,
];

const CHAINED_PAIRS: [(u32, u32); 6] = [
    (0x3F27_D712, 0xAE34_28D5),
    (0x8290_DF68, 0xF4BE_212E),
    (0x3F39_0565, 0x5501_8D06),
    (0x3C90_1B05, 0x5C74_5714),
    (0xD542_69FF, 0x7815_9020),
    (0x50CC_A8FC, 0x1400_6B34),
];

const STATES_AFTER_THE_SCENE_BEGINS: [u32; 5] = [
    FADE_STATE,
    START_WAIT_STATE,
    STARTING_STATE,
    FINISHING_STATE,
    DONE_STATE,
];

fn is_sorted_and_unique(ids: &[u32]) -> bool {
    ids.windows(2).all(|pair| pair[0] < pair[1])
}

#[test]
fn the_id_tables_are_sorted_and_have_no_repeats() {
    assert!(is_sorted_and_unique(&DENIED_SCENE_ACTION_IDS));
    assert!(is_sorted_and_unique(&FREE_ROAM_NEEDED_SCENE_ACTION_IDS));
}

#[test]
fn no_scene_is_both_denied_and_kept() {
    for id in DENIED_SCENE_ACTION_IDS {
        assert!(!scene_is_free_roam_needed(id), "{id:#X}");
    }
}

#[test]
fn every_denied_scene_is_held_in_every_state_before_it_begins() {
    for id in DENIED_SCENE_ACTION_IDS {
        for state in PHASE_EVENT_ACTION_PRE_START_STATES {
            for asked in [NOT_ASKED, ASKED] {
                assert_eq!(
                    update_hold(true, id, state, asked),
                    Some(UpdateHold::DeniedScene),
                    "{id:#X} in {state:#X} asked {asked}"
                );
            }
        }
    }
}

#[test]
fn a_denied_scene_is_still_held_when_a_script_or_command_loaded_it_first() {
    assert_eq!(
        update_hold(
            true,
            FIRST_AREA_SCENE,
            PHASE_EVENT_ACTION_LOADING_STATE,
            ASKED
        ),
        Some(UpdateHold::DeniedScene)
    );
    assert_eq!(
        update_hold(
            true,
            FIRST_AREA_SCENE,
            PHASE_EVENT_ACTION_LOADED_STATE,
            ASKED
        ),
        Some(UpdateHold::DeniedScene)
    );
    assert_eq!(
        update_hold(
            true,
            FIRST_AREA_SCENE,
            PHASE_EVENT_ACTION_LOAD_REFUSED_STATE,
            ASKED
        ),
        Some(UpdateHold::DeniedScene)
    );
}

#[test]
fn a_scene_that_already_began_is_never_stopped_halfway() {
    for id in DENIED_SCENE_ACTION_IDS {
        for state in STATES_AFTER_THE_SCENE_BEGINS {
            for asked in [NOT_ASKED, ASKED] {
                assert_eq!(update_hold(true, id, state, asked), None);
            }
        }
    }
}

#[test]
fn the_campaign_holds_nothing_for_any_listed_scene() {
    for id in DENIED_SCENE_ACTION_IDS
        .into_iter()
        .chain(FREE_ROAM_NEEDED_SCENE_ACTION_IDS)
    {
        for state in PHASE_EVENT_ACTION_PRE_START_STATES
            .into_iter()
            .chain(STATES_AFTER_THE_SCENE_BEGINS)
        {
            for asked in [NOT_ASKED, ASKED] {
                assert_eq!(update_hold(false, id, state, asked), None);
            }
        }
    }
}

#[test]
fn the_scenes_free_roam_needs_to_load_in_are_never_denied() {
    for id in FREE_ROAM_NEEDED_SCENE_ACTION_IDS {
        assert!(!scene_is_denied(id));
        for state in PHASE_EVENT_ACTION_PRE_START_STATES {
            assert_ne!(
                update_hold(true, id, state, ASKED),
                Some(UpdateHold::DeniedScene)
            );
        }
        assert_eq!(
            update_hold(true, id, PHASE_EVENT_ACTION_IDLE_STATE, ASKED),
            None
        );
        assert_eq!(
            update_hold(true, id, PHASE_EVENT_ACTION_LOADING_STATE, NOT_ASKED),
            None
        );
    }
    assert!(scene_is_free_roam_needed(LANDING_SCENE));
}

#[test]
fn an_unlisted_action_waiting_for_its_area_is_still_held_as_an_area_start() {
    let unlisted = 0x1234_5678;
    assert!(!scene_is_denied(unlisted));
    assert_eq!(
        update_hold(true, unlisted, PHASE_EVENT_ACTION_IDLE_STATE, NOT_ASKED),
        Some(UpdateHold::AreaStart)
    );
    assert_eq!(
        update_hold(true, unlisted, PHASE_EVENT_ACTION_IDLE_STATE, ASKED),
        None
    );
}

#[test]
fn a_chain_is_denied_from_its_root_to_every_follower() {
    for (root, follower) in CHAINED_PAIRS {
        assert!(scene_is_denied(root), "{root:#X}");
        assert!(scene_is_denied(follower), "{follower:#X}");
    }
    assert!(scene_is_denied(FIRST_AREA_SCENE));
    assert!(scene_is_denied(CHAINED_AFTER_FIRST_AREA_SCENE));
}

#[test]
fn a_chain_follower_cannot_be_pushed_to_play_after_a_denied_root() {
    assert_eq!(
        update_hold(
            true,
            CHAINED_AFTER_FIRST_AREA_SCENE,
            PHASE_EVENT_ACTION_LOADING_STATE,
            NOT_ASKED
        ),
        Some(UpdateHold::DeniedScene)
    );
}

#[test]
fn a_held_blocking_action_reports_still_running_to_the_hap() {
    assert_eq!(hap_step_result_for_held_update(BLOCKING_FLAGS), Some(1));
}

#[test]
fn a_held_action_that_does_not_block_leaves_the_hap_step_result_alone() {
    assert_eq!(hap_step_result_for_held_update(NON_BLOCKING_FLAGS), None);
    assert_eq!(
        hap_step_result_for_held_update(BLOCKING_WITHOUT_RESULT_FLAGS),
        None
    );
}

#[test]
fn the_hap_step_result_is_the_high_dword_of_the_step_context_pair() {
    assert_eq!(HAP_STEP_RESULT_OFFSET, 0xC);
}

#[test]
fn the_denied_line_names_the_action_and_says_it_never_starts() {
    let line = held_action_line(
        UpdateHold::DeniedScene,
        FIRST_AREA_SCENE,
        PHASE_EVENT_ACTION_IDLE_STATE,
        NOT_ASKED,
        1,
    );
    assert!(line.contains("0x3F27D712"));
    assert!(line.contains("never starts"));
}

#[test]
fn each_denied_scene_is_logged_once() {
    let mut held = HeldActions::new();
    let first_lines = DENIED_SCENE_ACTION_IDS
        .iter()
        .filter(|id| held.is_first_hold_of(**id))
        .count();
    assert_eq!(first_lines, DENIED_SCENE_ACTION_IDS.len());
    assert!(
        DENIED_SCENE_ACTION_IDS
            .iter()
            .all(|id| !held.is_first_hold_of(*id))
    );
}

#[test]
fn the_update_the_hold_hooks_is_the_noted_one_in_both_exe_variants() {
    assert_eq!(
        PHASE_EVENT_ACTION_UPDATE_FUNCTION.rva(ExeVariant::Main),
        0x3A_3C10
    );
    assert_eq!(
        PHASE_EVENT_ACTION_UPDATE_FUNCTION.rva(ExeVariant::Compat),
        0x39_DDE0
    );
}

#[test]
fn each_of_the_forty_nine_scenes_first_handed_back_is_now_denied_in_every_pre_start_state() {
    for id in SCENES_FIRST_HANDED_BACK_FOR_A_HAP_THAT_RUNS_BEFORE_THEM {
        assert!(scene_is_denied(id), "{id:#X}");
        for state in PHASE_EVENT_ACTION_PRE_START_STATES {
            assert_eq!(
                update_hold(true, id, state, ASKED),
                Some(UpdateHold::DeniedScene),
                "{id:#X} in {state:#X}"
            );
            assert_eq!(update_hold(false, id, state, ASKED), None, "{id:#X}");
        }
    }
}

#[test]
fn the_denied_table_is_the_eighty_three_plus_the_forty_nine() {
    assert_eq!(DENIED_SCENE_ACTION_IDS.len(), 83 + 49);
}
