//! What a shared actor's own AI or state step makes: sent by the host as actor cues, made by the
//! guest on its copy, which never runs those steps.

use core::cell::Cell;
use core::ffi::c_void;
use core::ptr;
use std::panic;
use std::sync::atomic::{AtomicPtr, AtomicU32, AtomicU64, Ordering};
use std::sync::{LazyLock, Mutex, OnceLock, PoisonError};

use windows_sys::Win32::System::Diagnostics::Debug::RtlCaptureStackBackTrace;

use nt_hook::line_budget::LineBudget;
use replicant_game::GameMemory;
use replicant_game::effects::{
    ActorCueCall, ActorCueMoment, FirstCues, MAX_ACTOR_CUES_PER_SECOND,
    actor_cue_goes_to_enemy_sync, actor_cue_is_sent, actor_cue_text, read_effect_transform,
    stack_holds_ai_step,
};
use replicant_game::offsets::{
    ACTOR_ATTACH_EFFECT_BY_ID_FUNCTION_RVA, ACTOR_REMOVE_ATTACHED_EFFECT_FUNCTION_RVA,
    BULLET_CREATE_EFFECTS_RVA_RANGE,
};
use replicant_proto::{
    ATTACHED_EFFECT_COLOUR_FLOATS, ATTACHED_EFFECT_STYLE_FLOATS, ActorCue, ActorCueKind,
    AttachedEffect, EnemyId, Message,
};

use crate::build_gate::VerifiedBuild;
use crate::bullets::create_effect;
use crate::diagnostics::log_line;
use crate::effects::hooks::{game_caller_rva, sending};
use crate::enemies::{EnemySyncRole, enemy_id_text, shared_actor_id, shared_actor_of};
use crate::enemy_bullets::{command_in_flight, play_shot_sound};
use crate::game_hooks::hook_address;
use crate::memory::InProcessMemory;
use crate::net::{self, net_clock_ms};

static HOOKED_BUILD: OnceLock<VerifiedBuild> = OnceLock::new();
static ORIGINAL_REMOVE_ATTACHED: AtomicPtr<c_void> = AtomicPtr::new(ptr::null_mut());
static FIRST_SENT: LazyLock<Mutex<FirstCues>> = LazyLock::new(Mutex::default);
static FIRST_PLAYED: LazyLock<Mutex<FirstCues>> = LazyLock::new(Mutex::default);
static SEND_WINDOW: Mutex<LineBudget> = Mutex::new(LineBudget::counting_from_zero());
static SENT: AtomicU32 = AtomicU32::new(0);
static REFUSED_BY_RATE_LIMIT: AtomicU32 = AtomicU32::new(0);
static PLAYED: AtomicU32 = AtomicU32::new(0);
static NOT_PLAYED: AtomicU32 = AtomicU32::new(0);
static LAST_SUMMARY_MS: AtomicU64 = AtomicU64::new(0);

const STACK_SEARCH_FRAMES: usize = 32;
const ACTOR_CUE_SUMMARY_INTERVAL_MS: u64 = 5000;
const SEND_WINDOW_MS: u64 = 1000;

thread_local! {
    static TICKING_ACTOR: Cell<u64> = const { Cell::new(0) };
    static REPLAYING: Cell<bool> = const { Cell::new(false) };
}

type RemoveAttached = unsafe extern "system" fn(*mut c_void, i32, i32, u8) -> u64;
type AttachEffectById = unsafe extern "system" fn(
    *mut c_void,
    u32,
    i32,
    u8,
    *const f32,
    *const f32,
    u32,
    u8,
    u32,
) -> u64;

pub(super) fn install(build: VerifiedBuild) {
    if HOOKED_BUILD.set(build).is_err() {
        return;
    }
    #[expect(
        unsafe_code,
        reason = "hooks the remove-attached-effect function by address"
    )]
    // SAFETY: the RVA starts the function (actor, effect id, bone, flag -> removed any) in the
    // verified build, and the replacement matches it.
    unsafe {
        hook_address(
            "Actor::RemoveAttachedEffect",
            (build.module_base() + u64::from(ACTOR_REMOVE_ATTACHED_EFFECT_FUNCTION_RVA)) as usize,
            remove_attached_detour as *const () as *mut c_void,
            &ORIGINAL_REMOVE_ATTACHED,
        )
    };
}

