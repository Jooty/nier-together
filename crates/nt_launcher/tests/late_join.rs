use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};

use nt_launcher::app::{LaunchHost, Launcher, STEAM_RUN_URL_PREFIX};
use nt_launcher::config::LauncherConfig as SavedConfig;
use nt_launcher::home::GameCard;
use nt_launcher::install::ModStatus;
use nt_launcher::net::{LOBBY_MODE_CONTINUE, LOBBY_MODE_NEW_GAME, LobbyReady};
use nt_launcher::saves::{
    SlotSummary, campaign_for_host_slot, coop_campaigns_path, coop_gamedata_path, read_campaigns,
    save_dir, seat_summary_of, store_campaign_seat,
};
use nt_launcher::steam::GameInstall;
use nt_launcher_core::{
    ALL_PROFILES, AUTOMATA, ControllerCheck, FREE_ROAM, GameProfile, LauncherConfig, LauncherCore,
    LauncherEvent, LauncherNet, LinkStatus, NetState, REPLICANT, TransportFactory,
};
use nt_net::{
    Channel, Invite, LINK_INVITE_PREFIX, LINK_LAUNCHER_PEER, LocalLinkTransport, LoopbackNetwork,
    Transport, TransportEvent,
};
use nt_wire::{Envelope, decode_enveloped, encode_enveloped};
use replicant_game::story_progress::{
    SLOT_BYTES, SLOTS_FILE_OFFSET, fix_header_checksum, seat_bytes_of,
};
use replicant_proto::{Message, ScaleUpAnswer, ScaleUpAsk};

use crate::scratch::fresh_dir;
use crate::seat_files::{seat_with_play, template_file};

pub const WAIT_LIMIT: Duration = Duration::from_secs(20);
pub const QUIET_MILLIS: u64 = 300;
pub const ACCOUNT_FOLDER: &str = "76561198000000000";
pub const HOST_SLOT: u8 = 0;
pub const HOST_PLAYER: &str = "Jay";
pub const SEAT_PLAYER: &str = "Seat";
pub const SEAT_WAIT_PAST_ANY_LIMIT_SECONDS: f64 = 100_000.0;

pub struct NoControllers;

impl ControllerCheck for NoControllers {
    fn controller_connected(&self) -> bool {
        false
    }
}

pub struct ScratchHost {
    opened: Arc<Mutex<Vec<String>>>,
    cards: Vec<GameCard>,
    root: PathBuf,
}

impl LaunchHost for ScratchHost {
    fn open_url(&self, url: &str) -> io::Result<()> {
        self.opened.lock().expect("opened").push(url.to_owned());
        Ok(())
    }

    fn game_pids(&self, _profile: &GameProfile) -> Vec<u32> {
        Vec::new()
    }

    fn logs_dir(&self) -> Option<PathBuf> {
        Some(self.root.join("logs"))
    }

    fn data_dir(&self) -> Option<PathBuf> {
        Some(self.root.join("data"))
    }

    fn coop_backup_root(&self) -> Option<PathBuf> {
        Some(self.root.join("data").join("coop-backups"))
    }

    fn cards(&self, _config: &SavedConfig) -> Vec<GameCard> {
        self.cards.clone()
    }
}

pub fn found_card(profile: &'static GameProfile, root: &Path) -> GameCard {
    let game_dir = root.join(profile.install_folder_name);
    std::fs::create_dir_all(&game_dir).expect("game folder");
    std::fs::write(game_dir.join(profile.exe_names[0]), b"").expect("fake exe");
    let dll = game_dir.join(profile.hook_dll_name);
    std::fs::write(&dll, b"").expect("fake dll");
    GameCard {
        profile,
        install: Some(GameInstall {
            profile,
            game_dir,
            steamapps_dir: root.to_path_buf(),
            proton_prefix: Some(root.join("prefix")),
        }),
        dll_source: Some(dll),
        mod_status: ModStatus::Current,
        installed_build: Some("3f2a91c".to_owned()),
        ready_build: None,
        exe_supported: true,
    }
}

