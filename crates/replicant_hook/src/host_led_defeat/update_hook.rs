use core::ffi::c_void;
use core::ptr;
use std::panic::{self, AssertUnwindSafe};
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::{OnceLock, PoisonError};

use replicant_game::GameMemory;
use replicant_game::dialogue::{
    TALK_UPDATE_INPUT_BUTTONS_START_OFFSET, TALK_UPDATE_INPUT_BYTES,
    TALK_UPDATE_INPUT_ENABLED_OFFSET, TALK_UPDATE_INPUT_POINTER_ACTIVE_OFFSET,
};
use replicant_game::host_led_defeat::{
    GAME_OVER_ROOT_SEQUENCE_UPDATE_RVA, GAME_OVER_SEQUENCE_CHOOSING_STATE,
    GAME_OVER_SEQUENCE_CURSOR_OFFSET, GuestDefeatAct, find_game_over_context, make_pick_on_screen,
    read_game_over_screen,
};
use replicant_game::stay_together::FollowRole;
use replicant_proto::Message;

use crate::build_gate::VerifiedBuild;
use crate::diagnostics::log_line;
use crate::game_hooks::hook_address;
use crate::host_led_defeat::state::{GUEST_FOLLOW, HOST_WATCH};
use crate::memory::InProcessMemory;
use crate::net::{self, NetSharedExt, NetState, net_clock_ms};

static HOOKED_BUILD: OnceLock<VerifiedBuild> = OnceLock::new();
static ORIGINAL_UPDATE: AtomicPtr<c_void> = AtomicPtr::new(ptr::null_mut());

type SequenceUpdate = unsafe extern "system" fn(*mut c_void, *mut u8);

enum GuestPlan {
    GameAsIs,
    ButtonsOff,
    PickMade,
}

pub fn install(build: VerifiedBuild) {
    if HOOKED_BUILD.set(build).is_err() {
        return;
    }
    #[expect(
        unsafe_code,
        reason = "hooks the defeat screen sequence's update by address"
    )]
    // SAFETY: the RVA starts GameOverRootSequence's update (vftable 0x140BC77B8 slot 0x20:
    // sequence, frame argument; no result) in the verified build, and the replacement has the
    // same signature.
    unsafe {
        hook_address(
            "GameOverRootSequence::Update",
            (build.module_base() + u64::from(GAME_OVER_ROOT_SEQUENCE_UPDATE_RVA)) as usize,
            update_detour as *const () as *mut c_void,
            &ORIGINAL_UPDATE,
        );
    }
}

extern "system" fn update_detour(sequence: *mut c_void, frame: *mut u8) {
    let original = ORIGINAL_UPDATE.load(Ordering::Acquire);
    if original.is_null() {
        return;
    }
    let Some(build) = HOOKED_BUILD.get() else {
        run_update(original, sequence, frame);
        return;
    };
    match session_role() {
        FollowRole::Host => host_frame(build, original, sequence, frame),
        FollowRole::Guest if !frame.is_null() => guest_frame(build, original, sequence, frame),
        FollowRole::Guest | FollowRole::Unknown => run_update(original, sequence, frame),
    }
}

fn session_role() -> FollowRole {
    let Some(net) = net::handle() else {
        return FollowRole::Unknown;
    };
    let view = net.view();
    if view.state != NetState::Connected || !view.partner_in_play() {
        return FollowRole::Unknown;
    }
    FollowRole::from_slot(view.local_slot)
}

fn host_frame(build: &VerifiedBuild, original: *mut c_void, sequence: *mut c_void, frame: *mut u8) {
    let memory = InProcessMemory::new(build);
    let before = read_game_over_screen(&memory, sequence as u64)
        .ok()
        .flatten();
    run_update(original, sequence, frame);
    let _ = panic::catch_unwind(AssertUnwindSafe(|| {
        let (Some(before), Ok(Some(after))) =
            (before, read_game_over_screen(&memory, sequence as u64))
        else {
            return;
        };
        let watched = HOST_WATCH
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .on_frame(before.state, after, net_clock_ms());
        if let Some(note) = watched.send
            && let Some(net) = net::handle()
        {
            net.send_relayed(Message::HostDefeatScreen(note));
        }
        for line in watched.lines {
            log_line(&line);
        }
    }));
}

