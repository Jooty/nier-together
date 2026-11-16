use replicant_game::offsets::{
    ASSET_NAME_SLOT_OFFSETS, ENVIRONMENT_AREA_OFFSET, ENVIRONMENT_BLEND_SECONDS_OFFSET,
    ENVIRONMENT_MANAGER_RVA, ENVIRONMENT_TABLE_RELOAD_OFFSET, WEATHER_COUNTER_RVA,
    WEATHER_OFFSET_COUNTER_RVA, WIND_AREA_OFFSET, WIND_MANAGER_RVA,
};
use replicant_game::sky::{
    APPLY_RETRY_MS, RecordAssembler, SKY_APPLY_LIMIT_MS, SkyBroadcast, SkyHold, SkyOutcome,
    SkyReading, ZONE_SETTLE_MS, merge_record, read_asset_names, read_sky, read_table_record,
    split_record, write_asset_pointers, write_table_record, write_weather_counters,
};

use crate::fake_memory::FakeMemory;

const PHASE: u16 = 7;

fn reading(env_area: i32) -> SkyReading {
    SkyReading {
        env_area,
        env_blend_seconds: 2.0,
        wind_area: 3,
        weather: 2,
        weather_offset: 1,
        table_reloading: false,
        sky_resources_ready: true,
    }
}

fn sky_memory(env_area: i32, reload: i32) -> FakeMemory {
    let mut memory = FakeMemory::new();
    let environment = ENVIRONMENT_MANAGER_RVA;
    memory.put_at_rva(
        environment + ENVIRONMENT_AREA_OFFSET as u32,
        &env_area.to_le_bytes(),
    );
    memory.put_at_rva(
        environment + ENVIRONMENT_BLEND_SECONDS_OFFSET as u32,
        &1.5_f32.to_le_bytes(),
    );
    memory.put_at_rva(
        environment + ENVIRONMENT_TABLE_RELOAD_OFFSET as u32,
        &reload.to_le_bytes(),
    );
    memory.put_at_rva(
        WIND_MANAGER_RVA + WIND_AREA_OFFSET as u32,
        &4_i32.to_le_bytes(),
    );
    memory.put_at_rva(WEATHER_COUNTER_RVA, &[2]);
    memory.put_at_rva(WEATHER_OFFSET_COUNTER_RVA, &[1]);
    memory
}

#[test]
fn the_sky_is_read_from_the_two_managers_and_the_counters() {
    let memory = sky_memory(12, 0);
    let read = read_sky(&memory).expect("readable");
    assert_eq!(read.env_area, 12);
    assert_eq!(read.wind_area, 4);
    assert_eq!((read.weather, read.weather_offset), (2, 1));
    assert!(read.is_settled());
}

#[test]
fn a_table_reload_or_no_area_yet_is_not_settled() {
    assert!(!read_sky(&sky_memory(12, 1)).expect("readable").is_settled());
    assert!(!read_sky(&sky_memory(-1, 0)).expect("readable").is_settled());
}

#[test]
fn weather_counters_are_written_back() {
    let mut memory = sky_memory(0, 0);
    write_weather_counters(&mut memory, 3, 0).expect("writable");
    let read = read_sky(&memory).expect("readable");
    assert_eq!((read.weather, read.weather_offset), (3, 0));
}

#[test]
fn the_host_sends_when_the_guest_joins() {
    let mut host = SkyBroadcast::default();
    assert!(host.next(0, PHASE, &reading(5), None).is_none());
    let sent = host
        .next(10, PHASE, &reading(5), Some(PHASE))
        .expect("sent");
    assert_eq!(sent.reason, "guest joined");
    assert_eq!(sent.state.env_area, 5);
    assert_eq!(sent.state.env_blend_seconds, 0.0);
}

#[test]
fn the_host_sends_no_change_made_later_in_the_same_zone() {
    let mut host = SkyBroadcast::default();
    host.next(0, PHASE, &reading(5), Some(PHASE));
    assert!(host.next(50, PHASE, &reading(5), Some(PHASE)).is_none());
    for (now, area) in [(100, 6), (150, 7), (10_000, 5)] {
        assert!(host.next(now, PHASE, &reading(area), Some(PHASE)).is_none());
    }
}

