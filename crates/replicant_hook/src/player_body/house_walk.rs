use core::ffi::c_void;
use std::ffi::CString;
use std::sync::{LazyLock, Mutex, PoisonError};

use replicant_game::GameMemory;
use replicant_game::move_log::{
    FeetDriveReading, FeetDriveStep, FeetDriveTally, FeetSecond, INDOOR_FEET_PLAY_BLEND_SECONDS,
    RUN_LOOP_MOTION_NAME, SPEED_CAP_LIFTED_VALUE, SpeedCapLift, SpeedCapReading, SpeedCapSecond,
    SpeedCapWatch, feet_drive_step, format_feet_second, format_speed_cap_log,
    format_speed_cap_second, house_pace_in_play, is_house_speed_cap_value, read_feet,
    read_move_sample, read_speed_cap, read_state_rva, write_feet_speed, write_lifted_speed_cap,
};
use replicant_game::offsets::{
    BODY_MOTION_LAYER, MOTION_PLAY_MODE_LOOP, MOTION_WORK_LAYER_SPEED_STRIDE,
    MOTION_WORK_LAYER_SPEEDS_OFFSET, MOTION_WORK_PLAY_BY_NAME_FUNCTION_RVA,
    PLAYABLE_ACTOR_MOTION_WORK_OFFSET,
};
use replicant_game::puppet::motion_name;
use replicant_proto::PartnerBody;

use crate::build_gate::VerifiedBuild;
use crate::diagnostics::log_line;
use crate::memory::InProcessMemory;
use crate::net::net_clock_ms;
use crate::player_body::missing_motion_detour::last_missed_motion_on;
use crate::player_body::real_body;

static SPEED_CAP_LIFT: LazyLock<Mutex<(SpeedCapLift, SpeedCapWatch, FeetDriveTally)>> =
    LazyLock::new(Mutex::default);

type PlayByName = unsafe extern "system" fn(*mut c_void, *const u8, u32, f32, f32, i32, u8);

pub fn lift_house_speed_cap(
    build: &VerifiedBuild,
    actor: u64,
    event_scene: bool,
    talk: bool,
    control_masked: bool,
) {
    let real = real_body();
    let kaine_swapped =
        real.is_some_and(|real| real.by_swap && real.body == PartnerBody::SeedKaineE);
    let phase = real.map_or(0, |real| real.phase_loads);
    let mut memory = InProcessMemory::new(build);
    let Some(value) = read_speed_cap(&memory, actor) else {
        return;
    };
    let now_ms = net_clock_ms();
    let mut state = SPEED_CAP_LIFT
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    let (lift, watch, tally) = &mut *state;
    let state_rva = read_state_rva(&memory, actor);
    let step = lift.observe(SpeedCapReading {
        phase,
        now_ms,
        value,
        kaine_swapped,
        event_scene,
        talk,
        control_masked,
        state_rva,
    });
    if step.write && !write_lifted_speed_cap(&mut memory, actor) {
        log_line("kaine walk: cannot write the speed cap");
    }
    if let Some(log) = step.log {
        log_line(&format_speed_cap_log(log, phase));
    }
    let house_cap_seen =
        is_house_speed_cap_value(value) || (value - SPEED_CAP_LIFTED_VALUE).abs() < f32::EPSILON;
    let mut reading = FeetDriveReading {
        kaine_swapped,
        state_rva,
        speed_cap: value,
        lifts_this_phase: lift.lifts_this_phase(),
        event_scene,
        talk,
        control_masked,
        step_per_second: None,
        ground_speed: 0.0,
        body_motion: None,
        body_motion_speed: None,
    };
    let line_due = watch.due(now_ms, house_cap_seen);
    if !house_pace_in_play(&reading) && !line_due {
        return;
    }
    let sample = read_move_sample(&memory, actor);
    let feet = read_feet(&memory, actor);
    let name = feet.and_then(|feet| motion_name(&memory, actor, feet.index).ok().flatten());
    reading.step_per_second = sample.applied_step_per_second;
    reading.ground_speed = sample.speed.unwrap_or(0.0);
    reading.body_motion = name.as_deref();
    reading.body_motion_speed = feet.map(|feet| feet.speed);
    let drive = feet_drive_step(&reading);
    match drive {
        FeetDriveStep::PlayRunLoop { speed } => {
            if !play_run_loop(build, &mut memory, actor, speed) {
                log_line("indoor feet: cannot play her run loop");
            }
        }
        FeetDriveStep::HoldSpeed { speed } => {
            if !write_feet_speed(&mut memory, actor, speed) {
                log_line("indoor feet: cannot write her motion speed");
            }
        }
        FeetDriveStep::Leave => {}
    }
    tally.count(drive);
    if !line_due {
        return;
    }
    log_line(&format_speed_cap_second(&SpeedCapSecond {
        phase,
        swapped_body: kaine_swapped,
        cap: read_speed_cap(&memory, actor),
        lifts_this_phase: lift.lifts_this_phase(),
        step_per_second: sample.applied_step_per_second,
        ground_speed: sample.speed,
        state_rva: sample.state_rva,
    }));
    if kaine_swapped {
        let last_missed = last_missed_motion_on(actor + PLAYABLE_ACTOR_MOTION_WORK_OFFSET);
        // read again after the drive: the line names what her body plays now, not what it played
        let feet_now = read_feet(&memory, actor);
        let name_now =
            feet_now.and_then(|feet| motion_name(&memory, actor, feet.index).ok().flatten());
        log_line(&format_feet_second(&FeetSecond {
            name: name_now.as_deref(),
            reading: feet_now,
            step_per_second: sample.applied_step_per_second,
            drive: tally.take(),
            last_missed: last_missed.as_deref(),
        }));
    }
}

fn play_run_loop(
    build: &VerifiedBuild,
    memory: &mut InProcessMemory,
    actor: u64,
    speed: f32,
) -> bool {
    let motion_work = actor + PLAYABLE_ACTOR_MOTION_WORK_OFFSET;
    // the play copies the layer's speed into the new slot, so the speed goes in first
    let layer_speed = motion_work
        + MOTION_WORK_LAYER_SPEEDS_OFFSET
        + u64::from(BODY_MOTION_LAYER) * MOTION_WORK_LAYER_SPEED_STRIDE;
    if memory.write_f32(layer_speed, speed).is_err() {
        return false;
    }
    let Ok(name) = CString::new(RUN_LOOP_MOTION_NAME) else {
        return false;
    };
    let function =
        (build.module_base() + u64::from(MOTION_WORK_PLAY_BY_NAME_FUNCTION_RVA)) as usize;
    #[expect(
        unsafe_code,
        reason = "calls MotionWork::PlayByName at its verified address on her own body"
    )]
    // SAFETY: the RVA starts PlayByName (motion work, NUL-terminated name, layer, blend seconds,
    // start frame, mode, sync start) in the verified build; the motion work is the live player's
    // own, read on the game thread, and the name outlives the call.
    unsafe {
        core::mem::transmute::<usize, PlayByName>(function)(
            motion_work as usize as *mut c_void,
            name.as_ptr().cast(),
            BODY_MOTION_LAYER,
            INDOOR_FEET_PLAY_BLEND_SECONDS,
            0.0,
            MOTION_PLAY_MODE_LOOP,
            0,
        );
    }
    true
}
