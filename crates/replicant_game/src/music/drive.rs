use replicant_proto::PartnerActivity;

use crate::cutscene_skip::PARTNER_SNAPSHOT_STALE_MS;

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
