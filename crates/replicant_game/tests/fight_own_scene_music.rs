use replicant_game::fight_sync::{
    GuestFightStream, GuestNotice, GuestPop, HostFightStream, HostStreamOut, PopSource, SlotState,
    StreamStall, format_stream_stall,
};
use replicant_proto::{Message, decode, encode};

const PACK: &str = "A_CENTER_VILLAGE_01";
const PHASE: u16 = 1;
const HERE: Option<u16> = Some(PHASE);
const FIGHT_START: &str = "EID_2000_a0050_Start";
const SCENE_START: &str = "EID_2000_a0082_Start";
const SCENE_MUSIC: &str = "BGM_EID_2000_a0082_01";
const SCENE_END: &str = "EID_2000_a0082_End";
const STREAM_SLOT: u64 = 0x1_4130_2070;
const OWN_TIMER_SLOT: u64 = 0x1_4130_2990;
const OWN_END_SLOT: u64 = 0x1_4130_2b40;
const SCENE_START_RAN_MS: u64 = 1_000;
const N3_HOSTS_MUSIC_HELD_MS: u64 = 7_314;
const N3_OWN_MUSIC_POPPED_MS: u64 = 7_330;
const N3_NEXT_TICK_MS: u64 = 7_346;
const SCENE_CLEARED_MS: u64 = 20_000;
const OWN_SCENE_END_POPPED_MS: u64 = 20_100;

fn over_the_wire(message: Message) -> Message {
    assert_eq!(message.validate(), Ok(()));
    decode(&encode(&message).unwrap()).unwrap()
}

fn messages(outs: &[HostStreamOut]) -> Vec<Message> {
    outs.iter()
        .filter_map(HostStreamOut::message)
        .map(over_the_wire)
        .collect()
}

fn host_in_the_fight() -> (HostFightStream, Vec<Message>) {
    let mut host = HostFightStream::new();
    host.enter_phase(PHASE, PACK);
    let mut outs = host.popped(FIGHT_START, Vec::new(), false);
    outs.extend(host.callback_registered());
    outs.extend(host.script_slot(SlotState::Idle));
    let stream = messages(&outs);
    (host, stream)
}

fn host_runs(host: &mut HostFightStream, name: &str) -> Vec<Message> {
    let mut outs = host.popped(name, Vec::new(), false);
    outs.extend(host.script_slot(SlotState::Idle));
    messages(&outs)
}

fn released(guest: &mut GuestFightStream, now_ms: u64, ran: &mut Vec<String>) -> Option<String> {
    let entry = guest.tick(SlotState::Idle, HERE, false, now_ms)?;
    guest.entry_queued_at(STREAM_SLOT);
    ran.push(entry.name.clone());
    Some(entry.name)
}

fn receive(guest: &mut GuestFightStream, stream: Vec<Message>) {
    for message in stream {
        guest.receive_message(message, HERE);
    }
}

fn guest_after_the_scene_start(stream: Vec<Message>, ran: &mut Vec<String>) -> GuestFightStream {
    let mut guest = GuestFightStream::new();
    receive(&mut guest, stream);
    assert_eq!(released(&mut guest, 0, ran).as_deref(), Some(FIGHT_START));
    assert_eq!(
        guest.pop(FIGHT_START, PopSource::Queued, STREAM_SLOT, HERE, false),
        GuestPop::RunStartEntry
    );
    assert_eq!(
        released(&mut guest, SCENE_START_RAN_MS, ran).as_deref(),
        Some(SCENE_START)
    );
    assert!(matches!(
        guest.pop(SCENE_START, PopSource::Queued, STREAM_SLOT, HERE, false),
        GuestPop::RunReplayed { .. }
    ));
    guest.take_notices();
    guest
}

fn guest_in_its_own_scene(stream: Vec<Message>, ran: &mut Vec<String>) -> GuestFightStream {
    let mut guest = guest_after_the_scene_start(stream, ran);
    guest.set_own_scene_playing(true);
    guest
}

fn own_music_pop(guest: &mut GuestFightStream) -> GuestPop {
    guest.pop(SCENE_MUSIC, PopSource::Fired, OWN_TIMER_SLOT, HERE, false)
}

fn scene_ends_and_its_own_end_pops(guest: &mut GuestFightStream, ran: &mut Vec<String>) {
    guest.set_own_scene_playing(false);
    assert_eq!(released(guest, SCENE_CLEARED_MS, ran), None);
    assert_eq!(
        guest.pop(SCENE_END, PopSource::Queued, OWN_END_SLOT, HERE, false),
        GuestPop::Hold
    );
    assert_eq!(
        released(guest, OWN_SCENE_END_POPPED_MS, ran).as_deref(),
        Some(SCENE_END)
    );
    assert_eq!(released(guest, OWN_SCENE_END_POPPED_MS + 16, ran), None);
}

