use replicant_game::dialogue::{
    MirrorAction, MirrorInput, OWN_STEP_COPY_AWAITED_MS, OwnStepCopyEvent, OwnStepCopyReading,
    OwnStepCopyWait, OwnTalkWindow, PartnerTalkMemory, PartnerTalkMirror, TalkLeader,
};
use replicant_proto::{DialogueLine, DialogueUpdate};

const FORCED_TALK: u32 = 1328;
const FORCED_TALK_LAST_LINE: u16 = 5;
const DREAM_FIRST_TALK: u32 = 1422;
const DREAM_SECOND_TALK: u32 = 1423;
const DREAM_START: &str = "EID_1500_a0005_010_01_Start";
const FORCED_TALK_OPEN_MS: u64 = 30_476;
const HOST_LINE_TIMES_MS: [u64; 5] = [31_246, 32_067, 32_494, 32_920, 33_399];
const HOST_FORCED_TALK_ENDED_MS: u64 = 34_027;
const HOST_DREAM_TALK_ARRIVES_MS: u64 = 40_542;

fn line(talk_id: u32, line: u16) -> DialogueLine {
    DialogueLine {
        talk_id,
        line,
        step: 0,
    }
}

fn guest_tick(
    mirror: &mut PartnerTalkMirror,
    host: &PartnerTalkMemory,
    own: Option<DialogueLine>,
    now_ms: u64,
) -> Option<MirrorAction> {
    let leader = if own.is_some() {
        TalkLeader::Partner
    } else {
        TalkLeader::Either
    };
    mirror.step(MirrorInput {
        partner_line: host.for_mirror(own, leader, true),
        partner_ended_talk: host.ended_for_follow(own, leader, now_ms),
        can_show: true,
        window_open: true,
        own_talk: own,
        leader,
        answer_open: false,
        now_ms,
    })
}

fn reading(
    host_story_talk: Option<u32>,
    own_step: Option<&str>,
    own_copy_open: bool,
    now_ms: u64,
) -> OwnStepCopyReading<'_> {
    OwnStepCopyReading {
        host_story_talk,
        story_step_running_here: own_step,
        own_copy_open,
        mirror_showing: false,
        now_ms,
    }
}

#[test]
fn run_1116a_the_talk_a_story_steps_talk_command_opened_is_the_guests_own_copy_and_the_host_leads_it()
 {
    // Run 1116a guest 08:29:30.476: EID_1500_a0005_040_Select01 forced talk 1328 ("own talk
    // started: false"), so the copy was neither followed nor mirrored and sat on line 0 until
    // the guest's own presses at 08:29:45-46, 12.7 s after the host had left it.
    let mut unmarked = OwnTalkWindow::new();
    assert!(!unmarked.observe(true, false));
    let mut host = PartnerTalkMemory::new();
    host.receive(
        DialogueUpdate::StoryLine(line(FORCED_TALK, 0)),
        FORCED_TALK_OPEN_MS,
    );
    let mut unled = PartnerTalkMirror::new();
    assert_eq!(
        guest_tick(&mut unled, &host, None, FORCED_TALK_OPEN_MS),
        None
    );

    let mut window = OwnTalkWindow::new();
    window.note_opened_by_a_story_steps_talk_command();
    assert!(window.observe(true, false));
    let mut mirror = PartnerTalkMirror::new();
    let own = Some(line(FORCED_TALK, 0));
    assert_eq!(
        guest_tick(&mut mirror, &host, own, FORCED_TALK_OPEN_MS),
        Some(MirrorAction::FollowStarted(FORCED_TALK))
    );
    for (index, at_ms) in HOST_LINE_TIMES_MS.into_iter().enumerate() {
        let host_line = u16::try_from(index + 1).unwrap();
        host.receive(
            DialogueUpdate::StoryLine(line(FORCED_TALK, host_line)),
            at_ms,
        );
        assert!(window.observe(true, false));
        assert_eq!(
            guest_tick(&mut mirror, &host, own, at_ms),
            Some(MirrorAction::FollowLine(host_line))
        );
    }
    host.receive(DialogueUpdate::Ended, HOST_FORCED_TALK_ENDED_MS);
    let own_on_the_last_line = Some(line(FORCED_TALK, FORCED_TALK_LAST_LINE));
    assert_eq!(
        guest_tick(
            &mut mirror,
            &host,
            own_on_the_last_line,
            HOST_FORCED_TALK_ENDED_MS
        ),
        Some(MirrorAction::FinishOwnTalk(FORCED_TALK))
    );
    assert!(!window.observe(false, false));
    assert!(!window.observe(true, false));
}

