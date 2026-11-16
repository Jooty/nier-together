use replicant_game::GameMemory;
use replicant_game::host_led_defeat::{
    GAME_OVER_CONTEXT_RESULT_OFFSET, GAME_OVER_CONTEXT_VFTABLE_RVA, GAME_OVER_CONTINUE_RESULT,
    GAME_OVER_LEAVE_RESULT, GAME_OVER_ROOT_SEQUENCE_VFTABLE_RVA, GAME_OVER_SCENE_ID,
    GAME_OVER_SEQUENCE_CURSOR_OFFSET, GAME_OVER_SEQUENCE_DONE_STATE,
    GAME_OVER_SEQUENCE_STATE_OFFSET, GameOverScreen, GuestDefeatAct, GuestDefeatFollow,
    HostDefeatScreenWatch, UI_SCENE_CONTEXT_LIST_OFFSET, UI_SCENE_FIRST_CONTEXT_LINK_OFFSET,
    find_game_over_context, make_pick_on_screen, read_game_over_screen,
};
use replicant_game::ui_scenes::PAUSE_MENU_SCENE_ID;
use replicant_proto::{DefeatPick, HostDefeatScreen};

use crate::fake_memory::{FAKE_MODULE_BASE, FAKE_UI_SCENE_HEAP, FakeMemory};

const FRAME_MS: u64 = 16;
const RELAY_MS: u64 = 18;
const GUEST_SCREEN_OPENS_MS: u64 = 14_065;
const HOST_SCREEN_OPENS_MS: u64 = 14_194;
const HOST_PRESSES_CONTINUE_MS: u64 = 18_100;
const HOST_SCREEN_CLOSES_MS: u64 = 18_682;
const GUEST_SCREEN_STILL_OPEN_AT_LOG_END_MS: u64 = 116_257;
const OTHER_CONTEXT: u64 = FAKE_UI_SCENE_HEAP + 0x8_0000;
const GAME_OVER_CONTEXT: u64 = FAKE_UI_SCENE_HEAP + 0x9_0000;
const SEQUENCE: u64 = FAKE_UI_SCENE_HEAP + 0xA_0000;

fn screen(state: i32, cursor: i32) -> GameOverScreen {
    GameOverScreen { state, cursor }
}

fn host_state_at(now_ms: u64) -> i32 {
    match now_ms {
        _ if now_ms < HOST_PRESSES_CONTINUE_MS => 0,
        _ if now_ms < HOST_SCREEN_CLOSES_MS - 3 * FRAME_MS => 2,
        _ if now_ms < HOST_SCREEN_CLOSES_MS - 2 * FRAME_MS => 3,
        _ if now_ms < HOST_SCREEN_CLOSES_MS - FRAME_MS => 4,
        _ => 5,
    }
}

fn host_notes(cursor_at: impl Fn(u64) -> i32) -> Vec<(u64, HostDefeatScreen)> {
    let mut host = HostDefeatScreenWatch::new();
    let mut notes = Vec::new();
    let mut before = 0;
    let mut now = HOST_SCREEN_OPENS_MS;
    while now < HOST_SCREEN_CLOSES_MS {
        let after = screen(host_state_at(now), cursor_at(now));
        if let Some(note) = host.on_frame(before, after, now).send {
            notes.push((now + RELAY_MS, note));
        }
        before = after.state;
        now += FRAME_MS;
    }
    notes
}

fn guest_acts(
    guest: &mut GuestDefeatFollow,
    from_ms: u64,
    to_ms: u64,
) -> Vec<(u64, GuestDefeatAct)> {
    (from_ms..=to_ms)
        .step_by(FRAME_MS as usize)
        .map(|now| (now, guest.on_frame(screen(0, 0), true, now).act))
        .collect()
}

fn put_u64(memory: &mut FakeMemory, address: u64, value: u64) {
    memory.put(address, &value.to_le_bytes());
}

