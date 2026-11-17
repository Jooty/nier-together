use replicant_game::fight_sync::{
    FIGHT_WINDOWS, GuestFightStream, GuestPop, HostFightStream, HostStreamOut, PopSource, SlotState,
};
use replicant_game::script_functions::SCRIPT_FUNCTIONS;
use replicant_proto::Message;

const PACK: &str = "A_CENTER_LIBRARY_01";
const PHASE: u16 = 7;
const HERE: Option<u16> = Some(PHASE);
const WAVE_END: &str = "EID_2000_a0090g_End";
const FIGHT_START: &str = "EID_2000_a0100_Start";
const SECOND_PART_START: &str = "EID_2000_a0129g_Start";
const SECOND_PART_MIDDLE: &str = "EID_2000_a0120_Start";
const FIRST_PART_CONTINUE: &str = "EID_2000_a0100_Continue";
const OTHER_PACK_FUNCTION: &str = "EID_2000_a0105g_End";
const OWN_SLOT: u64 = 0x1_4130_2070;

struct HookedHost {
    host: HostFightStream,
    streamed: Vec<HostStreamOut>,
    dropped: Vec<String>,
}

impl HookedHost {
    fn new() -> Self {
        let mut host = HostFightStream::new();
        host.enter_phase(PHASE, PACK);
        Self {
            host,
            streamed: Vec::new(),
            dropped: Vec::new(),
        }
    }

    fn pops(&mut self, name: &str) {
        if self.host.is_open_in(PHASE) && self.host.repeats_start_chain(name).is_some() {
            self.dropped.push(name.to_owned());
            return;
        }
        let outs = self.host.popped(name, Vec::new(), false);
        self.streamed.extend(outs);
    }

    fn registers_a_callback(&mut self) {
        let outs = self.host.callback_registered();
        self.streamed.extend(outs);
    }

    fn streamed_names(&self) -> Vec<&str> {
        self.streamed
            .iter()
            .filter_map(|out| match out {
                HostStreamOut::Entry(entry) => Some(entry.name.as_str()),
                _ => None,
            })
            .collect()
    }

    fn messages(&self) -> Vec<Message> {
        self.streamed
            .iter()
            .filter_map(HostStreamOut::message)
            .collect()
    }
}

fn host_through_the_first_boss_intro() -> HookedHost {
    let mut hooked = HookedHost::new();
    hooked.pops(WAVE_END);
    hooked.pops(FIGHT_START);
    hooked.registers_a_callback();
    hooked.pops(OTHER_PACK_FUNCTION);
    hooked.pops(SECOND_PART_MIDDLE);
    hooked.pops(SECOND_PART_START);
    hooked
}

#[test]
fn the_wave_end_popping_again_after_the_boss_fight_began_does_not_start_the_chain_twice() {
    let mut hooked = host_through_the_first_boss_intro();
    hooked.pops(WAVE_END);
    assert_eq!(hooked.dropped, [WAVE_END]);
    let starts = hooked
        .streamed_names()
        .iter()
        .filter(|name| **name == FIGHT_START)
        .count();
    assert_eq!(starts, 1, "{:?}", hooked.streamed_names());
    assert!(!hooked.streamed_names().contains(&WAVE_END));
}

#[test]
fn a_repeat_of_the_start_itself_is_dropped_too() {
    let mut hooked = host_through_the_first_boss_intro();
    hooked.pops(FIGHT_START);
    assert_eq!(hooked.dropped, [FIGHT_START]);
}

#[test]
fn the_first_wave_end_before_the_window_opens_runs() {
    let hooked = HookedHost::new();
    let mut host = hooked.host;
    assert!(host.repeats_start_chain(WAVE_END).is_none());
    assert!(host.open_window().is_none());
}

#[test]
fn the_guest_gets_no_second_start_and_holds_its_own_games_repeat() {
    let mut hooked = host_through_the_first_boss_intro();
    hooked.pops(WAVE_END);
    let mut guest = GuestFightStream::new();
    for message in hooked.messages() {
        guest.receive_message(message, HERE);
    }
    let mut starts_replayed = 0;
    let mut now_ms = 0;
    while let Some(entry) = guest.tick(SlotState::Idle, HERE, false, now_ms) {
        guest.entry_queued_at(OWN_SLOT);
        if entry.name == FIGHT_START {
            starts_replayed += 1;
        }
        now_ms += 16;
    }
    assert!(starts_replayed <= 1, "{starts_replayed}");
    assert_eq!(
        guest.pop(WAVE_END, PopSource::Fired, OWN_SLOT, HERE, false),
        GuestPop::Hold
    );
}

#[test]
fn a_defeat_reload_runs_the_chain_again() {
    let mut hooked = host_through_the_first_boss_intro();
    hooked.pops(WAVE_END);
    assert_eq!(hooked.dropped, [WAVE_END]);
    hooked.pops(FIRST_PART_CONTINUE);
    hooked.streamed.extend(hooked.host.restart());
    hooked.pops(WAVE_END);
    assert_eq!(hooked.dropped, [WAVE_END], "{:?}", hooked.dropped);
    assert_eq!(hooked.streamed_names().last(), Some(&WAVE_END));
    hooked.pops(OTHER_PACK_FUNCTION);
    hooked.pops(WAVE_END);
    assert_eq!(hooked.dropped, [WAVE_END, WAVE_END]);
}

#[test]
fn a_new_window_after_the_old_one_closed_runs_its_chain() {
    let mut hooked = host_through_the_first_boss_intro();
    hooked
        .streamed
        .extend(hooked.host.enter_phase(PHASE + 1, "OTHER_PACK"));
    hooked.streamed.extend(hooked.host.enter_phase(PHASE, PACK));
    hooked.pops(WAVE_END);
    hooked.pops(FIGHT_START);
    hooked.registers_a_callback();
    assert!(hooked.dropped.is_empty(), "{:?}", hooked.dropped);
    hooked.pops(OTHER_PACK_FUNCTION);
    hooked.pops(WAVE_END);
    assert_eq!(hooked.dropped, [WAVE_END]);
}

#[test]
fn no_window_drops_a_pop_of_its_own_first_pass_start_chain() {
    for window in FIGHT_WINDOWS.iter().filter(|row| !row.start_ids.is_empty()) {
        let mut host = HostFightStream::new();
        host.enter_phase(PHASE, window.pack);
        host.popped(window.start_ids[0], Vec::new(), false);
        host.callback_registered();
        let callers = SCRIPT_FUNCTIONS
            .iter()
            .filter(|(pack, _, _, _, callees)| {
                *pack == window.pack && callees.iter().any(|id| window.start_ids.contains(id))
            })
            .map(|(_, name, ..)| *name);
        let chain: Vec<&str> = window.start_ids[1..]
            .iter()
            .copied()
            .chain(callers)
            .collect();
        for name in chain {
            assert!(
                host.repeats_start_chain(name).is_none(),
                "{} {}: first pass {name}",
                window.number,
                window.part
            );
            host.popped(name, Vec::new(), false);
        }
    }
}
