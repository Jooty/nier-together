use replicant_proto::PartnerActivity;

use crate::cutscene_skip::PARTNER_SNAPSHOT_STALE_MS;
use crate::music::{
    FIGHTS_WHOSE_HOST_MUSIC_THE_GUEST_KEEPS_THROUGH_SILENCE, MUSIC_TARGET_STALE_IN_A_HELD_FIGHT_MS,
    MUSIC_TARGET_STALE_MS,
};

pub fn host_reports_a_scene(activity: PartnerActivity, snapshot_age_ms: u64) -> bool {
    activity == PartnerActivity::InCutscene && snapshot_age_ms < PARTNER_SNAPSHOT_STALE_MS
}

pub fn host_music_drives(
    host_music_fresh: bool,
    local_world_paused: bool,
    local_in_play: bool,
    host_in_scene: bool,
) -> bool {
    host_music_fresh && !local_world_paused && local_in_play && !host_in_scene
}

pub fn drive_release_replays_dropped_calls(
    host_music_fresh: bool,
    local_world_paused: bool,
    local_in_play: bool,
    host_in_scene: bool,
) -> bool {
    host_music_fresh && !local_world_paused && local_in_play && host_in_scene
}

pub fn host_music_stale_after_ms(fight: Option<&str>) -> u64 {
    let held = fight.is_some_and(|fight| {
        FIGHTS_WHOSE_HOST_MUSIC_THE_GUEST_KEEPS_THROUGH_SILENCE.contains(&fight)
    });
    if held {
        MUSIC_TARGET_STALE_IN_A_HELD_FIGHT_MS
    } else {
        MUSIC_TARGET_STALE_MS
    }
}
