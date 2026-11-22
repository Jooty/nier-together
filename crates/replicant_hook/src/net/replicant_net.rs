use nt_hook::diagnostics::log_line;
use nt_hook::net::{LinkAccess, NetGame, PlayerOf, log_readiness, net_clock_ms};
use nt_net::Channel;
use replicant_game::PHASE_TABLE;
use replicant_game::carts::partner_wants_to_ride_text;
use replicant_game::readiness::{HEARTBEAT_INTERVAL_MS, PartnerWatch, ReadinessChangeLog};
use replicant_game::scale_up_prompt::AskEffect;
use replicant_proto::{GameReadiness, Message, PartnerNotice, PlayerState};

use crate::event_log::post_event;
use crate::net::NetState;
use crate::net::link_state::ReplicantLinkState;
use crate::net::local_snapshot::LocalSnapshot;
use crate::net::shared::RemotePlayer;

const HOST_SLOT: u8 = 0;
const READINESS_SENT: &str = "sent";
const READINESS_SEND_REFUSED: &str = "refused by the link, retrying";
const READINESS_NOT_SENT_YET: &str = "nothing sent yet";

#[derive(Debug)]
pub struct ReplicantNet {
    readiness_sent: Option<(GameReadiness, u64)>,
    readiness_log: ReadinessChangeLog,
    last_readiness_send: &'static str,
}

impl Default for ReplicantNet {
    fn default() -> Self {
        Self {
            readiness_sent: None,
            readiness_log: ReadinessChangeLog::default(),
            last_readiness_send: READINESS_NOT_SENT_YET,
        }
    }
}

impl NetGame for ReplicantNet {
    type Message = Message;
    type Snapshot = LocalSnapshot;
    type PeerState = PlayerState;
    type PlayerWatch = PartnerWatch;
    type Extras = ReplicantLinkState;

    fn kind_name(message: &Message) -> &'static str {
        message.kind_name()
    }

    fn player_in_play(watch: &PartnerWatch) -> bool {
        watch.is_in_play()
    }

    fn snapshot_message(&mut self, snapshot: &LocalSnapshot, tick: u32) -> Message {
        Message::State(snapshot.to_player_state(tick))
    }

    fn on_message(&mut self, link: &LinkAccess<'_, Self>, from: u8, message: Message) {
        match message {
            Message::Readiness(readiness) => self.on_partner_readiness(link, from, readiness),
            Message::Notice(notice) => on_partner_notice(link, from, notice),
            Message::State(state) => on_partner_state(link, from, state),
            Message::SessionOptions(options) => link.shared().options = options.clamped(),
            Message::Settings(settings) => link.shared().pending_settings = Some(settings),
            Message::PathInfo(info) => {
                let mut shared = link.shared();
                shared.rtt = Some(std::time::Duration::from_micros(u64::from(info.rtt_micros)));
                shared.direct_path = Some(info.direct);
            }
            Message::ScaleUpAsk(ask) => {
                let effect = link.shared().scale_up.on_ask(ask);
                if let AskEffect::Line(line) = effect {
                    post_event(&line);
                }
            }
            Message::QuitGame => {
                log_line("net: the launcher asked this game to quit");
                link.shared().quit_requested = true;
            }
            relayed if relayed.is_relayed() => {
                let sender_in_play = link
                    .shared()
                    .player(from)
                    .is_some_and(|sender| sender.watch.is_in_play());
                if !sender_in_play {
                    return;
                }
                link.push_relayed(from, relayed);
            }
            _ => {}
        }
    }

    fn on_player_joined(&mut self, extras: &mut ReplicantLinkState, player: &mut PlayerOf<Self>) {
        let host_is_playing = extras.effective_local_readiness().is_in_play();
        if player.slot != HOST_SLOT && host_is_playing {
            log_line(&format!(
                "net: {} joined while this game plays, taking them in at a safe moment",
                player.name
            ));
            player
                .watch
                .partner_connected_while_host_plays(net_clock_ms());
        } else {
            player.watch.partner_connected(net_clock_ms());
        }
    }

    fn on_player_left(&mut self, extras: &mut ReplicantLinkState, _slot: u8) {
        self.on_peer_cleared(extras);
    }

    fn on_peer_cleared(&mut self, extras: &mut ReplicantLinkState) {
        extras.options = replicant_proto::SessionOptions::default();
        extras.scale_up.clear();
    }

    fn on_launcher_linked(&mut self) {
        self.readiness_sent = None;
        self.readiness_log.reset();
        self.last_readiness_send = READINESS_NOT_SENT_YET;
    }

    fn on_idle(
        &mut self,
        link: &LinkAccess<'_, Self>,
        send: &mut dyn FnMut(&Message, Channel) -> bool,
    ) {
        assume_silent_partner_in_play(link);
        take_in_joiners_at_a_safe_moment(link);
        self.send_readiness(link, send);
    }

    fn announce(&self, text: &str) {
        post_event(text);
    }
}

