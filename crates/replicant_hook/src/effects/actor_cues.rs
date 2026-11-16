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
    ActorCueCall, ActorCueMoment, AttachedCueEffect, AttachedCueEffects, CopyForCue, FirstCues,
    MAX_ACTOR_CUES_PER_SECOND, ReplayedLastingSound, ReplayedLastingSounds, SentLastingSounds,
    actor_cue_goes_to_enemy_sync, actor_cue_is_sent, actor_cue_replay_target, actor_cue_text,
    by_id_sound_cue, read_actor_position, read_effect_transform, stack_holds_ai_step,
};
use replicant_game::offsets::{
    ACTOR_ATTACH_EFFECT_BY_ID_FUNCTION_RVA, ACTOR_REMOVE_ATTACHED_EFFECT_FUNCTION_RVA,
    BULLET_CREATE_EFFECTS_RVA_RANGE, SOUND_STOP_FUNCTION_RVA,
};
use replicant_game::puppet::actor_is_live;
use replicant_proto::{
    ATTACHED_EFFECT_COLOUR_FLOATS, ATTACHED_EFFECT_STYLE_FLOATS, ActorCue, ActorCueKind,
    AttachedEffect, EnemyId, Message,
};

use crate::build_gate::VerifiedBuild;
use crate::bullets::create_effect;
use crate::diagnostics::log_line;
use crate::effects::hooks::sending;
use crate::enemies::{EnemySyncRole, enemy_id_text, is_held, shared_actor_id, shared_actor_of};
use crate::enemy_bullets::{
    bullet_start_in_flight, command_in_flight, move_shot_sound, play_shot_sound, stop_shot_sound,
};
use crate::game_hooks::hook_address;
use crate::memory::InProcessMemory;
use crate::net::{self, net_clock_ms};

