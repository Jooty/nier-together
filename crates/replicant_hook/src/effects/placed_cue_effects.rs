use std::sync::{LazyLock, Mutex, PoisonError};

use nt_hook::line_budget::LineBudget;
use replicant_game::effects::{
    ActorCueCall, EFFECT_LOG_WINDOW_MS, LiveEffects, PUPPET_EFFECT_MAX_LIFETIME_MS,
    SentPlacedCueEffect, SentPlacedCueEffects, effect_is_finished, finish_effect,
};
use replicant_proto::{ActorCue, ActorCueKind, EnemyId};

use crate::build_gate::VerifiedBuild;
use crate::diagnostics::log_line;
use crate::effects::actor_cues::send;
use crate::effects::hooks::sending;
use crate::enemies::enemy_id_text;
use crate::memory::InProcessMemory;
use crate::net::net_clock_ms;

const PLACED_EFFECT_LOG_LINES_PER_WINDOW: u32 = 40;

static SENT_PLACED: LazyLock<Mutex<SentPlacedCueEffects>> = LazyLock::new(Mutex::default);
static REPLAYED_PLACED: LazyLock<Mutex<LiveEffects>> = LazyLock::new(Mutex::default);
static LOG_WINDOW: Mutex<LineBudget> = Mutex::new(LineBudget::counting_from_zero());

fn log_placed(line: &str) {
    let admitted = LOG_WINDOW
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .admit(
            net_clock_ms(),
            EFFECT_LOG_WINDOW_MS,
            PLACED_EFFECT_LOG_LINES_PER_WINDOW,
        );
    if admitted {
        log_line(line);
    }
}

pub(super) fn note_sent(handle: u32, enemy: EnemyId, id: u32) {
    log_placed(&format!(
        "actor cues: placed effect {id} of {} sent (host handle {handle:#x})",
        enemy_id_text(enemy)
    ));
    let evicted = SENT_PLACED
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .note_sent(SentPlacedCueEffect { handle, enemy, id });
    if let Some(evicted) = evicted {
        send_ended(evicted);
    }
}

pub(super) fn send_ended_effects(build: &VerifiedBuild) {
    let ended = {
        let mut sent = SENT_PLACED.lock().unwrap_or_else(PoisonError::into_inner);
        if sent.is_empty() {
            return;
        }
        let memory = InProcessMemory::new(build);
        sent.take_ended(|handle| effect_is_finished(&memory, handle).unwrap_or(true))
    };
    if !sending() {
        return;
    }
    for effect in ended {
        send_ended(effect);
    }
}

fn send_ended(effect: SentPlacedCueEffect) {
    send(
        ActorCue {
            actor: effect.enemy,
            id: effect.id,
            kind: ActorCueKind::PlacedEffectEnded {
                handle: effect.handle,
            },
        },
        ActorCueCall::PlacedEffectEnded,
    );
}

pub(super) fn forget_sent() {
    SENT_PLACED
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .take_all();
}

pub(super) fn note_replayed(
    build: &VerifiedBuild,
    cue: &ActorCue,
    host_handle: u32,
    own_handle: Option<u32>,
) {
    log_placed(&format!(
        "actor cues: placed effect {} of {} (host handle {host_handle:#x}) made here: {}",
        cue.id,
        enemy_id_text(cue.actor),
        own_handle.map_or_else(|| "refused".to_owned(), |own| format!("handle {own:#x}"))
    ));
    let Some(own_handle) = own_handle else {
        return;
    };
    let evicted = REPLAYED_PLACED
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .track(own_handle, host_handle, net_clock_ms());
    if let Some(evicted) = evicted {
        finish(build, evicted);
    }
}

pub(super) fn end_by_host(build: &VerifiedBuild, host_handle: u32) {
    let own = REPLAYED_PLACED
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .take_by_sender(host_handle);
    log_placed(&format!(
        "actor cues: placed effect (host handle {host_handle:#x}) ended on the host, {}",
        if own.is_some() {
            "ended here too"
        } else {
            "not made here"
        }
    ));
    if let Some(own) = own {
        finish(build, own);
    }
}

pub(super) fn end_expired(build: &VerifiedBuild) {
    let expired = {
        let mut replayed = REPLAYED_PLACED
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        if replayed.is_empty() {
            return;
        }
        replayed.take_expired(net_clock_ms())
    };
    for (own, host_handle) in expired {
        log_placed(&format!(
            "actor cues: placed effect {own:#x} (host handle {host_handle:#x}) ended by the {PUPPET_EFFECT_MAX_LIFETIME_MS} ms limit, no stop arrived"
        ));
        finish(build, own);
    }
}

pub(super) fn end_all_replayed(build: &VerifiedBuild) {
    let all = REPLAYED_PLACED
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .take_all();
    for own in all {
        finish(build, own);
    }
}

fn finish(build: &VerifiedBuild, handle: u32) {
    let _ = finish_effect(&mut InProcessMemory::new(build), handle);
}
