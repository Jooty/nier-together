use std::path::{Path, PathBuf};

use nt_launcher::net::{LOBBY_MODE_CONTINUE, LOBBY_MODE_NEW_GAME};
use nt_launcher::saves::{coop_campaigns_path, read_campaigns};
use nt_launcher_core::{LauncherNet, REPLICANT, seat_with_name};
use nt_net::LoopbackNetwork;
use replicant_game::story_progress::{
    SLOT_BYTES, SLOTS_FILE_OFFSET, fix_header_checksum, seat_bytes_of,
};
use replicant_proto::ScaleUpAsk;

use crate::late_join::{
    FileFacts, HOST_SLOT, Peer, assert_untouched, facts_of, guest_with_a_template,
    host_a_running_game, host_with_a_campaign_save, run_quietly, run_until,
};
use crate::seat_files::{seat_with_play, template_file};

const GUEST_NAME: &str = "Guesty";
const GUEST_MEMBER_SLOT: u8 = 1;
const PAST_THE_SEAT_WAIT_SECONDS: f64 = 100_000.0;

fn open_ask() -> ScaleUpAsk {
    ScaleUpAsk::Open {
        slot: GUEST_MEMBER_SLOT,
        name: GUEST_NAME.to_owned(),
    }
}

fn no_save_ask() -> ScaleUpAsk {
    ScaleUpAsk::NoSaveYet {
        slot: GUEST_MEMBER_SLOT,
        name: GUEST_NAME.to_owned(),
    }
}

fn withdrawn_ask() -> ScaleUpAsk {
    ScaleUpAsk::Withdrawn {
        slot: GUEST_MEMBER_SLOT,
    }
}

struct Table {
    host: Peer,
    guest: Peer,
    ticket: String,
    host_facts: FileFacts,
    coop: PathBuf,
}

fn table_with_a_continue_host(name: &str) -> Table {
    let network = LoopbackNetwork::new();
    let mut host = Peer::new(&network, &REPLICANT, "Hosty", &format!("su_{name}_host"));
    let mut guest = Peer::new(
        &network,
        &REPLICANT,
        GUEST_NAME,
        &format!("su_{name}_guest"),
    );
    host_with_a_campaign_save(&mut host);
    guest_with_a_template(&mut guest);
    let coop = host.coop_file();
    let ticket = host_a_running_game(&mut host, LOBBY_MODE_CONTINUE);
    guest.follows_kind = false;
    let host_facts = facts_of(&coop);
    Table {
        host,
        guest,
        ticket,
        host_facts,
        coop,
    }
}

fn the_host_game_was_asked(table: &mut Table) -> bool {
    run_until(&mut [&mut table.host, &mut table.guest], |peers| {
        peers[0]
            .game
            .as_ref()
            .is_some_and(|game| game.asked.contains(&open_ask()))
    })
}

fn host_asks(table: &Table) -> Vec<ScaleUpAsk> {
    table
        .host
        .game
        .as_ref()
        .map(|game| game.asked.clone())
        .unwrap_or_default()
}

fn campaign_seat_is_stored(host: &Peer) -> bool {
    let record = coop_campaigns_path(&host.saves).expect("a record path");
    read_campaigns(&record)
        .get(&HOST_SLOT)
        .is_some_and(|campaign| campaign.seat.is_some())
}

fn session_files(folder: &Path) -> Vec<PathBuf> {
    std::fs::read_dir(folder)
        .map(|entries| {
            entries
                .filter_map(|entry| Some(entry.ok()?.path()))
                .filter(|path| {
                    path.file_name()
                        .and_then(|name| name.to_str())
                        .is_some_and(|name| name.starts_with("GAMEDATA.seat-"))
                })
                .collect()
        })
        .unwrap_or_default()
}

fn host_seat_in(file: &[u8]) -> Vec<u8> {
    seat_bytes_of(file, HOST_SLOT).expect("the host's seat")
}

fn the_host_game_saves_again(host: &Peer, seed: u8) -> Vec<u8> {
    let mut file = template_file();
    file[SLOTS_FILE_OFFSET..SLOTS_FILE_OFFSET + SLOT_BYTES]
        .copy_from_slice(&seat_with_play("Jay", 900.0, seed));
    file[0x6C] = 1;
    fix_header_checksum(&mut file).unwrap();
    std::fs::write(host.coop_file(), &file).unwrap();
    file
}

