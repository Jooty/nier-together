use core::ffi::c_void;
use core::ptr;
use std::panic;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::{Mutex, OnceLock, PoisonError};

use nt_hook::detour::hook_address;
use replicant_game::Replicant;
use replicant_game::fight_sync::read_native_numbers;
use replicant_game::side_view::{CHANGE_CAMERA_STATUS_NATIVE_RVA, ViewCall};
use replicant_game::stay_together::{FollowRole, local_phase_index};
use replicant_game::story_watch::{
    WATCHED_ROOM_STATUS_FRESH_MS, WatchedCameraRun, camera_call_is_followed,
    watched_camera_applies_here, watched_camera_run,
};
use replicant_proto::{
    Message, WATCHED_CAMERA_CALL_NUMBERS, WatchedCameraCall, WatchedCameraNative,
};

use crate::build_gate::VerifiedBuild;
use crate::diagnostics::log_line;
use crate::memory::InProcessMemory;
use crate::net::{self, NetState, net_clock_ms};
use crate::story_watch::this_game_is_watching;

static ORIGINAL_CHANGE_CAMERA_STATUS: AtomicPtr<c_void> = AtomicPtr::new(ptr::null_mut());
static STATUS_HOOK_BUILD: OnceLock<VerifiedBuild> = OnceLock::new();
static LAST_HOST_ROOM_STATUS: Mutex<Option<(i32, u64)>> = Mutex::new(None);

type LuaNative = unsafe extern "system" fn(*mut c_void);

pub(crate) fn install_status_hook(build: VerifiedBuild) {
    if STATUS_HOOK_BUILD.set(build).is_err() {
        return;
    }
    #[expect(unsafe_code, reason = "hooks the camera status native by address")]
    // SAFETY: the RVA starts the Lua native _ChangeCameraStatus in the verified build (argument
    // block in, nothing out), and the replacement has the same signature and calls the original.
    unsafe {
        hook_address(
            "ChangeCameraStatus_native",
            (build.module_base() + u64::from(CHANGE_CAMERA_STATUS_NATIVE_RVA)) as usize,
            change_camera_status_detour as *const () as *mut c_void,
            &ORIGINAL_CHANGE_CAMERA_STATUS,
        );
    }
}

pub(crate) fn host_camera_native_ran(
    build: &VerifiedBuild,
    native: WatchedCameraNative,
    args_address: u64,
) {
    let Some(net) = net::handle() else {
        return;
    };
    let view = net.view();
    if view.state != NetState::Connected
        || FollowRole::from_slot(view.local_slot) != FollowRole::Host
    {
        return;
    }
    let memory = InProcessMemory::new(build);
    let Some(phase) = local_phase_index(&Replicant::new(InProcessMemory::new(build))) else {
        return;
    };
    let read = read_native_numbers(&memory, args_address, argument_count(native));
    let mut numbers = [0.0_f32; WATCHED_CAMERA_CALL_NUMBERS];
    for (slot, value) in numbers.iter_mut().zip(&read) {
        *slot = value.unwrap_or(0.0) as f32;
    }
    if !camera_call_is_followed(native, &numbers) {
        return;
    }
    net.send_relayed(Message::WatchedCameraCall(WatchedCameraCall {
        phase,
        native,
        numbers,
    }));
}

pub(crate) fn fresh_host_room_status() -> Option<i32> {
    let (status, at_ms) = (*LAST_HOST_ROOM_STATUS
        .lock()
        .unwrap_or_else(PoisonError::into_inner))?;
    (net_clock_ms().saturating_sub(at_ms) <= WATCHED_ROOM_STATUS_FRESH_MS).then_some(status)
}

pub(super) fn apply_the_hosts_camera_calls(
    build: &VerifiedBuild,
    net: &net::NetThread,
    local_phase: Option<u16>,
) {
    let received = net.take_relayed(|message| matches!(message, Message::WatchedCameraCall(_)));
    if !this_game_is_watching() {
        return;
    }
    for message in received {
        let Message::WatchedCameraCall(call) = message else {
            continue;
        };
        if !watched_camera_applies_here(call.phase, local_phase) {
            continue;
        }
        let Some(run) = watched_camera_run(&call) else {
            continue;
        };
        if call.native == WatchedCameraNative::ChangeCameraStatus {
            *LAST_HOST_ROOM_STATUS
                .lock()
                .unwrap_or_else(PoisonError::into_inner) =
                Some((call.numbers[0].round() as i32, net_clock_ms()));
        }
        log_line(&format!(
            "watching: the host's {:?} {:?} runs here",
            call.native, call.numbers
        ));
        run_camera_call(build, run);
    }
}

fn run_camera_call(build: &VerifiedBuild, run: WatchedCameraRun) {
    crate::teleport::run_view_call(
        build,
        &ViewCall {
            native_rva: run.native_rva,
            args: run.args,
        },
    );
}

fn argument_count(native: WatchedCameraNative) -> usize {
    match native {
        WatchedCameraNative::SetFixedCamera => 6,
        WatchedCameraNative::ChangeCamera | WatchedCameraNative::ChangeCameraStatus => 3,
    }
}

#[expect(
    unsafe_code,
    reason = "calls the original native through the trampoline"
)]
extern "system" fn change_camera_status_detour(args: *mut c_void) {
    let original = ORIGINAL_CHANGE_CAMERA_STATUS.load(Ordering::Acquire);
    if original.is_null() {
        return;
    }
    if let Some(build) = STATUS_HOOK_BUILD.get() {
        let args_address = args as u64;
        let _ = panic::catch_unwind(|| {
            host_camera_native_ran(build, WatchedCameraNative::ChangeCameraStatus, args_address);
        });
    }
    // SAFETY: the trampoline continues into the original native, which has this signature.
    unsafe { core::mem::transmute::<*mut c_void, LuaNative>(original)(args) };
}