impl ReplicantNet {
    fn on_partner_readiness(
        &mut self,
        link: &LinkAccess<'_, Self>,
        from: u8,
        readiness: GameReadiness,
    ) {
        log_readiness(&format!("readiness: partner {readiness:?}"));
        let mut shared = link.shared();
        let Some(player) = shared.players.get_mut(&from) else {
            log_readiness(&format!(
                "readiness: no player in slot {from}, report dropped"
            ));
            return;
        };
        let was_in_play = player.watch.is_in_play();
        let before = player.watch.readiness();
        let just_closed = player.watch.partner_reported(readiness);
        let still_in_play = player.watch.is_in_play();
        let name = player.name.clone();
        if was_in_play != still_in_play {
            player.latest = None;
        }
        drop(shared);
        if just_closed {
            log_readiness("readiness: partner game closed");
            post_event(&format!("{name}'s game closed"));
        } else if still_in_play && !was_in_play {
            post_event(&format!("{name}'s game is in play"));
        } else if readiness == GameReadiness::Loading && before != GameReadiness::Loading {
            post_event(&format!("{name}'s game is loading"));
        }
        if was_in_play != still_in_play {
            link.clear_relayed();
        }
    }

    fn send_readiness(
        &mut self,
        link: &LinkAccess<'_, Self>,
        send: &mut dyn FnMut(&Message, Channel) -> bool,
    ) {
        let (state, current, waiting_on, reads) = {
            let shared = link.shared();
            (
                shared.state.clone(),
                shared.effective_local_readiness(),
                shared.local_readiness_wait(),
                shared.local_reads.describe(),
            )
        };
        if state == NetState::WaitingForLauncher {
            return;
        }
        let now_ms = net_clock_ms();
        let due = match self.readiness_sent {
            None => true,
            Some((sent, _)) if sent != current => true,
            Some((_, at_ms)) => now_ms.saturating_sub(at_ms) >= HEARTBEAT_INTERVAL_MS,
        };
        if due {
            if send(&Message::Readiness(current), Channel::ReliableOrdered) {
                self.readiness_sent = Some((current, now_ms));
                self.last_readiness_send = READINESS_SENT;
            } else {
                self.last_readiness_send = READINESS_SEND_REFUSED;
            }
        }
        if let Some(line) = self.readiness_log.observe(
            now_ms,
            current,
            &waiting_on,
            &reads,
            self.last_readiness_send,
        ) {
            log_readiness(&line);
        }
    }
}

fn on_partner_state(link: &LinkAccess<'_, ReplicantNet>, from: u8, state: PlayerState) {
    let mut shared = link.shared();
    let Some(player) = shared.players.get_mut(&from) else {
        return;
    };
    let promoted = player.watch.partner_sent_state();
    if player.watch.is_waiting_for_safe_moment() {
        return;
    }
    let latest = RemotePlayer {
        name: player.name.clone(),
        slot: from,
        state,
        received_at: std::time::Instant::now(),
    };
    let name = latest.name.clone();
    player.latest = Some(latest.clone());
    drop(shared);
    if promoted {
        log_readiness("readiness: partner in play (it sent its position)");
        post_event(&format!("{name}'s game is in play"));
    }
}

fn on_partner_notice(link: &LinkAccess<'_, ReplicantNet>, from: u8, notice: PartnerNotice) {
    let Some(name) = link.shared().player(from).map(|sender| sender.name.clone()) else {
        return;
    };
    match notice {
        PartnerNotice::WantsToChangeZone => {
            post_event(&format!("{name} wants to change zone"));
        }
        PartnerNotice::HostWantsToGo { phase } => {
            let area = phase
                .and_then(|phase| PHASE_TABLE.get(usize::from(phase)))
                .map_or("another area", |entry| entry.area);
            post_event(&format!("{name} wants to go to {area}"));
        }
        PartnerNotice::RejoiningYourArea => {
            post_event(&format!("{name} is rejoining your area"));
        }
        PartnerNotice::WantsToRideTheCart => {
            post_event(&partner_wants_to_ride_text(&name));
        }
    }
}

fn assume_silent_partner_in_play(link: &LinkAccess<'_, ReplicantNet>) {
    let now_ms = net_clock_ms();
    let assumed: Vec<String> = link
        .shared()
        .players
        .values_mut()
        .filter_map(|player| {
            player
                .watch
                .assume_in_play_if_silent(now_ms)
                .then(|| player.name.clone())
        })
        .collect();
    for name in assumed {
        log_readiness("readiness: partner never reported, assuming in play");
        post_event(&format!("{name}'s game is in play"));
    }
}

fn take_in_joiners_at_a_safe_moment(link: &LinkAccess<'_, ReplicantNet>) {
    let mut shared = link.shared();
    let safe = shared.safe_to_take_a_joiner();
    let taken_in: Vec<String> = shared
        .players
        .values_mut()
        .filter_map(|player| {
            player
                .watch
                .take_in_if_safe(safe)
                .then(|| player.name.clone())
        })
        .collect();
    drop(shared);
    for name in taken_in {
        log_readiness("readiness: a joiner is taken in, this game is at a safe moment");
        post_event(&format!("{name}'s game is in play"));
    }
}
