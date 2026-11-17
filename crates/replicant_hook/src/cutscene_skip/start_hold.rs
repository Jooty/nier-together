use core::ffi::c_void;
use core::ptr;
use std::panic;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::{Mutex, OnceLock, PoisonError};

use replicant_game::cutscene_skip::{
    EVENT_CUT_LOADED_BYTE_RVA, GUEST_OWN_START_HOLD_CAP_WHEN_HOST_NEVER_STARTS_MS, ParkedOwnStart,
    SCENE_ITEM_SCENE_ID_OFFSET, SCENE_ITEM_STATE_NOT_STARTED, SCENE_ITEM_STATE_OFFSET,
    SCENE_ITEM_STATE_STARTING, SCENE_ITEM_UPDATE_RVA, SCENE_START_HOLD_TIMEOUT_MS, SceneStartGate,
    StartVerdict,
};
use replicant_game::script_functions::script_function;
use replicant_game::shared_events::phase_name;
use replicant_game::stay_together::FollowRole;
use replicant_game::{GameMemory, PhaseEntry, Replicant};

use crate::build_gate::VerifiedBuild;
use crate::cutscene_skip::state::partner_in_cutscene;
use crate::diagnostics::log_line;
use crate::game_hooks::hook_address;
use crate::memory::InProcessMemory;
use crate::net::NetSharedExt;
use crate::net::{self, NetShared, NetState, net_clock_ms};

static HOOKED_BUILD: OnceLock<VerifiedBuild> = OnceLock::new();
static ORIGINAL_ITEM_UPDATE: AtomicPtr<c_void> = AtomicPtr::new(ptr::null_mut());
static GATE: Mutex<SceneStartGate> = Mutex::new(SceneStartGate::new());
static FUNCTION_GATE: Mutex<SceneStartGate> = Mutex::new(SceneStartGate::new());
static HELD_FUNCTION: Mutex<Option<HeldFunction>> = Mutex::new(None);
static ANNOUNCED_WHILE_HELD: Mutex<Option<String>> = Mutex::new(None);

struct HeldFunction {
    name: String,
    phase: u16,
    parked: ParkedOwnStart,
}

type ItemUpdate = unsafe extern "system" fn(*mut c_void);

pub(super) fn install(build: VerifiedBuild) {
    if HOOKED_BUILD.set(build).is_err() {
        return;
    }
    #[expect(
        unsafe_code,
        reason = "hooks the script event item's update by address"
    )]
    // SAFETY: the RVA starts ScriptEventItem::Update (item) in the verified build (see docs
    // off-hand/cutscenes.md) and the replacement has the same signature.
    unsafe {
        hook_address(
            "ScriptEventItem::Update",
            (build.module_base() + u64::from(SCENE_ITEM_UPDATE_RVA)) as usize,
            item_update_detour as *const () as *mut c_void,
            &ORIGINAL_ITEM_UPDATE,
        );
    }
}

pub(super) fn observe_host(build: &VerifiedBuild, view: &NetShared) {
    let mut gate = GATE.lock().unwrap_or_else(PoisonError::into_inner);
    if guest_waits_for_host(build, view) {
        gate.observe_host(partner_in_cutscene(view));
    } else {
        gate.reset();
        FUNCTION_GATE
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .reset();
    }
}

pub(super) fn note_host_function_start(phase_label: &str, name: &str) {
    if own_call_is_parked(name) {
        return;
    }
    if script_function(phase_label, name).is_some_and(|function| function.starts_scene) {
        FUNCTION_GATE
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .note_host_function_start();
    }
}

pub(super) fn take_announced_while_held(name: &str) -> bool {
    let mut announced = ANNOUNCED_WHILE_HELD
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    if announced.as_deref() == Some(name) {
        *announced = None;
        return true;
    }
    false
}

pub(super) fn own_call_is_parked(name: &str) -> bool {
    HELD_FUNCTION
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .as_ref()
        .is_some_and(|held| held.name == name && !held.parked.is_replaced())
}

pub(super) fn note_partner_copy_ran(name: &str) {
    let mut held = HELD_FUNCTION.lock().unwrap_or_else(PoisonError::into_inner);
    let Some(held) = held.as_mut().filter(|held| held.name == name) else {
        return;
    };
    held.parked.replace_with_partner_copy();
    take_announced_while_held(name);
}

pub(super) fn start_scene_function_the_host_never_started(build: &VerifiedBuild) {
    let now = net_clock_ms();
    let (phase, name) = {
        let mut held = HELD_FUNCTION.lock().unwrap_or_else(PoisonError::into_inner);
        let Some(held) = held
            .as_mut()
            .filter(|held| held.parked.host_never_started(now))
        else {
            return;
        };
        held.parked.replace_with_partner_copy();
        (held.phase, held.name.clone())
    };
    log_line(&format!(
        "scene hold: the host did not start its {name} within {GUEST_OWN_START_HOLD_CAP_WHEN_HOST_NEVER_STARTS_MS} ms, your {name} starts without the host"
    ));
    super::shared_functions::start_scene_function_here(build, phase, &name);
}

pub(super) fn forget_held_function_outside(phase: Option<u16>) {
    let mut held = HELD_FUNCTION.lock().unwrap_or_else(PoisonError::into_inner);
    if held.as_ref().is_some_and(|held| Some(held.phase) != phase) {
        *held = None;
    }
}

