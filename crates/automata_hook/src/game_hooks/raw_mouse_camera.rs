use core::ffi::c_void;
use core::ptr;
use std::sync::atomic::{AtomicBool, AtomicPtr, AtomicUsize, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

use automata_game::address::{
    ADD_FRAME_TIME_TO_HINT_RECORD_FUNCTION, BODY_OR_INPUT_BITS_FUNCTION,
    CAMERA_EASE_WINDOW_OPEN_FUNCTION, CAMERA_OPTION_FLAGS_WORD_VALUE,
    CAMERA_TARGET_AS_INPUT_BODY_FUNCTION, ENTITY_AS_PLAYER_BODY_FUNCTION, GameAddress,
    SENSITIVITY_X_VALUE, SENSITIVITY_Y_VALUE,
};
use automata_game::build::variant_of_image_size;
use automata_game::exe::ExeVariant;
use automata_game::raw_mouse::{
    AUTO_FOLLOW_SUPPRESS_VALUE, BODY_RECENTRED_BY_HAND_INPUT_BIT,
    CAMERA_AUTO_FOLLOW_SUPPRESS_FRAMES_OFFSET, CAMERA_BASIS_MATRIX_OFFSET,
    CAMERA_CUT_FRAMES_OFFSET, CAMERA_EASE_PITCH_LIMIT_OFFSET, CAMERA_EASE_PITCH_RATE_SOURCE_OFFSET,
    CAMERA_EASE_SAVED_PITCH_SOURCE_OFFSET, CAMERA_EASE_SAVED_YAW_SOURCE_OFFSET,
    CAMERA_EASE_WINDOW_DIRECTION_TIMER_OFFSET, CAMERA_EASE_YAW_RATE_SOURCE_OFFSET,
    CAMERA_EASED_EYE_OFFSET, CAMERA_EASED_EYE_TO_LOOK_DISTANCE_OFFSET, CAMERA_EASED_LOOK_AT_OFFSET,
    CAMERA_INVERT_PITCH_OFFSET, CAMERA_INVERT_YAW_OFFSET, CAMERA_LOWER_PITCH_LIMIT_ENABLE_OFFSET,
    CAMERA_LOWER_PITCH_LIMIT_OFFSET, CAMERA_MOVED_BY_HAND_OFFSET, CAMERA_PITCH_OFFSET,
    CAMERA_TARGET_BODY_POSITION_OFFSET, CAMERA_TARGET_OFFSET, CAMERA_TARGET_OWNER_OFFSET,
    CAMERA_UPPER_PITCH_LIMIT_ENABLE_OFFSET, CAMERA_UPPER_PITCH_LIMIT_OFFSET, CAMERA_YAW_OFFSET,
    CONTROLLER_MOUSE_STICK_LENGTH_OFFSET, CameraAngles, CapProbe, CapProbeLine, EaseArmSource,
    FollowProbe, FollowProbeLine, FollowSample, HINT_RECORD_CAMERA_MOVED_BY_HAND,
    HOOK_PROLOGUE_LEN, IDENTITY_BASIS, LookHold, ModeLimit, RawLook, RawMouseHook, SwingProbe,
    apply_raw_look, camera_field_writes, ease_arm_writes, eye_to_look_distance,
    pitch_inside_ease_limit, prologue_matches, raw_camera_plan, rotate_eased_offset,
    rotation_basis, sensitivity_multiplier, x_sensitivity_value, yaw_step_radians,
};
use nt_hook::build_gate::VerifiedBuild;
use nt_hook::diagnostics::log_line;
use nt_hook::memory::{GameMemory, InProcessMemory};

use crate::game_hooks::input_block::menu_is_open;
use crate::game_hooks::raw_mouse_counts::{
    game_window_report, raw_mouse_on, take_raw_mouse_frame_counts,
};

static ORIGINAL_APPLY_STICK: AtomicPtr<c_void> = AtomicPtr::new(ptr::null_mut());
static ORIGINAL_GET_CAMERA_STICK: AtomicPtr<c_void> = AtomicPtr::new(ptr::null_mut());
static RAW_BUILD: OnceLock<(VerifiedBuild, ExeVariant)> = OnceLock::new();
static INPUT_BLOCKED_ADDRESS: AtomicUsize = AtomicUsize::new(0);
static IN_CAMERA_STEP: AtomicBool = AtomicBool::new(false);
static GAME_MOUSE_REPLACED: AtomicBool = AtomicBool::new(false);
static FIRST_CONTROLLER_IN_STEP: AtomicUsize = AtomicUsize::new(0);
static HOOKED: AtomicBool = AtomicBool::new(false);
static FIRST_MOVE_LOGGED: AtomicBool = AtomicBool::new(false);
static LOOK_HOLD: Mutex<LookHold> = Mutex::new(LookHold::new());
static SWING_PROBE: Mutex<SwingProbe> = Mutex::new(SwingProbe::new());
static CAP_PROBE: Mutex<CapProbe> = Mutex::new(CapProbe::new());
static FOLLOW_PROBE: Mutex<FollowProbe> = Mutex::new(FollowProbe::new());
static LAST_TURN: Mutex<Option<(CameraAngles, CameraAngles)>> = Mutex::new(None);
static CLOCK_START: OnceLock<Instant> = OnceLock::new();

type ApplyStick = unsafe extern "system" fn(usize, u32, u32, f32, f32);
type GetCameraStick = unsafe extern "system" fn(usize, usize) -> usize;
type InputBlocked = unsafe extern "system" fn(usize) -> u32;
type AddFrameTimeToHintRecord = unsafe extern "system" fn(i32);
type EaseWindowOpen = unsafe extern "system" fn(usize) -> u32;
type CameraTargetAsInputBody = unsafe extern "system" fn(usize) -> usize;
type OrBodyInputBits = unsafe extern "system" fn(usize, u32);
type EntityAsPlayerBody = unsafe extern "system" fn(usize) -> usize;

pub fn raw_mouse_camera_hooked() -> bool {
    HOOKED.load(Ordering::Relaxed)
}

pub fn install_raw_mouse_camera(build: VerifiedBuild) {
    let Some(variant) = variant_of_image_size(build.size_of_image()) else {
        log_line("raw mouse: image size matches no known exe, not installed");
        return;
    };
    if RAW_BUILD.set((build, variant)).is_err() {
        return;
    }
    let base = build.module_base() as usize;
    let memory = InProcessMemory::new(&build);
    for hook in RawMouseHook::ALL {
        let address = base + hook.address().rva(variant) as usize;
        let found = memory.read_array::<HOOK_PROLOGUE_LEN>(address as u64);
        if !found.is_ok_and(|bytes| prologue_matches(hook, &bytes)) {
            log_line(&format!(
                "raw mouse: the first bytes of the {} are not the noted ones, nothing hooked",
                hook.label()
            ));
            return;
        }
    }
    INPUT_BLOCKED_ADDRESS.store(
        base + RawMouseHook::ControllerInputBlocked.address().rva(variant) as usize,
        Ordering::Release,
    );
    let apply = base + RawMouseHook::ApplyStickToAngles.address().rva(variant) as usize;
    let pick = base + RawMouseHook::GetCameraStick.address().rva(variant) as usize;
    let picked = hook_one(
        RawMouseHook::GetCameraStick,
        pick,
        get_camera_stick_detour as *const (),
        &ORIGINAL_GET_CAMERA_STICK,
    );
    let applied = picked
        && hook_one(
            RawMouseHook::ApplyStickToAngles,
            apply,
            apply_stick_detour as *const (),
            &ORIGINAL_APPLY_STICK,
        );
    HOOKED.store(applied, Ordering::Release);
    if applied {
        log_line("raw mouse: swing probe on (lines start with \"swing probe\")");
        log_line(
            "raw mouse: cap probe on (lines start with \"cap probe\"): windowed maxima of lX, lY, yaw step",
        );
        log_line(
            "raw mouse: follow probe on (lines start with \"follow probe\"): largest body to camera change while the hero moves",
        );
    }
}

fn hook_one(
    hook: RawMouseHook,
    address: usize,
    replacement: *const (),
    original: &AtomicPtr<c_void>,
) -> bool {
    #[expect(
        unsafe_code,
        reason = "hooks a game camera function; each replacement has the signature of its original"
    )]
    // SAFETY: the address passed the first-bytes check for this function of the verified build and
    // the replacement forwards every argument to the original.
    unsafe {
        nt_hook::detour::hook_address(hook.label(), address, replacement as *mut c_void, original)
    }
}