fn game_over_scene_memory() -> FakeMemory {
    let mut memory = FakeMemory::new();
    memory.put_ui_scenes(&[PAUSE_MENU_SCENE_ID, GAME_OVER_SCENE_ID], false);
    let scene = FAKE_UI_SCENE_HEAP + 0x1000;
    let ring_end = scene + UI_SCENE_CONTEXT_LIST_OFFSET;
    put_u64(&mut memory, ring_end, ring_end);
    put_u64(
        &mut memory,
        scene + UI_SCENE_FIRST_CONTEXT_LINK_OFFSET,
        OTHER_CONTEXT + 8,
    );
    put_u64(&mut memory, OTHER_CONTEXT, FAKE_MODULE_BASE + 0x123_4560);
    put_u64(&mut memory, OTHER_CONTEXT + 8, OTHER_CONTEXT);
    put_u64(&mut memory, OTHER_CONTEXT + 0x10, GAME_OVER_CONTEXT + 8);
    put_u64(
        &mut memory,
        GAME_OVER_CONTEXT,
        FAKE_MODULE_BASE + u64::from(GAME_OVER_CONTEXT_VFTABLE_RVA),
    );
    put_u64(&mut memory, GAME_OVER_CONTEXT + 8, GAME_OVER_CONTEXT);
    put_u64(&mut memory, GAME_OVER_CONTEXT + 0x10, ring_end);
    memory.put(GAME_OVER_CONTEXT + 0x20, &[0; 8]);
    put_u64(
        &mut memory,
        SEQUENCE,
        FAKE_MODULE_BASE + u64::from(GAME_OVER_ROOT_SEQUENCE_VFTABLE_RVA),
    );
    memory.put(
        SEQUENCE + GAME_OVER_SEQUENCE_CURSOR_OFFSET,
        &0_i32.to_le_bytes(),
    );
    memory.put(
        SEQUENCE + GAME_OVER_SEQUENCE_STATE_OFFSET,
        &0_i32.to_le_bytes(),
    );
    memory
}

#[test]
fn the_guests_screen_from_the_run_1116a_defeat_takes_no_buttons_and_makes_the_hosts_continue() {
    // G run0 08:47:14.065 and H 14.194 `ui scenes [1, 14]`; H 18.682 `ui scenes [1]` then a phase
    // load and EID_2000_a0070_Continue; G stayed on scene 14 to the log's end at 08:48:56.
    let mut notes = host_notes(|_| 0).into_iter().peekable();
    let mut guest = GuestDefeatFollow::new();
    let mut acts = Vec::new();
    let mut now = GUEST_SCREEN_OPENS_MS;
    while now < GUEST_SCREEN_STILL_OPEN_AT_LOG_END_MS {
        while let Some((_, note)) = notes.next_if(|(at, _)| *at <= now) {
            guest.hear(note, now);
        }
        let frame = guest.on_frame(screen(0, 0), true, now);
        acts.push((now, frame.act));
        if let GuestDefeatAct::MakeHostsPick(_) = frame.act {
            break;
        }
        now += FRAME_MS;
    }
    assert!(
        acts.iter()
            .all(|(_, act)| *act != GuestDefeatAct::OwnButtons),
        "the guest's own buttons never work while the host's screen leads"
    );
    let (picked_at, act) = acts.last().copied().unwrap();
    assert_eq!(act, GuestDefeatAct::MakeHostsPick(DefeatPick::Continue));
    assert!(picked_at <= HOST_SCREEN_CLOSES_MS + RELAY_MS + FRAME_MS);
    assert!(acts.iter().any(|(at, act)| *at < HOST_SCREEN_OPENS_MS
        && *act == GuestDefeatAct::ButtonsOff { show_cursor: None }));
    assert!(acts.iter().any(|(_, act)| *act
        == GuestDefeatAct::ButtonsOff {
            show_cursor: Some(0)
        }));
}

#[test]
fn the_hosts_pick_is_the_screens_own_result_write_and_its_highlight_is_sent_on_each_move() {
    let mut host = HostDefeatScreenWatch::new();
    let frames = [
        (0, 0, 0),
        (0, 0, 1),
        (0, 0, 2),
        (0, 0, 60),
        (1, 1, 61),
        (1, 2, 70),
        (2, 3, 100),
        (3, 4, 101),
        (4, 5, 102),
        (5, 5, 103),
    ];
    let mut sent = Vec::new();
    for (before, state, frame) in frames {
        let cursor = i32::from(frame >= 2);
        let at = 1_000 + frame * FRAME_MS;
        if let Some(note) = host.on_frame(before, screen(state, cursor), at).send {
            sent.push((frame, note));
        }
    }
    let note = |defeat, cursor, pick| HostDefeatScreen {
        defeat,
        cursor,
        pick,
    };
    assert_eq!(
        sent,
        vec![
            (0, note(1, 0, None)),
            (2, note(1, 1, None)),
            (70, note(1, 1, None)),
            (102, note(1, 1, Some(DefeatPick::Leave))),
        ]
    );
    let next = host.on_frame(0, screen(0, 0), 30_000).send.unwrap();
    assert_eq!((next.defeat, next.pick), (2, None));
}

