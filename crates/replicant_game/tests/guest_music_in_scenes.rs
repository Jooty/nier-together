use replicant_game::music::{DroppedCall, DroppedCalls, host_music_drives, host_reports_a_scene};
use replicant_proto::PartnerActivity;

const FIGHT_TRACK: i32 = 100_047;
const HOST_SCENE_BEGAN_MS: u64 = 0;
const GUEST_MENU_CLOSED_MS: u64 = 250;
const GUEST_ENTERED_SCENE_MS: u64 = 4_900;
const GUEST_OWN_PLAY_MS: u64 = 4_890;
const TICK_MS: u64 = 16;
const FRESH_SNAPSHOT_AGE_MS: u64 = 40;
const STALE_SNAPSHOT_AGE_MS: u64 = 3_000;
const NO_FADE_SECONDS: f32 = 0.0;
const FULL_VOLUME: f32 = 1.0;
const LAYER_COUNT: i32 = 4;

fn drive_at(now_ms: u64) -> bool {
    let host_in_scene = host_reports_a_scene(PartnerActivity::InCutscene, FRESH_SNAPSHOT_AGE_MS);
    let menu_open = now_ms < GUEST_MENU_CLOSED_MS;
    let guest_in_play = now_ms < GUEST_ENTERED_SCENE_MS;
    host_music_drives(true, menu_open, guest_in_play, host_in_scene)
}

fn own_play() -> DroppedCall {
    DroppedCall::Play {
        track: FIGHT_TRACK,
        volume: FULL_VOLUME,
        fade_seconds: NO_FADE_SECONDS,
    }
}

fn own_master() -> DroppedCall {
    DroppedCall::Master {
        volume: FULL_VOLUME,
        seconds: NO_FADE_SECONDS,
    }
}

fn own_layer(layer: i32) -> DroppedCall {
    DroppedCall::Layer {
        layer,
        volume: FULL_VOLUME,
        seconds: NO_FADE_SECONDS,
    }
}

#[test]
fn the_hosts_music_never_drives_a_guest_while_the_host_is_in_a_scene() {
    let mut now_ms = HOST_SCENE_BEGAN_MS;
    while now_ms <= GUEST_ENTERED_SCENE_MS + 100 {
        assert!(!drive_at(now_ms), "the drive was on at {now_ms} ms");
        now_ms += TICK_MS;
    }
}

#[test]
fn the_guests_own_play_inside_the_late_join_gap_is_not_ignored() {
    let drives = drive_at(GUEST_OWN_PLAY_MS);
    let ignored = drives;
    assert!(!ignored);
}

#[test]
fn the_hosts_music_drives_a_guest_in_play_when_the_host_is_not_in_a_scene() {
    assert!(host_music_drives(true, false, true, false));
    assert!(!host_music_drives(false, false, true, false));
    assert!(!host_music_drives(true, true, true, false));
    assert!(!host_music_drives(true, false, false, false));
}

#[test]
fn an_old_or_other_host_activity_is_not_a_scene() {
    assert!(!host_reports_a_scene(
        PartnerActivity::InCutscene,
        STALE_SNAPSHOT_AGE_MS
    ));
    assert!(!host_reports_a_scene(
        PartnerActivity::InMenu,
        FRESH_SNAPSHOT_AGE_MS
    ));
}

#[test]
fn dropped_calls_are_replayed_once_when_the_drive_lets_go() {
    let mut dropped = DroppedCalls::new();
    dropped.record(own_play());
    dropped.record(own_master());
    for layer in 0..LAYER_COUNT {
        dropped.record(own_layer(layer));
    }
    let mut expected = vec![own_play(), own_master()];
    expected.extend((0..LAYER_COUNT).map(own_layer));
    assert_eq!(dropped.take_replay(), expected);
    assert!(dropped.take_replay().is_empty());
}

#[test]
fn only_the_last_call_of_each_kind_is_replayed() {
    let mut dropped = DroppedCalls::new();
    dropped.record(own_play());
    dropped.record(DroppedCall::Stop {
        fade_seconds: NO_FADE_SECONDS,
    });
    dropped.record(DroppedCall::Master {
        volume: 0.5,
        seconds: NO_FADE_SECONDS,
    });
    dropped.record(own_master());
    assert_eq!(
        dropped.take_replay(),
        vec![
            DroppedCall::Stop {
                fade_seconds: NO_FADE_SECONDS
            },
            own_master()
        ]
    );
}

#[test]
fn calls_dropped_before_a_new_drive_are_forgotten() {
    let mut dropped = DroppedCalls::new();
    dropped.record(own_play());
    dropped.clear();
    assert!(dropped.take_replay().is_empty());
}

#[test]
fn a_layer_outside_the_known_layers_is_not_kept() {
    let mut dropped = DroppedCalls::new();
    dropped.record(own_layer(LAYER_COUNT));
    dropped.record(own_layer(-1));
    assert!(dropped.take_replay().is_empty());
}
