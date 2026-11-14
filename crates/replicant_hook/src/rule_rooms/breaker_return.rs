use core::ffi::c_void;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, PoisonError};

use replicant_game::dialogue::{CONTROL_MASK_SET_SLOT_ZERO_RVA, GAME_STATE_RVA};
use replicant_game::fight_sync::CHANGE_CAMERA_TO_FREE_CAMERA_KIND;
use replicant_game::marker::CONTROL_MASK_FIRST_RVA;
use replicant_game::own_ride::SET_PLAYER_DEFAULT_STATUS_NATIVE_RVA;
use replicant_game::rule_rooms::{
    BLACK_FADE_DEPTH, BLACK_FADE_KIND, BreakerBodyHold, BreakerReturn,
    CHANGE_CAMERA_BACK_TO_THE_PLAYER_SECONDS, CHANGE_CAMERA_BLEND_KIND,
    RESET_FREE_CAMERA_NATIVE_RVA, RETURN_FADE_IN_SECONDS, RETURN_FADE_OUT_SECONDS,
    RETURN_INVINCIBLE_SECONDS, ReturnSpot, RuleAct, RulePlayer, RuleRoom, RuleState,
    SET_PLAYER_VISIBLE_NATIVE_RVA, SET_SNOW_ACTIVE_AREA_BY_PLAYER_POSITION_NATIVE_RVA,
    SET_UNIQUE_ACTOR_INVINCIBLE_NATIVE_RVA, body_held_line, body_let_go_line,
    cleared_during_return_line, ignored_break_line, return_spot_of, returned_line,
    returned_without_a_spot_line,
};
use replicant_game::side_view::{CHANGE_CAMERA_NATIVE_RVA, CHANGE_CAMERA_STATUS_NATIVE_RVA};
use replicant_game::story_jump::PLAYER_UNIQUE_ACTOR_ID;
use replicant_game::{GameMemory, Position};

use crate::build_gate::VerifiedBuild;
use crate::diagnostics::log_line;
use crate::memory::InProcessMemory;
use crate::net::net_clock_ms;
use crate::rule_rooms::rearm::arm_the_breaks_again;
use crate::rule_rooms::state::WATCH;
use crate::story_jump::{NativeArg, call_native_with};
use crate::teleport::move_player_facing;

static RETURN: Mutex<BreakerReturn> = Mutex::new(BreakerReturn::new());
static BREAKER_INPUT_BLOCKED: AtomicBool = AtomicBool::new(false);
static BODY_HOLD: Mutex<BreakerBodyHold> = Mutex::new(BreakerBodyHold::new());

type SetMaskSlot = unsafe extern "system" fn(*mut c_void, u32);

pub(crate) fn input_blocked() -> bool {
    BREAKER_INPUT_BLOCKED.load(Ordering::Relaxed)
}

pub(super) fn begin(room: &'static RuleRoom, player: RulePlayer) {
    let started =
        RETURN
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .begin(room, player, net_clock_ms());
    if !started {
        log_line(&ignored_break_line(room, player));
        return;
    }
    BREAKER_INPUT_BLOCKED.store(true, Ordering::Relaxed);
    crate::fade_sync::fade_on_this_game_only(
        true,
        BLACK_FADE_KIND,
        RETURN_FADE_OUT_SECONDS,
        BLACK_FADE_DEPTH,
    );
}

pub(super) fn tick(build: &VerifiedBuild) {
    let now_ms = net_clock_ms();
    let (due, arm_due, input_blocked) = {
        let mut state = RETURN.lock().unwrap_or_else(PoisonError::into_inner);
        (
            state.take_if_due(now_ms),
            state.take_arm_if_due(now_ms),
            state.input_blocked(now_ms),
        )
    };
    BREAKER_INPUT_BLOCKED.store(input_blocked, Ordering::Relaxed);
    hold_the_breakers_body(build, input_blocked);
    if let Some(room) = arm_due
        && !room_is_cleared(room)
    {
        arm_the_breaks_again(build, room);
    }
    let Some((room, player)) = due else {
        return;
    };
    if room_is_cleared(room) {
        fade_the_screen_back_in();
        log_line(&cleared_during_return_line(room, player));
        return;
    }
    let Some(spot) = return_spot_of(room) else {
        log_line(&returned_without_a_spot_line(room, RuleAct::TimeUp));
        return;
    };
    put_the_breaker_back(build, spot);
    fade_the_screen_back_in();
    log_line(&returned_line(room, player, spot));
}