pub fn loopback_factory(network: &LoopbackNetwork) -> TransportFactory {
    let network = network.clone();
    Arc::new(move || Ok(Box::new(network.endpoint()) as Box<dyn Transport>))
}

pub struct FakeGame {
    pub link: LocalLinkTransport,
    pub asked: Vec<ScaleUpAsk>,
}

impl FakeGame {
    fn attach(core: &LauncherCore, profile: &GameProfile) -> Self {
        let mut link = LocalLinkTransport::game(LauncherConfig::for_game(profile, "x").build);
        let invite =
            Invite::new(format!("{LINK_INVITE_PREFIX}{}", core.link_port())).expect("invite");
        link.join(&invite).expect("dials");
        Self {
            link,
            asked: Vec::new(),
        }
    }

    fn keep_alive(&mut self) {
        let mut events: Vec<TransportEvent> = Vec::new();
        self.link.poll(&mut events);
        for event in events {
            if let TransportEvent::Message { payload, .. } = event
                && let Ok((_, Message::ScaleUpAsk(ask))) = decode_enveloped::<Message>(&payload)
            {
                self.asked.push(ask);
            }
        }
    }

    pub fn answer(&mut self, slot: u8, yes: bool) {
        let bytes = encode_enveloped(
            &Envelope { from: 0, to: None },
            &Message::ScaleUpAnswer(ScaleUpAnswer { slot, yes }),
        )
        .expect("encodes");
        let _ = self
            .link
            .send(LINK_LAUNCHER_PEER, Channel::ReliableOrdered, &bytes);
    }
}

pub struct Peer {
    pub name: String,
    pub profile: &'static GameProfile,
    pub launcher: Launcher,
    pub core: LauncherCore,
    pub opened: Arc<Mutex<Vec<String>>>,
    pub game_dir: PathBuf,
    pub saves: PathBuf,
    pub game: Option<FakeGame>,
    pub ready_sent: Option<LobbyReady>,
    pub clock: Instant,
    pub skew_seconds: f64,
    pub events: Vec<LauncherEvent>,
    pub follows_kind: bool,
}

impl Peer {
    pub fn new(
        network: &LoopbackNetwork,
        profile: &'static GameProfile,
        name: &str,
        scratch_name: &str,
    ) -> Self {
        let root = fresh_dir(scratch_name);
        let cards: Vec<GameCard> = ALL_PROFILES
            .iter()
            .map(|profile| found_card(profile, &root))
            .collect();
        let core = LauncherCore::for_game(profile, name, loopback_factory(network)).expect("core");
        let mut launcher = Launcher::with_config(SavedConfig::default());
        launcher.profile = profile;
        launcher.config.player_name = Some(name.to_owned());
        launcher.config.keyboard_mouse.asked = true;
        launcher.config_file = Some(root.join("config.toml"));
        launcher.link_port = Some(core.link_port());
        launcher.controller_check = Box::new(NoControllers);
        let opened = Arc::new(Mutex::new(Vec::new()));
        launcher.host = Box::new(ScratchHost {
            opened: opened.clone(),
            cards: cards.clone(),
            root: root.clone(),
        });
        launcher.refresh();
        let install = launcher.game.clone().expect("a game");
        let saves = save_dir(&install).unwrap_or_else(|| root.join("no-saves"));
        Self {
            name: name.to_owned(),
            profile,
            game_dir: install.game_dir.clone(),
            saves,
            opened,
            launcher,
            core,
            game: None,
            ready_sent: None,
            clock: Instant::now(),
            skew_seconds: 0.0,
            events: Vec::new(),
            follows_kind: true,
        }
    }

    pub fn account_folder(&self) -> PathBuf {
        self.saves.join("Steam").join(ACCOUNT_FOLDER)
    }