#[test]
fn a_guest_only_defeat_gives_the_guest_its_own_buttons_after_the_wait_for_the_host() {
    let mut guest = GuestDefeatFollow::new();
    let acts = guest_acts(&mut guest, 5_000, 7_000);
    assert!(acts.iter().all(
        |(at, act)| (*at < 7_000) == (*act == GuestDefeatAct::ButtonsOff { show_cursor: None })
    ));
    assert_eq!(acts.last().unwrap().1, GuestDefeatAct::OwnButtons);
}

#[test]
fn a_host_screen_that_stops_sending_without_a_pick_gives_the_guest_its_buttons_back() {
    let mut guest = GuestDefeatFollow::new();
    guest.hear(
        HostDefeatScreen {
            defeat: 4,
            cursor: 1,
            pick: None,
        },
        1_000,
    );
    assert_eq!(
        guest.on_frame(screen(0, 0), true, 1_016).act,
        GuestDefeatAct::ButtonsOff {
            show_cursor: Some(1)
        }
    );
    let mut now = 1_032;
    while now <= 4_000 {
        assert_ne!(
            guest.on_frame(screen(0, 0), true, now).act,
            GuestDefeatAct::OwnButtons
        );
        now += FRAME_MS;
    }
    assert_eq!(
        guest.on_frame(screen(0, 0), true, 4_016).act,
        GuestDefeatAct::OwnButtons
    );
}

#[test]
fn a_host_only_defeat_drops_the_hosts_pick_and_never_makes_it_on_a_later_screen() {
    let mut guest = GuestDefeatFollow::new();
    let pick = HostDefeatScreen {
        defeat: 1,
        cursor: 0,
        pick: Some(DefeatPick::Continue),
    };
    guest.hear(pick, 1_000);
    assert!(guest.age(11_000).is_empty());
    assert_eq!(guest.age(11_001).len(), 1);
    guest.hear(pick, 11_010);
    let acts = guest_acts(&mut guest, 12_000, 14_000);
    assert!(
        acts.iter()
            .all(|(_, act)| *act != GuestDefeatAct::MakeHostsPick(DefeatPick::Continue))
    );
    assert_eq!(acts.last().unwrap().1, GuestDefeatAct::OwnButtons);
}

#[test]
fn a_host_pick_waits_while_the_guests_screen_is_not_taking_a_pick_or_its_result_word_is_missing() {
    let mut guest = GuestDefeatFollow::new();
    guest.hear(
        HostDefeatScreen {
            defeat: 2,
            cursor: 1,
            pick: Some(DefeatPick::Leave),
        },
        1_000,
    );
    assert_eq!(
        guest.on_frame(screen(2, 0), true, 1_016).act,
        GuestDefeatAct::ButtonsOff { show_cursor: None }
    );
    assert_eq!(
        guest.on_frame(screen(0, 0), false, 1_032).act,
        GuestDefeatAct::ButtonsOff { show_cursor: None }
    );
    assert_eq!(
        guest.on_frame(screen(0, 0), true, 1_048).act,
        GuestDefeatAct::MakeHostsPick(DefeatPick::Leave)
    );
}

#[test]
fn the_defeat_scenes_result_word_is_found_and_the_pick_is_written_as_the_screens_steps_write_it() {
    let mut memory = game_over_scene_memory();
    assert_eq!(
        read_game_over_screen(&memory, SEQUENCE).unwrap(),
        Some(screen(0, 0))
    );
    assert_eq!(read_game_over_screen(&memory, OTHER_CONTEXT).unwrap(), None);
    let context = find_game_over_context(&memory).unwrap().unwrap();
    assert_eq!(context, GAME_OVER_CONTEXT);
    make_pick_on_screen(&mut memory, SEQUENCE, context, DefeatPick::Continue).unwrap();
    assert_eq!(
        memory
            .read_i32(context + GAME_OVER_CONTEXT_RESULT_OFFSET)
            .unwrap(),
        GAME_OVER_CONTINUE_RESULT
    );
    assert_eq!(
        read_game_over_screen(&memory, SEQUENCE).unwrap(),
        Some(screen(GAME_OVER_SEQUENCE_DONE_STATE, 0))
    );
    make_pick_on_screen(&mut memory, SEQUENCE, context, DefeatPick::Leave).unwrap();
    assert_eq!(
        memory
            .read_i32(context + GAME_OVER_CONTEXT_RESULT_OFFSET)
            .unwrap(),
        GAME_OVER_LEAVE_RESULT
    );
}

#[test]
fn no_defeat_scene_means_no_result_word() {
    let mut memory = FakeMemory::new();
    memory.put_ui_scenes(&[PAUSE_MENU_SCENE_ID], false);
    assert_eq!(find_game_over_context(&memory).unwrap(), None);
}
