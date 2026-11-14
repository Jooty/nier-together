use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

use replicant_game::Replicant;
use replicant_game::rule_rooms::BreakerBodyHold;
use replicant_game::stay_together::FollowRole;
use replicant_game::story_watch::{
    GUEST_WORD_RESENT_EVERY_MS, GuestWatch, RemovalChange, WATCH_LINE_EVERY_MS, WatchEnd,
    WatchFacts, WatchStep, camera_line, ended_line, started_line, watch_hold_line,
    watch_let_go_line, watching_host_text,
};
use replicant_proto::{GuestWatches, Message};

use crate::build_gate::VerifiedBuild;
use crate::diagnostics::log_line;
use crate::event_log::{clear_persistent_event, set_persistent_event};
use crate::memory::InProcessMemory;
use crate::net::{self, NetState, net_clock_ms};
use crate::slot_zero::{read_slot_zero, write_slot_zero};
use crate::story_watch::body::{Placed, WatchedBody};
use crate::story_watch::host_side::{forget_guest, tick_host};
use crate::story_watch::removal_natives::{forget_removal, step_removal, story_has_taken_her};
use crate::story_watch::setting::guest_watches_setting;

const HOST_POSITION_FRESH_FOR: Duration = Duration::from_millis(1_500);

static GUEST: Mutex<GuestState> = Mutex::new(GuestState::new());
static THIS_GAME_IS_WATCHING: AtomicBool = AtomicBool::new(false);
static WATCH_HOLD: Mutex<BreakerBodyHold> = Mutex::new(BreakerBodyHold::new());

struct GuestState {
    watch: GuestWatch,
    body: WatchedBody,
    last_word: Option<(bool, u64)>,
    last_line_ms: u64,
    last_move_line_ms: u64,
    last_pack: String,
    controls_locked_in: Option<String>,
    host_position: Option<([f32; 3], Instant)>,
    last_placed: &'static str,
    last_error: f32,
    last_own: [f32; 3],
}

impl GuestState {
    const fn new() -> Self {
        Self {
            watch: GuestWatch::new(),
            body: WatchedBody::new(),
            last_word: None,
            last_line_ms: 0,
            last_move_line_ms: 0,
            last_pack: String::new(),
            controls_locked_in: None,
            host_position: None,
            last_placed: "no body placed yet",
            last_error: 0.0,
            last_own: [0.0; 3],
        }
    }
}

pub(crate) fn this_game_is_watching() -> bool {
    THIS_GAME_IS_WATCHING.load(Ordering::Relaxed)
}

pub(crate) fn tick_story_watch(build: &VerifiedBuild) {
    if let Some(change) = step_removal() {
        log_line(match change {
            RemovalChange::TakenOut => {
                "watching: the story has kept the companion out of play for the settle time"
            }
            RemovalChange::BroughtBack => {
                "watching: the story has had the companion in play for the settle time"
            }
        });
    }
    let Some(net) = net::handle() else {
        end_without_session(build);
        forget_guest();
        forget_removal();
        return;
    };
    let view = net.view();
    let in_session = view.state == NetState::Connected && view.partner_state().is_some();
    let partner_name = view.partner_state().map(|remote| remote.name.clone());
    match FollowRole::from_slot(view.local_slot) {
        FollowRole::Host => tick_host(net, partner_name.as_deref().filter(|_| in_session)),
        FollowRole::Guest => tick_guest(build, net, in_session, partner_name.as_deref()),
        FollowRole::Unknown => {}
    }
    if !in_session {
        forget_removal();
    }
}

pub(crate) fn follow_the_host(build: &VerifiedBuild, host: [f32; 3]) {
    let mut guest = GUEST.lock().unwrap_or_else(PoisonError::into_inner);
    guest.host_position = Some((host, Instant::now()));
    if !guest.watch.watching() {
        return;
    }
    let (placed, error, own) = guest.body.place_at(build, host);
    guest.last_placed = match placed {
        Placed::Left => "placed",
        Placed::Written => "placed (written)",
        Placed::Teleported => "placed (the game's own move)",
        Placed::NoBody => "no body to place",
    };
    guest.last_error = error;
    guest.last_own = own;
}

