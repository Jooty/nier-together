use std::sync::{Arc, Mutex, PoisonError};
use std::thread;
use std::time::{Duration, Instant};

use automata_game::build::MAIN_PE_IDENTITY;
use automata_game::character::Character;
use automata_game::enemies::{
    EnemyHit, EnemyHits, EnemyId, EnemyRosterPart, EnemySighting, EnemySpawn, GeneratorPhase,
    GeneratorState, GeneratorStates, LiveGenerators,
};
use automata_game::enemy_lead::{EnemyLead, EnemyPose, LeadTarget};
use automata_game::event_follow::{EventAsk, EventAskWhat, EventId};
use automata_game::exe::ExeVariant;
use automata_game::late_join::joiner_waits_line;
use automata_game::load_gate::StoryPlace;
use automata_game::outfit_look::{HOST_SLOT, PlayerLook};
use automata_game::partner::BodySnapshot;
use automata_game::pickups::ItemPickup;
use automata_game::session_jump::SessionJumpOrder;
use automata_game::shared_defeat::{DefeatNote, DefeatWay};
use automata_game::story_follow::{AskVerdict, StoryAsk, StoryAskAnswer, StoryMove, StoryRequest};
use automata_game::story_steps::{StepAnswer, StoryStepAnswer, StoryStepAsk, StoryStepDone};
use automata_game::travel::{JumpRole, TeleportRequest};
use automata_hook::net::{
    AutomataNet, NetSharedExt, SessionModeExt, held_arrivals_of, load_role_of, partner_place_of,
    take_announced_lines_where, take_in_held_players,
};
use automata_proto::Message;
use nt_hook::net::{LinkPeer, NetThread};
use nt_launcher_core::{
    AUTOMATA, FREE_ROAM_MODE, LauncherCore, LauncherNet, TransportFactory, rules_for_mode,
};
use nt_net::{LocalLinkTransport, LoopbackNetwork, Transport};
use nt_wire::{BuildGate, STORY_MODE, SessionRules};

const WAIT_LIMIT: Duration = Duration::from_secs(20);
const MAIN_BUILD: BuildGate = BuildGate {
    time_date_stamp: MAIN_PE_IDENTITY.time_date_stamp,
    size_of_image: MAIN_PE_IDENTITY.size_of_image,
};
const MOST_MARKER_ACCESSORIES: u8 = 20;
const GUEST_NAME: &str = "Latecomer";
const HOST_NAME: &str = "Hostelry";
const CAMPAIGN_RULES: SessionRules = SessionRules {
    mode: STORY_MODE,
    max_players: 2,
    map: 0,
};

struct HostRig {
    _launcher: LauncherCore,
    game: NetThread<AutomataNet>,
    ticket: String,
    network: LoopbackNetwork,
}

fn wait_until(mut condition: impl FnMut() -> bool) -> bool {
    let deadline = Instant::now() + WAIT_LIMIT;
    while Instant::now() < deadline {
        if condition() {
            return true;
        }
        thread::sleep(Duration::from_millis(3));
    }
    false
}

fn launcher_on(network: &LoopbackNetwork, name: &str) -> LauncherCore {
    let network = network.clone();
    let factory: TransportFactory =
        Arc::new(move || Ok(Box::new(network.endpoint()) as Box<dyn Transport>));
    LauncherCore::for_game(&AUTOMATA, name, factory).expect("launcher starts")
}

fn game_linked_to(launcher: &LauncherCore) -> NetThread<AutomataNet> {
    game_with(
        launcher,
        AutomataNet::new(Character::TwoB, ExeVariant::Main),
    )
}

fn late_game_linked_to(
    launcher: &LauncherCore,
    joins_are_late: fn() -> bool,
) -> NetThread<AutomataNet> {
    let net = AutomataNet::new(Character::TwoB, ExeVariant::Main)
        .with_late_joins_decided_by(joins_are_late);
    game_with(launcher, net)
}