#[test]
fn a_rejoin_makes_the_host_send_again() {
    let mut host = SkyBroadcast::default();
    host.next(0, PHASE, &reading(5), Some(PHASE));
    assert!(host.next(1_000, PHASE, &reading(6), None).is_none());
    let sent = host
        .next(1_010, PHASE, &reading(6), Some(PHASE))
        .expect("sent");
    assert_eq!((sent.reason, sent.state.env_area), ("guest joined", 6));
}

#[test]
fn the_host_resends_after_the_guest_loads_a_zone_once_both_are_there() {
    let mut host = SkyBroadcast::default();
    host.next(0, PHASE, &reading(5), Some(PHASE));
    assert!(
        host.next(100, PHASE, &reading(5), Some(PHASE + 1))
            .is_none()
    );
    let late = 100 + ZONE_SETTLE_MS;
    assert!(
        host.next(late, PHASE, &reading(5), Some(PHASE + 1))
            .is_none()
    );
    assert!(
        host.next(late + 1, PHASE, &reading(5), Some(PHASE))
            .is_none()
    );
    let sent = host
        .next(late + 1 + ZONE_SETTLE_MS, PHASE, &reading(5), Some(PHASE))
        .expect("sent");
    assert_eq!(sent.reason, "guest zone loaded");
}

#[test]
fn the_host_waits_for_a_new_zone_to_settle_before_sending() {
    let mut host = SkyBroadcast::default();
    host.next(0, PHASE, &reading(5), Some(PHASE));
    assert!(
        host.next(100, PHASE + 1, &reading(5), Some(PHASE + 1))
            .is_none()
    );
    let sent = host
        .next(
            100 + ZONE_SETTLE_MS,
            PHASE + 1,
            &reading(9),
            Some(PHASE + 1),
        )
        .expect("sent");
    assert_eq!((sent.reason, sent.state.env_area), ("host zone loaded", 9));
    assert_eq!(sent.state.phase, PHASE + 1);
}

#[test]
fn the_host_sends_nothing_while_its_own_table_is_loading() {
    let mut host = SkyBroadcast::default();
    let loading = SkyReading {
        table_reloading: true,
        ..reading(5)
    };
    assert!(host.next(0, PHASE, &loading, Some(PHASE)).is_none());
    assert!(host.next(10, PHASE, &reading(5), Some(PHASE)).is_some());
}

fn host_state(area: i32, blend: f32) -> replicant_proto::SkyState {
    replicant_proto::SkyState {
        phase: PHASE,
        env_area: area,
        env_blend_seconds: blend,
        wind_area: 3,
        weather: 2,
        weather_offset: 1,
    }
}

#[test]
fn the_guest_applies_a_fresh_host_sky_with_the_host_blend() {
    let mut guest = SkyHold::default();
    guest.receive(host_state(8, 2.5));
    let apply = guest.step(0, PHASE, &reading(0));
    assert_eq!(apply.env, Some((8, 2.5)));
    assert_eq!(apply.wind, None);
    assert_eq!(apply.weather, None);
}

#[test]
fn a_guest_whose_own_script_changes_the_area_after_the_apply_keeps_its_own_area() {
    let mut guest = SkyHold::default();
    guest.receive(host_state(8, 2.5));
    assert_eq!(guest.step(0, PHASE, &reading(0)).env, Some((8, 2.5)));
    assert!(guest.step(10, PHASE, &reading(8)).is_empty());
    assert_eq!(
        guest.take_outcome(),
        Some(SkyOutcome {
            phase: PHASE,
            timed_out: false
        })
    );
    for now in [20, 300, 5_000] {
        assert!(guest.step(now, PHASE, &reading(4)).is_empty());
    }
    assert_eq!(guest.take_outcome(), None);
}

#[test]
fn a_new_zone_load_or_a_rejoin_applies_the_host_sky_again() {
    let mut guest = SkyHold::default();
    guest.receive(host_state(8, 0.0));
    guest.step(0, PHASE, &reading(8));
    assert!(guest.take_outcome().is_some());
    assert!(guest.step(100, PHASE, &reading(4)).is_empty());
    guest.receive(host_state(9, 0.0));
    assert_eq!(guest.step(200, PHASE, &reading(4)).env, Some((9, 0.0)));
}

