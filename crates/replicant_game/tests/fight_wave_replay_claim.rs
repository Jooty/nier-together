use replicant_game::fight_sync::{
    GuestFightStream, GuestPop, HostFightStream, HostStreamOut, PopSource, SlotState,
};
use replicant_proto::{Message, decode, encode};

const PACK: &str = "A_CENTER_VILLAGE_01";
const PHASE: u16 = 1;
const HERE: Option<u16> = Some(PHASE);
const FIGHT_START: &str = "EID_2000_a0050_Start";
const WAVE_CALL: &str = "PUT_ENEMY_02";
const QUEUED_SLOT: u64 = 0x1_4130_2070;
const OTHER_QUEUED_SLOT: u64 = 0x1_4130_2818;
const FIRED_SLOT: u64 = 0x1_4122_ec08;
const REFUSALS_BEFORE_THE_CLAIM: usize = 3;
const START_RAN_MS: u64 = 10_000;
const FIRST_WAVE_MS: u64 = 20_000;
const SECOND_WAVE_MS: u64 = 20_100;

fn over_the_wire(message: Message) -> Message {
    assert_eq!(message.validate(), Ok(()));
    decode(&encode(&message).unwrap()).unwrap()
}

fn guest_with_two_waves_queued_behind_the_start() -> GuestFightStream {
    let mut host = HostFightStream::new();
    host.enter_phase(PHASE, PACK);
    let mut outs = host.popped(FIGHT_START, Vec::new(), false);
    outs.extend(host.callback_registered());
    outs.extend(host.script_slot(SlotState::Idle));
    for _ in 0..2 {
        outs.extend(host.popped(WAVE_CALL, Vec::new(), false));
        outs.extend(host.script_slot(SlotState::Idle));
    }
    let mut guest = GuestFightStream::new();
    for message in outs
        .iter()
        .filter_map(HostStreamOut::message)
        .map(over_the_wire)
    {
        guest.receive_message(message, HERE);
    }
    assert!(guest.tick(SlotState::Idle, HERE, false, 0).is_some());
    guest.entry_queued_at(QUEUED_SLOT);
    assert_eq!(
        guest.pop(FIGHT_START, PopSource::Queued, QUEUED_SLOT, HERE, false),
        GuestPop::RunStartEntry
    );
    guest
}

fn refuse_the_games_own_wave_calls(guest: &mut GuestFightStream) {
    for _ in 0..REFUSALS_BEFORE_THE_CLAIM {
        assert_eq!(
            guest.pop(WAVE_CALL, PopSource::Fired, FIRED_SLOT, HERE, false),
            GuestPop::Hold
        );
    }
}

fn guest_with_the_first_wave_queued() -> GuestFightStream {
    let mut guest = guest_with_two_waves_queued_behind_the_start();
    let entry = guest
        .tick(SlotState::Idle, HERE, false, START_RAN_MS)
        .expect("the first wave is released");
    assert_eq!(entry.name, WAVE_CALL);
    guest.entry_queued_at(QUEUED_SLOT);
    guest
}

#[test]
fn the_games_own_fired_wave_call_runs_as_the_queued_host_wave_once_refused_three_times() {
    let mut guest = guest_with_the_first_wave_queued();
    refuse_the_games_own_wave_calls(&mut guest);
    let verdict = guest.pop(WAVE_CALL, PopSource::Fired, FIRED_SLOT, HERE, false);
    assert!(
        matches!(verdict, GuestPop::RunReplayed { .. }),
        "{verdict:?}"
    );
}

#[test]
fn a_queued_wave_call_in_another_slot_is_still_held() {
    let mut guest = guest_with_the_first_wave_queued();
    assert_eq!(
        guest.pop(WAVE_CALL, PopSource::Queued, OTHER_QUEUED_SLOT, HERE, false),
        GuestPop::Hold
    );
}

#[test]
fn an_object_armed_wave_call_runs_as_the_queued_host_wave_once_refused_three_times() {
    let mut guest = guest_with_the_first_wave_queued();
    for _ in 0..REFUSALS_BEFORE_THE_CLAIM {
        assert_eq!(
            guest.pop_by_player(WAVE_CALL, Vec::new(), HERE, false, FIRST_WAVE_MS),
            GuestPop::Hold
        );
    }
    let verdict = guest.pop_by_player(WAVE_CALL, Vec::new(), HERE, false, FIRST_WAVE_MS);
    assert!(
        matches!(verdict, GuestPop::RunReplayed { .. }),
        "{verdict:?}"
    );
}

#[test]
fn the_next_wave_is_released_after_the_claimed_one_ran() {
    let mut guest = guest_with_the_first_wave_queued();
    refuse_the_games_own_wave_calls(&mut guest);
    assert!(matches!(
        guest.pop(WAVE_CALL, PopSource::Fired, FIRED_SLOT, HERE, false),
        GuestPop::RunReplayed { seq: 1 }
    ));
    let entry = guest.tick(SlotState::Idle, HERE, false, SECOND_WAVE_MS);
    assert_eq!(entry.map(|entry| entry.name).as_deref(), Some(WAVE_CALL));
}

#[test]
fn a_fired_call_of_another_name_is_still_held() {
    let mut guest = guest_with_the_first_wave_queued();
    assert_eq!(
        guest.pop("PUT_ENEMY_03", PopSource::Fired, FIRED_SLOT, HERE, false),
        GuestPop::Hold
    );
}
