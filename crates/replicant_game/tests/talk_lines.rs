use replicant_game::dialogue::{
    ChoiceFollowInput, ChoiceFollower, ChoiceStep, MirrorAction, MirrorInput, PartnerTalkMirror,
    TalkLeader, talk_command_came_from_a_story_step,
};
use replicant_game::host_led_stretch::host_run_replays_on_the_watching_guest;
use replicant_game::script_functions::{traced_bare_unreachable_function, traced_function_name};
use replicant_proto::{DialogueLine, TalkChoice, TalkChoiceKind};

const DREAM_FIRST_TALK: u32 = 1422;
const DREAM_SECOND_TALK: u32 = 1423;
const GUEST_MIRROR_OPENS_1422_MS: u64 = 2_406_979;
const HOST_LINE_1_OF_1422_MS: u64 = 2_409_983;
const HOST_ON_1423_MS: u64 = 2_411_507;
const HOST_LINE_1_OF_1423_MS: u64 = 2_417_174;
const PROMPT_TALK: u32 = 1323;
const OPTION_TALK: u32 = 1324;
const HOST_PICK_RECEIVED_MS: u64 = 2_396_660;
const GUESTS_OWN_PROMPT_OPENS_MS: u64 = 2_398_470;
const STEP_WITH_A_FORCED_TALK: &str = "EID_1500_a0005_040_Select01";
const FORCED_TALK_STEP_POP_MS: u64 = 2_396_843;
const FORCED_TALK_WINDOW_MS: u64 = 2_396_858;
const LAST_POP_BEFORE_THE_PLAYERS_TALK: &str = "From_1F_to_2F_03";
const LAST_POP_BEFORE_THE_PLAYERS_TALK_MS: u64 = 2_391_189;
const PLAYERS_TALK_WINDOW_MS: u64 = 2_392_514;
const MANOR_ROOM_TALK: u32 = 1359;
const GUEST_MIRROR_OPENS_1359_MS: u64 = 2_728_802;
const GUESTS_OWN_1359_OPENS_MS: u64 = 2_728_819;
const HOST_LINE_1_OF_1359_MS: u64 = 2_731_013;
const MANOR: &str = "A_SOUTH_MANSION_01";

fn line(talk_id: u32, line: u16) -> DialogueLine {
    DialogueLine { talk_id, line }
}

fn watching(partner_line: DialogueLine, window_open: bool, now_ms: u64) -> MirrorInput {
    MirrorInput {
        partner_line: Some(partner_line),
        partner_ended_talk: None,
        can_show: true,
        window_open,
        own_talk: None,
        leader: TalkLeader::Either,
        answer_open: false,
        now_ms,
    }
}

fn hosts_pick() -> TalkChoice {
    TalkChoice {
        talk_id: PROMPT_TALK,
        line: 0,
        kind: TalkChoiceKind::TalkList,
        index: 0,
        target: OPTION_TALK,
    }
}

#[test]
fn run_1115g_the_dreams_next_talk_opens_in_the_window_the_mirror_already_has_open() {
    let mut mirror = PartnerTalkMirror::new();
    assert_eq!(
        mirror.step(watching(
            line(DREAM_FIRST_TALK, 0),
            false,
            GUEST_MIRROR_OPENS_1422_MS
        )),
        Some(MirrorAction::Open(line(DREAM_FIRST_TALK, 0)))
    );
    assert_eq!(
        mirror.step(watching(
            line(DREAM_FIRST_TALK, 1),
            true,
            HOST_LINE_1_OF_1422_MS
        )),
        Some(MirrorAction::ShowLine(1))
    );
    assert_eq!(
        mirror.step(watching(line(DREAM_SECOND_TALK, 0), true, HOST_ON_1423_MS)),
        Some(MirrorAction::SwitchTalk(line(DREAM_SECOND_TALK, 0)))
    );
    assert_eq!(
        mirror.step(watching(
            line(DREAM_SECOND_TALK, 1),
            true,
            HOST_LINE_1_OF_1423_MS
        )),
        Some(MirrorAction::ShowLine(1))
    );
    assert!(mirror.is_showing());
}

#[test]
fn run_1115g_a_talk_command_the_story_step_issued_is_the_steps_talk() {
    assert!(talk_command_came_from_a_story_step(
        Some((STEP_WITH_A_FORCED_TALK, FORCED_TALK_STEP_POP_MS)),
        FORCED_TALK_WINDOW_MS
    ));
    assert!(!talk_command_came_from_a_story_step(
        Some((
            LAST_POP_BEFORE_THE_PLAYERS_TALK,
            LAST_POP_BEFORE_THE_PLAYERS_TALK_MS
        )),
        PLAYERS_TALK_WINDOW_MS
    ));
    assert!(!talk_command_came_from_a_story_step(
        None,
        PLAYERS_TALK_WINDOW_MS
    ));
}