    pub fn coop_file(&self) -> PathBuf {
        coop_gamedata_path(&self.saves).expect("a co-op file path")
    }

    pub fn seconds(&self) -> f64 {
        self.clock.elapsed().as_secs_f64() + self.skew_seconds
    }

    pub fn frame(&mut self) {
        let events = self.core.drain_events();
        self.launcher.note_events(&events, &self.core);
        self.events.extend(events);
        self.launcher.link = self.core.link_status();
        if self.follows_kind {
            self.launcher.follow_the_hosts_kind();
        }
        let now = self.seconds();
        self.launcher.step_seat_round(&self.core, now);
        let ready = self.launcher.own_lobby_ready();
        if self.ready_sent != Some(ready) {
            self.core.send_lobby_ready(ready);
            self.ready_sent = Some(ready);
        }
        self.launcher.step_guest_launch(now);
        if let Some(game) = self.game.as_mut() {
            game.keep_alive();
        }
    }

    pub fn game_opens(&self) -> usize {
        self.opened
            .lock()
            .expect("opened")
            .iter()
            .filter(|url| **url == format!("{STEAM_RUN_URL_PREFIX}{}", self.profile.steam_app_id))
            .count()
    }

    pub fn start_game_link(&mut self) {
        self.game = Some(FakeGame::attach(&self.core, self.profile));
    }

    pub fn log_has(&self, text: &str) -> bool {
        self.launcher.log.iter().any(|line| line.contains(text))
    }

    pub fn link_file(&self) -> String {
        std::fs::read_to_string(self.game_dir.join(self.profile.link_file_name)).unwrap_or_default()
    }

    pub fn is_connected(&self) -> bool {
        matches!(self.core.status().state, NetState::Connected { .. })
    }
}

pub fn run_until(
    peers: &mut [&mut Peer],
    mut condition: impl FnMut(&mut [&mut Peer]) -> bool,
) -> bool {
    let deadline = Instant::now() + WAIT_LIMIT;
    while Instant::now() < deadline {
        for peer in peers.iter_mut() {
            peer.frame();
        }
        if condition(peers) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(3));
    }
    false
}

pub fn run_quietly(peers: &mut [&mut Peer]) {
    let until = Instant::now() + Duration::from_millis(QUIET_MILLIS);
    while Instant::now() < until {
        for peer in peers.iter_mut() {
            peer.frame();
        }
        std::thread::sleep(Duration::from_millis(3));
    }
}

pub fn name_the_guest(peer: &mut Peer) {
    peer.launcher.new_game_name = peer.name.clone();
}

pub fn host_a_running_game(host: &mut Peer, mode: u8) -> String {
    let ticket = host.core.host().expect("ticket");
    host.frame();
    host.launcher.lobby_mode = mode;
    host.launcher.press_start(&host.core, mode, host.seconds());
    assert_eq!(host.game_opens(), 1, "the host's game was asked for once");
    host.start_game_link();
    ticket
}

pub fn host_with_a_campaign_save(host: &mut Peer) {
    let account = host.account_folder();
    std::fs::create_dir_all(&account).unwrap();
    let mut file = template_file();
    file[SLOTS_FILE_OFFSET..SLOTS_FILE_OFFSET + SLOT_BYTES].copy_from_slice(&seat_with_play(
        HOST_PLAYER,
        500.0,
        1,
    ));
    file[0x6C] = 1;
    fix_header_checksum(&mut file).unwrap();
    std::fs::write(account.join("GAMEDATA.coop"), file).unwrap();
    host.launcher.refresh_saves();
    assert!(
        host.launcher
            .slots
            .iter()
            .any(|slot| slot.index == HOST_SLOT && slot.used),
        "the host's slot reads as used"
    );
}

pub fn guest_with_a_template(guest: &mut Peer) {
    let account = guest.account_folder();
    std::fs::create_dir_all(&account).unwrap();
    std::fs::write(account.join("GAMEDATA"), template_file()).unwrap();
    guest.launcher.refresh_saves();
}