fn game_with(launcher: &LauncherCore, net: AutomataNet) -> NetThread<AutomataNet> {
    NetThread::spawn(
        net,
        LocalLinkTransport::game(MAIN_BUILD),
        LinkPeer::launcher_on_port(launcher.link_port()),
    )
    .expect("the net worker starts")
}

fn joins_are_late() -> bool {
    true
}

fn joins_are_not_late() -> bool {
    false
}

fn clear_announced_lines() {
    take_announced_lines_where(|line| line.contains(GUEST_NAME) || line.contains(HOST_NAME));
}

fn hosting_alone(rules: SessionRules, joins: fn() -> bool) -> HostRig {
    let network = LoopbackNetwork::new();
    let launcher = launcher_on(&network, HOST_NAME);
    launcher.set_rules(rules);
    let ticket = launcher.host().expect("hosts");
    let game = late_game_linked_to(&launcher, joins);
    assert!(wait_until(|| launcher.link_status().game_linked));
    assert!(wait_until(|| game.view().extras.session_open));
    HostRig {
        _launcher: launcher,
        game,
        ticket,
        network,
    }
}

fn a_body_state() -> BodySnapshot {
    BodySnapshot {
        speed: 1.0,
        position: [10.0, 2.0, -30.0],
        health: 4200,
        max_health: 5000,
        free: true,
        phase_id: 0x200,
        ..BodySnapshot::default()
    }
}

fn free_roam_rules() -> SessionRules {
    rules_for_mode(&AUTOMATA, FREE_ROAM_MODE).expect("automata has free roam")
}

#[test]
fn a_late_joiner_is_unseen_by_every_reader_until_taken_in_even_after_its_game_arrives() {
    let rig = hosting_alone(CAMPAIGN_RULES, joins_are_late);
    let guest = launcher_on(&rig.network, GUEST_NAME);
    let guest_game = game_linked_to(&guest);
    guest.join(&rig.ticket).expect("guest dials");
    thread::sleep(Duration::from_millis(150));
    clear_announced_lines();
    assert!(wait_until(|| rig.game.view().partner().is_some()));
    assert!(wait_until(|| guest_game.wants_snapshots()));
    let lobby_only = rig.game.view();
    assert!(lobby_only.partner().is_some_and(|p| p.watch.joined_late));
    assert!(!lobby_only.campaign_link_is_up());
    assert!(!lobby_only.counts_as_connected());
    assert!(lobby_only.campaign_partner().is_none());
    assert!(lobby_only.companion_player().is_none());
    assert_eq!(load_role_of(&lobby_only), JumpRole::Solo);
    guest_game.publish(a_body_state());
    assert!(wait_until(|| held_arrivals_of(&rig.game.view()) == 1));
    let arrived = rig.game.view();
    assert!(arrived.partner().is_some_and(|p| p.latest.is_none()));
    assert!(!arrived.partner().is_some_and(|p| p.watch.heard));
    assert!(!arrived.campaign_link_is_up());
    assert!(arrived.companion_player().is_none());
    assert_eq!(arrived.counted_others().count(), 0);
    let taken = rig.game.update(take_in_held_players);
    assert_eq!(taken, vec![GUEST_NAME.to_owned()]);
    guest_game.publish(a_body_state());
    assert!(wait_until(|| rig
        .game
        .view()
        .companion_player()
        .is_some_and(|p| p.latest.is_some())));
    let in_game = rig.game.view();
    assert!(in_game.campaign_link_is_up());
    assert!(in_game.campaign_partner().is_some());
    assert_eq!(held_arrivals_of(&in_game), 0);
    guest.leave();
    thread::sleep(Duration::from_millis(150));
    clear_announced_lines();
    assert!(wait_until(|| rig.game.view().partner().is_none()));
    let again = launcher_on(&rig.network, GUEST_NAME);
    again.join(&rig.ticket).expect("guest dials again");
    thread::sleep(Duration::from_millis(150));
    clear_announced_lines();
    assert!(wait_until(|| rig.game.view().partner().is_some()));
    assert!(!rig.game.view().campaign_link_is_up());
    clear_announced_lines();
}

