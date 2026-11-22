use automata_game::exe::ExeVariant;
use automata_game::free_roam_scenes::{
    HAP_STEP_FUNCTION, HAP_STEP_START, HAP_WALK_CAPS, HapCapCache, capped_hap_line, hap_walk_cap,
};

const GATE_WITH_SIDE_EFFECTS_BEHIND_IT: u32 = 0xF46E_AF71;
const TRANSFER_BAN_HAP_FIRST_ACTION: u32 = 0x85D0_6139;
const ENDING_JUMP_SCENE: u32 = 0x51AF_CF84;
const GATE_INDEX_OF_THAT_HAP: u32 = 0;
const SCRIPT_GATE_OF_THE_SECOND_SCRIPT_PLAY_HAP: u32 = 0xF0B3_105C;
const SCRIPT_GATE_INDEX: u32 = 3;

fn hap_with_actions(ids: &[u32]) -> impl FnMut(u32) -> Option<u32> + '_ {
    |index| ids.get(index as usize).copied()
}

#[test]
fn the_caps_are_unique_by_gate_action() {
    let mut ids: Vec<u32> = HAP_WALK_CAPS.iter().map(|cap| cap.action_id).collect();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), HAP_WALK_CAPS.len());
}

#[test]
fn a_hap_with_a_known_gate_in_its_list_is_capped_at_the_gate_index() {
    let mut ids = vec![0x1111_1111; 29];
    ids[GATE_INDEX_OF_THAT_HAP as usize] = GATE_WITH_SIDE_EFFECTS_BEHIND_IT;
    let cap = hap_walk_cap(true, 29, 0, hap_with_actions(&ids)).expect("capped");
    assert_eq!(cap.index, GATE_INDEX_OF_THAT_HAP);
    assert_eq!(cap.action_id, GATE_WITH_SIDE_EFFECTS_BEHIND_IT);
}

#[test]
fn the_script_haps_are_capped_at_their_script() {
    let mut ids = vec![0x2222_2222; 17];
    ids[SCRIPT_GATE_INDEX as usize] = SCRIPT_GATE_OF_THE_SECOND_SCRIPT_PLAY_HAP;
    let cap = hap_walk_cap(true, 17, 2, hap_with_actions(&ids)).expect("capped");
    assert_eq!(cap.index, SCRIPT_GATE_INDEX);
}

#[test]
fn every_listed_gate_caps_a_hap_that_has_it_at_its_index_and_nothing_else_does() {
    for cap in HAP_WALK_CAPS {
        let mut ids = vec![0x3333_3333; cap.index as usize + 5];
        ids[cap.index as usize] = cap.action_id;
        let found = hap_walk_cap(true, ids.len() as u32, 0, hap_with_actions(&ids));
        assert_eq!(found, Some(cap), "{:#X}", cap.action_id);
        let other = vec![0x3333_3333; cap.index as usize + 5];
        assert_eq!(
            hap_walk_cap(true, other.len() as u32, 0, hap_with_actions(&other)),
            None
        );
    }
}

#[test]
fn the_campaign_never_caps_a_hap() {
    for cap in HAP_WALK_CAPS {
        let mut ids = vec![0x3333_3333; cap.index as usize + 5];
        ids[cap.index as usize] = cap.action_id;
        assert_eq!(
            hap_walk_cap(false, ids.len() as u32, 0, hap_with_actions(&ids)),
            None
        );
    }
}

#[test]
fn a_hap_that_already_walked_past_the_gate_is_left_alone() {
    let mut ids = vec![0x4444_4444; 10];
    ids[3] = 0xF0B3_105C;
    assert_eq!(hap_walk_cap(true, 10, 4, hap_with_actions(&ids)), None);
}

#[test]
fn a_gate_index_past_the_action_count_is_never_read() {
    let ids = [0x5555_5555; 2];
    let mut reads = 0;
    let found = hap_walk_cap(true, 2, 0, |index| {
        reads += 1;
        assert!(index < 2);
        ids.get(index as usize).copied()
    });
    assert_eq!(found, None);
    assert!(reads > 0);
}

#[test]
fn the_hap_step_is_the_noted_one_in_both_exe_variants() {
    assert_eq!(HAP_STEP_FUNCTION.rva(ExeVariant::Main), 0x3E_F570);
    assert_eq!(HAP_STEP_FUNCTION.rva(ExeVariant::Compat), 0x3E_98B0);
    assert_eq!(HAP_STEP_START.len(), 20);
}

#[test]
fn the_cap_line_names_the_gate_and_says_nothing_behind_it_runs() {
    let line = capped_hap_line(0xF46E_AF71, 0, 29, 1);
    assert!(line.contains("0xF46EAF71"));
    assert!(line.contains("nothing behind the gate runs"));
}

