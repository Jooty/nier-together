use automata_game::event_follow::{EventId, EventMachine, EventReadings};
use automata_game::load_gate::StoryPlace;
use automata_game::travel::JumpRole;

fn event(id: i32) -> EventId {
    EventId {
        kind: 0,
        id,
        sub: -1,
    }
}

fn readings(now_ms: u64) -> EventReadings {
    EventReadings {
        now_ms,
        role: JumpRole::Solo,
        partner_present: false,
        local_in_play: true,
        partner_in_play: false,
        current_phase_id: 0x100,
        current_place: StoryPlace::new(0x100, "A"),
        partner_place: StoryPlace::default(),
        partner_playing: None,
        slots: Vec::new(),
        event_flags_say_playing: false,
        started_heard: Vec::new(),
        ended_heard: Vec::new(),
        asks: Vec::new(),
        partner_report: None,
        local_unattended: None,
        skip_asks_heard: Vec::new(),
        local_event_pause_open: false,
        pauses_heard: Vec::new(),
        progress: None,
    }
}

fn scene_lines(machine: &mut EventMachine, now_ms: u64) -> Vec<String> {
    machine
        .step(&readings(now_ms))
        .logs
        .into_iter()
        .filter(|line| line.starts_with("scene: "))
        .collect()
}

#[test]
fn a_started_event_logs_a_numbered_scene_start_with_its_ids() {
    let mut machine = EventMachine::new(true);
    machine.on_start(event(0x123), 10);
    assert_eq!(
        scene_lines(&mut machine, 20),
        ["scene: #1 started event kind 0 id 0x123 sub -1"]
    );
}

#[test]
fn the_end_carries_the_number_of_the_start() {
    let mut machine = EventMachine::new(true);
    machine.on_start(event(1), 10);
    machine.on_start(event(2), 11);
    scene_lines(&mut machine, 20);
    machine.on_end(event(1), false, 30);
    assert_eq!(scene_lines(&mut machine, 40), ["scene: #1 ended"]);
    machine.on_end(event(2), true, 50);
    assert_eq!(scene_lines(&mut machine, 60), ["scene: #2 ended"]);
}

#[test]
fn the_numbers_keep_counting_for_every_start_of_the_session() {
    let mut machine = EventMachine::new(true);
    for (round, id) in [5, 6, 7].into_iter().enumerate() {
        let now_ms = 100 * (round as u64 + 1);
        machine.on_start(event(id), now_ms);
        let started = scene_lines(&mut machine, now_ms + 1);
        assert!(started[0].starts_with(&format!("scene: #{} started", round + 1)));
        machine.on_end(event(id), false, now_ms + 2);
        assert_eq!(
            scene_lines(&mut machine, now_ms + 3),
            [format!("scene: #{} ended", round + 1)]
        );
    }
}

#[test]
fn an_end_with_no_start_here_logs_no_scene_line() {
    let mut machine = EventMachine::new(true);
    machine.on_end(event(9), false, 10);
    assert!(scene_lines(&mut machine, 20).is_empty());
}