#[test]
fn a_player_who_joined_before_the_start_is_never_held() {
    let rig = hosting_alone(CAMPAIGN_RULES, joins_are_not_late);
    let guest = launcher_on(&rig.network, GUEST_NAME);
    let guest_game = game_linked_to(&guest);
    guest.join(&rig.ticket).expect("guest dials");
    thread::sleep(Duration::from_millis(150));
    clear_announced_lines();
    assert!(wait_until(|| rig.game.view().campaign_link_is_up()));
    assert!(wait_until(|| guest_game.wants_snapshots()));
    guest_game.publish(a_body_state());
    assert!(wait_until(|| rig
        .game
        .view()
        .partner()
        .is_some_and(|p| p.latest.is_some())));
    let view = rig.game.view();
    assert!(!view.partner().is_some_and(|p| p.watch.held_back));
    assert!(view.companion_player().is_some());
    assert_eq!(held_arrivals_of(&view), 0);
    clear_announced_lines();
}

#[test]
fn in_free_roam_a_late_joiner_is_left_out_of_the_other_players_until_taken_in() {
    let rig = hosting_alone(free_roam_rules(), joins_are_late);
    let guest = launcher_on(&rig.network, GUEST_NAME);
    let guest_game = game_linked_to(&guest);
    guest.join(&rig.ticket).expect("guest dials");
    thread::sleep(Duration::from_millis(150));
    clear_announced_lines();
    assert!(wait_until(|| rig.game.view().others().next().is_some()));
    assert!(wait_until(|| guest_game.wants_snapshots()));
    assert_eq!(rig.game.view().counted_others().count(), 0);
    assert!(rig.game.view().companion_player().is_none());
    guest_game.publish(a_body_state());
    assert!(wait_until(|| held_arrivals_of(&rig.game.view()) == 1));
    assert_eq!(rig.game.view().counted_others().count(), 0);
    rig.game.update(take_in_held_players);
    assert_eq!(rig.game.view().counted_others().count(), 1);
    assert!(rig.game.view().companion_player().is_some());
    let place = partner_place_of(&guest_game.view());
    assert_eq!(place, StoryPlace::default());
    clear_announced_lines();
}

fn a_place() -> StoryPlace {
    StoryPlace::new(0x100, "00_30_A_Place")
}

fn an_event() -> EventId {
    EventId {
        kind: 0,
        id: 0x123,
        sub: -1,
    }
}

fn an_enemy() -> EnemyId {
    EnemyId::Placed {
        layout: 0x11,
        entry: 0x22,
    }
}

