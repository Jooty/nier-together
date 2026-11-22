use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use automata_game::address::MOUSE_INPUT_SOURCE;
use automata_game::exe::ExeVariant;
use automata_game::raw_mouse::{
    MOUSE_SOURCE_DX_OFFSET, MOUSE_SOURCE_DY_OFFSET, MOUSE_SOURCE_GOOD_READ_OFFSET,
    RawMouseAccumulator, ScreenRect, clip_cursor_is_set, counts_of_device_move,
};
use nt_hook::build_gate::VerifiedBuild;
use nt_hook::memory::{GameMemory, InProcessMemory};
use windows_sys::Win32::Foundation::{HWND, LPARAM, POINT, RECT};
use windows_sys::Win32::Graphics::Gdi::ClientToScreen;
use windows_sys::Win32::System::Threading::GetCurrentProcessId;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GW_OWNER, GetClientRect, GetClipCursor, GetCursorPos, GetSystemMetrics, GetWindow,
    GetWindowThreadProcessId, IsWindowVisible, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN,
    SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN,
};

static RAW_MOUSE_ON: AtomicBool = AtomicBool::new(false);
static ACCUMULATOR: Mutex<RawMouseAccumulator> = Mutex::new(RawMouseAccumulator::new());

pub fn raw_mouse_on() -> bool {
    RAW_MOUSE_ON.load(Ordering::Relaxed)
}

pub fn set_raw_mouse_on(on: bool) {
    RAW_MOUSE_ON.store(on, Ordering::Relaxed);
    if !on {
        with_accumulator(RawMouseAccumulator::discard_all);
    }
}

pub(super) fn collect_raw_mouse_counts(
    build: &VerifiedBuild,
    variant: ExeVariant,
    menu_open: bool,
) {
    if !raw_mouse_on() {
        with_accumulator(RawMouseAccumulator::discard_all);
        return;
    }
    let counts = if menu_open {
        (0, 0)
    } else {
        counts_of_the_device_source(build, variant).unwrap_or((0, 0))
    };
    with_accumulator(|accumulator| {
        accumulator.add(counts.0, counts.1);
        accumulator.begin_frame();
    });
}

pub(super) fn take_raw_mouse_frame_counts() -> (i32, i32) {
    with_accumulator(RawMouseAccumulator::take_frame_counts)
}

fn counts_of_the_device_source(build: &VerifiedBuild, variant: ExeVariant) -> Option<(i32, i32)> {
    let memory = InProcessMemory::new(build);
    let source = build.module_base() + u64::from(MOUSE_INPUT_SOURCE.rva(variant));
    if memory
        .read_i32(source + MOUSE_SOURCE_GOOD_READ_OFFSET)
        .ok()?
        == 0
    {
        return None;
    }
    let dx = memory.read_f32(source + MOUSE_SOURCE_DX_OFFSET).ok()?;
    let dy = memory.read_f32(source + MOUSE_SOURCE_DY_OFFSET).ok()?;
    Some(counts_of_device_move(dx, dy))
}

fn with_accumulator<R>(change: impl FnOnce(&mut RawMouseAccumulator) -> R) -> R {
    let mut guard = ACCUMULATOR
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    change(&mut guard)
}

pub(super) fn game_window_report() -> String {
    let Some(window) = own_top_window() else {
        return "window=none".to_string();
    };
    let mut client = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut origin = POINT { x: 0, y: 0 };
    let mut cursor = POINT { x: 0, y: 0 };
    let mut clip = client;
    #[expect(
        unsafe_code,
        reason = "reads the window rectangle, cursor and clip rectangle"
    )]
    // SAFETY: every out pointer is a valid local; the window handle came from EnumWindows just now.
    let known = unsafe {
        GetClientRect(window, &raw mut client) != 0
            && ClientToScreen(window, &raw mut origin) != 0
            && GetCursorPos(&raw mut cursor) != 0
            && GetClipCursor(&raw mut clip) != 0
    };
    if !known {
        return "window=unreadable".to_string();
    }
    #[expect(unsafe_code, reason = "reads the virtual screen size")]
    // SAFETY: GetSystemMetrics has no preconditions.
    let whole_screen = unsafe {
        let (left, top) = (
            GetSystemMetrics(SM_XVIRTUALSCREEN),
            GetSystemMetrics(SM_YVIRTUALSCREEN),
        );
        ScreenRect {
            left,
            top,
            right: left + GetSystemMetrics(SM_CXVIRTUALSCREEN),
            bottom: top + GetSystemMetrics(SM_CYVIRTUALSCREEN),
        }
    };
    let clip_rect = ScreenRect {
        left: clip.left,
        top: clip.top,
        right: clip.right,
        bottom: clip.bottom,
    };
    format!(
        "client={}x{} at ({},{}) cursor=({},{}) clip_set={} clip=({},{},{},{})",
        client.right - client.left,
        client.bottom - client.top,
        origin.x,
        origin.y,
        cursor.x,
        cursor.y,
        clip_cursor_is_set(clip_rect, whole_screen),
        clip.left,
        clip.top,
        clip.right,
        clip.bottom,
    )
}

fn own_top_window() -> Option<HWND> {
    let mut found: HWND = core::ptr::null_mut();
    #[expect(
        unsafe_code,
        reason = "walks the top level windows for this process's own"
    )]
    // SAFETY: the callback only reads the pointer passed as lparam, which outlives the call.
    unsafe {
        EnumWindows(Some(pick_own_window), &raw mut found as LPARAM);
    }
    (!found.is_null()).then_some(found)
}

#[expect(unsafe_code, reason = "window enumeration callback")]
unsafe extern "system" fn pick_own_window(window: HWND, found: LPARAM) -> i32 {
    let mut process_id = 0_u32;
    // SAFETY: the out pointer is a valid local; the handle comes from EnumWindows.
    let (owner_pid, own_pid, visible, has_owner) = unsafe {
        GetWindowThreadProcessId(window, &raw mut process_id);
        (
            process_id,
            GetCurrentProcessId(),
            IsWindowVisible(window) != 0,
            !GetWindow(window, GW_OWNER).is_null(),
        )
    };
    if owner_pid == own_pid && visible && !has_owner {
        // SAFETY: lparam is the address of the HWND local in own_top_window, alive during the call.
        unsafe { *(found as *mut HWND) = window };
        return 0;
    }
    1
}