#[expect(
    unsafe_code,
    reason = "calls the original camera stick pick through the trampoline MinHook built"
)]
extern "system" fn get_camera_stick_detour(controller: usize, out: usize) -> usize {
    let original = ORIGINAL_GET_CAMERA_STICK.load(Ordering::Acquire);
    if original.is_null() {
        return out;
    }
    let call = || {
        // SAFETY: the trampoline continues into the original with the two arguments received.
        unsafe { core::mem::transmute::<*mut c_void, GetCameraStick>(original)(controller, out) }
    };
    if !IN_CAMERA_STEP.load(Ordering::Acquire) {
        return call();
    }
    let _ = FIRST_CONTROLLER_IN_STEP.compare_exchange(
        0,
        controller,
        Ordering::AcqRel,
        Ordering::Acquire,
    );
    if !GAME_MOUSE_REPLACED.load(Ordering::Acquire) {
        return call();
    }
    let Some((build, _)) = RAW_BUILD.get() else {
        return call();
    };
    let mut memory = InProcessMemory::new(build);
    let length_address = controller as u64 + CONTROLLER_MOUSE_STICK_LENGTH_OFFSET;
    let Ok(kept_length) = memory.read_f32(length_address) else {
        return call();
    };
    // the pick falls through to the keyboard and pad sticks while the mouse stick has no length
    let zeroed = memory.write_f32(length_address, 0.0).is_ok();
    let result = call();
    if zeroed {
        let _ = memory.write_f32(length_address, kept_length);
    }
    result
}