#[test]
fn a_guest_without_a_save_is_asked_about_and_waits_without_a_give_up_until_the_host_answers() {
    let mut table = table_with_a_continue_host("waits");
    table.guest.core.join(&table.ticket).expect("join");
    assert!(
        the_host_game_was_asked(&mut table),
        "{:#?}",
        table.host.launcher.log
    );
    assert!(
        run_until(&mut [&mut table.host, &mut table.guest], |peers| peers[1]
            .launcher
            .guest_waits_for_the_hosts_decision()),
        "the guest never learned the host is being asked"
    );
    table.guest.skew_seconds += PAST_THE_SEAT_WAIT_SECONDS;
    run_quietly(&mut [&mut table.host, &mut table.guest]);
    assert_eq!(table.guest.game_opens(), 0, "no launch before the answer");
    assert!(
        !table.guest.log_has("did not arrive in time"),
        "{:#?}",
        table.guest.launcher.log
    );
    assert!(table.guest.launcher.guest_waits_for_the_hosts_decision());
    assert_eq!(host_asks(&table), vec![open_ask()], "asked once");
    assert_untouched(&table.coop, &table.host_facts, "while the host decides");
    assert!(!campaign_seat_is_stored(&table.host));
}

#[test]
fn yes_gives_the_guest_a_copy_of_the_hosts_save_under_its_name_and_the_guest_launches_into_it() {
    let mut table = table_with_a_continue_host("yes");
    let host_seat = host_seat_in(&std::fs::read(&table.coop).unwrap());
    table.guest.core.join(&table.ticket).expect("join");
    assert!(the_host_game_was_asked(&mut table));
    table
        .host
        .game
        .as_mut()
        .unwrap()
        .answer(GUEST_MEMBER_SLOT, true);
    let launched = run_until(&mut [&mut table.host, &mut table.guest], |peers| {
        peers[1].game_opens() == 1
    });
    assert!(
        launched,
        "host log {:#?}\nguest log {:#?}",
        table.host.launcher.log, table.guest.launcher.log
    );
    run_quietly(&mut [&mut table.host, &mut table.guest]);
    assert_eq!(table.guest.game_opens(), 1, "launched once");
    let files = session_files(&table.guest.account_folder());
    assert_eq!(files.len(), 1, "one session file with the seat in it");
    let seat = host_seat_in(&std::fs::read(&files[0]).unwrap());
    assert_eq!(
        seat,
        seat_with_name(&host_seat, GUEST_NAME).expect("renamed"),
        "the guest boots into a copy of the host's save under its own name"
    );
    assert!(campaign_seat_is_stored(&table.host));
    assert_untouched(&table.coop, &table.host_facts, "after the copy");
    assert!(!table.guest.launcher.guest_waits_for_the_hosts_decision());
}

#[test]
fn not_now_starts_the_guest_on_its_own_new_game_and_copies_nothing() {
    let mut table = table_with_a_continue_host("no");
    table.guest.core.join(&table.ticket).expect("join");
    assert!(the_host_game_was_asked(&mut table));
    table
        .host
        .game
        .as_mut()
        .unwrap()
        .answer(GUEST_MEMBER_SLOT, false);
    assert!(run_until(
        &mut [&mut table.host, &mut table.guest],
        |peers| peers[1].game_opens() == 1
    ));
    run_quietly(&mut [&mut table.host, &mut table.guest]);
    assert!(session_files(&table.guest.account_folder()).is_empty());
    assert!(!campaign_seat_is_stored(&table.host));
    assert_untouched(&table.coop, &table.host_facts, "after not now");
}