fn every_message_that_acts_on_the_receivers_game() -> Vec<Message> {
    vec![
        Message::BossWait {
            slot: 0,
            waiting: true,
        },
        Message::Teleport(TeleportRequest::FollowJump { point: 8 }),
        Message::EnemyRoster(EnemyRosterPart {
            round: 3,
            last: true,
            enemies: vec![EnemySighting {
                id: an_enemy(),
                object_id: 0x5000,
                health: 120,
                max_health: 200,
            }],
        }),
        Message::EnemyHits(EnemyHits {
            hits: vec![EnemyHit {
                id: an_enemy(),
                amount: 15,
                max_health: 200,
            }],
        }),
        Message::EnemyDefeated(an_enemy()),
        Message::EnemySpawned(EnemySpawn {
            generator: 0x33,
            serial: 4,
            set_index: 1,
            object_id: 0x5000,
            level: Some(7),
            uses_the_second_set_values: true,
            object_read_variant: 2,
            set_values: [1, 2, 3, 4],
            position: [1.5, -2.0, 3.25, 1.0],
            rotation: [0.0, 0.5, 0.0, 1.0],
        }),
        Message::EnemyGone(an_enemy()),
        Message::GeneratorStates(GeneratorStates {
            states: vec![GeneratorState {
                generator: 0x33,
                phase: GeneratorPhase::Spawning,
                spawned: 5,
                counts: None,
            }],
        }),
        Message::LiveGenerators(LiveGenerators {
            route_hash: 0xCAFE_F00D,
            part: 0,
            parts: 1,
            ids: vec![0x33, 0x34],
        }),
        Message::Pickup(ItemPickup {
            item_id: 80,
            count: 2,
            key_item: false,
        }),
        Message::StoryMove(StoryMove {
            number: 2,
            from: a_place(),
            request: StoryRequest::Full {
                id: 0x40,
                name: "Step".to_owned(),
                params: [1, 2, 3, 4, 5, 6],
            },
        }),
        Message::StoryAsk(StoryAsk {
            number: 3,
            at: a_place(),
            request: StoryRequest::SubPhase {
                id: 0x41,
                name: "Sub".to_owned(),
            },
        }),
        Message::StoryAskAnswer(StoryAskAnswer {
            number: 3,
            verdict: AskVerdict::RefusedOtherPlace,
        }),
        Message::EventStarted { id: an_event() },
        Message::EventEnded {
            id: an_event(),
            skipped: true,
        },
        Message::EventAsk(EventAsk {
            id: an_event(),
            what: EventAskWhat::Skip,
        }),
        Message::HostSaved { number: 7 },
        Message::SkipAsk { id: an_event() },
        Message::DefeatNote(DefeatNote::Way {
            way: DefeatWay::RestorePoint,
            restore: None,
        }),
        Message::StoryStepDone(StoryStepDone {
            number: 2,
            id: 0x1234,
            value: -1,
            place: a_place(),
        }),
        Message::StoryStepAsk(StoryStepAsk {
            id: 0x1234,
            value: 3,
            place: a_place(),
        }),
        Message::StoryStepAnswer(StoryStepAnswer {
            id: 0x1234,
            answer: StepAnswer::NotStarted,
        }),
        Message::EventPause {
            id: an_event(),
            paused: true,
        },
        Message::SessionJump(SessionJumpOrder { start_index: 41 }),
        Message::EnemyLead(EnemyLead {
            poses: vec![EnemyPose {
                id: an_enemy(),
                position: [1.5, -2.0, 3.25],
                facing: 0.5,
                state: [1, 2, 3, 4],
                starts: 9,
                target: LeadTarget::HostPlayer,
            }],
            starts: Vec::new(),
        }),
    ]
}

fn a_marker_look(number: u8) -> PlayerLook {
    PlayerLook {
        outfit_variant: 2,
        accessory_code: number % MOST_MARKER_ACCESSORIES,
        hair_colour: number / MOST_MARKER_ACCESSORIES,
        pod_skin: 1,
        gun_skin: 255,
    }
}

fn the_marker_arrived(rig: &HostRig, number: u8) -> bool {
    rig.game
        .view()
        .partner()
        .is_some_and(|partner| partner.watch.look == Some(a_marker_look(number)))
}

fn nothing_reached_the_hosts_game(rig: &HostRig) -> Vec<String> {
    let mut reached: Vec<String> = rig
        .game
        .take_relayed_from(|_| true)
        .into_iter()
        .map(|(from, message)| format!("{} from slot {from} in the inbox", message.kind_name()))
        .collect();
    let leads = rig.game.view().extras.enemy_leads.len();
    if leads > 0 {
        reached.push(format!("{leads} enemy leads waiting"));
    }
    reached
}