fn counted<'a>(
    ids: &'a [u32],
    reads: &'a std::cell::Cell<u32>,
) -> impl FnMut(u32) -> Option<u32> + 'a {
    move |index| {
        reads.set(reads.get() + 1);
        ids.get(index as usize).copied()
    }
}

#[test]
fn the_hap_that_sets_the_transfer_ban_is_capped_before_the_set() {
    let cap = HAP_WALK_CAPS
        .iter()
        .find(|cap| cap.action_id == TRANSFER_BAN_HAP_FIRST_ACTION)
        .expect("listed");
    assert_eq!(cap.index, 0);
}

#[test]
fn the_ending_jump_hap_is_capped_at_its_scene_before_the_jumps() {
    let cap = HAP_WALK_CAPS
        .iter()
        .find(|cap| cap.action_id == ENDING_JUMP_SCENE)
        .expect("listed");
    assert_eq!(cap.index, 5);
}

#[test]
fn an_uncapped_hap_costs_at_most_one_id_read_per_listed_gate_once_and_none_after() {
    let ids = [0x6666_6666_u32; 64];
    let reads = std::cell::Cell::new(0);
    let mut cache = HapCapCache::new();
    assert_eq!(
        cache.cap_for(0x1000, 0x2000, 64, 0, counted(&ids, &reads)),
        None
    );
    assert!(reads.get() as usize <= HAP_WALK_CAPS.len());
    let after_first = reads.get();
    for _ in 0..100 {
        assert_eq!(
            cache.cap_for(0x1000, 0x2000, 64, 7, counted(&ids, &reads)),
            None
        );
    }
    assert_eq!(reads.get(), after_first);
}

#[test]
fn a_small_hap_reads_only_the_slots_a_gate_could_be_in() {
    let ids = [0x6666_6666_u32; 2];
    let reads = std::cell::Cell::new(0);
    let mut cache = HapCapCache::new();
    cache.cap_for(1, 2, 2, 0, counted(&ids, &reads));
    assert!(reads.get() <= 2 * HAP_WALK_CAPS.len() as u32);
    let empty: [u32; 0] = [];
    let none = std::cell::Cell::new(0);
    cache.cap_for(3, 4, 0, 0, counted(&empty, &none));
    assert_eq!(none.get(), 0);
}

#[test]
fn a_capped_hap_is_judged_once_and_then_costs_no_id_reads() {
    let mut ids = vec![0x7777_7777_u32; 29];
    ids[0] = GATE_WITH_SIDE_EFFECTS_BEHIND_IT;
    let reads = std::cell::Cell::new(0);
    let mut cache = HapCapCache::new();
    let first = cache.cap_for(0x10, 0x20, 29, 0, counted(&ids, &reads));
    assert_eq!(first.map(|cap| cap.index), Some(0));
    let after_first = reads.get();
    assert!(after_first as usize <= HAP_WALK_CAPS.len());
    for _ in 0..100 {
        assert!(
            cache
                .cap_for(0x10, 0x20, 29, 0, counted(&ids, &reads))
                .is_some()
        );
    }
    assert_eq!(reads.get(), after_first);
}

#[test]
fn a_reused_address_with_another_list_or_count_is_judged_again() {
    let mut ids = vec![0x7777_7777_u32; 29];
    ids[0] = GATE_WITH_SIDE_EFFECTS_BEHIND_IT;
    let plain = vec![0x7777_7777_u32; 29];
    let reads = std::cell::Cell::new(0);
    let mut cache = HapCapCache::new();
    assert!(
        cache
            .cap_for(0x10, 0x20, 29, 0, counted(&ids, &reads))
            .is_some()
    );
    assert!(
        cache
            .cap_for(0x10, 0x30, 29, 0, counted(&plain, &reads))
            .is_none()
    );
    assert!(
        cache
            .cap_for(0x10, 0x30, 29, 0, counted(&ids, &reads))
            .is_none()
    );
    assert!(
        cache
            .cap_for(0x10, 0x30, 30, 0, counted(&ids, &reads))
            .is_some()
    );
}

#[test]
fn a_cached_cap_stops_applying_once_the_cursor_is_past_the_gate() {
    let mut ids = vec![0x7777_7777_u32; 10];
    ids[3] = SCRIPT_GATE_OF_THE_SECOND_SCRIPT_PLAY_HAP;
    let reads = std::cell::Cell::new(0);
    let mut cache = HapCapCache::new();
    assert!(cache.cap_for(5, 6, 10, 3, counted(&ids, &reads)).is_some());
    assert!(cache.cap_for(5, 6, 10, 4, counted(&ids, &reads)).is_none());
}