#[test]
fn one_answer_is_one_action_even_when_the_game_says_it_twice() {
    let mut table = table_with_a_continue_host("twice");
    table.guest.core.join(&table.ticket).expect("join");
    assert!(the_host_game_was_asked(&mut table));
    let game = table.host.game.as_mut().unwrap();
    game.answer(GUEST_MEMBER_SLOT, true);
    game.answer(GUEST_MEMBER_SLOT, true);
    assert!(run_until(
        &mut [&mut table.host, &mut table.guest],
        |peers| peers[1].game_opens() == 1
    ));
    run_quietly(&mut [&mut table.host, &mut table.guest]);
    let copies = table
        .host
        .launcher
        .log
        .iter()
        .filter(|line| line.contains("gave Guesty a copy"))
        .count();
    assert_eq!(copies, 1, "{:#?}", table.host.launcher.log);
}

#[test]
fn a_guest_who_leaves_withdraws_the_question_and_a_late_yes_copies_nothing() {
    let mut table = table_with_a_continue_host("leaves");
    table.guest.core.join(&table.ticket).expect("join");
    assert!(the_host_game_was_asked(&mut table));
    table.guest.core.leave();
    assert!(run_until(
        &mut [&mut table.host, &mut table.guest],
        |peers| {
            peers[0]
                .game
                .as_ref()
                .is_some_and(|game| game.asked.contains(&withdrawn_ask()))
        }
    ));
    table
        .host
        .game
        .as_mut()
        .unwrap()
        .answer(GUEST_MEMBER_SLOT, true);
    run_quietly(&mut [&mut table.host, &mut table.guest]);
    assert!(!campaign_seat_is_stored(&table.host), "nothing was copied");
    assert_eq!(table.guest.game_opens(), 0);
    assert!(!table.guest.launcher.guest_waits_for_the_hosts_decision());
    assert_untouched(&table.coop, &table.host_facts, "after a late yes");
}

#[test]
fn a_guest_who_rejoins_is_asked_again_and_the_new_answer_counts() {
    let mut table = table_with_a_continue_host("rejoins");
    table.guest.core.join(&table.ticket).expect("join");
    assert!(the_host_game_was_asked(&mut table));
    table.guest.core.leave();
    assert!(run_until(
        &mut [&mut table.host, &mut table.guest],
        |peers| {
            peers[0]
                .game
                .as_ref()
                .is_some_and(|game| game.asked.contains(&withdrawn_ask()))
        }
    ));
    table.guest.core.join(&table.ticket).expect("join again");
    assert!(run_until(
        &mut [&mut table.host, &mut table.guest],
        |peers| {
            peers[0].game.as_ref().is_some_and(|game| {
                game.asked.iter().filter(|ask| **ask == open_ask()).count() == 2
            })
        }
    ));
    table
        .host
        .game
        .as_mut()
        .unwrap()
        .answer(GUEST_MEMBER_SLOT, true);
    assert!(run_until(
        &mut [&mut table.host, &mut table.guest],
        |peers| peers[1].game_opens() == 1
    ));
    assert!(campaign_seat_is_stored(&table.host));
}

#[test]
fn a_host_whose_game_closes_withdraws_the_question_and_the_guest_stops_waiting() {
    let mut table = table_with_a_continue_host("host_quits");
    table.guest.core.join(&table.ticket).expect("join");
    assert!(the_host_game_was_asked(&mut table));
    assert!(run_until(
        &mut [&mut table.host, &mut table.guest],
        |peers| peers[1].launcher.guest_waits_for_the_hosts_decision()
    ));
    let mut game = table.host.game.take().expect("the host's game");
    run_quietly(&mut [&mut table.host, &mut table.guest]);
    game.answer(GUEST_MEMBER_SLOT, true);
    drop(game);
    assert!(
        run_until(&mut [&mut table.host, &mut table.guest], |peers| !peers[1]
            .launcher
            .guest_waits_for_the_hosts_decision()),
        "the guest still waits: {:#?}\nhost: {:#?}",
        table.guest.launcher.log,
        table.host.launcher.log
    );
    run_quietly(&mut [&mut table.host, &mut table.guest]);
    assert_eq!(table.guest.game_opens(), 0);
    assert!(!campaign_seat_is_stored(&table.host));
    assert_untouched(&table.coop, &table.host_facts, "after the host quit");
}

