use replicant_game::story_progress::{
    HOST_STORY_WAITS_FOR_THE_STAGE_START_AT_MOST_MS, HostStoryApply, HostStoryApplyGate,
    HostStoryRelease, STAGE_START_SCRIPT_FUNCTION,
};

const GUEST_IN_THE_FIELD_MS: u64 = 6_773_000;
const GUEST_PHASE_CHANGE_MS: u64 = 6_773_467;
const HOSTS_STAGE_START_FLAGS_ARRIVE_MS: u64 = 6_774_068;
const GUEST_PHASE_LOADED_MS: u64 = 6_775_480;
const GUEST_STAGE_START_MS: u64 = 6_779_441;
const LOADS_BEFORE: u32 = 5;
const LOADS_AFTER: u32 = 6;
const FIELD_PHASE: Option<u16> = Some(19);
const MANOR_PHASE: Option<u16> = Some(20);

#[test]
fn run_1115j_the_hosts_changes_wait_from_the_guests_phase_change_until_its_own_stage_start_has_run()
{
    let mut gate = HostStoryApplyGate::new();
    assert_eq!(
        gate.step(true, LOADS_BEFORE, FIELD_PHASE, GUEST_IN_THE_FIELD_MS),
        HostStoryApply::Now
    );
    assert_eq!(
        gate.step(true, LOADS_BEFORE, MANOR_PHASE, GUEST_PHASE_CHANGE_MS),
        HostStoryApply::HeldForTheLoad,
        "01:52:53.467: the guest's phase name changed with its ChangePhase, while it was still in play"
    );
    assert_eq!(
        gate.step(
            true,
            LOADS_BEFORE,
            MANOR_PHASE,
            HOSTS_STAGE_START_FLAGS_ARRIVE_MS
        ),
        HostStoryApply::HeldForTheLoad,
        "01:52:54.068: g503 from the host's stage start reached the guest, still reported in play and 1.4 s before its load; applied then, its own StartStage skipped the step that places the escort actor"
    );
    assert_eq!(
        gate.step(true, LOADS_AFTER, MANOR_PHASE, GUEST_PHASE_LOADED_MS),
        HostStoryApply::HeldForTheLoad
    );
    assert_eq!(
        gate.step(true, LOADS_AFTER, MANOR_PHASE, GUEST_STAGE_START_MS - 1),
        HostStoryApply::HeldForTheLoad,
        "01:52:59.440: the guest's StartStage came 3.96 s after its load"
    );
    gate.note_function_ran(STAGE_START_SCRIPT_FUNCTION);
    assert_eq!(
        gate.step(true, LOADS_AFTER, MANOR_PHASE, GUEST_STAGE_START_MS + 16),
        HostStoryApply::Now
    );
    assert_eq!(gate.last_release(), Some(HostStoryRelease::StageStart));
}

#[test]
fn a_stage_start_that_never_shows_releases_the_hold_after_the_cap_counted_from_the_load() {
    let mut gate = HostStoryApplyGate::new();
    gate.step(true, LOADS_BEFORE, FIELD_PHASE, GUEST_IN_THE_FIELD_MS);
    gate.step(true, LOADS_BEFORE, MANOR_PHASE, GUEST_PHASE_CHANGE_MS);
    gate.step(true, LOADS_AFTER, MANOR_PHASE, GUEST_PHASE_LOADED_MS);
    gate.note_function_ran("Play_Env_05_10");
    assert_eq!(
        gate.step(
            true,
            LOADS_AFTER,
            MANOR_PHASE,
            GUEST_PHASE_LOADED_MS + HOST_STORY_WAITS_FOR_THE_STAGE_START_AT_MOST_MS - 1
        ),
        HostStoryApply::HeldForTheLoad
    );
    assert_eq!(
        gate.step(
            true,
            LOADS_AFTER,
            MANOR_PHASE,
            GUEST_PHASE_LOADED_MS + HOST_STORY_WAITS_FOR_THE_STAGE_START_AT_MOST_MS
        ),
        HostStoryApply::Now
    );
    assert_eq!(gate.last_release(), Some(HostStoryRelease::Cap));
}

#[test]
fn the_first_look_at_the_phase_and_the_load_count_starts_no_wait() {
    let mut gate = HostStoryApplyGate::new();
    assert_eq!(
        gate.step(true, LOADS_AFTER, MANOR_PHASE, GUEST_PHASE_LOADED_MS),
        HostStoryApply::Now
    );
    assert_eq!(gate.last_release(), None);
}

#[test]
fn a_load_without_a_phase_change_still_waits_for_the_stage_start() {
    let mut gate = HostStoryApplyGate::new();
    gate.step(true, LOADS_BEFORE, MANOR_PHASE, GUEST_IN_THE_FIELD_MS);
    assert_eq!(
        gate.step(true, LOADS_AFTER, MANOR_PHASE, GUEST_PHASE_LOADED_MS),
        HostStoryApply::HeldForTheLoad,
        "a jump in place reloads the phase the game is in"
    );
    gate.note_function_ran(STAGE_START_SCRIPT_FUNCTION);
    assert_eq!(
        gate.step(true, LOADS_AFTER, MANOR_PHASE, GUEST_STAGE_START_MS),
        HostStoryApply::Now
    );
}

#[test]
fn a_phase_name_that_cannot_be_read_changes_nothing() {
    let mut gate = HostStoryApplyGate::new();
    gate.step(true, LOADS_BEFORE, FIELD_PHASE, GUEST_IN_THE_FIELD_MS);
    assert_eq!(
        gate.step(true, LOADS_BEFORE, None, GUEST_IN_THE_FIELD_MS + 100),
        HostStoryApply::Now
    );
    assert_eq!(
        gate.step(true, LOADS_BEFORE, FIELD_PHASE, GUEST_IN_THE_FIELD_MS + 200),
        HostStoryApply::Now
    );
}

#[test]
fn a_game_out_of_play_without_a_load_still_holds_the_hosts_changes() {
    let mut gate = HostStoryApplyGate::new();
    gate.step(true, LOADS_BEFORE, FIELD_PHASE, GUEST_IN_THE_FIELD_MS);
    assert_eq!(
        gate.step(
            false,
            LOADS_BEFORE,
            FIELD_PHASE,
            GUEST_IN_THE_FIELD_MS + 100
        ),
        HostStoryApply::HeldForTheLoad
    );
    assert_eq!(
        gate.step(true, LOADS_BEFORE, FIELD_PHASE, GUEST_IN_THE_FIELD_MS + 200),
        HostStoryApply::Now
    );
}