#[test]
fn the_guest_retries_until_the_apply_has_taken_then_stops() {
    let mut guest = SkyHold::default();
    guest.receive(host_state(8, 0.0));
    assert!(guest.step(0, PHASE, &reading(0)).env.is_some());
    assert!(guest.step(100, PHASE, &reading(0)).is_empty());
    assert!(guest.step(APPLY_RETRY_MS, PHASE, &reading(0)).env.is_some());
    assert_eq!(guest.take_outcome(), None);
    assert!(
        guest
            .step(APPLY_RETRY_MS + 10, PHASE, &reading(8))
            .is_empty()
    );
    assert_eq!(
        guest.take_outcome(),
        Some(SkyOutcome {
            phase: PHASE,
            timed_out: false
        })
    );
    assert!(guest.step(5_000, PHASE, &reading(0)).is_empty());
}

#[test]
fn the_guest_gives_up_after_the_limit_and_says_so() {
    let mut guest = SkyHold::default();
    guest.receive(host_state(8, 0.0));
    assert!(guest.step(0, PHASE, &reading(0)).env.is_some());
    assert!(
        guest
            .step(SKY_APPLY_LIMIT_MS, PHASE, &reading(0))
            .is_empty()
    );
    assert_eq!(
        guest.take_outcome(),
        Some(SkyOutcome {
            phase: PHASE,
            timed_out: true
        })
    );
    assert!(
        guest
            .step(SKY_APPLY_LIMIT_MS + 1_000, PHASE, &reading(0))
            .is_empty()
    );
}

#[test]
fn the_guest_retries_no_faster_than_the_retry_interval() {
    let mut guest = SkyHold::default();
    guest.receive(host_state(8, 0.0));
    assert!(!guest.step(0, PHASE, &reading(0)).is_empty());
    assert!(
        guest
            .step(APPLY_RETRY_MS - 1, PHASE, &reading(0))
            .is_empty()
    );
    assert!(!guest.step(APPLY_RETRY_MS, PHASE, &reading(0)).is_empty());
}

#[test]
fn the_guest_takes_the_host_weather_counters() {
    let mut guest = SkyHold::default();
    guest.receive(host_state(8, 0.0));
    let local = SkyReading {
        weather: 3,
        weather_offset: 0,
        ..reading(8)
    };
    assert_eq!(guest.step(0, PHASE, &local).weather, Some((2, 1)));
}

#[test]
fn the_guest_holds_nothing_in_another_zone_or_while_loading() {
    let mut guest = SkyHold::default();
    guest.receive(host_state(8, 0.0));
    assert!(guest.step(0, PHASE + 1, &reading(0)).is_empty());
    let loading = SkyReading {
        env_area: -1,
        ..reading(0)
    };
    assert!(guest.step(10, PHASE, &loading).is_empty());
    assert!(!guest.step(20, PHASE, &reading(0)).is_empty());
}

#[test]
fn an_unset_host_wind_is_left_alone() {
    let mut guest = SkyHold::default();
    guest.receive(replicant_proto::SkyState {
        wind_area: -1,
        ..host_state(8, 0.0)
    });
    assert_eq!(guest.step(0, PHASE, &reading(8)).wind, None);
}

#[test]
fn the_guest_reapplies_the_area_once_sky_resources_become_ready() {
    let mut guest = SkyHold::default();
    guest.receive(host_state(8, 0.0));
    let not_ready = SkyReading {
        sky_resources_ready: false,
        ..reading(0)
    };
    assert!(guest.step(0, PHASE, &not_ready).env.is_some());
    let matching_not_ready = SkyReading {
        sky_resources_ready: false,
        ..reading(8)
    };
    assert!(guest.step(300, PHASE, &matching_not_ready).is_empty());
    let again = guest.step(600, PHASE, &reading(8));
    assert_eq!(again.env, Some((8, 0.0)));
    assert!(guest.step(900, PHASE, &reading(8)).is_empty());
}

#[test]
fn the_host_sends_again_after_a_reload_of_the_same_phase() {
    let mut host = SkyBroadcast::default();
    host.note_zone_loads(0, 1);
    let joined = host
        .next(0, PHASE, &reading(30), Some(PHASE))
        .expect("join send");
    assert_eq!(joined.reason, "guest joined");
    host.note_zone_loads(600, 2);
    assert!(host.next(600, PHASE, &reading(30), Some(PHASE)).is_none());
    let sent = host
        .next(600 + ZONE_SETTLE_MS, PHASE, &reading(31), Some(PHASE))
        .expect("sent after the reload settled");
    assert_eq!((sent.reason, sent.state.env_area), ("host zone loaded", 31));
    assert!(
        host.next(700 + ZONE_SETTLE_MS, PHASE, &reading(32), Some(PHASE))
            .is_none()
    );
}

