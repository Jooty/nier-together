use core::ffi::c_void;
use core::ptr;
use std::panic;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicPtr, Ordering};

use replicant_game::host_led_stretch::{window_size_mirrored_line, window_size_sent_line};
use replicant_game::offsets::SET_TALK_WINDOW_SIZE_NATIVE_RVA;
use replicant_game::quiet_end::{
    LUA_ARG_PAYLOAD_OFFSET, LUA_ARG_TAG_OFFSET, lua_arg_as_int, lua_arg_offset,
};
use replicant_proto::{Message, TalkWindowSize};

use crate::build_gate::VerifiedBuild;
use crate::diagnostics::log_line;
use crate::game_hooks::hook_address;
use crate::net::{self, NetThread};
use crate::story_jump::{NativeArg, call_native_with};

static HOOKED_BUILD: OnceLock<VerifiedBuild> = OnceLock::new();
static ORIGINAL_SET_TALK_WINDOW_SIZE: AtomicPtr<c_void> = AtomicPtr::new(ptr::null_mut());

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
        call_native_with(
            build,
            SET_TALK_WINDOW_SIZE_NATIVE_RVA,
            &[NativeArg::Integer(size)],
        );
        log_line(&window_size_mirrored_line(size));
    }
}

#[expect(
    unsafe_code,
    reason = "calls the original _SetTalkWindowSize through the trampoline MinHook built"
)]
extern "system" fn set_talk_window_size_detour(args: *mut c_void) -> i32 {
    let _ = panic::catch_unwind(|| send_while_the_host_leads(args));
    let original = ORIGINAL_SET_TALK_WINDOW_SIZE.load(Ordering::Acquire);
    if original.is_null() {
        return 0;
    }
    // SAFETY: the trampoline continues into the original native with this exact signature.
    unsafe { core::mem::transmute::<*mut c_void, LuaNative>(original)(args) }
}

fn send_while_the_host_leads(args: *mut c_void) {
    let Some(first_step) = crate::host_led_stretch::hosts_stretch_first_step() else {
        return;
    };
    let Some(net) = net::handle() else {
        return;
    };
    let size = read_size(args);
    net.send_relayed(Message::TalkWindowSize(TalkWindowSize { size }));
    log_line(&window_size_sent_line(size, first_step));
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