#[test]
fn a_held_joiner_cannot_act_on_the_hosts_game_with_any_message_it_sends() {
    let rig = hosting_alone(CAMPAIGN_RULES, joins_are_late);
    let guest = launcher_on(&rig.network, GUEST_NAME);
    let guest_game = game_linked_to(&guest);
    guest.join(&rig.ticket).expect("guest dials");
    thread::sleep(Duration::from_millis(150));
    clear_announced_lines();
    assert!(wait_until(|| rig.game.view().partner().is_some()));
    assert!(wait_until(|| guest_game.wants_snapshots()));
    guest_game.publish(a_body_state());
    assert!(wait_until(|| held_arrivals_of(&rig.game.view()) == 1));
    let mut broken = Vec::new();
    for (number, message) in (0u8..).zip(every_message_that_acts_on_the_receivers_game()) {
        let kind = message.kind_name();
        guest_game.send_relayed(message);
        guest_game.send_relayed(Message::Look(a_marker_look(number)));
        assert!(
            wait_until(|| the_marker_arrived(&rig, number)),
            "{kind}: the message after it never arrived"
        );
        broken.extend(
            nothing_reached_the_hosts_game(&rig)
                .into_iter()
                .map(|reached| format!("{kind}: {reached}")),
        );
    }
    assert!(
        broken.is_empty(),
        "a held joiner acted on the host: {broken:?}"
    );
    let view = rig.game.view();
    assert!(view.partner().is_some_and(|p| p.watch.held_back));
    assert!(view.partner().is_some_and(|p| p.latest.is_none()));
    assert!(view.campaign_partner().is_none());
    assert_eq!(held_arrivals_of(&view), 1);
    clear_announced_lines();
}

#[test]
fn once_taken_in_a_joiners_messages_reach_the_hosts_game() {
    let rig = hosting_alone(CAMPAIGN_RULES, joins_are_late);
    let guest = launcher_on(&rig.network, GUEST_NAME);
    let guest_game = game_linked_to(&guest);
    guest.join(&rig.ticket).expect("guest dials");
    thread::sleep(Duration::from_millis(150));
    clear_announced_lines();
    assert!(wait_until(|| guest_game.wants_snapshots()));
    guest_game.publish(a_body_state());
    assert!(wait_until(|| held_arrivals_of(&rig.game.view()) == 1));
    rig.game.update(take_in_held_players);
    guest_game.send_relayed(Message::Pickup(ItemPickup {
        item_id: 80,
        count: 2,
        key_item: false,
    }));
    assert!(wait_until(|| !rig
        .game
        .take_relayed_from(|message| matches!(message, Message::Pickup(_)))
        .is_empty()));
    clear_announced_lines();
}

#[test]
fn a_guest_never_holds_the_host_even_when_its_own_link_came_up_late() {
    let network = LoopbackNetwork::new();
    let host_launcher = launcher_on(&network, HOST_NAME);
    host_launcher.set_rules(CAMPAIGN_RULES);
    let ticket = host_launcher.host().expect("hosts");
    let host_game = game_linked_to(&host_launcher);
    assert!(wait_until(|| host_launcher.link_status().game_linked));
    let guest = launcher_on(&network, GUEST_NAME);
    let guest_game = late_game_linked_to(&guest, joins_are_late);
    guest.join(&ticket).expect("guest dials");
    thread::sleep(Duration::from_millis(150));
    clear_announced_lines();
    assert!(wait_until(|| host_game.wants_snapshots()));
    host_game.publish(a_body_state());
    host_game.send_relayed(Message::Teleport(TeleportRequest::ComeToSender));
    assert!(wait_until(|| guest_game
        .view()
        .player(HOST_SLOT)
        .is_some_and(|host| host.latest.is_some())));
    let view = guest_game.view();
    assert!(!view.player_watch(HOST_SLOT).held_back);
    assert!(view.companion_player().is_some());
    assert!(wait_until(|| !guest_game
        .take_relayed_from(|message| matches!(message, Message::Teleport(_)))
        .is_empty()));
    clear_announced_lines();
}