#[test]
fn replay_a_lost_join_send_then_a_same_phase_jump_reload_on_both_sides() {
    let mut host = SkyBroadcast::default();
    let mut guest = SkyHold::default();
    host.note_zone_loads(0, 1);
    assert!(!guest.note_zone_loads(0, PHASE, 1));
    assert!(host.next(0, PHASE, &reading(30), Some(PHASE)).is_some());
    host.note_zone_loads(600, 2);
    assert!(!guest.note_zone_loads(623, PHASE, 2));
    let mut sent = None;
    for now in (600..600 + 2 * ZONE_SETTLE_MS).step_by(16) {
        sent = sent.or(host.next(now, PHASE, &reading(30), Some(PHASE)));
    }
    let sent = sent.expect("the host sends after its jump reload");
    guest.receive(sent.state);
    assert!(guest.step(1_360, PHASE, &reading(0)).is_empty());
    assert_eq!(
        guest.step(623 + ZONE_SETTLE_MS, PHASE, &reading(0)).env,
        Some((30, 0.0))
    );
    assert!(
        guest
            .step(640 + ZONE_SETTLE_MS, PHASE, &reading(30))
            .is_empty()
    );
    assert_eq!(
        guest.take_outcome(),
        Some(SkyOutcome {
            phase: PHASE,
            timed_out: false
        })
    );
}

#[test]
fn a_host_sky_applied_just_before_the_guest_reloads_is_applied_again_after_it() {
    let mut guest = SkyHold::default();
    guest.note_zone_loads(0, PHASE, 1);
    guest.receive(host_state(30, 0.0));
    assert!(guest.step(10, PHASE, &reading(0)).env.is_some());
    assert!(guest.step(20, PHASE, &reading(30)).is_empty());
    assert!(guest.take_outcome().is_some());
    assert!(guest.note_zone_loads(500, PHASE, 2));
    assert!(guest.waits_for_zone_settle(600));
    assert!(guest.step(600, PHASE, &reading(0)).is_empty());
    assert_eq!(
        guest.step(500 + ZONE_SETTLE_MS, PHASE, &reading(0)).env,
        Some((30, 0.0))
    );
    assert!(
        guest
            .step(510 + ZONE_SETTLE_MS, PHASE, &reading(30))
            .is_empty()
    );
    assert!(guest.take_outcome().is_some());
    assert!(guest.step(5_000, PHASE, &reading(4)).is_empty());
}

#[test]
fn a_guest_load_into_another_phase_does_not_bring_back_the_old_sky() {
    let mut guest = SkyHold::default();
    guest.note_zone_loads(0, PHASE, 1);
    guest.receive(host_state(30, 0.0));
    guest.step(10, PHASE, &reading(30));
    assert!(guest.take_outcome().is_some());
    assert!(!guest.note_zone_loads(500, PHASE + 1, 2));
    assert!(guest.step(5_000, PHASE + 1, &reading(0)).is_empty());
    assert!(guest.step(6_000, PHASE, &reading(0)).is_empty());
}

#[test]
fn the_first_zone_load_count_the_guest_sees_is_not_a_reload() {
    let mut guest = SkyHold::default();
    assert!(!guest.note_zone_loads(0, PHASE, 5));
    assert!(!guest.waits_for_zone_settle(0));
    guest.receive(host_state(30, 0.0));
    assert!(guest.step(0, PHASE, &reading(0)).env.is_some());
}

fn record_with(words: &[u64]) -> Vec<u8> {
    let mut record = vec![0_u8; 0xAA0];
    for (index, word) in words.iter().enumerate() {
        record[index * 8..index * 8 + 8].copy_from_slice(&word.to_le_bytes());
    }
    record
}