#[test]
fn a_host_who_closes_the_lobby_while_asked_ends_the_wait_and_copies_nothing() {
    let mut table = table_with_a_continue_host("closes");
    table.guest.core.join(&table.ticket).expect("join");
    assert!(the_host_game_was_asked(&mut table));
    table.host.core.leave();
    assert!(run_until(
        &mut [&mut table.host, &mut table.guest],
        |peers| {
            peers[0]
                .game
                .as_ref()
                .is_some_and(|game| game.asked.contains(&withdrawn_ask()))
        }
    ));
    table
        .host
        .game
        .as_mut()
        .unwrap()
        .answer(GUEST_MEMBER_SLOT, true);
    run_quietly(&mut [&mut table.host, &mut table.guest]);
    assert!(!campaign_seat_is_stored(&table.host));
    assert_eq!(table.guest.game_opens(), 0);
    assert!(!table.guest.launcher.guest_waits_for_the_hosts_decision());
}

#[test]
fn a_host_who_started_a_new_game_with_nothing_saved_is_not_asked_and_is_told_why() {
    let network = LoopbackNetwork::new();
    let mut host = Peer::new(&network, &REPLICANT, "Hosty", "su_newgame_host");
    let mut guest = Peer::new(&network, &REPLICANT, GUEST_NAME, "su_newgame_guest");
    let ticket = host_a_running_game(&mut host, LOBBY_MODE_NEW_GAME);
    guest.follows_kind = false;
    guest.core.join(&ticket).expect("join");
    assert!(run_until(&mut [&mut host, &mut guest], |peers| peers[1]
        .game_opens()
        == 1));
    run_quietly(&mut [&mut host, &mut guest]);
    let asks = host.game.as_ref().map(|game| game.asked.clone()).unwrap();
    assert_eq!(
        asks,
        vec![no_save_ask()],
        "told why, never asked: {:#?}",
        host.launcher.log
    );
    assert!(host.log_has("has no save here and you have no finished save"));
    assert_eq!(guest.game_opens(), 1);
    assert!(!guest.launcher.guest_waits_for_the_hosts_decision());
}

#[test]
fn a_host_who_started_a_new_game_and_has_since_saved_is_asked_and_yes_puts_the_guest_into_the_copy()
{
    let network = LoopbackNetwork::new();
    let mut host = Peer::new(&network, &REPLICANT, "Hosty", "su_newsaved_host");
    let mut guest = Peer::new(&network, &REPLICANT, GUEST_NAME, "su_newsaved_guest");
    guest_with_a_template(&mut guest);
    let ticket = host_a_running_game(&mut host, LOBBY_MODE_NEW_GAME);
    host_with_a_campaign_save(&mut host);
    let saved = std::fs::read(host.coop_file()).unwrap();
    let facts = facts_of(&host.coop_file());
    guest.core.join(&ticket).expect("join");
    assert!(run_until(&mut [&mut host, &mut guest], |peers| peers[0]
        .game
        .as_ref()
        .is_some_and(|game| game.asked.contains(&open_ask()))));
    host.game.as_mut().unwrap().answer(GUEST_MEMBER_SLOT, true);
    let launched = run_until(&mut [&mut host, &mut guest], |peers| {
        peers[1].game_opens() == 1
    });
    assert!(
        launched,
        "host log {:#?}\nguest log {:#?}",
        host.launcher.log, guest.launcher.log
    );
    run_quietly(&mut [&mut host, &mut guest]);
    let files = session_files(&guest.account_folder());
    assert_eq!(files.len(), 1, "the guest boots into a session file");
    let seat = host_seat_in(&std::fs::read(&files[0]).unwrap());
    assert_eq!(
        seat,
        seat_with_name(&host_seat_in(&saved), GUEST_NAME).expect("renamed")
    );
    assert_untouched(&host.coop_file(), &facts, "after the copy");
}

#[test]
fn a_save_that_is_half_written_when_the_guest_joins_is_not_asked_about_until_it_is_whole() {
    let mut table = table_with_a_continue_host("torn_join");
    let whole = std::fs::read(&table.coop).unwrap();
    std::fs::write(&table.coop, &whole[..whole.len() / 2]).unwrap();
    let torn = facts_of(&table.coop);
    table.guest.core.join(&table.ticket).expect("join");
    run_quietly(&mut [&mut table.host, &mut table.guest]);
    assert!(host_asks(&table).is_empty(), "asked about a torn save");
    assert_eq!(table.guest.game_opens(), 0);
    assert_untouched(&table.coop, &torn, "while the save was torn");
    std::fs::write(&table.coop, &whole).unwrap();
    assert!(the_host_game_was_asked(&mut table));
}