pub fn give_the_campaign_a_seat(host: &Peer) -> Vec<u8> {
    let coop = host.coop_file();
    let slot = host
        .launcher
        .slots
        .iter()
        .find(|slot| slot.index == HOST_SLOT)
        .cloned()
        .expect("the host slot");
    campaign_for_host_slot(&coop, &slot_for_record(&slot)).expect("a campaign");
    let seat = seat_with_play(SEAT_PLAYER, 100.0, 2);
    store_campaign_seat(
        &coop,
        HOST_SLOT,
        &seat,
        seat_summary_of(&seat).expect("a summary"),
        1,
    )
    .expect("stored");
    seat
}

pub fn slot_for_record(slot: &SlotSummary) -> SlotSummary {
    slot.clone()
}

pub struct FileFacts {
    pub bytes: Vec<u8>,
    pub modified: SystemTime,
}

pub fn facts_of(path: &Path) -> FileFacts {
    FileFacts {
        bytes: std::fs::read(path).expect("readable"),
        modified: std::fs::metadata(path)
            .and_then(|meta| meta.modified())
            .expect("a modified time"),
    }
}

pub fn assert_untouched(path: &Path, before: &FileFacts, when: &str) {
    let after = facts_of(path);
    assert!(
        after.bytes == before.bytes,
        "{when}: the file's bytes changed"
    );
    assert!(
        after.modified == before.modified,
        "{when}: the file was written again"
    );
}

pub fn connected_both(host: &mut Peer, guest: &mut Peer) -> bool {
    run_until(&mut [host, guest], |peers| {
        peers.iter().all(|peer| peer.is_connected())
    })
}

pub fn the_cases() -> Vec<(&'static GameProfile, nt_wire::SessionRules)> {
    vec![
        (&REPLICANT, REPLICANT.modes[0].rules()),
        (&AUTOMATA, AUTOMATA.modes[0].rules()),
        (&AUTOMATA, FREE_ROAM.rules()),
    ]
}

#[test]
fn a_player_who_joins_a_host_already_in_game_launches_by_itself_in_every_game_and_mode() {
    for (index, (profile, rules)) in the_cases().into_iter().enumerate() {
        let network = LoopbackNetwork::new();
        let mut host = Peer::new(&network, profile, "Hosty", &format!("lj_new_host_{index}"));
        let mut guest = Peer::new(
            &network,
            profile,
            "Guesty",
            &format!("lj_new_guest_{index}"),
        );
        host.core.set_rules(rules);
        let ticket = host_a_running_game(&mut host, LOBBY_MODE_NEW_GAME);
        guest.core.join(&ticket).expect("join");
        guest.follows_kind = false;
        assert!(
            run_until(&mut [&mut host, &mut guest], |peers| peers[1].game_opens()
                == 1),
            "{} {:?}: the guest's game was never launched\n{:#?}",
            profile.display_name,
            rules.mode,
            guest.launcher.log
        );
        run_quietly(&mut [&mut host, &mut guest]);
        assert_eq!(guest.game_opens(), 1, "launched once, not again");
        assert_eq!(
            host.game_opens(),
            1,
            "the host's game was never started again"
        );
        assert!(
            !guest.link_file().contains("started_alone = true"),
            "{}: the late guest's link file says it started alone:\n{}",
            profile.display_name,
            guest.link_file()
        );
    }
}