#[test]
fn the_guests_own_scene_music_runs_at_its_scene_time_and_the_hosts_held_copy_is_dropped() {
    // N3 (2026-11-16-102331) guest 15:27:07.314: the host's BGM_EID_2000_a0082_01 (seq 25) waits
    // for the guest's own a0082 scene; at 07.330 the guest's own copy popped and was dropped as
    // Hold, so the scene's music started only after the scene ended.
    let (mut host, mut stream) = host_in_the_fight();
    for name in [SCENE_START, SCENE_MUSIC, SCENE_END] {
        stream.extend(host_runs(&mut host, name));
    }
    let mut ran = Vec::new();
    let mut guest = guest_in_its_own_scene(stream, &mut ran);
    assert_eq!(released(&mut guest, N3_HOSTS_MUSIC_HELD_MS, &mut ran), None);
    let Some((music_seq, _)) = guest.next_waiting_entry() else {
        panic!("the host's music entry waits");
    };
    assert!(guest.is_held_for_own_scene(music_seq));
    assert!(matches!(
        guest.take_notices().as_slice(),
        [GuestNotice::EntryWaitsForOwnScene { name, .. }] if name == SCENE_MUSIC
    ));
    assert_eq!(own_music_pop(&mut guest), GuestPop::RunMusicInOwnScene);
    assert_eq!(released(&mut guest, N3_NEXT_TICK_MS, &mut ran), None);
    assert!(matches!(
        guest.take_notices().as_slice(),
        [GuestNotice::HostCopyOfOwnSceneMusicDropped { seq, name }]
            if *seq == music_seq && name == SCENE_MUSIC
    ));
    assert!(!guest.is_held_for_own_scene(music_seq));
    scene_ends_and_its_own_end_pops(&mut guest, &mut ran);
    assert_eq!(ran, [FIGHT_START, SCENE_START, SCENE_END]);
}

#[test]
fn the_hosts_copy_that_comes_after_the_guests_own_scene_music_ran_is_dropped() {
    let (mut host, mut stream) = host_in_the_fight();
    stream.extend(host_runs(&mut host, SCENE_START));
    let mut ran = Vec::new();
    let mut guest = guest_in_its_own_scene(stream, &mut ran);
    assert_eq!(own_music_pop(&mut guest), GuestPop::RunMusicInOwnScene);
    let mut later = host_runs(&mut host, SCENE_MUSIC);
    later.extend(host_runs(&mut host, SCENE_END));
    receive(&mut guest, later);
    assert_eq!(released(&mut guest, N3_OWN_MUSIC_POPPED_MS, &mut ran), None);
    assert_eq!(released(&mut guest, N3_NEXT_TICK_MS, &mut ran), None);
    assert!(matches!(
        guest.take_notices().as_slice(),
        [GuestNotice::HostCopyOfOwnSceneMusicDropped { name, .. }, ..] if name == SCENE_MUSIC
    ));
    scene_ends_and_its_own_end_pops(&mut guest, &mut ran);
    assert_eq!(ran, [FIGHT_START, SCENE_START, SCENE_END]);
}

#[test]
fn outside_a_scene_of_the_guests_own_its_music_pop_is_held_and_the_hosts_copy_runs() {
    let (mut host, mut stream) = host_in_the_fight();
    for name in [SCENE_START, SCENE_MUSIC] {
        stream.extend(host_runs(&mut host, name));
    }
    let mut ran = Vec::new();
    let mut guest = guest_after_the_scene_start(stream, &mut ran);
    assert_eq!(own_music_pop(&mut guest), GuestPop::Hold);
    assert_eq!(
        released(&mut guest, N3_NEXT_TICK_MS, &mut ran).as_deref(),
        Some(SCENE_MUSIC)
    );
}

#[test]
fn the_stall_line_names_the_own_scene_hold_when_it_holds_the_entry() {
    let stall = StreamStall {
        seq: 25,
        name: SCENE_MUSIC,
        waited_ms: 5_010,
        holder: None,
        held_by_own_scene: true,
        last_yielding_native: None,
        radio: None,
    };
    assert_eq!(
        format_stream_stall(&stall),
        "fight stream: seq 25 BGM_EID_2000_a0082_01 waits behind the hold for your game's own scene, until its end pops (in the slot ? ms, no yielding native since the pop, radio unread), waited 5010 ms"
    );
}
