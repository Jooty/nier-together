mod end_follow;
mod end_queue;
mod gate;
mod names;
mod own_start_pause;
mod partner_copies;
mod ran_ends;
mod relayed_scenes;
mod scene_pull;
mod stretch_end;

pub use end_follow::{
    EndArrival, EndDrop, EndFollow, MaskIdleTimer, PendingEnd, WaitStep, control_slots_free,
    end_follow, is_end_callback, own_end_is_the_copy_a_waiting_host_end_needs,
    slots_an_end_waits_on, wait_step,
};
pub use end_queue::{EndQueue, ParkedCall, ParkedKind, QueueEvent, Released};
pub use gate::{CallbackGate, GateRole, GateStep, LocalFire, PartnerFire};
pub use names::{is_event_talk, is_gated_callback, is_scene_talk};
pub use own_start_pause::{
    LastPauseOn, OwnStartPauseFacts, OwnStartPauseRelease, own_start_of_end,
    own_start_pause_release,
};
pub use partner_copies::{PartnerStartedCopies, event_base, event_family};
pub use ran_ends::RanEnds;
pub use relayed_scenes::{PlayedStart, RelayedScenesPlayed, scene_base, start_base};
pub use scene_pull::{ScenePull, ScenePullFacts, ScenePullSkip, scene_pull};
pub use stretch_end::end_follow_around_a_stretch;

pub const HOST_STEPS_THAT_PLACE_OR_MOVE_A_STORY_ACTOR: [&str; 2] =
    ["EID_2000_a0026g_02_Start", "EID_2000_a0026g_03_Start"];
pub const GATED_CALLBACK_NAMES: [&str; 1] = ["Boss_Battle_Start"];
pub const GATED_EVENT_SUFFIX: &str = "_Start";
pub const SCENE_PLAY_SUFFIXES: [&str; 3] = ["_PLAY_EVENT", "_EVENT", "_Event"];
pub const STORY_EVENT_PREFIXES: [&str; 2] = ["EID_", "SEID_"];
pub const CALLBACK_GATE_PAIRING_WINDOW_MS: u64 = 20_000;
pub const CALLBACK_GATE_HOST_GRACE_MS: u64 = 3_000;
pub const CALLBACK_GATE_AFTER_DIALOG_CLOSE_MS: u64 = 600;
pub const CALLBACK_GATE_GUEST_GIVE_UP_MS: u64 = 20_000;
pub const SCENE_TALK_PREFIX: &str = "SEID_";
pub const END_FOLLOW_EVENT_SUFFIX: &str = "_End";
pub const END_FOLLOW_STILL_WAITING_LOG_MS: u64 = 60_000;
pub const END_FOLLOW_MASK_IDLE_MS: u64 = 500;
pub const ENDS_THAT_RUN_UNDER_THEIR_OWN_STARTS_PAUSE: [&str; 1] = ["EID_1500_a0005_010_01_End"];
pub const ENDS_THAT_RUN_ANYWAY_AFTER_WAITING: [&str; 2] =
    ["EID_1500_a0005_010_01_End", "EID_1500_a0005_010_01_End02"];
pub const END_RUNS_UNDER_ITS_OWN_STARTS_PAUSE_AFTER_MS: u64 = 2_000;
pub const END_RUNS_ANYWAY_AFTER_WAITING_MS: u64 = 20_000;
pub const TALK_EVENT_SUFFIX: &str = "_Talk";
pub const TALK_START_AFTER_COMMAND_MS: u64 = 2_000;