fn guest_frame(
    build: &VerifiedBuild,
    original: *mut c_void,
    sequence: *mut c_void,
    frame: *mut u8,
) {
    let plan = panic::catch_unwind(AssertUnwindSafe(|| guest_plan(build, sequence as u64)))
        .unwrap_or(GuestPlan::GameAsIs);
    match plan {
        GuestPlan::GameAsIs => run_update(original, sequence, frame),
        GuestPlan::ButtonsOff => {
            let mut quiet = [0_u8; TALK_UPDATE_INPUT_BYTES];
            #[expect(unsafe_code, reason = "copies the game's own frame argument")]
            // SAFETY: `frame` is the UI manager's per-frame argument (time step at 0, pad from 4,
            // pointer flag at 0x44), the same TALK_UPDATE_INPUT_BYTES block the talk window gets.
            unsafe {
                ptr::copy_nonoverlapping(frame, quiet.as_mut_ptr(), TALK_UPDATE_INPUT_BYTES);
            }
            // Every button and pointer test of the screen needs the enabled flag (0x78130 and kin).
            quiet[TALK_UPDATE_INPUT_ENABLED_OFFSET] = 0;
            quiet[TALK_UPDATE_INPUT_POINTER_ACTIVE_OFFSET] = 0;
            quiet[TALK_UPDATE_INPUT_BUTTONS_START_OFFSET..TALK_UPDATE_INPUT_ENABLED_OFFSET].fill(0);
            run_update(original, sequence, quiet.as_mut_ptr());
        }
        GuestPlan::PickMade => {}
    }
}

fn guest_plan(build: &VerifiedBuild, sequence: u64) -> GuestPlan {
    let mut memory = InProcessMemory::new(build);
    let Ok(Some(screen)) = read_game_over_screen(&memory, sequence) else {
        return GuestPlan::GameAsIs;
    };
    let context = find_game_over_context(&memory).ok().flatten();
    let followed = GUEST_FOLLOW
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .on_frame(screen, context.is_some(), net_clock_ms());
    for line in &followed.lines {
        log_line(line);
    }
    match followed.act {
        GuestDefeatAct::OwnButtons => GuestPlan::GameAsIs,
        GuestDefeatAct::ButtonsOff { show_cursor } => {
            if let Some(cursor) = show_cursor
                && screen.state == GAME_OVER_SEQUENCE_CHOOSING_STATE
                && cursor != screen.cursor
            {
                let _ = memory.write_i32(sequence + GAME_OVER_SEQUENCE_CURSOR_OFFSET, cursor);
            }
            GuestPlan::ButtonsOff
        }
        GuestDefeatAct::MakeHostsPick(pick) => {
            let made = context.is_some_and(|context| {
                make_pick_on_screen(&mut memory, sequence, context, pick).is_ok()
            });
            if made {
                return GuestPlan::PickMade;
            }
            log_line(&format!(
                "defeat screen: the host's pick {} could not be written here, your buttons stay off",
                pick.label()
            ));
            GuestPlan::ButtonsOff
        }
    }
}

#[expect(
    unsafe_code,
    reason = "calls the original sequence update through the trampoline MinHook built"
)]
fn run_update(original: *mut c_void, sequence: *mut c_void, frame: *mut u8) {
    // SAFETY: the trampoline continues into the original update, which has this signature; the
    // frame argument is the game's own or a copy of it.
    unsafe { core::mem::transmute::<*mut c_void, SequenceUpdate>(original)(sequence, frame) }
}