fn tick_guest(
    build: &VerifiedBuild,
    net: &net::NetThread,
    in_session: bool,
    host_name: Option<&str>,
) {
    let game = Replicant::new(InProcessMemory::new(build));
    let phase = game.current_phase().ok();
    let mut guest = GUEST.lock().unwrap_or_else(PoisonError::into_inner);
    if let Some(pack) = &phase {
        pack.clone_into(&mut guest.last_pack);
    }
    let pack = guest.last_pack.clone();
    let facts = WatchFacts {
        setting_on: guest_watches_setting(),
        guest_in_session: in_session,
        story_has_taken_her: story_has_taken_her(),
        pack: &pack,
    };
    let host = host_name.unwrap_or("the host");
    match guest.watch.step(&facts) {
        WatchStep::Started => {
            log_line(&started_line(host, &pack));
            guest.controls_locked_in = None;
        }
        WatchStep::Ended(end) => end_watch(build, &mut guest, end),
        WatchStep::Idle | WatchStep::Continues => {}
    }
    THIS_GAME_IS_WATCHING.store(guest.watch.watching(), Ordering::Relaxed);
    hold_the_body_still(build, guest.watch.watching());
    if guest.watch.watching() {
        keep_watching(build, &mut guest, phase.as_deref(), host);
    }
    tell_the_host(net, &mut guest);
}

fn keep_watching(build: &VerifiedBuild, guest: &mut GuestState, phase: Option<&str>, host: &str) {
    set_persistent_event(&watching_host_text(host));
    let Some(phase) = phase else {
        guest.controls_locked_in = None;
        return;
    };
    guest.body.hide_and_protect(build);
    if guest.controls_locked_in.as_deref() != Some(phase) {
        guest.body.lock_controls(build);
        guest.controls_locked_in = Some(phase.to_owned());
    }
    let now = net_clock_ms();
    if now.saturating_sub(guest.last_move_line_ms) >= WATCH_LINE_EVERY_MS
        && let Some((speed, keys)) = guest.body.moves_on_keys(build)
    {
        guest.last_move_line_ms = now;
        log_line(&format!(
            "watching: this game's hidden body moves on held keys {keys:?} at speed {speed:.2}, the player-input hold is not enough"
        ));
    }
    if now.saturating_sub(guest.last_line_ms) >= WATCH_LINE_EVERY_MS {
        guest.last_line_ms = now;
        let host_at = guest.host_position.map_or([0.0; 3], |(at, _)| at);
        log_line(&camera_line(
            guest.last_own,
            host_at,
            guest.last_error,
            guest.last_placed,
        ));
    }
}

fn end_watch(build: &VerifiedBuild, guest: &mut GuestState, end: WatchEnd) {
    log_line(&ended_line(end));
    let host = guest
        .host_position
        .filter(|(_, at)| at.elapsed() < HOST_POSITION_FRESH_FOR)
        .map(|(at, _)| at);
    let in_play = Replicant::new(InProcessMemory::new(build))
        .current_phase()
        .is_ok();
    if in_play {
        guest.body.give_back(build, host);
    }
    guest.controls_locked_in = None;
    clear_persistent_event();
}

fn end_without_session(build: &VerifiedBuild) {
    let mut guest = GUEST.lock().unwrap_or_else(PoisonError::into_inner);
    let facts = WatchFacts {
        setting_on: guest_watches_setting(),
        guest_in_session: false,
        story_has_taken_her: false,
        pack: "",
    };
    if let WatchStep::Ended(end) = guest.watch.step(&facts) {
        end_watch(build, &mut guest, end);
    }
    THIS_GAME_IS_WATCHING.store(guest.watch.watching(), Ordering::Relaxed);
    hold_the_body_still(build, guest.watch.watching());
}

fn hold_the_body_still(build: &VerifiedBuild, watching: bool) {
    let Some(slot_zero) = read_slot_zero(build) else {
        return;
    };
    let mut hold = WATCH_HOLD.lock().unwrap_or_else(PoisonError::into_inner);
    let Some(written) = hold.slot_zero_to_write(slot_zero, watching) else {
        return;
    };
    let holding = hold.is_holding();
    drop(hold);
    write_slot_zero(build, written);
    log_line(&if holding {
        watch_hold_line(slot_zero, written)
    } else {
        watch_let_go_line(slot_zero, written)
    });
}

fn tell_the_host(net: &net::NetThread, guest: &mut GuestState) {
    let now = net_clock_ms();
    let watching = guest.watch.watching();
    let due = match guest.last_word {
        Some((said, at)) => {
            said != watching || (watching && now.saturating_sub(at) >= GUEST_WORD_RESENT_EVERY_MS)
        }
        None => watching,
    };
    if !due {
        return;
    }
    net.send_relayed(Message::GuestWatches(GuestWatches { watching }));
    guest.last_word = Some((watching, now));
}
