use replicant_game::fight_sync::{
    FIGHT_WINDOWS, HostFightStream, HostRequestOut, HostStreamOut, PopSource, Replayer,
    RequestReceipt, SlotState, StreamEntry,
};
use replicant_proto::{FightRequest, FightRequestKind};

const PHASE: u16 = 6;
const EPOCH: u32 = 3;
const SCENE_END: &str = "EID_1450_a0080_End";
const NEXT_ENTRY: &str = "EID_1450_a0083g_End";

fn entry(seq: u64, name: &str) -> StreamEntry {
    StreamEntry {
        phase: PHASE,
        epoch: EPOCH,
        seq,
        name: name.to_owned(),
        args: Vec::new(),
    }
}

#[test]
fn a_guest_does_not_run_a_repeated_stream_end_of_a_scene_it_already_ended_in_the_fight() {
    let mut replayer = Replayer::new(Vec::<String>::new());
    replayer.receive(entry(0, SCENE_END));
    replayer.receive(entry(1, SCENE_END));
    replayer.receive(entry(2, NEXT_ENTRY));
    let first = replayer
        .tick(SlotState::Idle)
        .expect("the first End is released");
    assert_eq!((first.seq, first.name.as_str()), (0, SCENE_END));
    replayer.entry_queued_at(7);
    replayer.pop(SCENE_END, PopSource::Queued, 7);
    assert!(replayer.tick(SlotState::Busy).is_none());
    let next = replayer
        .tick(SlotState::Idle)
        .expect("the entry after the repeat is released");
    assert_eq!((next.seq, next.name.as_str()), (2, NEXT_ENTRY));
}

#[test]
fn a_new_epoch_runs_the_same_end_again() {
    let mut replayer = Replayer::new(Vec::<String>::new());
    replayer.receive(entry(0, SCENE_END));
    assert!(replayer.tick(SlotState::Idle).is_some());
    let mut again = entry(0, SCENE_END);
    again.epoch = EPOCH + 1;
    replayer.receive(again);
    assert_eq!(replayer.tick(SlotState::Idle).map(|e| e.seq), Some(0));
}

#[test]
fn the_host_knows_a_name_its_stream_already_ran_in_this_fight() {
    let window = FIGHT_WINDOWS
        .iter()
        .find(|window| window.pack == "A_DESERT_TEMPLE_02")
        .expect("the desert temple fight window exists");
    let mut host = HostFightStream::new();
    host.enter_phase(PHASE, window.pack);
    let start = window.start_ids.first().copied().expect("a start id");
    host.popped(start, Vec::new(), false);
    host.callback_registered();
    assert!(!host.streamed_in_this_fight(SCENE_END));
    let outs = host.popped(SCENE_END, Vec::new(), false);
    assert!(
        outs.iter()
            .any(|out| matches!(out, HostStreamOut::Entry(_)))
    );
    assert!(host.streamed_in_this_fight(SCENE_END));
}

#[test]
fn run_135601_the_host_does_not_run_a_guest_request_for_an_end_it_ran_just_before_the_window_opened()
 {
    const LIBRARY_PHASE: u16 = 7;
    const FIGHT_END: &str = "EID_2000_a0100_End";
    let window = FIGHT_WINDOWS
        .iter()
        .find(|window| window.window_functions.contains(&FIGHT_END))
        .expect("a fight window lists the library fight's End");
    let mut host = HostFightStream::new();
    host.enter_phase(LIBRARY_PHASE, window.pack);
    host.popped(FIGHT_END, Vec::new(), false);
    let start = window.start_ids.first().copied().expect("a start id");
    host.popped(start, Vec::new(), false);
    host.callback_registered();
    assert!(host.is_open_in(LIBRARY_PHASE));
    let request = FightRequest {
        phase: LIBRARY_PHASE,
        epoch: host.epoch(),
        kind: FightRequestKind::RunPop {
            name: FIGHT_END.to_owned(),
            args: Vec::new(),
            copy_number: 1,
        },
    };
    assert_eq!(
        host.receive_request(request),
        HostRequestOut::Run(RequestReceipt::AlreadyRanHere)
    );
}