#[expect(
    unsafe_code,
    reason = "calls the original camera step through the trampoline MinHook built"
)]
extern "system" fn apply_stick_detour(
    camera: usize,
    pitch_on: u32,
    yaw_on: u32,
    pitch_rate: f32,
    yaw_rate: f32,
) {
    let original = ORIGINAL_APPLY_STICK.load(Ordering::Acquire);
    if original.is_null() {
        return;
    }
    let setting_on = raw_mouse_on();
    FIRST_CONTROLLER_IN_STEP.store(0, Ordering::Release);
    GAME_MOUSE_REPLACED.store(setting_on, Ordering::Release);
    IN_CAMERA_STEP.store(true, Ordering::Release);
    // SAFETY: the trampoline continues into the original with the arguments received.
    unsafe {
        core::mem::transmute::<*mut c_void, ApplyStick>(original)(
            camera, pitch_on, yaw_on, pitch_rate, yaw_rate,
        );
    }
    IN_CAMERA_STEP.store(false, Ordering::Release);
    GAME_MOUSE_REPLACED.store(false, Ordering::Release);
    let controller = FIRST_CONTROLLER_IN_STEP.swap(0, Ordering::AcqRel);
    if !setting_on || controller == 0 || controller_input_is_blocked(controller) {
        return;
    }
    let plan = raw_camera_plan(true, menu_is_open(), take_raw_mouse_frame_counts());
    let turn = if plan.counts == (0, 0) {
        None
    } else {
        write_raw_look(
            camera,
            plan.counts,
            pitch_on & 0xFF != 0,
            yaw_on & 0xFF != 0,
        )
    };
    let yaw_step = turn.map(|(before, after)| yaw_step_radians(before.yaw, after.yaw));
    let now_ms = now_ms();
    probe_the_follow(camera, turn.is_some());
    if let Some((before, after)) = turn {
        carry_the_eased_eye_through_the_turn(camera, before, after);
    }
    with_lock(&LAST_TURN, |last| *last = turn);
    let held = with_lock(&LOOK_HOLD, |hold| hold.step(now_ms, yaw_step.is_some()));
    write_camera_fields(camera);
    if yaw_step.is_some() {
        run_the_originals_moving_path(camera);
    } else if held {
        hold_moved_by_hand(camera);
    }
    let probe_line = with_lock(&SWING_PROBE, |probe| {
        probe.step(now_ms, yaw_step.unwrap_or(0.0))
    });
    if probe_line {
        log_swing_probe_line(camera, yaw_step.unwrap_or(0.0));
    }
    if let Some(step) = yaw_step {
        let cap_line = with_lock(&CAP_PROBE, |probe| probe.step(now_ms, plan.counts, step));
        if let Some(line) = cap_line {
            log_cap_probe_line(&line);
        }
    }
}