#[test]
fn a_start_that_reaches_a_guest_who_is_not_ready_launches_the_game_once_it_is() {
    let network = LoopbackNetwork::new();
    let mut host = Peer::new(&network, &REPLICANT, "Hosty", "lj_notready_host");
    let mut guest = Peer::new(&network, &REPLICANT, "Guesty", "lj_notready_guest");
    guest.launcher.new_game_name = String::new();
    let ticket = host_a_running_game(&mut host, LOBBY_MODE_NEW_GAME);
    guest.core.join(&ticket).expect("join");
    guest.follows_kind = false;
    assert!(run_until(&mut [&mut host, &mut guest], |peers| {
        peers[1]
            .events
            .iter()
            .any(|event| matches!(event, LauncherEvent::LobbyStartReceived { .. }))
    }));
    run_quietly(&mut [&mut host, &mut guest]);
    assert!(!guest.launcher.own_lobby_ready().ready, "still not ready");
    assert_eq!(guest.game_opens(), 0, "nothing launched while not ready");
    name_the_guest(&mut guest);
    assert!(run_until(&mut [&mut host, &mut guest], |peers| {
        peers[1].game_opens() == 1
    }));
}

#[test]
fn a_late_guest_in_continue_gets_the_seat_before_its_game_starts_and_the_hosts_file_is_never_written()
 {
    let network = LoopbackNetwork::new();
    let mut host = Peer::new(&network, &REPLICANT, "Hosty", "lj_seat_host");
    let mut guest = Peer::new(&network, &REPLICANT, "Guesty", "lj_seat_guest");
    host_with_a_campaign_save(&mut host);
    guest_with_a_template(&mut guest);
    let seat = give_the_campaign_a_seat(&host);
    let coop = host.coop_file();
    let before = facts_of(&coop);
    let campaigns = coop_campaigns_path(&host.saves).expect("a record path");
    let ticket = host_a_running_game(&mut host, LOBBY_MODE_CONTINUE);
    assert_untouched(&coop, &before, "after the host started");
    guest.core.join(&ticket).expect("join");
    guest.follows_kind = false;
    let mut seat_file_when_launched = None;
    let launched = run_until(&mut [&mut host, &mut guest], |peers| {
        assert_untouched(&coop, &before, "while the guest joined");
        if peers[1].game_opens() == 1 && seat_file_when_launched.is_none() {
            seat_file_when_launched = peers[1]
                .launcher
                .campaign_card()
                .map(|_| peers[1].account_folder());
        }
        peers[1].game_opens() == 1
    });
    assert!(launched, "{:#?}", guest.launcher.log);
    let folder = seat_file_when_launched.expect("a folder");
    let session_files: Vec<PathBuf> = std::fs::read_dir(&folder)
        .unwrap()
        .filter_map(|entry| Some(entry.ok()?.path()))
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("GAMEDATA.seat-"))
        })
        .collect();
    assert_eq!(
        session_files.len(),
        1,
        "one session file with the seat in it"
    );
    let session = std::fs::read(&session_files[0]).unwrap();
    assert!(
        seat_bytes_of(&session, 0).expect("a seat") == seat,
        "the guest's game boots into the host's seat"
    );
    let link = guest.link_file();
    assert!(
        link.contains("coop_save_file"),
        "the link names the session file:\n{link}"
    );
    run_quietly(&mut [&mut host, &mut guest]);
    assert_untouched(&coop, &before, "at the end");
    assert!(
        campaigns.is_file(),
        "the campaign record is beside the host's file"
    );
    assert!(read_campaigns(&campaigns)[&HOST_SLOT].seat.is_some());
}