/// Runs one actor's tick, so an effect or a sound made inside it knows whose tick it was.
pub(crate) fn with_ticking_actor<R>(actor: u64, tick: impl FnOnce() -> R) -> R {
    let outer = TICKING_ACTOR.with(|cell| cell.replace(actor));
    let result = tick();
    TICKING_ACTOR.with(|cell| cell.set(outer));
    result
}

/// From the attach-by-id detour, after the game attached effect `id` to `actor`.
pub(super) fn note_attach(
    actor: u64,
    id: u32,
    bone: i32,
    unique: u8,
    colour: u64,
    style: u64,
    mode: u32,
    flags: u8,
    extra: u32,
) {
    if REPLAYING.with(Cell::get) {
        return;
    }
    let Some((build, shared)) = hosted_shared(actor) else {
        return;
    };
    if !actor_cue_is_sent(moment(false, false)) {
        return;
    }
    let memory = InProcessMemory::new(build);
    let (Some(colour), Some(style)) = (
        read_floats::<ATTACHED_EFFECT_COLOUR_FLOATS>(&memory, colour),
        read_floats::<ATTACHED_EFFECT_STYLE_FLOATS>(&memory, style),
    ) else {
        return;
    };
    // The game keeps the bone as a 16-bit value, so the cue does too.
    let bone = bone as i16;
    let attached = AttachedEffect {
        bone,
        unique: unique != 0,
        colour,
        style,
        mode: mode.cast_signed(),
        flags,
        extra,
    };
    send(
        ActorCue {
            actor: shared,
            id,
            kind: ActorCueKind::AttachEffect(attached),
        },
        ActorCueCall::AttachById,
    );
}

/// From the create-by-id detour, after the game made effect `id` into `handle`.
pub(super) fn note_create_by_id(handle: u32, id: u32, caller_rva: u32, nested: bool) {
    let actor = TICKING_ACTOR.with(Cell::get);
    if actor == 0 || REPLAYING.with(Cell::get) {
        return;
    }
    let Some((build, shared)) = hosted_shared(actor) else {
        return;
    };
    let made_for_a_bullet =
        BULLET_CREATE_EFFECTS_RVA_RANGE.contains(&caller_rva) || command_in_flight().is_some();
    if !actor_cue_is_sent(moment(nested, made_for_a_bullet)) {
        return;
    }
    let Ok(Some(transform)) = read_effect_transform(&InProcessMemory::new(build), handle) else {
        return;
    };
    send(
        ActorCue {
            actor: shared,
            id,
            kind: ActorCueKind::PlaceEffect { transform },
        },
        ActorCueCall::CreateById,
    );
}

/// From the play-sound-by-id detour, with the sound's position pointer and its two levels.
pub(super) fn note_sound_by_id(id: u32, position: u64, levels: [f32; 2]) {
    let actor = TICKING_ACTOR.with(Cell::get);
    if actor == 0 || position == 0 || REPLAYING.with(Cell::get) {
        return;
    }
    let Some((build, shared)) = hosted_shared(actor) else {
        return;
    };
    if !actor_cue_is_sent(moment(false, command_in_flight().is_some())) {
        return;
    }
    let Some(position) = read_floats::<3>(&InProcessMemory::new(build), position) else {
        return;
    };
    send(
        ActorCue {
            actor: shared,
            id,
            kind: ActorCueKind::Sound { position, levels },
        },
        ActorCueCall::SoundById,
    );
}

/// True for an outermost by-name spawn inside a shared actor's own AI or state step on the
/// host: it is sent as a world effect.
pub(super) fn spawn_is_actor_cue(outermost: bool) -> bool {
    let actor = TICKING_ACTOR.with(Cell::get);
    if actor == 0 || !outermost {
        return false;
    }
    let Some((_, shared)) = hosted_shared(actor) else {
        return false;
    };
    let caller_rva = game_caller_rva();
    let made_for_a_bullet = BULLET_CREATE_EFFECTS_RVA_RANGE.contains(&caller_rva);
    let sent = actor_cue_is_sent(moment(false, made_for_a_bullet));
    if sent
        && FIRST_SENT
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .is_first(shared, ActorCueCall::SpawnByName, caller_rva)
    {
        log_line(&format!(
            "actor cues: a by-name effect from {}'s own step (caller {caller_rva:#x}) is sent as a world effect",
            enemy_id_text(shared)
        ));
    }
    sent
}

