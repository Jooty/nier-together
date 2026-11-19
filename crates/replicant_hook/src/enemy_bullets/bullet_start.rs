//! The host's side: what an ordinary bullet makes in its create-effects step (its start effect and
//! sounds, which no effect slot keeps), caught as the game makes them.

use core::cell::Cell;
use core::ffi::c_void;
use core::ptr;
use std::panic;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicPtr, Ordering};

use replicant_game::offsets::NORMAL_BULLET_CREATE_EFFECTS_FUNCTION_RVA;

use crate::build_gate::VerifiedBuild;
use crate::enemy_bullets::shot_cues::{begin_command, end_bullet_start, restore_outer_command};
use crate::game_hooks::hook_address;

static ORIGINAL_CREATE_EFFECTS: AtomicPtr<c_void> = AtomicPtr::new(ptr::null_mut());
static HOOKED_BUILD: OnceLock<VerifiedBuild> = OnceLock::new();

thread_local! {
    static STARTING_BULLET: Cell<u64> = const { Cell::new(0) };
}

type CreateEffects = unsafe extern "system" fn(u64) -> u64;

pub fn install(build: VerifiedBuild) {
    if HOOKED_BUILD.set(build).is_err() {
        return;
    }
    let address =
        (build.module_base() + u64::from(NORMAL_BULLET_CREATE_EFFECTS_FUNCTION_RVA)) as usize;
    #[expect(
        unsafe_code,
        reason = "hooks the ordinary bullet's create-effects step by its address"
    )]
    // SAFETY: the RVA starts `CNormalBullet::CreateEffects` (vslot 0x510) in the verified build
    // (docs symbols.tsv): one pointer argument, the bullet, and a result in rax. The replacement
    // has the same signature and forwards it.
    unsafe {
        hook_address(
            "CNormalBullet::CreateEffects",
            address,
            create_effects_detour as *const () as *mut c_void,
            &ORIGINAL_CREATE_EFFECTS,
        );
    }
}

/// `true` while an ordinary bullet's create-effects step runs on this thread.
pub(crate) fn bullet_start_in_flight() -> bool {
    STARTING_BULLET.with(Cell::get) != 0
}

extern "system" fn create_effects_detour(bullet: u64) -> u64 {
    let trampoline = ORIGINAL_CREATE_EFFECTS.load(Ordering::Acquire);
    if trampoline.is_null() {
        return 0;
    }
    let outer = STARTING_BULLET.with(|cell| cell.replace(bullet));
    let outer_made = begin_command();
    let result = call_original(trampoline, bullet);
    let _ = panic::catch_unwind(|| end_bullet_start(bullet));
    restore_outer_command(outer_made);
    STARTING_BULLET.with(|cell| cell.set(outer));
    result
}

#[expect(
    unsafe_code,
    reason = "calls the original create-effects step through the trampoline"
)]
fn call_original(trampoline: *mut c_void, bullet: u64) -> u64 {
    // SAFETY: the trampoline continues into the original function, which has this signature.
    unsafe { core::mem::transmute::<*mut c_void, CreateEffects>(trampoline)(bullet) }
}