fn hold_the_breakers_body(build: &VerifiedBuild, hold: bool) {
    let memory = InProcessMemory::new(build);
    let base = memory.module_base();
    let Ok(slot_zero) = memory.read_i32(base + u64::from(CONTROL_MASK_FIRST_RVA)) else {
        return;
    };
    let slot_zero = slot_zero.cast_unsigned();
    let mut body_hold = BODY_HOLD.lock().unwrap_or_else(PoisonError::into_inner);
    let Some(written) = body_hold.slot_zero_to_write(slot_zero, hold) else {
        return;
    };
    let holding = body_hold.is_holding();
    drop(body_hold);
    set_slot_zero(base, written);
    log_line(&if holding {
        body_held_line(slot_zero, written)
    } else {
        body_let_go_line(slot_zero, written)
    });
}

#[expect(
    unsafe_code,
    reason = "calls the control mask slot 0 setter by address"
)]
fn set_slot_zero(base: u64, value: u32) {
    // SAFETY: the slot 0 setter (game state, value) in the verified build, called on the game
    // thread as the stuck menu mask recovery does.
    unsafe {
        core::mem::transmute::<usize, SetMaskSlot>(
            (base + u64::from(CONTROL_MASK_SET_SLOT_ZERO_RVA)) as usize,
        )((base + u64::from(GAME_STATE_RVA)) as *mut c_void, value);
    }
}

fn room_is_cleared(room: &RuleRoom) -> bool {
    WATCH
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .state_of(room)
        == Some(RuleState::Cleared)
}

fn fade_the_screen_back_in() {
    crate::fade_sync::fade_on_this_game_only(
        false,
        BLACK_FADE_KIND,
        RETURN_FADE_IN_SECONDS,
        BLACK_FADE_DEPTH,
    );
}

fn put_the_breaker_back(build: &VerifiedBuild, spot: &ReturnSpot) {
    call_native_with(build, SET_PLAYER_DEFAULT_STATUS_NATIVE_RVA, &[]);
    call_native_with(
        build,
        SET_PLAYER_VISIBLE_NATIVE_RVA,
        &[NativeArg::Integer(1)],
    );
    let [x, y, z] = spot.position;
    move_player_facing(build, Position { x, y, z }, spot.heading_degrees);
    call_native_with(
        build,
        CHANGE_CAMERA_NATIVE_RVA,
        &[
            NativeArg::Integer(CHANGE_CAMERA_TO_FREE_CAMERA_KIND),
            NativeArg::Real(CHANGE_CAMERA_BACK_TO_THE_PLAYER_SECONDS),
            NativeArg::Integer(CHANGE_CAMERA_BLEND_KIND),
        ],
    );
    call_native_with(
        build,
        CHANGE_CAMERA_STATUS_NATIVE_RVA,
        &[
            NativeArg::Integer(spot.camera_status_kind),
            NativeArg::Real(spot.camera_status_seconds),
            NativeArg::Integer(CHANGE_CAMERA_BLEND_KIND),
        ],
    );
    call_native_with(build, RESET_FREE_CAMERA_NATIVE_RVA, &[]);
    call_native_with(
        build,
        SET_UNIQUE_ACTOR_INVINCIBLE_NATIVE_RVA,
        &[
            NativeArg::Integer(PLAYER_UNIQUE_ACTOR_ID),
            NativeArg::Real(RETURN_INVINCIBLE_SECONDS),
        ],
    );
    call_native_with(
        build,
        SET_SNOW_ACTIVE_AREA_BY_PLAYER_POSITION_NATIVE_RVA,
        &[],
    );
}
