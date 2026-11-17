use core::ffi::c_void;
use core::ptr;
use std::panic;
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};
use std::sync::{Mutex, OnceLock, PoisonError};

use replicant_game::music::{DroppedCall, DroppedCalls};
use replicant_game::offsets::{
    PLAY_BGM_FUNCTION_RVA, SET_BGM_LAYER_VOLUME_FUNCTION_RVA, SET_BGM_MASTER_VOLUME_FUNCTION_RVA,
    STOP_BGM_FUNCTION_RVA,
};

use crate::build_gate::VerifiedBuild;
use crate::game_hooks::hook_address;
use crate::music::report::{count_ignored_call, log_limited};

static HOOKED_BUILD: OnceLock<VerifiedBuild> = OnceLock::new();
static ORIGINAL_PLAY_BGM: AtomicPtr<c_void> = AtomicPtr::new(ptr::null_mut());
static ORIGINAL_STOP_BGM: AtomicPtr<c_void> = AtomicPtr::new(ptr::null_mut());
static ORIGINAL_SET_MASTER: AtomicPtr<c_void> = AtomicPtr::new(ptr::null_mut());
static ORIGINAL_SET_LAYER: AtomicPtr<c_void> = AtomicPtr::new(ptr::null_mut());
static HOST_OWNS_LOCAL_MUSIC: AtomicBool = AtomicBool::new(false);
static APPLYING_HOST_MUSIC: AtomicBool = AtomicBool::new(false);
static DROPPED_CALLS: Mutex<DroppedCalls> = Mutex::new(DroppedCalls::new());

type PlayBgm = unsafe extern "system" fn(*mut c_void, i32, f32, f32) -> u64;
type StopBgm = unsafe extern "system" fn(*mut c_void, f32) -> u64;
type SetMasterVolume = unsafe extern "system" fn(*mut c_void, f32, f32);
type SetLayerVolume = unsafe extern "system" fn(*mut c_void, i32, f32, f32);

pub(super) struct ApplyingGuard;

impl Drop for ApplyingGuard {
    fn drop(&mut self) {
        APPLYING_HOST_MUSIC.store(false, Ordering::Relaxed);
    }
}

pub(super) fn applying() -> ApplyingGuard {
    APPLYING_HOST_MUSIC.store(true, Ordering::Relaxed);
    ApplyingGuard
}

pub(super) fn set_host_owns_local_music(owns: bool) {
    HOST_OWNS_LOCAL_MUSIC.store(owns, Ordering::Relaxed);
}

pub(super) fn forget_dropped_calls() {
    DROPPED_CALLS
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .clear();
}

pub(super) fn take_dropped_calls() -> Vec<DroppedCall> {
    DROPPED_CALLS
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .take_replay()
}

pub fn install(build: VerifiedBuild) {
    if HOOKED_BUILD.set(build).is_err() {
        return;
    }
    let base = build.module_base();
    let hooks: [(&str, u32, *mut c_void, &AtomicPtr<c_void>); 4] = [
        (
            "CAudioMan::PlayBgm",
            PLAY_BGM_FUNCTION_RVA,
            play_bgm_detour as *const () as *mut c_void,
            &ORIGINAL_PLAY_BGM,
        ),
        (
            "CAudioMan::StopBgm",
            STOP_BGM_FUNCTION_RVA,
            stop_bgm_detour as *const () as *mut c_void,
            &ORIGINAL_STOP_BGM,
        ),
        (
            "CAudioMan::SetBgmMasterVolume",
            SET_BGM_MASTER_VOLUME_FUNCTION_RVA,
            set_master_detour as *const () as *mut c_void,
            &ORIGINAL_SET_MASTER,
        ),
        (
            "CAudioMan::SetBgmLayerVolume",
            SET_BGM_LAYER_VOLUME_FUNCTION_RVA,
            set_layer_detour as *const () as *mut c_void,
            &ORIGINAL_SET_LAYER,
        ),
    ];
    for (label, rva, detour, original) in hooks {
        #[expect(unsafe_code, reason = "hooks a music function of the audio manager")]
        // SAFETY: each RVA starts the member function whose label it carries in the verified
        // build, and each detour has the same signature as the function it replaces.
        unsafe {
            hook_address(label, (base + u64::from(rva)) as usize, detour, original)
        };
    }
}

fn local_call_is_ignored(call: DroppedCall) -> bool {
    if !HOST_OWNS_LOCAL_MUSIC.load(Ordering::Relaxed) || APPLYING_HOST_MUSIC.load(Ordering::Relaxed)
    {
        return false;
    }
    count_ignored_call();
    DROPPED_CALLS
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .record(call);
    log_limited(&format!(
        "music: ignored the game's own {} ({}), the host's music is playing",
        call.name(),
        call.detail()
    ));
    true
}

fn ignored(call: DroppedCall) -> bool {
    panic::catch_unwind(|| local_call_is_ignored(call)).unwrap_or(false)
}

#[expect(
    unsafe_code,
    reason = "calls the original PlayBgm through the trampoline MinHook built"
)]
extern "system" fn play_bgm_detour(this: *mut c_void, track: i32, volume: f32, fade: f32) -> u64 {
    if ignored(DroppedCall::Play {
        track,
        volume,
        fade_seconds: fade,
    }) {
        return 0;
    }
    let original = ORIGINAL_PLAY_BGM.load(Ordering::Acquire);
    if original.is_null() {
        return 0;
    }
    // SAFETY: the trampoline continues into the original function, which has this signature.
    unsafe { core::mem::transmute::<*mut c_void, PlayBgm>(original)(this, track, volume, fade) }
}

#[expect(
    unsafe_code,
    reason = "calls the original StopBgm through the trampoline MinHook built"
)]
extern "system" fn stop_bgm_detour(this: *mut c_void, fade: f32) -> u64 {
    if ignored(DroppedCall::Stop { fade_seconds: fade }) {
        return 0;
    }
    let original = ORIGINAL_STOP_BGM.load(Ordering::Acquire);
    if original.is_null() {
        return 0;
    }
    // SAFETY: the trampoline continues into the original function, which has this signature.
    unsafe { core::mem::transmute::<*mut c_void, StopBgm>(original)(this, fade) }
}

#[expect(
    unsafe_code,
    reason = "calls the original SetBgmMasterVolume through the trampoline MinHook built"
)]
extern "system" fn set_master_detour(this: *mut c_void, volume: f32, seconds: f32) {
    if ignored(DroppedCall::Master { volume, seconds }) {
        return;
    }
    let original = ORIGINAL_SET_MASTER.load(Ordering::Acquire);
    if original.is_null() {
        return;
    }
    // SAFETY: the trampoline continues into the original function, which has this signature.
    unsafe { core::mem::transmute::<*mut c_void, SetMasterVolume>(original)(this, volume, seconds) }
}

#[expect(
    unsafe_code,
    reason = "calls the original SetBgmLayerVolume through the trampoline MinHook built"
)]
extern "system" fn set_layer_detour(this: *mut c_void, layer: i32, volume: f32, seconds: f32) {
    if ignored(DroppedCall::Layer {
        layer,
        volume,
        seconds,
    }) {
        return;
    }
    let original = ORIGINAL_SET_LAYER.load(Ordering::Acquire);
    if original.is_null() {
        return;
    }
    // SAFETY: the trampoline continues into the original function, which has this signature.
    unsafe {
        core::mem::transmute::<*mut c_void, SetLayerVolume>(original)(this, layer, volume, seconds)
    }
}