#[test]
fn a_save_that_tears_after_a_yes_is_never_copied_and_the_finished_one_is() {
    let mut table = table_with_a_continue_host("torn_yes");
    table.guest.core.join(&table.ticket).expect("join");
    assert!(the_host_game_was_asked(&mut table));
    let whole = std::fs::read(&table.coop).unwrap();
    std::fs::write(&table.coop, &whole[..whole.len() / 3]).unwrap();
    let torn = facts_of(&table.coop);
    table
        .host
        .game
        .as_mut()
        .unwrap()
        .answer(GUEST_MEMBER_SLOT, true);
    run_quietly(&mut [&mut table.host, &mut table.guest]);
    assert!(
        !campaign_seat_is_stored(&table.host),
        "a torn read was used"
    );
    assert_eq!(table.guest.game_opens(), 0, "the guest waits for the copy");
    assert_untouched(&table.coop, &torn, "while the save was torn");
    let finished = the_host_game_saves_again(&table.host, 9);
    let finished_facts = facts_of(&table.coop);
    assert!(run_until(
        &mut [&mut table.host, &mut table.guest],
        |peers| peers[1].game_opens() == 1
    ));
    let files = session_files(&table.guest.account_folder());
    let seat = host_seat_in(&std::fs::read(&files[0]).unwrap());
    assert_eq!(
        seat,
        seat_with_name(&host_seat_in(&finished), GUEST_NAME).expect("renamed")
    );
    assert_untouched(&table.coop, &finished_facts, "after the copy");
}

#[test]
fn a_save_that_stays_torn_gives_up_with_words_and_the_guest_starts_its_own_game() {
    let mut table = table_with_a_continue_host("torn_forever");
    table.guest.core.join(&table.ticket).expect("join");
    assert!(the_host_game_was_asked(&mut table));
    let whole = std::fs::read(&table.coop).unwrap();
    std::fs::write(&table.coop, &whole[..whole.len() / 3]).unwrap();
    table
        .host
        .game
        .as_mut()
        .unwrap()
        .answer(GUEST_MEMBER_SLOT, true);
    run_quietly(&mut [&mut table.host, &mut table.guest]);
    table.host.skew_seconds += 30.0;
    assert!(run_until(
        &mut [&mut table.host, &mut table.guest],
        |peers| peers[1].game_opens() == 1
    ));
    assert!(table.host.log_has("could not copy your save for"));
    assert!(!campaign_seat_is_stored(&table.host));
    assert!(session_files(&table.guest.account_folder()).is_empty());
}

#[test]
fn joining_before_the_start_still_offers_the_launchers_own_dialog_and_asks_the_game_nothing() {
    let network = LoopbackNetwork::new();
    let mut host = Peer::new(&network, &REPLICANT, "Hosty", "su_before_host");
    let mut guest = Peer::new(&network, &REPLICANT, GUEST_NAME, "su_before_guest");
    host_with_a_campaign_save(&mut host);
    guest_with_a_template(&mut guest);
    host.launcher.lobby_mode = LOBBY_MODE_CONTINUE;
    let ticket = host.core.host().expect("ticket");
    host.start_game_link();
    guest.core.join(&ticket).expect("join");
    assert!(run_until(&mut [&mut host, &mut guest], |peers| peers
        .iter()
        .all(|peer| peer.is_connected())));
    host.launcher.refresh_saves();
    host.launcher.selected_slot = Some(HOST_SLOT);
    assert!(run_until(&mut [&mut host, &mut guest], |peers| peers[0]
        .launcher
        .scale_up_offered()));
    run_quietly(&mut [&mut host, &mut guest]);
    assert!(host.game.as_ref().unwrap().asked.is_empty());
    assert!(!guest.launcher.guest_waits_for_the_hosts_decision());
}