fn write_camera_fields(camera: usize) {
    let Some((build, _)) = RAW_BUILD.get() else {
        return;
    };
    let writes = camera_field_writes(raw_mouse_on());
    if writes.suppress_auto_follow {
        let _ = InProcessMemory::new(build).write_i32(
            camera as u64 + CAMERA_AUTO_FOLLOW_SUPPRESS_FRAMES_OFFSET,
            AUTO_FOLLOW_SUPPRESS_VALUE,
        );
    }
}

fn carry_the_eased_eye_through_the_turn(camera: usize, before: CameraAngles, after: CameraAngles) {
    let Some((build, _)) = RAW_BUILD.get() else {
        return;
    };
    let mut memory = InProcessMemory::new(build);
    let camera = camera as u64;
    let (Some(eye), Some(look_at)) = (
        read_vec3(&memory, camera + CAMERA_EASED_EYE_OFFSET),
        read_vec3(&memory, camera + CAMERA_EASED_LOOK_AT_OFFSET),
    ) else {
        return;
    };
    let basis = read_basis(&memory, camera);
    let offset = rotate_eased_offset(subtract(eye, look_at), before, after, &basis);
    if offset.iter().any(|axis| !axis.is_finite()) {
        return;
    }
    for axis in 0..3 {
        let _ = memory.write_f32(
            camera + CAMERA_EASED_EYE_OFFSET + 4 * axis as u64,
            look_at[axis] + offset[axis],
        );
    }
}

fn probe_the_follow(camera: usize, raw_turned: bool) {
    let Some((build, _)) = RAW_BUILD.get() else {
        return;
    };
    let memory = InProcessMemory::new(build);
    let camera = camera as u64;
    let Ok(target) = memory.read_u64(camera + CAMERA_TARGET_OFFSET) else {
        return;
    };
    if target == 0 {
        return;
    }
    let (Some(body), Some(eye), Some(look_at)) = (
        read_vec3(&memory, target + CAMERA_TARGET_BODY_POSITION_OFFSET),
        read_vec3(&memory, camera + CAMERA_EASED_EYE_OFFSET),
        read_vec3(&memory, camera + CAMERA_EASED_LOOK_AT_OFFSET),
    ) else {
        return;
    };
    let offset = subtract(eye, look_at);
    let eye_from_look_at_turn_removed = with_lock(&LAST_TURN, |last| match *last {
        Some((before, after)) => {
            rotate_eased_offset(offset, after, before, &read_basis(&memory, camera))
        }
        None => offset,
    });
    let sample = FollowSample {
        body,
        look_at_from_body: subtract(look_at, body),
        eye_from_look_at_turn_removed,
        raw_turned,
    };
    let line = with_lock(&FOLLOW_PROBE, |probe| probe.step(now_micros(), sample));
    if let Some(line) = line {
        log_follow_probe_line(&line);
    }
}

fn log_follow_probe_line(line: &FollowProbeLine) {
    log_line(&format!(
        "follow probe: largest_body_to_camera_change={:.4} largest_frame_ms={:.1} raw_turned={} frames={}",
        line.largest_jump, line.largest_frame_ms, line.raw_turned, line.frames
    ));
}

fn read_vec3(memory: &InProcessMemory, address: u64) -> Option<[f32; 3]> {
    Some([
        memory.read_f32(address).ok()?,
        memory.read_f32(address + 4).ok()?,
        memory.read_f32(address + 8).ok()?,
    ])
}

fn read_basis(memory: &InProcessMemory, camera: u64) -> [[f32; 3]; 3] {
    let row = |index: u64| read_vec3(memory, camera + CAMERA_BASIS_MATRIX_OFFSET + 16 * index);
    let rows = [row(0), row(1), row(2)];
    match rows {
        [Some(a), Some(b), Some(c)] => rotation_basis([a, b, c]).unwrap_or(IDENTITY_BASIS),
        _ => IDENTITY_BASIS,
    }
}