#[test]
fn a_record_split_into_chunks_assembles_again_in_any_order() {
    let record: Vec<u8> = (0..0xAA0_u32).map(|value| (value % 251) as u8).collect();
    let mut chunks = split_record(PHASE, 30, &record);
    assert!(chunks.iter().all(|chunk| chunk.validate().is_ok()));
    chunks.reverse();
    let mut assembler = RecordAssembler::default();
    let mut done = None;
    for chunk in &chunks {
        done = assembler.receive(chunk);
    }
    assert_eq!(done, Some(record));
}

#[test]
fn an_incomplete_record_is_not_assembled() {
    let chunks = split_record(PHASE, 30, &vec![1; 0xAA0]);
    let mut assembler = RecordAssembler::default();
    assert!(assembler.receive(&chunks[0]).is_none());
    assert!(assembler.receive(&chunks[2]).is_none());
}

#[test]
fn merging_keeps_only_the_five_asset_pointers_and_takes_everything_else_from_the_host() {
    let mut host = record_with(&[]);
    let mut local = record_with(&[]);
    for (index, byte) in host.iter_mut().enumerate() {
        *byte = (index % 251) as u8 | 1;
    }
    for offset in ASSET_NAME_SLOT_OFFSETS {
        local[offset..offset + 8].copy_from_slice(&0x0000_015A_598B_97E0_u64.to_le_bytes());
    }
    let merged = merge_record(&host, &local);
    for offset in ASSET_NAME_SLOT_OFFSETS {
        assert_eq!(merged[offset..offset + 8], local[offset..offset + 8]);
    }
    let pointer_bytes: Vec<usize> = ASSET_NAME_SLOT_OFFSETS
        .iter()
        .flat_map(|offset| *offset..*offset + 8)
        .collect();
    for index in (0..merged.len()).filter(|index| !pointer_bytes.contains(index)) {
        assert_eq!(merged[index], host[index]);
    }
}

#[test]
fn a_float_word_with_a_zero_high_half_is_taken_from_the_host() {
    let float_word = u64::from(0.75_f32.to_bits());
    let host = record_with(&[float_word, 7]);
    let local = record_with(&[u64::from(0.25_f32.to_bits()), 0]);
    let merged = merge_record(&host, &local);
    assert_eq!(merged[..16], host[..16]);
}

#[test]
fn a_host_pointer_below_four_gigabytes_never_reaches_a_pointer_field() {
    let wine_heap_pointer = 0x0AC3_C4D8_u64;
    let windows_heap_pointer = 0x0000_015A_598B_97E0_u64;
    let mut host = record_with(&[]);
    let mut local = record_with(&[]);
    host[0x350..0x358].copy_from_slice(&wine_heap_pointer.to_le_bytes());
    local[0x350..0x358].copy_from_slice(&windows_heap_pointer.to_le_bytes());
    let merged = merge_record(&host, &local);
    assert_eq!(merged[0x350..0x358], windows_heap_pointer.to_le_bytes());
}

#[test]
fn a_table_record_is_read_and_written_through_the_table_pointer() {
    let table = 0x1_5000_0000_u64;
    let mut memory = FakeMemory::new();
    memory.put_at_rva(ENVIRONMENT_MANAGER_RVA + 0x1FE8, &table.to_le_bytes());
    memory.put(table + 2 * 0xAA0, &vec![0; 0xAA0]);
    write_table_record(&mut memory, 2, &vec![9; 0xAA0]).expect("writable");
    assert_eq!(
        read_table_record(&memory, 2).expect("readable"),
        vec![9; 0xAA0]
    );
}

#[test]
fn asset_names_are_read_through_the_record_pointers_and_pointers_are_rewritten() {
    let table = 0x1_5000_0000_u64;
    let strings = 0x1_6000_0000_u64;
    let mut memory = FakeMemory::new();
    memory.put_at_rva(ENVIRONMENT_MANAGER_RVA + 0x1FE8, &table.to_le_bytes());
    memory.put(table + 0xAA0, &vec![0; 0xAA0]);
    memory.put(strings, b"bg/skylight/a\0");
    memory.put(table + 0xAA0 + 0x350, &strings.to_le_bytes());
    let names = read_asset_names(&memory, 1).expect("readable");
    assert_eq!(names[0], "bg/skylight/a");
    assert_eq!(names[1], "");
    write_asset_pointers(&mut memory, 1, &[0, 0, 0, 0, 0]).expect("writable");
    assert_eq!(read_asset_names(&memory, 1).expect("readable")[0], "");
}