static HOOKED_BUILD: OnceLock<VerifiedBuild> = OnceLock::new();
static ORIGINAL_REMOVE_ATTACHED: AtomicPtr<c_void> = AtomicPtr::new(ptr::null_mut());
static ORIGINAL_STOP_SOUND: AtomicPtr<c_void> = AtomicPtr::new(ptr::null_mut());
static SENT_LASTING_SOUNDS: LazyLock<Mutex<SentLastingSounds>> = LazyLock::new(Mutex::default);
static SENT_ATTACHES: LazyLock<Mutex<AttachedCueEffects>> = LazyLock::new(Mutex::default);
static REPLAYED_ATTACHES: LazyLock<Mutex<AttachedCueEffects>> = LazyLock::new(Mutex::default);
static REPLAYED_LASTING_SOUNDS: LazyLock<Mutex<ReplayedLastingSounds>> =
    LazyLock::new(Mutex::default);
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
type StopSound = unsafe extern "system" fn(*mut c_void, *mut c_void, f32);
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
    #[expect(unsafe_code, reason = "hooks the stop-sound function by address")]
    // SAFETY: the RVA starts CAudioMan::StopSound (audio manager, sound, fade) in the verified
    // build, and the replacement matches it.
    unsafe {
        hook_address(
            "CAudioMan::StopSound",
            (build.module_base() + u64::from(SOUND_STOP_FUNCTION_RVA)) as usize,
            stop_sound_detour as *const () as *mut c_void,
            &ORIGINAL_STOP_SOUND,
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
    relayed_as_cube_glow: bool,
) {
    if REPLAYING.with(Cell::get) {
        return;
    }
    let Some((build, shared)) = hosted_shared(actor) else {
        return;
    };
    let attach_moment = ActorCueMoment {
        made_for_a_cube_glow: relayed_as_cube_glow,
        ..moment(false, false)
    };
    if !actor_cue_is_sent(attach_moment) {
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
    let sent = send(
        ActorCue {
            actor: shared,
            id,
            kind: ActorCueKind::AttachEffect(attached),
        },
        ActorCueCall::AttachById,
    );
    if sent {
        SENT_ATTACHES
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .note_attached(AttachedCueEffect {
                actor,
                enemy: shared,
                id,
                bone,
            });
    }
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
    let made_for_a_bullet = BULLET_CREATE_EFFECTS_RVA_RANGE.contains(&caller_rva)
        || command_in_flight().is_some()
        || bullet_start_in_flight();
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

/// From the play-sound-by-id detour, with the sound's position pointer, its two levels and the
/// handle the game got for it.
pub(super) fn note_sound_by_id(
    id: u32,
    position: u64,
    levels: [f32; 2],
    caller_rva: u32,
    handle: u64,
) {
    let actor = TICKING_ACTOR.with(Cell::get);
    if actor == 0 || position == 0 || REPLAYING.with(Cell::get) {
        return;
    }
    let Some((build, shared)) = hosted_shared(actor) else {
        return;
    };
    let made_for_a_bullet = command_in_flight().is_some() || bullet_start_in_flight();
    if !actor_cue_is_sent(moment(false, made_for_a_bullet)) {
        return;
    }
    let Some(position) = read_floats::<3>(&InProcessMemory::new(build), position) else {
        return;
    };
    let Some(kind) = by_id_sound_cue(caller_rva, position, levels, handle) else {
        return;
    };
    let cue = ActorCue {
        actor: shared,
        id,
        kind,
    };
    if send(cue, ActorCueCall::of_kind(&kind)) && matches!(kind, ActorCueKind::LastingSound { .. })
    {
        SENT_LASTING_SOUNDS
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .note_sent(handle, shared, id);
    }
}

#[expect(
    unsafe_code,
    reason = "calls the original stop-sound function through the trampoline MinHook built"
)]
extern "system" fn stop_sound_detour(manager: *mut c_void, sound: *mut c_void, fade: f32) {
    let original = ORIGINAL_STOP_SOUND.load(Ordering::Acquire);
    if original.is_null() {
        return;
    }
    // SAFETY: the trampoline continues into the original function, which has this signature.
    unsafe {
        core::mem::transmute::<*mut c_void, StopSound>(original)(manager, sound, fade);
    }
    let sound = sound as usize as u64;
    let _ = panic::catch_unwind(|| note_stop_sound(sound));
}

fn note_stop_sound(handle: u64) {
    if handle == 0 {
        return;
    }
    let stopped = {
        let mut sent = SENT_LASTING_SOUNDS
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        if sent.is_empty() {
            return;
        }
        sent.take_stopped(handle)
    };
    let Some((actor, id)) = stopped else {
        return;
    };
    if sending() {
        send(
            ActorCue {
                actor,
                id,
                kind: ActorCueKind::StopSound { handle },
            },
            ActorCueCall::StopSound,
        );
    }
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
    keep_lasting_sounds(build, active);
    send_removes_of_gone_actors(build);
    remove_replays_from_unheld_copies(build, active);
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

fn replay_target(build: &VerifiedBuild, id: EnemyId) -> Option<u64> {
    let copy = shared_actor_of(id).map(|actor| CopyForCue {
        actor,
        held: is_held(actor),
        live: actor_is_live(&InProcessMemory::new(build), actor).unwrap_or(false),
    });
    actor_cue_replay_target(copy)
}

fn play(build: &VerifiedBuild, cue: ActorCue) {
    let copy = replay_target(build, cue.actor);
    let outcome = match (cue.kind, copy) {
        (ActorCueKind::AttachEffect(attached), Some(copy)) => {
            let made = attach_on_copy(build, copy, cue.id, &attached);
            if made {
                REPLAYED_ATTACHES
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .note_attached(AttachedCueEffect {
                        actor: copy,
                        enemy: cue.actor,
                        id: cue.id,
                        bone: attached.bone,
                    });
            }
            made.then_some("played")
        }
        (ActorCueKind::RemoveEffect { bone, all_matching }, Some(copy)) => {
            remove_on_copy(build, copy, cue.id, bone, all_matching);
            REPLAYED_ATTACHES
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .take_removed(copy, cue.id, bone);
            Some("played")
        }
        (ActorCueKind::PlaceEffect { transform }, Some(_)) => {
            let id = i32::try_from(cue.id).unwrap_or(i32::MAX);
            create_effect(build, id, &transform).map(|_| "played")
        }
        (ActorCueKind::Sound { position, levels }, Some(_)) => {
            play_shot_sound(build, cue.id, position, levels).map(|_| "played")
        }
        (
            ActorCueKind::LastingSound {
                position,
                levels,
                handle,
            },
            Some(copy),
        ) => play_lasting_sound(build, &cue, copy, position, levels, handle).then_some("played"),
        (ActorCueKind::StopSound { handle }, _) => {
            stop_lasting_sound(build, &cue, handle);
            return;
        }
        (ActorCueKind::TurnedToStone(_) | ActorCueKind::StoneBroken, _) => return,
        (_, None) => None,
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
            (None, None) => "not played, no live copy held here",
            (None, Some(_)) => "not played, the game refused it",
        };
        log_line(&format!(
            "actor cues: {} ({why})",
            actor_cue_text(&enemy_id_text(cue.actor), &cue)
        ));
    }
}

fn play_lasting_sound(
    build: &VerifiedBuild,
    cue: &ActorCue,
    copy: u64,
    position: [f32; 3],
    levels: [f32; 2],
    sender_handle: u64,
) -> bool {
    let Some(own_handle) = play_shot_sound(build, cue.id, position, levels) else {
        return false;
    };
    let copy_position = read_actor_position(&InProcessMemory::new(build), copy).unwrap_or(position);
    let replaced = REPLAYED_LASTING_SOUNDS
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .track(ReplayedLastingSound {
            actor: cue.actor,
            id: cue.id,
            sender_handle,
            own_handle,
            offset_from_copy: core::array::from_fn(|axis| position[axis] - copy_position[axis]),
        });
    for own in replaced {
        stop_shot_sound(build, own);
    }
    true
}

fn stop_lasting_sound(build: &VerifiedBuild, cue: &ActorCue, sender_handle: u64) {
    let own = REPLAYED_LASTING_SOUNDS
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .take_stopped(cue.actor, cue.id, sender_handle);
    let Some(own) = own else {
        return;
    };
    stop_shot_sound(build, own);
    let first = FIRST_PLAYED
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .is_first(cue.actor, ActorCueCall::StopSound, cue.id);
    if first {
        log_line(&format!(
            "actor cues: {} (stopped the replay, own handle {own:#x})",
            actor_cue_text(&enemy_id_text(cue.actor), cue)
        ));
    }
}

fn keep_lasting_sounds(build: &VerifiedBuild, active: bool) {
    let mut replayed = REPLAYED_LASTING_SOUNDS
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    if replayed.playing().is_empty() {
        return;
    }
    let unheld = replayed.take_unheld(|actor| active && replay_target(build, actor).is_some());
    let playing: Vec<ReplayedLastingSound> = replayed.playing().to_vec();
    drop(replayed);
    for sound in unheld {
        stop_shot_sound(build, sound.own_handle);
        log_line(&format!(
            "actor cues: lasting sound {} of {} stopped here, its copy is no longer held",
            sound.id,
            enemy_id_text(sound.actor)
        ));
    }
    let memory = InProcessMemory::new(build);
    for sound in playing {
        let Some(copy) = shared_actor_of(sound.actor) else {
            continue;
        };
        let Ok(copy_position) = read_actor_position(&memory, copy) else {
            continue;
        };
        let position =
            core::array::from_fn(|axis| copy_position[axis] + sound.offset_from_copy[axis]);
        if !move_shot_sound(build, sound.own_handle, position) {
            REPLAYED_LASTING_SOUNDS
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .forget(sound.own_handle);
        }
    }
}

fn send_removes_of_gone_actors(build: &VerifiedBuild) {
    let gone = {
        let mut sent = SENT_ATTACHES.lock().unwrap_or_else(PoisonError::into_inner);
        if sent.is_empty() {
            return;
        }
        let memory = InProcessMemory::new(build);
        sent.take_gone(|actor, enemy| {
            shared_actor_id(actor) == Some((EnemySyncRole::Host, enemy))
                && actor_is_live(&memory, actor).unwrap_or(false)
        })
    };
    if gone.is_empty() || !sending() {
        return;
    }
    for effect in gone {
        let sent = send(
            ActorCue {
                actor: effect.enemy,
                id: effect.id,
                kind: ActorCueKind::RemoveEffect {
                    bone: i32::from(effect.bone),
                    all_matching: true,
                },
            },
            ActorCueCall::RemoveAttached,
        );
        log_line(&format!(
            "actor cues: {} is gone or no longer shared here, removal of its effect {} (bone {}) sent {sent}",
            enemy_id_text(effect.enemy),
            effect.id,
            effect.bone
        ));
    }
}

fn remove_replays_from_unheld_copies(build: &VerifiedBuild, active: bool) {
    let gone = {
        let mut replayed = REPLAYED_ATTACHES
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        if replayed.is_empty() {
            return;
        }
        replayed.take_gone(|copy, enemy| active && replay_target(build, enemy) == Some(copy))
    };
    let memory = InProcessMemory::new(build);
    for effect in gone {
        let live = actor_is_live(&memory, effect.actor).unwrap_or(false);
        if live {
            remove_on_copy(build, effect.actor, effect.id, i32::from(effect.bone), true);
        }
        log_line(&format!(
            "actor cues: {}'s copy is no longer held here, its replayed effect {} (bone {}) {}",
            enemy_id_text(effect.enemy),
            effect.id,
            effect.bone,
            if live {
                "removed"
            } else {
                "went with the copy"
            }
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
    if removed & 0xFF != 0 && !REPLAYING.with(Cell::get) {
        let actor = actor as usize as u64;
        let _ = panic::catch_unwind(|| note_remove(actor, id, bone, all_matching != 0));
    }
    removed
}

fn note_remove(actor: u64, id: i32, bone: i32, all_matching: bool) {
    let shared = {
        let mut sent = SENT_ATTACHES.lock().unwrap_or_else(PoisonError::into_inner);
        if sent.is_empty() {
            return;
        }
        sent.take_removed(actor, id.cast_unsigned(), bone)
    };
    let Some(shared) = shared else {
        return;
    };
    if !sending() {
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
        made_for_a_cube_glow: false,
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

fn send(cue: ActorCue, call: ActorCueCall) -> bool {
    if cue.validate().is_err() {
        return false;
    }
    let admitted = matches!(
        cue.kind,
        ActorCueKind::RemoveEffect { .. } | ActorCueKind::StopSound { .. }
    ) || SEND_WINDOW
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .admit(net_clock_ms(), SEND_WINDOW_MS, MAX_ACTOR_CUES_PER_SECOND);
    if !admitted {
        REFUSED_BY_RATE_LIMIT.fetch_add(1, Ordering::Relaxed);
        return false;
    }
    let Some(net) = net::handle() else {
        return false;
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
    true
}