pub(super) fn tick(build: &VerifiedBuild, active: bool) {
    let Some(net) = net::handle() else {
        return;
    };
    let arrived = net.take_relayed(|message| {
        matches!(message, Message::ActorCue(cue) if !actor_cue_goes_to_enemy_sync(&cue.kind))
    });
    if active {
        for message in arrived {
            if let Message::ActorCue(cue) = message {
                play(build, cue);
            }
        }
    }
    let now_ms = net_clock_ms();
    if now_ms.saturating_sub(LAST_SUMMARY_MS.load(Ordering::Relaxed))
        >= ACTOR_CUE_SUMMARY_INTERVAL_MS
    {
        LAST_SUMMARY_MS.store(now_ms, Ordering::Relaxed);
        let counts = [&SENT, &REFUSED_BY_RATE_LIMIT, &PLAYED, &NOT_PLAYED]
            .map(|counter| counter.swap(0, Ordering::Relaxed));
        if counts.iter().any(|count| *count > 0) {
            log_line(&format!(
                "actor cues: last 5 s sent {} ({} refused by the rate limit), played {}, not played {}",
                counts[0], counts[1], counts[2], counts[3]
            ));
        }
    }
}

fn play(build: &VerifiedBuild, cue: ActorCue) {
    let copy = shared_actor_of(cue.actor);
    let outcome = match (cue.kind, copy) {
        (ActorCueKind::AttachEffect(attached), Some(copy)) => {
            attach_on_copy(build, copy, cue.id, &attached).then_some("played")
        }
        (ActorCueKind::RemoveEffect { bone, all_matching }, Some(copy)) => {
            remove_on_copy(build, copy, cue.id, bone, all_matching);
            Some("played")
        }
        (ActorCueKind::AttachEffect(_) | ActorCueKind::RemoveEffect { .. }, None) => None,
        (ActorCueKind::PlaceEffect { transform }, _) => {
            let id = i32::try_from(cue.id).unwrap_or(i32::MAX);
            create_effect(build, id, &transform).map(|_| "played")
        }
        (ActorCueKind::Sound { position, levels }, _) => {
            play_shot_sound(build, cue.id, position, levels).map(|_| "played")
        }
        (ActorCueKind::TurnedToStone(_), _) => return,
    };
    let counter = if outcome.is_some() {
        &PLAYED
    } else {
        &NOT_PLAYED
    };
    counter.fetch_add(1, Ordering::Relaxed);
    let first = FIRST_PLAYED
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .is_first(cue.actor, ActorCueCall::of_kind(&cue.kind), cue.id);
    if first {
        let why = match (outcome, copy) {
            (Some(played), _) => played,
            (None, None) => "not played, no copy here",
            (None, Some(_)) => "not played, the game refused it",
        };
        log_line(&format!(
            "actor cues: {} ({why})",
            actor_cue_text(&enemy_id_text(cue.actor), &cue)
        ));
    }
}

fn attach_on_copy(build: &VerifiedBuild, copy: u64, id: u32, attached: &AttachedEffect) -> bool {
    let function =
        (build.module_base() + u64::from(ACTOR_ATTACH_EFFECT_BY_ID_FUNCTION_RVA)) as usize;
    let replayed = REPLAYING.with(|cell| cell.replace(true));
    #[expect(unsafe_code, reason = "calls the game's attach-effect-by-id function")]
    // SAFETY: the RVA starts Actor::AttachEffectById (actor, id, bone, unique, colour[4],
    // style[4], mode, flags, extra -> attached) in the verified build; the copy is a live shared
    // actor of this frame, both arrays outlive the call, and this runs on the game thread.
    let result = unsafe {
        core::mem::transmute::<usize, AttachEffectById>(function)(
            copy as usize as *mut c_void,
            id,
            i32::from(attached.bone),
            u8::from(attached.unique),
            attached.colour.as_ptr(),
            attached.style.as_ptr(),
            attached.mode.cast_unsigned(),
            attached.flags,
            attached.extra,
        )
    };
    REPLAYING.with(|cell| cell.set(replayed));
    result & 0xFF != 0
}

fn remove_on_copy(build: &VerifiedBuild, copy: u64, id: u32, bone: i32, all_matching: bool) {
    let function =
        (build.module_base() + u64::from(ACTOR_REMOVE_ATTACHED_EFFECT_FUNCTION_RVA)) as usize;
    let replayed = REPLAYING.with(|cell| cell.replace(true));
    #[expect(
        unsafe_code,
        reason = "calls the game's remove-attached-effect function"
    )]
    // SAFETY: the RVA starts Actor::RemoveAttachedEffect (actor, id, bone, flag) in the verified
    // build; the copy is a live shared actor of this frame, on the game thread.
    unsafe {
        core::mem::transmute::<usize, RemoveAttached>(function)(
            copy as usize as *mut c_void,
            id.cast_signed(),
            bone,
            u8::from(all_matching),
        );
    }
    REPLAYING.with(|cell| cell.set(replayed));
}

