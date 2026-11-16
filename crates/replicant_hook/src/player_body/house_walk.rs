use core::ffi::c_void;
use std::ffi::CString;
use std::sync::{LazyLock, Mutex, PoisonError};

use replicant_game::move_log::{
    BODY_SPEED_RATE_WRITE_TOLERANCE, FeetDrive, FeetDriveReading, FeetDriveTally, FeetSecond,
    GAMES_OWN_BODY_SPEED_RATE, INDOOR_FEET_PLAY_BLEND_SECONDS, RUN_LOOP_MOTION_NAME,
    SpeedCapSecond, SpeedCapWatch, format_feet_second, format_speed_cap_second, house_pace_in_play,
    is_house_speed_cap_value, read_body_speed_rate, read_feet, read_move_sample, read_speed_cap,
    read_state_rva, write_body_speed_rate,
};
use replicant_game::offsets::{
    BODY_MOTION_LAYER, MOTION_PLAY_MODE_LOOP, MOTION_WORK_PLAY_BY_NAME_FUNCTION_RVA,
    PLAYABLE_ACTOR_APPLY_MOTION_SPEED_RATES_FUNCTION_RVA, PLAYABLE_ACTOR_MOTION_WORK_OFFSET,
};
use replicant_game::puppet::motion_name;
use replicant_proto::PartnerBody;

use crate::build_gate::VerifiedBuild;
use crate::diagnostics::log_line;
use crate::memory::InProcessMemory;
use crate::net::net_clock_ms;
use crate::player_body::missing_motion_detour::last_missed_motion_on;
use crate::player_body::real_body;

static FEET_DRIVE: LazyLock<Mutex<(FeetDrive, SpeedCapWatch, FeetDriveTally)>> =
    LazyLock::new(Mutex::default);

type PlayByName = unsafe extern "system" fn(*mut c_void, *const u8, u32, f32, f32, i32, u8);
type ApplyMotionSpeedRates = unsafe extern "system" fn(*mut c_void);

pub fn drive_indoor_feet(
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
    let Some(cap) = read_speed_cap(&memory, actor) else {
        return;
    };
    let now_ms = net_clock_ms();
    let mut state = FEET_DRIVE.lock().unwrap_or_else(PoisonError::into_inner);
    let (drive, watch, tally) = &mut *state;
    let line_due = watch.due(now_ms, is_house_speed_cap_value(cap));
    let rate_now = read_body_speed_rate(&memory, actor);
    let ours_in_place = rate_now.is_some_and(|rate| {
        (rate - GAMES_OWN_BODY_SPEED_RATE).abs() > BODY_SPEED_RATE_WRITE_TOLERANCE
    });
    let mut reading = FeetDriveReading {
        kaine_swapped,
        state_rva: read_state_rva(&memory, actor),
        speed_cap: cap,
        event_scene,
        talk,
        control_masked,
        step_per_second: None,
        ground_speed: 0.0,
        body_motion: None,
    };
    if !house_pace_in_play(&reading) && !ours_in_place && !line_due {
        drive.tick(&reading);
        return;
    }
    let sample = read_move_sample(&memory, actor);
    let feet = read_feet(&memory, actor);
    let name = feet.and_then(|feet| motion_name(&memory, actor, feet.index).ok().flatten());
    reading.step_per_second = sample.applied_step_per_second;
    reading.ground_speed = sample.speed.unwrap_or(0.0);
    reading.body_motion = name.as_deref();
    let step = drive.tick(&reading);
    let rate_off = rate_now
        .is_some_and(|rate| (rate - step.body_speed_rate).abs() > BODY_SPEED_RATE_WRITE_TOLERANCE);
    if rate_off {
        if write_body_speed_rate(&mut memory, actor, step.body_speed_rate) {
            apply_motion_speed_rates(build, actor);
            tally.count_rate_write();
        } else {
            log_line("indoor feet: cannot write her body speed rate");
        }
    }
    // after the rates are applied, so the new slot takes the layer speed they set
    if step.play_run_loop {
        if play_run_loop(build, actor) {
            tally.count_play();
        } else {
            log_line("indoor feet: cannot play her run loop");
        }
    }
    if !line_due {
        return;
    }
    log_line(&format_speed_cap_second(&SpeedCapSecond {
        phase,
        swapped_body: kaine_swapped,
        cap: read_speed_cap(&memory, actor),
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
            body_speed_rate: read_body_speed_rate(&memory, actor),
            pace_per_second: drive.pace().average_step_per_second(),
            drive: tally.take(),
            last_missed: last_missed.as_deref(),
        }));
    }
}

fn apply_motion_speed_rates(build: &VerifiedBuild, actor: u64) {
    let function = (build.module_base()
        + u64::from(PLAYABLE_ACTOR_APPLY_MOTION_SPEED_RATES_FUNCTION_RVA))
        as usize;
    #[expect(
        unsafe_code,
        reason = "calls the playable actor's own rate apply at its verified address"
    )]
    // SAFETY: the RVA starts the apply (playable actor; sets its motion work's layer 2 and body
    // layer speeds to the rate products, 0x670A40, through MotionWork vslot 0x80) in the verified
    // build; the actor is the live player, read on the game thread.
    unsafe {
        core::mem::transmute::<usize, ApplyMotionSpeedRates>(function)(
            actor as usize as *mut c_void,
        );
    }
}

fn play_run_loop(build: &VerifiedBuild, actor: u64) -> bool {
    let motion_work = actor + PLAYABLE_ACTOR_MOTION_WORK_OFFSET;
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
