use core::ffi::c_void;
use core::ptr;
use std::panic;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicPtr, AtomicU32, Ordering};

use replicant_game::GameMemory;
use replicant_game::kaine::{HerAiHoldReading, her_ai_is_held};
use replicant_game::marker::read_world_flags;
use replicant_game::offsets::{
    PARTNER_KAINE_ACTOR_RVA, PARTNER_SCRIPT_MODE_OFFSET, PARTNER_UPDATE_FUNCTION_RVA,
};

use crate::build_gate::VerifiedBuild;
use crate::game_hooks::hook_address;
use crate::kaine::companion_off::keeps_her_off;
use crate::memory::InProcessMemory;

const UPDATE_KEEPS_THE_ACTOR: u8 = 1;

static ORIGINAL_UPDATE: AtomicPtr<c_void> = AtomicPtr::new(ptr::null_mut());
static HOOKED: OnceLock<VerifiedBuild> = OnceLock::new();
static HELD_TICKS: AtomicU32 = AtomicU32::new(0);

type PartnerUpdate = unsafe extern "system" fn(*mut c_void, f32) -> u8;

pub(crate) fn install(build: VerifiedBuild) {
    if HOOKED.set(build).is_err() {
        return;
    }
    #[expect(unsafe_code, reason = "hooks CActorPartner::Update by address")]
    // SAFETY: the RVA starts CActorPartner::Update (vtable slot 0x4E0 of every partner class:
    // this, step time; returns false only to destroy the actor) in the verified build, and the
    // replacement has the same signature.
    unsafe {
        hook_address(
            "CActorPartner_Update",
            (build.module_base() + u64::from(PARTNER_UPDATE_FUNCTION_RVA)) as usize,
            partner_update_detour as *const () as *mut c_void,
            &ORIGINAL_UPDATE,
        );
    }
}

/// How many of her updates were held since the last call.
pub(crate) fn take_held_ticks() -> u32 {
    HELD_TICKS.swap(0, Ordering::Relaxed)
}

#[expect(
    unsafe_code,
    reason = "calls the original partner update through the trampoline MinHook built"
)]
extern "system" fn partner_update_detour(this: *mut c_void, step_time: f32) -> u8 {
    let original = ORIGINAL_UPDATE.load(Ordering::Acquire);
    if original.is_null() {
        return UPDATE_KEEPS_THE_ACTOR;
    }
    if panic::catch_unwind(|| her_update_is_held(this as u64)).unwrap_or(false) {
        HELD_TICKS.fetch_add(1, Ordering::Relaxed);
        return UPDATE_KEEPS_THE_ACTOR;
    }
    // SAFETY: the trampoline continues into the original update, which has this exact signature.
    unsafe { core::mem::transmute::<*mut c_void, PartnerUpdate>(original)(this, step_time) }
}

fn her_update_is_held(actor: u64) -> bool {
    let Some(build) = HOOKED.get() else {
        return false;
    };
    let memory = InProcessMemory::new(build);
    if actor != memory.module_base() + u64::from(PARTNER_KAINE_ACTOR_RVA) {
        return false;
    }
    her_ai_is_held(HerAiHoldReading {
        kept_off: keeps_her_off(),
        her_script_mode: memory
            .read_i32(actor + PARTNER_SCRIPT_MODE_OFFSET)
            .is_ok_and(|mode| mode != 0),
        event_scene: read_world_flags(&memory).is_ok_and(|flags| flags.event_scene),
    })
}