#[test]
fn a_guest_that_first_hears_the_host_on_a_loading_screen_never_counts_as_linking_after_the_host_played()
 {
    let network = LoopbackNetwork::new();
    let host_launcher = launcher_on(&network, HOST_NAME);
    host_launcher.set_rules(CAMPAIGN_RULES);
    let ticket = host_launcher.host().expect("hosts");
    let host_game = game_linked_to(&host_launcher);
    assert!(wait_until(|| host_launcher.link_status().game_linked));
    let guest = launcher_on(&network, GUEST_NAME);
    let guest_game = game_linked_to(&guest);
    guest.join(&ticket).expect("guest dials");
    thread::sleep(Duration::from_millis(150));
    clear_announced_lines();
    assert!(wait_until(|| host_game.wants_snapshots()));
    host_game.publish(BodySnapshot::default());
    assert!(wait_until(|| guest_game
        .view()
        .player_watch(HOST_SLOT)
        .first_state_heard_in_play
        .is_some()));
    host_game.publish(a_body_state());
    assert!(wait_until(|| guest_game
        .view()
        .player(HOST_SLOT)
        .is_some_and(|host| host.latest.is_some())));
    assert!(!guest_game.view().host_was_in_play_when_first_heard());
    clear_announced_lines();
}

#[test]
fn a_guest_that_first_hears_the_host_in_play_counts_as_linking_after_the_host_played() {
    let network = LoopbackNetwork::new();
    let host_launcher = launcher_on(&network, HOST_NAME);
    host_launcher.set_rules(CAMPAIGN_RULES);
    let ticket = host_launcher.host().expect("hosts");
    let host_game = game_linked_to(&host_launcher);
    assert!(wait_until(|| host_launcher.link_status().game_linked));
    let guest = launcher_on(&network, GUEST_NAME);
    let guest_game = game_linked_to(&guest);
    guest.join(&ticket).expect("guest dials");
    thread::sleep(Duration::from_millis(150));
    clear_announced_lines();
    assert!(wait_until(|| host_game.wants_snapshots()));
    host_game.publish(a_body_state());
    assert!(wait_until(|| guest_game
        .view()
        .host_was_in_play_when_first_heard()));
    clear_announced_lines();
}

static BOSS_LINE_TESTS: Mutex<()> = Mutex::new(());
const BOSS_HOST_NAME: &str = "Bossbound Host";
const BOSS_GUEST_NAME: &str = "Bossbound Guest";

struct BossRig {
    host: HostRig,
    guest: LauncherCore,
    guest_game: NetThread<AutomataNet>,
    guest_slot: u8,
}

fn a_held_joiner_on_a_host_named(host_name: &str, guest_name: &str) -> BossRig {
    let network = LoopbackNetwork::new();
    let launcher = launcher_on(&network, host_name);
    launcher.set_rules(CAMPAIGN_RULES);
    let ticket = launcher.host().expect("hosts");
    let game = late_game_linked_to(&launcher, joins_are_late);
    assert!(wait_until(|| launcher.link_status().game_linked));
    assert!(wait_until(|| game.view().extras.session_open));
    let guest = launcher_on(&network, guest_name);
    let guest_game = game_linked_to(&guest);
    guest.join(&ticket).expect("guest dials");
    assert!(wait_until(|| guest_game.wants_snapshots()));
    guest_game.publish(a_body_state());
    assert!(wait_until(|| held_arrivals_of(&game.view()) == 1));
    assert!(wait_until(|| guest_game.view().partner().is_some()
        && guest_game.update(|shared| shared.local_slot).is_some()));
    let guest_slot = guest_game
        .update(|shared| shared.local_slot)
        .expect("the guest has a slot");
    BossRig {
        host: HostRig {
            _launcher: launcher,
            game,
            ticket,
            network,
        },
        guest,
        guest_game,
        guest_slot,
    }
}