#[test]
fn run_1115g_the_guests_late_own_prompt_takes_the_hosts_answer_and_never_asks() {
    let mut follower = ChoiceFollower::new();
    follower.receive(hosts_pick(), HOST_PICK_RECEIVED_MS);
    let only_watching = ChoiceFollowInput {
        follows_host: false,
        own_talk: None,
        prompt: None,
        now_ms: HOST_PICK_RECEIVED_MS + 1,
    };
    assert_eq!(
        follower.step(only_watching),
        Some(ChoiceStep::Drop(hosts_pick()))
    );
    let own_prompt = ChoiceFollowInput {
        follows_host: false,
        own_talk: Some(line(PROMPT_TALK, 0)),
        prompt: Some(TalkChoiceKind::TalkList),
        now_ms: GUESTS_OWN_PROMPT_OPENS_MS,
    };
    assert_eq!(
        follower.step(own_prompt),
        Some(ChoiceStep::TakeKept(hosts_pick()))
    );
    assert_eq!(
        follower.step(ChoiceFollowInput {
            now_ms: GUESTS_OWN_PROMPT_OPENS_MS + 16,
            ..own_prompt
        }),
        None
    );
    assert!(follower.is_waiting());
    let option_talk = ChoiceFollowInput {
        own_talk: Some(line(OPTION_TALK, 0)),
        prompt: None,
        now_ms: GUESTS_OWN_PROMPT_OPENS_MS + 200,
        ..own_prompt
    };
    assert_eq!(follower.step(option_talk), None);
    assert!(!follower.is_waiting());
    assert_eq!(
        follower.step(ChoiceFollowInput {
            now_ms: GUESTS_OWN_PROMPT_OPENS_MS + 300,
            ..own_prompt
        }),
        None
    );
}

#[test]
fn a_kept_answer_is_not_taken_for_another_prompt_or_after_its_time() {
    let mut follower = ChoiceFollower::new();
    follower.receive(hosts_pick(), 0);
    let other_prompt = ChoiceFollowInput {
        follows_host: false,
        own_talk: Some(line(PROMPT_TALK + 1, 0)),
        prompt: Some(TalkChoiceKind::TalkList),
        now_ms: 10,
    };
    assert_eq!(
        follower.step(other_prompt),
        Some(ChoiceStep::Drop(hosts_pick()))
    );
    assert_eq!(follower.step(other_prompt), None);
    let too_late = ChoiceFollowInput {
        own_talk: Some(line(PROMPT_TALK, 0)),
        now_ms: replicant_game::dialogue::HOST_ANSWER_KEPT_FOR_A_LATE_PROMPT_MS,
        ..other_prompt
    };
    assert_eq!(follower.step(too_late), None);
}

#[test]
fn run_1115g_the_guests_own_copy_of_the_manor_room_talk_follows_the_host_while_the_host_is_on_it() {
    let mut mirror = PartnerTalkMirror::new();
    assert_eq!(
        mirror.step(watching(
            line(MANOR_ROOM_TALK, 0),
            false,
            GUEST_MIRROR_OPENS_1359_MS
        )),
        Some(MirrorAction::Open(line(MANOR_ROOM_TALK, 0)))
    );
    let own = |host_line, now_ms| MirrorInput {
        own_talk: Some(line(MANOR_ROOM_TALK, 0)),
        leader: TalkLeader::Partner,
        ..watching(line(MANOR_ROOM_TALK, host_line), true, now_ms)
    };
    assert_eq!(
        mirror.step(own(0, GUESTS_OWN_1359_OPENS_MS)),
        Some(MirrorAction::FollowStarted(MANOR_ROOM_TALK))
    );
    assert_eq!(
        mirror.step(own(1, HOST_LINE_1_OF_1359_MS)),
        Some(MirrorAction::FollowLine(1))
    );
    assert!(mirror.holds_own_talk());
}

#[test]
fn in_the_watched_manor_the_hosts_story_steps_replay_and_its_doors_do_not() {
    assert!(host_run_replays_on_the_watching_guest(
        "EID_1100_a0030_GoBack"
    ));
    assert!(host_run_replays_on_the_watching_guest("EID_1100_a0045g_02"));
    assert!(!host_run_replays_on_the_watching_guest("Open_Door_01"));
    assert!(!host_run_replays_on_the_watching_guest("RETRY_FLAG_ON_001"));
}

#[test]
fn the_scene_start_a_door_calls_is_traced_from_its_bare_name() {
    assert_eq!(
        traced_bare_unreachable_function(MANOR, "EID_1100_a0050_01_Start"),
        Some("EID_1100_a0050_01_Start")
    );
    assert_eq!(
        traced_bare_unreachable_function(MANOR, "EID_1100_a0040_01_End"),
        None
    );
    assert_eq!(traced_function_name("EID_1100_a0050_01_Start"), None);
}