fn subtract(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn now_micros() -> u64 {
    CLOCK_START.get_or_init(Instant::now).elapsed().as_micros() as u64
}

fn with_lock<T, R>(lock: &Mutex<T>, change: impl FnOnce(&mut T) -> R) -> R {
    let mut guard = lock
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    change(&mut guard)
}

fn now_ms() -> u64 {
    CLOCK_START.get_or_init(Instant::now).elapsed().as_millis() as u64
}

fn hold_moved_by_hand(camera: usize) {
    let Some((build, _)) = RAW_BUILD.get() else {
        return;
    };
    let _ = InProcessMemory::new(build).write_i32(camera as u64 + CAMERA_MOVED_BY_HAND_OFFSET, 1);
}

fn game_function(address: GameAddress) -> Option<usize> {
    let (build, variant) = RAW_BUILD.get()?;
    Some(build.module_base() as usize + address.rva(*variant) as usize)
}

#[expect(
    unsafe_code,
    reason = "calls the game functions the original camera step calls when the camera was moved by hand"
)]
fn run_the_originals_moving_path(camera: usize) {
    let Some((build, _)) = RAW_BUILD.get() else {
        return;
    };
    let mut memory = InProcessMemory::new(build);
    let target = memory
        .read_u64(camera as u64 + CAMERA_TARGET_OFFSET)
        .unwrap_or(0) as usize;
    if let Some(add_frame_time) = game_function(ADD_FRAME_TIME_TO_HINT_RECORD_FUNCTION) {
        // SAFETY: the address is Hint::addFrameTime of the verified build; the original camera step
        // calls it with the same record id on the same thread.
        unsafe {
            core::mem::transmute::<usize, AddFrameTimeToHintRecord>(add_frame_time)(
                HINT_RECORD_CAMERA_MOVED_BY_HAND,
            );
        }
    }
    if let (Some(cast), Some(or_bits)) = (
        game_function(CAMERA_TARGET_AS_INPUT_BODY_FUNCTION),
        game_function(BODY_OR_INPUT_BITS_FUNCTION),
    ) {
        // SAFETY: both addresses are of the verified build; the cast takes 0 or the camera's own
        // target and returns 0 or a body, as the original camera step uses them.
        unsafe {
            let body = core::mem::transmute::<usize, CameraTargetAsInputBody>(cast)(target);
            if body != 0 {
                core::mem::transmute::<usize, OrBodyInputBits>(or_bits)(
                    body,
                    BODY_RECENTRED_BY_HAND_INPUT_BIT,
                );
            }
        }
    }
    arm_the_ease_window(&mut memory, camera as u64, target as u64);
}

#[expect(
    unsafe_code,
    reason = "calls the game's ease window test and entity to player body cast"
)]
fn arm_the_ease_window(memory: &mut InProcessMemory, camera: u64, target: u64) {
    let (Some(window_open), Some(as_player)) = (
        game_function(CAMERA_EASE_WINDOW_OPEN_FUNCTION),
        game_function(ENTITY_AS_PLAYER_BODY_FUNCTION),
    ) else {
        return;
    };
    // SAFETY: the address is CameraGame::easeWindowOpen of the verified build; it reads fields of
    // the camera it is given.
    let open =
        unsafe { core::mem::transmute::<usize, EaseWindowOpen>(window_open)(camera as usize) };
    if open & 0xFF == 0 || target == 0 {
        return;
    }
    let Ok(owner) = memory.read_u64(target + CAMERA_TARGET_OWNER_OFFSET) else {
        return;
    };
    // SAFETY: the address is Entity::asPl0000 of the verified build; the owner is the pointer the
    // original camera step passes to it, read this frame.
    let player =
        unsafe { core::mem::transmute::<usize, EntityAsPlayerBody>(as_player)(owner as usize) };
    if player == 0 {
        return;
    }
    let read = |memory: &InProcessMemory, offset| memory.read_i32(camera + offset);
    let (Ok(pitch_to_ease_from), Ok(yaw_to_ease_from), Ok(pitch_rate), Ok(yaw_rate)) = (
        read(memory, CAMERA_EASE_SAVED_PITCH_SOURCE_OFFSET),
        read(memory, CAMERA_EASE_SAVED_YAW_SOURCE_OFFSET),
        read(memory, CAMERA_EASE_PITCH_RATE_SOURCE_OFFSET),
        read(memory, CAMERA_EASE_YAW_RATE_SOURCE_OFFSET),
    ) else {
        return;
    };
    for write in ease_arm_writes(EaseArmSource {
        pitch_to_ease_from,
        yaw_to_ease_from,
        pitch_rate,
        yaw_rate,
    }) {
        let _ = memory.write_i32(camera + write.offset, write.value);
    }
    let (Ok(timer), Ok(limit), Ok(pitch)) = (
        memory.read_f32(camera + CAMERA_EASE_WINDOW_DIRECTION_TIMER_OFFSET),
        memory.read_f32(camera + CAMERA_EASE_PITCH_LIMIT_OFFSET),
        memory.read_f32(camera + CAMERA_PITCH_OFFSET),
    ) else {
        return;
    };
    if let Some(clamped) = pitch_inside_ease_limit(pitch, limit, timer) {
        let _ = memory.write_f32(camera + CAMERA_PITCH_OFFSET, clamped);
    }
}