#[test]
fn a_late_guests_save_comes_back_to_the_host_without_the_hosts_file_being_written() {
    let network = LoopbackNetwork::new();
    let mut host = Peer::new(&network, &REPLICANT, "Hosty", "lj_back_host");
    let mut guest = Peer::new(&network, &REPLICANT, "Guesty", "lj_back_guest");
    host_with_a_campaign_save(&mut host);
    guest_with_a_template(&mut guest);
    give_the_campaign_a_seat(&host);
    let coop = host.coop_file();
    let before = facts_of(&coop);
    let ticket = host_a_running_game(&mut host, LOBBY_MODE_CONTINUE);
    guest.core.join(&ticket).expect("join");
    guest.follows_kind = false;
    assert!(run_until(&mut [&mut host, &mut guest], |peers| peers[1]
        .game_opens()
        == 1));
    let session = guest
        .launcher
        .guest_session_path()
        .expect("the guest's session file");
    let newer = seat_with_play(SEAT_PLAYER, 900.0, 3);
    let mut file = std::fs::read(&session).unwrap();
    file[SLOTS_FILE_OFFSET..SLOTS_FILE_OFFSET + SLOT_BYTES].copy_from_slice(&newer);
    fix_header_checksum(&mut file).unwrap();
    std::fs::write(&session, file).unwrap();
    let saved = [LauncherEvent::GameSaved { succeeded: true }];
    guest.launcher.note_events(&saved, &guest.core);
    let campaigns = coop_campaigns_path(&host.saves).expect("a record path");
    assert!(run_until(&mut [&mut host, &mut guest], |_| {
        read_campaigns(&campaigns)
            .get(&HOST_SLOT)
            .and_then(|campaign| campaign.seat.as_ref())
            .is_some_and(|seat| seat.play_seconds > 800.0)
    }));
    assert_untouched(&coop, &before, "after the guest's save arrived");
}

#[test]
fn a_guest_joining_while_the_hosts_game_is_still_launching_waits_for_it() {
    let network = LoopbackNetwork::new();
    let mut host = Peer::new(&network, &AUTOMATA, "Hosty", "lj_launching_host");
    let mut guest = Peer::new(&network, &AUTOMATA, "Guesty", "lj_launching_guest");
    let ticket = host.core.host().expect("ticket");
    host.frame();
    host.launcher
        .press_start(&host.core, LOBBY_MODE_NEW_GAME, host.seconds());
    assert_eq!(host.game_opens(), 1);
    guest.core.join(&ticket).expect("join");
    assert!(run_until(&mut [&mut host, &mut guest], |peers| {
        peers[1].launcher.lobby_started
    }));
    run_quietly(&mut [&mut host, &mut guest]);
    assert_eq!(guest.game_opens(), 0, "the host's game is not up yet");
    host.start_game_link();
    assert!(run_until(&mut [&mut host, &mut guest], |peers| peers[1]
        .game_opens()
        == 1));
}

#[test]
fn two_guests_joining_a_running_free_roam_host_at_once_each_launch_once() {
    let network = LoopbackNetwork::new();
    let mut host = Peer::new(&network, &AUTOMATA, "Hosty", "lj_three_host");
    let mut first = Peer::new(&network, &AUTOMATA, "First", "lj_three_first");
    let mut second = Peer::new(&network, &AUTOMATA, "Second", "lj_three_second");
    host.core.set_rules(FREE_ROAM.rules());
    let ticket = host_a_running_game(&mut host, LOBBY_MODE_NEW_GAME);
    first.core.join(&ticket).expect("join");
    second.core.join(&ticket).expect("join");
    assert!(
        run_until(&mut [&mut host, &mut first, &mut second], |peers| {
            peers[1].game_opens() == 1 && peers[2].game_opens() == 1
        }),
        "{:#?}\n{:#?}",
        first.launcher.log,
        second.launcher.log
    );
    run_quietly(&mut [&mut host, &mut first, &mut second]);
    assert_eq!(first.game_opens(), 1);
    assert_eq!(second.game_opens(), 1);
    assert_eq!(host.game_opens(), 1);
}

#[test]
fn a_guest_who_leaves_while_another_waits_does_not_cancel_the_other() {
    let network = LoopbackNetwork::new();
    let mut host = Peer::new(&network, &AUTOMATA, "Hosty", "lj_other_host");
    let mut first = Peer::new(&network, &AUTOMATA, "First", "lj_other_first");
    let mut second = Peer::new(&network, &AUTOMATA, "Second", "lj_other_second");
    host.core.set_rules(FREE_ROAM.rules());
    let ticket = host.core.host().expect("ticket");
    host.frame();
    host.launcher
        .press_start(&host.core, LOBBY_MODE_NEW_GAME, host.seconds());
    first.core.join(&ticket).expect("join");
    second.core.join(&ticket).expect("join");
    assert!(run_until(
        &mut [&mut host, &mut first, &mut second],
        |peers| { peers[1].launcher.lobby_started && peers[2].launcher.lobby_started }
    ));
    first.core.leave();
    run_quietly(&mut [&mut host, &mut first, &mut second]);
    assert!(
        second.launcher.lobby_started,
        "the second guest still waits for the host's game"
    );
    host.start_game_link();
    assert!(run_until(&mut [&mut host, &mut second], |peers| peers[1]
        .game_opens()
        == 1));
    assert_eq!(first.game_opens(), 0);
}