pub(super) fn forget_held_function() {
    *HELD_FUNCTION.lock().unwrap_or_else(PoisonError::into_inner) = None;
    *ANNOUNCED_WHILE_HELD
        .lock()
        .unwrap_or_else(PoisonError::into_inner) = None;
}

pub(crate) fn hold_own_scene_function(build: &VerifiedBuild, name: &str, args: u64) -> bool {
    let already_parked = HELD_FUNCTION
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .as_ref()
        .is_some_and(|held| held.name == name && held.parked.is_call(args));
    if already_parked {
        return true;
    }
    let Some(net) = net::handle() else {
        return false;
    };
    if !guest_waits_for_host(build, &net.view()) {
        return false;
    }
    let game = Replicant::new(InProcessMemory::new(build));
    let Some(label) = game.current_phase().ok() else {
        return false;
    };
    let Some(phase) = PhaseEntry::index_of(&label).and_then(|index| u16::try_from(index).ok())
    else {
        return false;
    };
    if !is_a_first_scene_start_nobody_popped(phase, &label, name) {
        return false;
    }
    let now = net_clock_ms();
    let verdict = FUNCTION_GATE
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .ask_with_cap(now, GUEST_OWN_START_HOLD_CAP_WHEN_HOST_NEVER_STARTS_MS);
    match verdict {
        StartVerdict::Hold { .. } => {
            *HELD_FUNCTION.lock().unwrap_or_else(PoisonError::into_inner) = Some(HeldFunction {
                name: name.to_owned(),
                phase,
                parked: ParkedOwnStart::new(args, now),
            });
            log_line(&format!(
                "scene hold: your {name} waits for the host to start its own (at most {GUEST_OWN_START_HOLD_CAP_WHEN_HOST_NEVER_STARTS_MS} ms), the host's copy starts it here"
            ));
            *ANNOUNCED_WHILE_HELD
                .lock()
                .unwrap_or_else(PoisonError::into_inner) = Some(name.to_owned());
            super::shared_functions::send_to_partner(phase, name);
            true
        }
        StartVerdict::Start { .. } | StartVerdict::StartAfterTimeout { .. } => false,
    }
}

fn is_a_first_scene_start_nobody_popped(phase: u16, label: &str, name: &str) -> bool {
    script_function(label, name).is_some_and(|function| function.starts_scene)
        && phase_name(phase) == Some(label)
        && !super::shared_functions::popped_callback_is_pending(name)
        && !super::shared_functions::function_ran_here(phase, name)
}

#[expect(
    unsafe_code,
    reason = "calls the original item update through the trampoline MinHook built"
)]
extern "system" fn item_update_detour(item: *mut c_void) {
    if panic::catch_unwind(|| holds_scene_start(item as u64)).unwrap_or(false) {
        return;
    }
    let original = ORIGINAL_ITEM_UPDATE.load(Ordering::Acquire);
    if original.is_null() {
        return;
    }
    // SAFETY: the trampoline continues into the original update, which has this signature.
    unsafe { core::mem::transmute::<*mut c_void, ItemUpdate>(original)(item) };
}

fn guest_waits_for_host(build: &VerifiedBuild, view: &NetShared) -> bool {
    if view.state != NetState::Connected
        || FollowRole::from_slot(view.local_slot) != FollowRole::Guest
        || !view.partner_in_play()
    {
        return false;
    }
    let game = Replicant::new(InProcessMemory::new(build));
    let local_phase = game
        .current_phase()
        .ok()
        .and_then(|name| PhaseEntry::index_of(&name))
        .and_then(|index| u16::try_from(index).ok());
    view.partner_state()
        .is_some_and(|remote| Some(remote.state.phase) == local_phase)
}

fn holds_scene_start(item: u64) -> bool {
    let Some(build) = HOOKED_BUILD.get() else {
        return false;
    };
    let Some(net) = net::handle() else {
        return false;
    };
    let memory = InProcessMemory::new(build);
    let state = memory
        .read_i32(item + SCENE_ITEM_STATE_OFFSET)
        .unwrap_or(-1)
        .cast_unsigned();
    let cut_loaded = memory
        .read_u8(build.module_base() + u64::from(EVENT_CUT_LOADED_BYTE_RVA))
        .is_ok_and(|byte| byte != 0);
    if !cut_loaded || (state != SCENE_ITEM_STATE_NOT_STARTED && state != SCENE_ITEM_STATE_STARTING)
    {
        return false;
    }
    let view = net.view();
    if !guest_waits_for_host(build, &view) {
        return false;
    }
    let verdict = GATE
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .ask(net_clock_ms());
    let scene_id = memory
        .read_i32(item + SCENE_ITEM_SCENE_ID_OFFSET)
        .unwrap_or(-1);
    match verdict {
        StartVerdict::Hold { first } => {
            if first {
                log_line(&format!(
                    "scene hold: your scene {scene_id} waits for the host to enter its own (item state {state:#x}, at most {SCENE_START_HOLD_TIMEOUT_MS} ms)"
                ));
            }
            true
        }
        StartVerdict::Start { waited_ms } => {
            if waited_ms > 0 {
                log_line(&format!(
                    "scene hold: the host is in its scene, your scene {scene_id} starts after {waited_ms} ms"
                ));
            }
            false
        }
        StartVerdict::StartAfterTimeout { waited_ms } => {
            log_line(&format!(
                "scene hold: timed out after {waited_ms} ms, your scene {scene_id} starts without the host"
            ));
            false
        }
    }
}