fn log_swing_probe_line(camera: usize, yaw_step: f32) {
    let Some((build, _)) = RAW_BUILD.get() else {
        return;
    };
    let memory = InProcessMemory::new(build);
    let camera = camera as u64;
    let float = |offset: u64| memory.read_f32(camera + offset).unwrap_or(f32::NAN);
    let hand = memory
        .read_i32(camera + CAMERA_MOVED_BY_HAND_OFFSET)
        .unwrap_or(-1);
    let distance = eye_to_look_distance(
        [float(0x140), float(0x144), float(0x148)],
        [float(0x150), float(0x154), float(0x158)],
    );
    let eased_distance = eye_to_look_distance(
        [
            float(CAMERA_EASED_EYE_OFFSET),
            float(CAMERA_EASED_EYE_OFFSET + 4),
            float(CAMERA_EASED_EYE_OFFSET + 8),
        ],
        [
            float(CAMERA_EASED_LOOK_AT_OFFSET),
            float(CAMERA_EASED_LOOK_AT_OFFSET + 4),
            float(CAMERA_EASED_LOOK_AT_OFFSET + 8),
        ],
    );
    let int = |offset: u64| memory.read_i32(camera + offset).unwrap_or(-1);
    log_line(&format!(
        "swing probe: yaw_step={yaw_step:.4} d3B0={:.3} d550={:.3} d184={:.3} hand400={hand} \
         off594={:.3} off59C={:.3} cam_to_look={distance:.3} d134={:.3} cut20C={} follow8394={} \
         eased_eye_to_look={eased_distance:.3}",
        float(0x3B0),
        float(0x550),
        float(0x184),
        float(0x594),
        float(0x59C),
        float(CAMERA_EASED_EYE_TO_LOOK_DISTANCE_OFFSET),
        int(CAMERA_CUT_FRAMES_OFFSET),
        int(CAMERA_AUTO_FOLLOW_SUPPRESS_FRAMES_OFFSET),
    ));
}

fn log_cap_probe_line(line: &CapProbeLine) {
    let Some((build, variant)) = RAW_BUILD.get() else {
        return;
    };
    let memory = InProcessMemory::new(build);
    let level = |address: GameAddress| {
        memory
            .read_i32(memory.module_base() + u64::from(address.rva(*variant)))
            .unwrap_or(-1)
    };
    let (x_level, y_level) = (level(SENSITIVITY_X_VALUE), level(SENSITIVITY_Y_VALUE));
    let multipliers = (
        sensitivity_multiplier_of(&memory, *variant, SENSITIVITY_X_VALUE, true),
        sensitivity_multiplier_of(&memory, *variant, SENSITIVITY_Y_VALUE, false),
    );
    let window = if line.with_window_report {
        game_window_report()
    } else {
        String::new()
    };
    log_line(&format!(
        "cap probe: max_lX={} max_lY={} max_yaw_step={:.4} frames={} sens_x={x_level} sens_y={y_level} \
         mult_x={:.3} mult_y={:.3} {window}",
        line.largest_lx,
        line.largest_ly,
        line.largest_yaw_step,
        line.frames,
        multipliers.0.unwrap_or(f32::NAN),
        multipliers.1.unwrap_or(f32::NAN),
    ));
}