#[test]
fn a_guest_who_leaves_and_joins_again_is_started_again() {
    let network = LoopbackNetwork::new();
    let mut host = Peer::new(&network, &AUTOMATA, "Hosty", "lj_again_host");
    let mut guest = Peer::new(&network, &AUTOMATA, "Guesty", "lj_again_guest");
    let ticket = host_a_running_game(&mut host, LOBBY_MODE_NEW_GAME);
    guest.core.join(&ticket).expect("join");
    assert!(run_until(&mut [&mut host, &mut guest], |peers| peers[1]
        .game_opens()
        == 1));
    guest.core.leave();
    assert!(run_until(&mut [&mut host, &mut guest], |peers| {
        peers[0]
            .events
            .iter()
            .any(|event| matches!(event, LauncherEvent::MemberLeft { .. }))
    }));
    guest.core.join(&ticket).expect("join again");
    assert!(
        run_until(&mut [&mut host, &mut guest], |peers| peers[1].game_opens()
            == 2),
        "{:#?}",
        guest.launcher.log
    );
    assert_eq!(host.game_opens(), 1);
}

#[test]
fn a_host_who_closes_the_lobby_while_a_guest_waits_cancels_the_start() {
    let network = LoopbackNetwork::new();
    let mut host = Peer::new(&network, &AUTOMATA, "Hosty", "lj_quit_host");
    let mut guest = Peer::new(&network, &AUTOMATA, "Guesty", "lj_quit_guest");
    let ticket = host.core.host().expect("ticket");
    host.frame();
    host.launcher
        .press_start(&host.core, LOBBY_MODE_NEW_GAME, host.seconds());
    guest.core.join(&ticket).expect("join");
    assert!(run_until(&mut [&mut host, &mut guest], |peers| {
        peers[1].launcher.lobby_started
    }));
    host.core.leave();
    assert!(run_until(&mut [&mut host, &mut guest], |peers| {
        !peers[1].launcher.lobby_started
    }));
    host.start_game_link();
    run_quietly(&mut [&mut host, &mut guest]);
    assert_eq!(
        guest.game_opens(),
        0,
        "a closed lobby never launches a guest"
    );
}

#[test]
fn a_seat_that_never_arrives_stops_the_late_guests_launch_with_words() {
    let network = LoopbackNetwork::new();
    let mut host = Peer::new(&network, &REPLICANT, "Hosty", "lj_noseat_host");
    let mut guest = Peer::new(&network, &REPLICANT, "Guesty", "lj_noseat_guest");
    host_with_a_campaign_save(&mut host);
    give_the_campaign_a_seat(&host);
    let ticket = host_a_running_game(&mut host, LOBBY_MODE_CONTINUE);
    guest.core.join(&ticket).expect("join");
    guest.follows_kind = false;
    run_until(&mut [&mut host, &mut guest], |peers| {
        peers[1]
            .events
            .iter()
            .any(|event| matches!(event, LauncherEvent::LobbyStartReceived { .. }))
    });
    guest.skew_seconds = SEAT_WAIT_PAST_ANY_LIMIT_SECONDS;
    guest.frame();
    guest.frame();
    assert_eq!(guest.game_opens(), 0);
    assert!(
        guest.log_has("save for your seat"),
        "the guest is told why: {:#?}",
        guest.launcher.log
    );
}

