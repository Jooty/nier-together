use replicant_game::fight_sync::{
    GuestFightStream, GuestNotice, GuestPop, HostFightStream, HostStreamOut, PopSource, SlotState,
};
use replicant_proto::{Message, decode, encode};

const PACK: &str = "A_CENTER_VILLAGE_01";
const PHASE: u16 = 1;
const HERE: Option<u16> = Some(PHASE);
const FIGHT_START: &str = "EID_2000_a0050_Start";
const SCENE_START: &str = "EID_2000_a0080_Start";
const SCENE_END: &str = "EID_2000_a0080_End";
const NEXT_SCENE_START: &str = "EID_2000_a0082_Start";
const STREAM_SLOT: u64 = 0x1_4130_2070;
const SCENE_START_RAN_MS: u64 = 10_000;
const HOST_SCENE_END_QUEUED_MS: u64 = 32_911;
const GUEST_SCENE_OVER_MS: u64 = 32_928;

fn over_the_wire(message: Message) -> Message {
    assert_eq!(message.validate(), Ok(()));
    decode(&encode(&message).unwrap()).unwrap()
}

fn host_stream_of_the_fight() -> Vec<Message> {
    let mut host = HostFightStream::new();
    host.enter_phase(PHASE, PACK);
    let mut outs = host.popped(FIGHT_START, Vec::new(), false);
    outs.extend(host.callback_registered());
    outs.extend(host.script_slot(SlotState::Idle));
    for name in [SCENE_START, SCENE_END, NEXT_SCENE_START] {
        outs.extend(host.popped(name, Vec::new(), false));
        outs.extend(host.script_slot(SlotState::Idle));
    }
    outs.iter()
        .filter_map(HostStreamOut::message)
        .map(over_the_wire)
        .collect()
}

fn released(guest: &mut GuestFightStream, now_ms: u64) -> Option<String> {
    let entry = guest.tick(SlotState::Idle, HERE, false, now_ms)?;
    guest.entry_queued_at(STREAM_SLOT);
    Some(entry.name)
}

fn guest_in_the_scene_the_stream_started() -> GuestFightStream {
    let mut guest = GuestFightStream::new();
    for message in host_stream_of_the_fight() {
        guest.receive_message(message, HERE);
    }
    assert_eq!(released(&mut guest, 0).as_deref(), Some(FIGHT_START));
    assert_eq!(
        guest.pop(FIGHT_START, PopSource::Queued, STREAM_SLOT, HERE, false),
        GuestPop::RunStartEntry
    );
    assert_eq!(
        released(&mut guest, SCENE_START_RAN_MS).as_deref(),
        Some(SCENE_START)
    );
    assert!(matches!(
        guest.pop(SCENE_START, PopSource::Queued, STREAM_SLOT, HERE, false),
        GuestPop::RunReplayed { .. }
    ));
    guest.take_notices();
    guest
}

#[test]
fn the_hosts_scene_end_waits_until_the_guests_own_scene_has_ended() {
    // L35 (build b1c7351525f0) guest 11:52:32.912: "fight stream: the host's EID_2000_a0080_End
    // (seq 45) runs here" while the guest's own scene still played (its lend ended at 32.928). That
    // end loads the next cut inline; at a0082 the guest died at rva 0x3FE7E0 with no cut loaded.
    let mut guest = guest_in_the_scene_the_stream_started();
    guest.set_own_scene_playing(true);
    for now_ms in (HOST_SCENE_END_QUEUED_MS..GUEST_SCENE_OVER_MS).step_by(16) {
        assert_eq!(released(&mut guest, now_ms), None);
    }
    assert!(matches!(
        guest.take_notices().as_slice(),
        [GuestNotice::EntryWaitsForOwnScene { name, .. }] if name == SCENE_END
    ));
    guest.set_own_scene_playing(false);
    assert_eq!(
        released(&mut guest, GUEST_SCENE_OVER_MS).as_deref(),
        Some(SCENE_END)
    );
    assert!(matches!(
        guest.take_notices().as_slice(),
        [GuestNotice::EntryGoesAfterOwnScene { name, waited_ms, .. }]
            if name == SCENE_END && *waited_ms == GUEST_SCENE_OVER_MS - HOST_SCENE_END_QUEUED_MS
    ));
}

#[test]
fn the_wait_for_the_guests_scene_is_logged_once_however_long_the_scene_lasts() {
    let mut guest = guest_in_the_scene_the_stream_started();
    guest.set_own_scene_playing(true);
    for tick in 0..2_000 {
        assert_eq!(
            released(&mut guest, HOST_SCENE_END_QUEUED_MS + tick * 16),
            None
        );
    }
    assert_eq!(guest.take_notices().len(), 1);
}

#[test]
fn a_guest_with_no_scene_of_its_own_runs_the_hosts_scene_end_at_once() {
    let mut guest = guest_in_the_scene_the_stream_started();
    guest.set_own_scene_playing(false);
    assert_eq!(
        released(&mut guest, HOST_SCENE_END_QUEUED_MS).as_deref(),
        Some(SCENE_END)
    );
    assert_eq!(guest.take_notices(), Vec::new());
}