#[expect(
    unsafe_code,
    reason = "calls the game's own controller input blocked test"
)]
fn controller_input_is_blocked(controller: usize) -> bool {
    let address = INPUT_BLOCKED_ADDRESS.load(Ordering::Acquire);
    if address == 0 {
        return true;
    }
    // SAFETY: the address passed the first-bytes check; the function takes the controller it was
    // already handed by the camera step this frame and returns a flag in eax.
    unsafe { core::mem::transmute::<usize, InputBlocked>(address)(controller) != 0 }
}

fn write_raw_look(
    camera: usize,
    counts: (i32, i32),
    pitch_enabled: bool,
    yaw_enabled: bool,
) -> Option<(CameraAngles, CameraAngles)> {
    let (build, variant) = RAW_BUILD.get()?;
    let mut memory = InProcessMemory::new(build);
    let camera = camera as u64;
    let (angles, look) = read_look(&memory, camera, *variant, pitch_enabled, yaw_enabled)?;
    let moved = apply_raw_look(angles, counts, &look)?;
    let written = memory
        .write_f32(camera + CAMERA_PITCH_OFFSET, moved.pitch)
        .and_then(|()| memory.write_f32(camera + CAMERA_YAW_OFFSET, moved.yaw))
        .and_then(|()| memory.write_i32(camera + CAMERA_MOVED_BY_HAND_OFFSET, 1));
    if written.is_ok() && !FIRST_MOVE_LOGGED.swap(true, Ordering::Relaxed) {
        log_line("raw mouse: the camera moved from raw mouse movement");
    }
    written.is_ok().then_some((angles, moved))
}

fn read_look(
    memory: &InProcessMemory,
    camera: u64,
    variant: ExeVariant,
    pitch_enabled: bool,
    yaw_enabled: bool,
) -> Option<(CameraAngles, RawLook)> {
    let angles = CameraAngles {
        pitch: memory.read_f32(camera + CAMERA_PITCH_OFFSET).ok()?,
        yaw: memory.read_f32(camera + CAMERA_YAW_OFFSET).ok()?,
    };
    let look = RawLook {
        yaw_multiplier: sensitivity_multiplier_of(memory, variant, SENSITIVITY_X_VALUE, true)?,
        pitch_multiplier: sensitivity_multiplier_of(memory, variant, SENSITIVITY_Y_VALUE, false)?,
        invert_pitch: memory.read_i32(camera + CAMERA_INVERT_PITCH_OFFSET).ok()? != 0,
        invert_yaw: memory.read_i32(camera + CAMERA_INVERT_YAW_OFFSET).ok()? != 0,
        pitch_enabled,
        yaw_enabled,
        lower_limit: ModeLimit {
            value: memory
                .read_f32(camera + CAMERA_LOWER_PITCH_LIMIT_OFFSET)
                .ok()?,
            enable: memory
                .read_f32(camera + CAMERA_LOWER_PITCH_LIMIT_ENABLE_OFFSET)
                .ok()?,
        },
        upper_limit: ModeLimit {
            value: memory
                .read_f32(camera + CAMERA_UPPER_PITCH_LIMIT_OFFSET)
                .ok()?,
            enable: memory
                .read_f32(camera + CAMERA_UPPER_PITCH_LIMIT_ENABLE_OFFSET)
                .ok()?,
        },
    };
    Some((angles, look))
}

fn sensitivity_multiplier_of(
    memory: &InProcessMemory,
    variant: ExeVariant,
    value: GameAddress,
    horizontal: bool,
) -> Option<f32> {
    let at = |address: GameAddress| memory.module_base() + u64::from(address.rva(variant));
    let mut level = memory.read_i32(at(value)).ok()?;
    if horizontal {
        let flags = memory.read_i32(at(CAMERA_OPTION_FLAGS_WORD_VALUE)).ok()? as u32;
        level = x_sensitivity_value(level, flags);
    }
    Some(sensitivity_multiplier(level))
}