fn lines_named(line: &str) -> Vec<String> {
    take_announced_lines_where(|queued| queued == line)
}

fn guest_has_heard_the_marker(rig: &BossRig, number: u8) -> bool {
    rig.guest_game
        .view()
        .partner()
        .is_some_and(|partner| partner.watch.look == Some(a_marker_look(number)))
}

#[test]
fn the_joiner_is_told_why_it_waits_once_per_fight_and_the_end_withdraws_an_unshown_line() {
    let _one_at_a_time = BOSS_LINE_TESTS
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    let rig = a_held_joiner_on_a_host_named(BOSS_HOST_NAME, BOSS_GUEST_NAME);
    let line = joiner_waits_line(BOSS_HOST_NAME);
    thread::sleep(Duration::from_millis(150));
    lines_named(&line);
    let begins = Message::BossWait {
        slot: rig.guest_slot,
        waiting: true,
    };
    rig.host.game.send_relayed(begins.clone());
    rig.host.game.send_relayed(begins.clone());
    rig.host.game.send_relayed(Message::Look(a_marker_look(1)));
    assert!(wait_until(|| guest_has_heard_the_marker(&rig, 1)));
    assert_eq!(lines_named(&line), vec![line.clone()]);
    rig.host.game.send_relayed(Message::BossWait {
        slot: rig.guest_slot,
        waiting: false,
    });
    rig.host.game.send_relayed(begins.clone());
    rig.host.game.send_relayed(Message::BossWait {
        slot: rig.guest_slot,
        waiting: false,
    });
    rig.host.game.send_relayed(Message::Look(a_marker_look(2)));
    assert!(wait_until(|| guest_has_heard_the_marker(&rig, 2)));
    assert!(
        lines_named(&line).is_empty(),
        "a wait that ended before the line was shown still posted it"
    );
    rig.host.game.send_relayed(begins);
    rig.host.game.send_relayed(Message::Look(a_marker_look(3)));
    assert!(wait_until(|| guest_has_heard_the_marker(&rig, 3)));
    assert_eq!(lines_named(&line), vec![line]);
    rig.guest.leave();
}

#[test]
fn a_notice_for_another_slot_tells_this_joiner_nothing() {
    let _one_at_a_time = BOSS_LINE_TESTS
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    let rig = a_held_joiner_on_a_host_named("Bossbound Other", "Bossbound Third");
    let line = joiner_waits_line("Bossbound Other");
    lines_named(&line);
    rig.host.game.send_relayed(Message::BossWait {
        slot: rig.guest_slot.wrapping_add(1),
        waiting: true,
    });
    rig.host.game.send_relayed(Message::Look(a_marker_look(4)));
    assert!(wait_until(|| guest_has_heard_the_marker(&rig, 4)));
    assert!(lines_named(&line).is_empty());
    rig.guest.leave();
}

#[test]
fn a_guest_sending_the_boss_wait_notice_does_nothing_to_the_host_or_to_another_guest() {
    let _one_at_a_time = BOSS_LINE_TESTS
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    let rig = a_held_joiner_on_a_host_named("Bossbound Sender", "Bossbound Sent");
    let host_slot = rig.host.game.update(|shared| shared.local_slot);
    take_announced_lines_where(|line| line.starts_with("Waiting for"));
    for slot in [host_slot.unwrap_or(HOST_SLOT), rig.guest_slot] {
        rig.guest_game.send_relayed(Message::BossWait {
            slot,
            waiting: true,
        });
    }
    rig.guest_game.send_relayed(Message::Look(a_marker_look(5)));
    assert!(wait_until(|| the_marker_arrived(&rig.host, 5)));
    assert_eq!(
        take_announced_lines_where(|line| line.starts_with("Waiting for")),
        Vec::<String>::new()
    );
    assert_eq!(
        nothing_reached_the_hosts_game(&rig.host),
        Vec::<String>::new()
    );
    rig.guest.leave();
}
