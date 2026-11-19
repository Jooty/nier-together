use core::ffi::c_void;
use core::ptr;
use std::panic;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::{Mutex, OnceLock, PoisonError};

use replicant_game::GameMemory;
use replicant_game::dialogue::{
    MirrorWindowSize, TALK_WINDOW_LAYOUT_SIZE_OFFSET, TALK_WINDOW_RVA, layout_size_text,
};
use replicant_game::host_led_stretch::{window_size_mirrored_line, window_size_sent_line};
use replicant_game::offsets::SET_TALK_WINDOW_SIZE_NATIVE_RVA;
use replicant_game::quiet_end::{
    LUA_ARG_PAYLOAD_OFFSET, LUA_ARG_TAG_OFFSET, lua_arg_as_int, lua_arg_offset,
};
use replicant_game::stay_together::FollowRole;
use replicant_proto::{Message, TalkWindowSize};

use crate::build_gate::VerifiedBuild;
use crate::diagnostics::log_line;
use crate::game_hooks::hook_address;
use crate::memory::InProcessMemory;
use crate::net::{self, NetState, NetThread};
use crate::story_jump::{NativeArg, call_native_with, mod_native_call_running};

static HOOKED_BUILD: OnceLock<VerifiedBuild> = OnceLock::new();
static ORIGINAL_SET_TALK_WINDOW_SIZE: AtomicPtr<c_void> = AtomicPtr::new(ptr::null_mut());
static SIZES: Mutex<MirrorWindowSize> = Mutex::new(MirrorWindowSize::new());

type LuaNative = unsafe extern "system" fn(*mut c_void) -> i32;

pub(super) fn install(build: VerifiedBuild) {
    let _ = HOOKED_BUILD.set(build);
    let base = build.module_base();
    #[expect(unsafe_code, reason = "hooks the _SetTalkWindowSize native by address")]
    // SAFETY: the RVA starts the `_SetTalkWindowSize` Lua native (args block in, result code out)
    // in the verified build (ledger row 0x7021C0, docs off-hand/story-choices.md), and the
    // replacement has the same signature and passes the original's result through.
    unsafe {
        hook_address(
            "_SetTalkWindowSize native",
            (base + u64::from(SET_TALK_WINDOW_SIZE_NATIVE_RVA)) as usize,
            set_talk_window_size_detour as *const () as *mut c_void,
            &ORIGINAL_SET_TALK_WINDOW_SIZE,
        );
    }
}

pub(super) fn take_hosts_window_sizes(build: &VerifiedBuild, net: &NetThread) {
    for message in net.take_relayed(|message| matches!(message, Message::TalkWindowSize(_))) {
        let Message::TalkWindowSize(TalkWindowSize { size }) = message else {
            continue;
        };
        if crate::host_led_stretch::guest_watches_a_stretch() {
            set_size_here(build, size);
            sizes().note_own_size(size);
            log_line(&window_size_mirrored_line(size));
            continue;
        }
        sizes().note_host_size(size);
        log_line(&format!(
            "dialogue: the host's script set the talk window size {size}, kept for the host's story talks your window shows"
        ));
    }
}

/// Right before the mirror opens the host's story talk: the window takes the host's size, so
/// the talk is laid out as on the host (a full page where the host's is one).
pub(super) fn lend_the_hosts_size_to_the_mirror(build: &VerifiedBuild) {
    let Some(size) = sizes().size_for_the_mirror() else {
        return;
    };
    set_size_here(build, size);
    log_line(&format!(
        "dialogue: your talk window takes the host's size {size} for the host's story talk"
    ));
}

/// Once no talk window is open: a size the mirror took is given back to this game.
pub(super) fn give_back_the_own_size(build: &VerifiedBuild) {
    let Some(size) = sizes().size_to_give_back() else {
        return;
    };
    set_size_here(build, size);
    log_line(&format!(
        "dialogue: the host's talk is over here, your talk window has its own size {size} again"
    ));
}

pub(super) fn window_layout_size_text(build: &VerifiedBuild) -> String {
    let memory = InProcessMemory::new(build);
    let window = memory.module_base() + u64::from(TALK_WINDOW_RVA);
    memory
        .read_i32(window + TALK_WINDOW_LAYOUT_SIZE_OFFSET)
        .map_or_else(|_| "unreadable".to_owned(), layout_size_text)
}

fn sizes() -> std::sync::MutexGuard<'static, MirrorWindowSize> {
    SIZES.lock().unwrap_or_else(PoisonError::into_inner)
}

fn set_size_here(build: &VerifiedBuild, size: i32) {
    call_native_with(
        build,
        SET_TALK_WINDOW_SIZE_NATIVE_RVA,
        &[NativeArg::Integer(size)],
    );
}

#[expect(
    unsafe_code,
    reason = "calls the original _SetTalkWindowSize through the trampoline MinHook built"
)]
extern "system" fn set_talk_window_size_detour(args: *mut c_void) -> i32 {
    let _ = panic::catch_unwind(|| note_own_script_size(args));
    let original = ORIGINAL_SET_TALK_WINDOW_SIZE.load(Ordering::Acquire);
    if original.is_null() {
        return 0;
    }
    // SAFETY: the trampoline continues into the original native with this exact signature.
    unsafe { core::mem::transmute::<*mut c_void, LuaNative>(original)(args) }
}

fn note_own_script_size(args: *mut c_void) {
    if mod_native_call_running() {
        return;
    }
    let size = read_size(args);
    sizes().note_own_size(size);
    let Some(net) = net::handle() else {
        return;
    };
    let view = net.view();
    let hosting = FollowRole::from_slot(view.local_slot) == FollowRole::Host
        && view.state == NetState::Connected;
    if !hosting {
        return;
    }
    net.send_relayed(Message::TalkWindowSize(TalkWindowSize { size }));
    match crate::host_led_stretch::hosts_stretch_first_step() {
        Some(first_step) => log_line(&window_size_sent_line(size, first_step)),
        None => log_line(&format!(
            "dialogue: your script set the talk window size {size}, sent to the guest for your story talks its window shows"
        )),
    }
}

#[expect(
    unsafe_code,
    reason = "reads the native's first Lua argument from its argument block"
)]
fn read_size(args: *mut c_void) -> i32 {
    let arg = lua_arg_offset(0) as usize;
    // SAFETY: the game passes a live native block holding the one argument the script gave.
    unsafe {
        let tag = *args
            .cast::<u8>()
            .add(arg + LUA_ARG_TAG_OFFSET as usize)
            .cast::<i32>();
        let payload = *args
            .cast::<u8>()
            .add(arg + LUA_ARG_PAYLOAD_OFFSET as usize)
            .cast::<[u8; 8]>();
        lua_arg_as_int(tag, payload)
    }
}