#[test]
fn run_1116a_the_hosts_dream_talk_waits_for_the_copy_the_guests_own_step_opens() {
    // Run 1116a guest: the mirror opened 1422 itself at 08:29:46.876, at the guest's own window
    // size (a plain window), and its hold stopped the guest's own step's `_Wait(1.0)` before
    // `_SetTalk(1422)` until the host's talk was over (08:30:22.066 -> 08:30:23.053).
    let mut wait = OwnStepCopyWait::new();
    assert_eq!(
        wait.step(&reading(
            Some(DREAM_FIRST_TALK),
            Some(DREAM_START),
            false,
            HOST_DREAM_TALK_ARRIVES_MS
        )),
        (
            true,
            Some(OwnStepCopyEvent::Waits {
                talk_id: DREAM_FIRST_TALK,
                step: DREAM_START.to_owned(),
            })
        )
    );
    assert_eq!(
        wait.step(&reading(
            Some(DREAM_FIRST_TALK),
            Some(DREAM_START),
            false,
            HOST_DREAM_TALK_ARRIVES_MS + 200
        )),
        (true, None)
    );
    assert_eq!(
        wait.step(&reading(
            Some(DREAM_FIRST_TALK),
            Some(DREAM_START),
            true,
            HOST_DREAM_TALK_ARRIVES_MS + 350
        )),
        (
            false,
            Some(OwnStepCopyEvent::OwnCopyCame {
                talk_id: DREAM_FIRST_TALK,
                waited_ms: 350,
            })
        )
    );
}

#[test]
fn a_step_that_never_opens_the_hosts_talk_lets_it_open_here_after_the_wait_once() {
    let mut wait = OwnStepCopyWait::new();
    let at = |now_ms| reading(Some(DREAM_FIRST_TALK), Some(DREAM_START), false, now_ms);
    assert!(wait.step(&at(0)).0);
    assert!(wait.step(&at(OWN_STEP_COPY_AWAITED_MS - 1)).0);
    assert_eq!(
        wait.step(&at(OWN_STEP_COPY_AWAITED_MS)),
        (
            false,
            Some(OwnStepCopyEvent::GaveUp {
                talk_id: DREAM_FIRST_TALK,
                step: DREAM_START.to_owned(),
                waited_ms: OWN_STEP_COPY_AWAITED_MS,
            })
        )
    );
    assert_eq!(wait.step(&at(OWN_STEP_COPY_AWAITED_MS + 16)), (false, None));
}

#[test]
fn the_wait_keeps_its_first_clock_when_the_host_chains_on_to_its_next_talk() {
    let mut wait = OwnStepCopyWait::new();
    assert!(
        wait.step(&reading(
            Some(DREAM_FIRST_TALK),
            Some(DREAM_START),
            false,
            0
        ))
        .0
    );
    assert!(
        wait.step(&reading(
            Some(DREAM_SECOND_TALK),
            Some(DREAM_START),
            false,
            OWN_STEP_COPY_AWAITED_MS / 2
        ))
        .0
    );
    assert!(matches!(
        wait.step(&reading(
            Some(DREAM_SECOND_TALK),
            Some(DREAM_START),
            false,
            OWN_STEP_COPY_AWAITED_MS
        )),
        (
            false,
            Some(OwnStepCopyEvent::GaveUp {
                talk_id: DREAM_SECOND_TALK,
                ..
            })
        )
    ));
}

#[test]
fn a_guest_running_no_story_step_or_already_showing_the_talk_is_not_held() {
    let mut wait = OwnStepCopyWait::new();
    assert_eq!(
        wait.step(&reading(Some(DREAM_FIRST_TALK), None, false, 0)),
        (false, None)
    );
    let showing = OwnStepCopyReading {
        mirror_showing: true,
        ..reading(Some(DREAM_FIRST_TALK), Some(DREAM_START), false, 16)
    };
    assert_eq!(wait.step(&showing), (false, None));
    assert_eq!(
        wait.step(&reading(None, Some(DREAM_START), false, 32)),
        (false, None)
    );
}