#[test]
fn joining_before_the_start_is_unchanged_the_guest_launches_when_the_host_presses_start() {
    for (index, (profile, rules)) in the_cases().into_iter().enumerate() {
        let network = LoopbackNetwork::new();
        let mut host = Peer::new(
            &network,
            profile,
            "Hosty",
            &format!("lj_before_host_{index}"),
        );
        let mut guest = Peer::new(
            &network,
            profile,
            "Guesty",
            &format!("lj_before_guest_{index}"),
        );
        host.core.set_rules(rules);
        let ticket = host.core.host().expect("ticket");
        guest.core.join(&ticket).expect("join");
        assert!(connected_both(&mut host, &mut guest));
        assert!(run_until(&mut [&mut host, &mut guest], |peers| {
            peers[0]
                .launcher
                .link
                .partner()
                .and_then(|partner| partner.lobby)
                .is_some_and(|lobby| lobby.ready)
        }));
        run_quietly(&mut [&mut host, &mut guest]);
        assert_eq!(guest.game_opens(), 0, "nothing starts before Start");
        host.launcher
            .press_start(&host.core, LOBBY_MODE_NEW_GAME, host.seconds());
        host.start_game_link();
        assert!(run_until(&mut [&mut host, &mut guest], |peers| peers[1]
            .game_opens()
            == 1));
        run_quietly(&mut [&mut host, &mut guest]);
        assert_eq!(guest.game_opens(), 1);
        assert_eq!(host.game_opens(), 1);
    }
}

#[test]
fn a_third_player_cannot_join_a_two_player_campaign_and_the_two_play_on() {
    let network = LoopbackNetwork::new();
    let mut host = Peer::new(&network, &REPLICANT, "Hosty", "lj_full_host");
    let mut guest = Peer::new(&network, &REPLICANT, "Guesty", "lj_full_guest");
    let mut third = Peer::new(&network, &REPLICANT, "Third", "lj_full_third");
    let ticket = host_a_running_game(&mut host, LOBBY_MODE_NEW_GAME);
    guest.core.join(&ticket).expect("join");
    assert!(run_until(&mut [&mut host, &mut guest], |peers| peers[1]
        .game_opens()
        == 1));
    third.core.join(&ticket).expect("join starts");
    run_quietly(&mut [&mut host, &mut guest, &mut third]);
    assert_eq!(third.game_opens(), 0);
    assert!(guest.is_connected());
    assert_eq!(guest.game_opens(), 1);
    assert_eq!(host.game_opens(), 1);
    let status: LinkStatus = host.core.link_status();
    assert_eq!(status.members.len(), 1);
}

#[test]
fn a_guest_whose_tab_says_continue_still_launches_the_hosts_new_game() {
    for (index, (profile, rules)) in the_cases().into_iter().enumerate() {
        let network = LoopbackNetwork::new();
        let mut host = Peer::new(&network, profile, "Hosty", &format!("lj_rev_host_{index}"));
        let mut guest = Peer::new(
            &network,
            profile,
            "Guesty",
            &format!("lj_rev_guest_{index}"),
        );
        host.core.set_rules(rules);
        let ticket = host_a_running_game(&mut host, LOBBY_MODE_NEW_GAME);
        guest.follows_kind = false;
        guest.launcher.lobby_mode = LOBBY_MODE_CONTINUE;
        guest.core.join(&ticket).expect("join");
        assert!(
            run_until(&mut [&mut host, &mut guest], |peers| peers[1].game_opens()
                == 1),
            "{:#?}",
            guest.launcher.log
        );
        assert_eq!(guest.launcher.lobby_mode, LOBBY_MODE_NEW_GAME);
        assert!(
            profile.game_id != REPLICANT.game_id || guest.link_file().contains("new_game"),
            "{}: the guest starts a new game:\n{}",
            profile.display_name,
            guest.link_file()
        );
    }
}