#[expect(
    unsafe_code,
    reason = "calls the original remove-attached-effect function through the trampoline MinHook built"
)]
extern "system" fn remove_attached_detour(
    actor: *mut c_void,
    id: i32,
    bone: i32,
    all_matching: u8,
) -> u64 {
    let original = ORIGINAL_REMOVE_ATTACHED.load(Ordering::Acquire);
    if original.is_null() {
        return 0;
    }
    // SAFETY: the trampoline continues into the original function, which has this signature.
    let removed = unsafe {
        core::mem::transmute::<*mut c_void, RemoveAttached>(original)(actor, id, bone, all_matching)
    };
    if removed & 0xFF != 0 && sending() && !REPLAYING.with(Cell::get) {
        let actor = actor as usize as u64;
        let _ = panic::catch_unwind(|| note_remove(actor, id, bone, all_matching != 0));
    }
    removed
}

fn note_remove(actor: u64, id: i32, bone: i32, all_matching: bool) {
    let Some((_, shared)) = hosted_shared(actor) else {
        return;
    };
    if !actor_cue_is_sent(moment(false, false)) {
        return;
    }
    send(
        ActorCue {
            actor: shared,
            id: id.cast_unsigned(),
            kind: ActorCueKind::RemoveEffect { bone, all_matching },
        },
        ActorCueCall::RemoveAttached,
    );
}

fn hosted_shared(actor: u64) -> Option<(&'static VerifiedBuild, EnemyId)> {
    let build = HOOKED_BUILD.get()?;
    if !sending() {
        return None;
    }
    match shared_actor_id(actor)? {
        (EnemySyncRole::Host, id) => Some((build, id)),
        (EnemySyncRole::Guest, _) => None,
    }
}

fn moment(nested: bool, made_for_a_bullet: bool) -> ActorCueMoment {
    // Only called once `hosted_shared` found a shared actor on the host.
    ActorCueMoment {
        hosting: true,
        shared_actor: true,
        in_ai_step: stack_in_ai_step(),
        nested,
        made_for_a_bullet,
    }
}

#[expect(
    unsafe_code,
    reason = "walks the stack to find the actor tick's AI or state step call"
)]
fn stack_in_ai_step() -> bool {
    let Some(build) = HOOKED_BUILD.get() else {
        return false;
    };
    let mut frames = [ptr::null_mut::<c_void>(); STACK_SEARCH_FRAMES];
    // SAFETY: the buffer holds the frames asked for; the hash out-pointer may be null.
    let captured = unsafe {
        RtlCaptureStackBackTrace(
            0,
            STACK_SEARCH_FRAMES as u32,
            frames.as_mut_ptr(),
            ptr::null_mut(),
        )
    };
    let rvas: Vec<u32> = frames
        .iter()
        .take(usize::from(captured))
        .filter_map(|frame| {
            u32::try_from((*frame as usize as u64).wrapping_sub(build.module_base())).ok()
        })
        .collect();
    stack_holds_ai_step(&rvas)
}

fn read_floats<const COUNT: usize>(memory: &InProcessMemory, address: u64) -> Option<[f32; COUNT]> {
    if address == 0 {
        return None;
    }
    let mut values = [0.0_f32; COUNT];
    for (index, value) in (0_u64..).zip(values.iter_mut()) {
        *value = memory.read_f32(address + index * 4).ok()?;
    }
    Some(values)
}

fn send(cue: ActorCue, call: ActorCueCall) {
    if cue.validate().is_err() {
        return;
    }
    let admitted = SEND_WINDOW
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .admit(net_clock_ms(), SEND_WINDOW_MS, MAX_ACTOR_CUES_PER_SECOND);
    if !admitted {
        REFUSED_BY_RATE_LIMIT.fetch_add(1, Ordering::Relaxed);
        return;
    }
    let Some(net) = net::handle() else {
        return;
    };
    let first = FIRST_SENT
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .is_first(cue.actor, call, cue.id);
    if first {
        log_line(&format!(
            "actor cues: sent {}",
            actor_cue_text(&enemy_id_text(cue.actor), &cue)
        ));
    }
    net.send_relayed(Message::ActorCue(cue));
    SENT.fetch_add(1, Ordering::Relaxed);
}
