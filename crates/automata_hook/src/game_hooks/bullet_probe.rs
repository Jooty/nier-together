use core::ffi::c_void;
use core::ptr;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::{Mutex, OnceLock};

use automata_game::address::BULLET_SHOT_SETUP_FUNCTION;
use automata_game::bullet_probe::{
    BULLET_BEHAVIOR_KIND_OFFSET, BULLET_POSITION_OFFSET, BULLET_SHOT_LINES_LOGGED_PER_MINUTE,
    BULLET_SHOT_SETUP_FUNCTION_START, BulletShot, OWNER_OBJECT_ID_OFFSET,
    SHOT_INFO_ATTACK_GROUP_OFFSET, SHOT_INFO_ATTACK_INDEX_OFFSET, SHOT_INFO_KIND_OFFSET,
};
use automata_game::enemies::ENTITY_BEHAVIOR_POINTER_OFFSET;
use automata_game::minute_line_budget::MinuteLineBudget;
use automata_game::partner::{ENTITY_HANDLE_OFFSET, code_starts_with};
use nt_hook::diagnostics::log_line;

use crate::budgeted_log::log_within;
use crate::game_calls::GameCalls;

static ORIGINAL_BULLET_SHOT_SETUP: AtomicPtr<c_void> = AtomicPtr::new(ptr::null_mut());
static PROBE_CALLS: OnceLock<GameCalls> = OnceLock::new();
static SHOT_LINES: Mutex<MinuteLineBudget> =
    Mutex::new(MinuteLineBudget::new(BULLET_SHOT_LINES_LOGGED_PER_MINUTE));

type BulletShotSetup =
    unsafe extern "system" fn(usize, usize, usize, i32, u32, usize, usize, u32, u32);

/// Hooks the game's bullet shot setup to log each shot of a boss-class owner; changes nothing
/// the game does. False, with nothing hooked, when the function does not start as noted.
pub fn install_bullet_probe(calls: GameCalls) -> bool {
    let start_matches = calls
        .read_bytes(
            BULLET_SHOT_SETUP_FUNCTION,
            BULLET_SHOT_SETUP_FUNCTION_START.len(),
        )
        .is_ok_and(|found| code_starts_with(&found, &BULLET_SHOT_SETUP_FUNCTION_START));
    if !start_matches || PROBE_CALLS.set(calls).is_err() {
        log_line("bullets: the shot setup function does not start as noted, it is not probed");
        return false;
    }
    #[expect(
        unsafe_code,
        reason = "hooks the bullet shot setup; the replacement has the signature of the original and forwards every argument"
    )]
    // SAFETY: the address is the bullet shot setup function of the verified build and its
    // start was just checked.
    let hooked = unsafe {
        nt_hook::detour::hook_address(
            "bullet shot setup (a probe: one line per boss-class shot)",
            calls.address_value(BULLET_SHOT_SETUP_FUNCTION) as usize,
            bullet_shot_setup_detour as *const () as *mut c_void,
            &ORIGINAL_BULLET_SHOT_SETUP,
        )
    };
    if !hooked {
        log_line("bullets: the shot setup function could not be hooked, shots are not probed");
    }
    hooked
}

#[expect(
    unsafe_code,
    reason = "calls the original bullet shot setup through the trampoline MinHook built"
)]
extern "system" fn bullet_shot_setup_detour(
    bullet: usize,
    info: usize,
    owner: usize,
    a: i32,
    b: u32,
    c: usize,
    d: usize,
    e: u32,
    f: u32,
) {
    let original = ORIGINAL_BULLET_SHOT_SETUP.load(Ordering::Acquire);
    if original.is_null() {
        return;
    }
    // SAFETY: the trampoline continues into the original function with the arguments received.
    unsafe {
        core::mem::transmute::<*mut c_void, BulletShotSetup>(original)(
            bullet, info, owner, a, b, c, d, e, f,
        );
    }
    let Some(calls) = PROBE_CALLS.get() else {
        return;
    };
    if let Some(shot) = read_shot(calls, bullet, info, owner)
        && shot.is_by_a_boss_class_owner()
    {
        log_within(&SHOT_LINES, "bullets: shot lines", || shot.line());
    }
}

fn read_shot(calls: &GameCalls, bullet: usize, info: usize, owner: usize) -> Option<BulletShot> {
    if bullet == 0 || info == 0 || owner == 0 {
        return None;
    }
    let owner_at = owner as u64;
    let owner_body = calls
        .read_u64_at(owner_at + u64::from(ENTITY_BEHAVIOR_POINTER_OFFSET))
        .ok()
        .filter(|body| *body != 0)? as usize;
    let info_at = info as u64;
    Some(BulletShot {
        owner_object: calls
            .read_u32_at(owner_at + u64::from(OWNER_OBJECT_ID_OFFSET))
            .ok()?,
        owner_handle: calls
            .read_u32_at(owner_at + u64::from(ENTITY_HANDLE_OFFSET))
            .ok()?,
        owner_health: calls.read_enemy_health(owner_body)?,
        owner_max_health: calls.read_max_health(owner_body)?,
        shot_kind: calls.read_i32_at(info_at + SHOT_INFO_KIND_OFFSET).ok()?,
        behavior_kind: calls
            .read_i32_at(bullet as u64 + BULLET_BEHAVIOR_KIND_OFFSET)
            .ok()?,
        attack_group: calls
            .read_i32_at(info_at + SHOT_INFO_ATTACK_GROUP_OFFSET)
            .ok()?,
        attack_index: calls
            .read_i32_at(info_at + SHOT_INFO_ATTACK_INDEX_OFFSET)
            .ok()?,
        position: calls
            .read_f32s_at::<3>(bullet as u64 + BULLET_POSITION_OFFSET)
            .ok()?,
    })
}
