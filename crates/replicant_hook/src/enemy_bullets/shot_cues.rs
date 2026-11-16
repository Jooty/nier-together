//! The host's side: the effects and sounds a shared enemy's bullet script or bullet start makes,
//! caught as the game makes them and read back once the step that made them returns.

use core::cell::RefCell;
use std::collections::HashSet;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{LazyLock, Mutex, OnceLock, PoisonError};

use replicant_game::GameMemory;
use replicant_game::effects::{effect_instance_address, read_effect_transform};
use replicant_game::enemy_bullets::{
    BulletStartMade, bullet_start_cue_kind, effect_cue_of_command, end_cue_of_command,
    sound_cue_of_command,
};
use replicant_game::offsets::{
    ACTOR_TRANSFORM_BLOCK_BYTES, ACTOR_TRANSFORM_BLOCK_OFFSET, EFFECT_INSTANCE_STYLE_OFFSET,
};
use replicant_proto::{
    BULLET_STYLE_FLOATS, BULLET_TRANSFORM_FLOATS, SHOT_SOUND_LEVELS, ShotCueKind,
};

use crate::build_gate::VerifiedBuild;
use crate::diagnostics::log_line;
use crate::enemy_bullets::bullet_start::bullet_start_in_flight;
use crate::enemy_bullets::script_commands::{bullet_is_shared, command_in_flight};
use crate::memory::InProcessMemory;

const MAX_CAPTURED_SHOT_CUES: usize = 512;
const MAX_FIRST_BULLET_START_CUES_LOGGED: usize = 64;
const FLOAT_BYTES: usize = 4;
const STYLE_BYTES: usize = BULLET_STYLE_FLOATS * FLOAT_BYTES;

static HOOKED_BUILD: OnceLock<VerifiedBuild> = OnceLock::new();
static CAPTURED: Mutex<Vec<CapturedCue>> = Mutex::new(Vec::new());
static OVERFLOWED: AtomicU32 = AtomicU32::new(0);
static FIRST_BULLET_START_CUES: LazyLock<Mutex<HashSet<(ShotCueKind, u32)>>> =
    LazyLock::new(Mutex::default);

thread_local! {
    static MADE_IN_COMMAND: RefCell<Vec<Made>> = const { RefCell::new(Vec::new()) };
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum Made {
    Effect {
        id: u32,
        handle: u32,
    },
    Sound {
        id: u32,
        position: [f32; 3],
        levels: [f32; SHOT_SOUND_LEVELS],
    },
}

/// One effect or sound a shared enemy's bullet made, with where the bullet was when it did.
#[derive(Debug, Clone, Copy)]
pub(crate) struct CapturedCue {
    pub bullet: u64,
    pub kind: ShotCueKind,
    pub id: u32,
    pub transform: [f32; BULLET_TRANSFORM_FLOATS],
    pub offset: [f32; 3],
    pub style: [f32; BULLET_STYLE_FLOATS],
    pub sound_levels: [f32; SHOT_SOUND_LEVELS],
}

pub(crate) fn install(build: VerifiedBuild) {
    let _ = HOOKED_BUILD.set(build);
}

/// From the create-effect-by-id detour, after the game made effect `id` into `handle`.
pub(crate) fn note_effect_made(id: u32, handle: u32) {
    if !bullet_start_in_flight() {
        let Some((_, command)) = command_in_flight() else {
            return;
        };
        if effect_cue_of_command(command).is_none() {
            return;
        }
    }
    MADE_IN_COMMAND.with(|made| made.borrow_mut().push(Made::Effect { id, handle }));
}

/// From the play-sound-by-id detour, with the sound's position and its two level arguments.
pub(crate) fn note_sound_made(id: u32, position: [f32; 3], levels: [f32; SHOT_SOUND_LEVELS]) {
    if command_in_flight().is_none() && !bullet_start_in_flight() {
        return;
    }
    MADE_IN_COMMAND.with(|made| {
        made.borrow_mut().push(Made::Sound {
            id,
            position,
            levels,
        });
    });
}

pub(super) fn begin_command() -> Vec<Made> {
    MADE_IN_COMMAND.with(RefCell::take)
}

pub(super) fn restore_outer_command(outer: Vec<Made>) {
    MADE_IN_COMMAND.with(|made| *made.borrow_mut() = outer);
}

pub(super) fn end_command(bullet: u64, command: i32) {
    let made = MADE_IN_COMMAND.with(RefCell::take);
    let end = end_cue_of_command(command);
    if made.is_empty() && end.is_none() {
        return;
    }
    if !bullet_is_shared(bullet) {
        return;
    }
    let Some(build) = HOOKED_BUILD.get() else {
        return;
    };
    let memory = InProcessMemory::new(build);
    let Some(bullet_transform) = read_bullet_transform(&memory, bullet) else {
        return;
    };
    let mut cues: Vec<CapturedCue> = made
        .into_iter()
        .filter_map(|made| {
            let kind = match made {
                Made::Effect { .. } => effect_cue_of_command(command)?,
                Made::Sound { .. } => sound_cue_of_command(command),
            };
            Some(cue_of(&memory, bullet, kind, &bullet_transform, made))
        })
        .collect();
    if let Some(kind) = end {
        cues.push(CapturedCue {
            bullet,
            kind,
            id: 0,
            transform: bullet_transform,
            offset: [0.0; 3],
            style: [0.0; BULLET_STYLE_FLOATS],
            sound_levels: [0.0; SHOT_SOUND_LEVELS],
        });
    }
    capture(cues);
}

/// After an ordinary bullet's create-effects step: its start effect and sounds become shot cues
/// when a shared enemy fired it; its slot effects are left to the flights.
pub(super) fn end_bullet_start(bullet: u64) {
    let made = MADE_IN_COMMAND.with(RefCell::take);
    if made.is_empty() || !bullet_is_shared(bullet) {
        return;
    }
    let Some(build) = HOOKED_BUILD.get() else {
        return;
    };
    let memory = InProcessMemory::new(build);
    let Some(bullet_transform) = read_bullet_transform(&memory, bullet) else {
        return;
    };
    let cues: Vec<CapturedCue> = made
        .into_iter()
        .filter_map(|made| {
            let start_made = match made {
                Made::Effect { handle, .. } => BulletStartMade::Effect { handle },
                Made::Sound { id, .. } => BulletStartMade::Sound { id },
            };
            let kind = bullet_start_cue_kind(&memory, bullet, start_made)?;
            Some(cue_of(&memory, bullet, kind, &bullet_transform, made))
        })
        .collect();
    for cue in &cues {
        log_first_bullet_start_cue(cue);
    }
    capture(cues);
}

/// The cues caught since the last call, and how many did not fit.
pub(crate) fn take_captured() -> (Vec<CapturedCue>, u32) {
    let cues = core::mem::take(&mut *CAPTURED.lock().unwrap_or_else(PoisonError::into_inner));
    (cues, OVERFLOWED.swap(0, Ordering::Relaxed))
}

fn capture(cues: Vec<CapturedCue>) {
    let mut captured = CAPTURED.lock().unwrap_or_else(PoisonError::into_inner);
    for cue in cues {
        if captured.len() >= MAX_CAPTURED_SHOT_CUES {
            OVERFLOWED.fetch_add(1, Ordering::Relaxed);
            continue;
        }
        captured.push(cue);
    }
}

fn log_first_bullet_start_cue(cue: &CapturedCue) {
    let mut first = FIRST_BULLET_START_CUES
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    if first.len() >= MAX_FIRST_BULLET_START_CUES_LOGGED || !first.insert((cue.kind, cue.id)) {
        return;
    }
    drop(first);
    log_line(&format!(
        "enemy bullets: a shared enemy's ordinary bullet made {:?} {} as it started, sent as a shot cue at [{:.1}, {:.1}, {:.1}] (style {:?})",
        cue.kind, cue.id, cue.transform[3], cue.transform[7], cue.transform[11], cue.style
    ));
}

fn read_bullet_transform(
    memory: &InProcessMemory,
    bullet: u64,
) -> Option<[f32; BULLET_TRANSFORM_FLOATS]> {
    memory
        .read_array::<ACTOR_TRANSFORM_BLOCK_BYTES>(bullet + ACTOR_TRANSFORM_BLOCK_OFFSET)
        .ok()
        .map(|bytes| floats_of::<BULLET_TRANSFORM_FLOATS>(&bytes))
}

fn cue_of(
    memory: &InProcessMemory,
    bullet: u64,
    kind: ShotCueKind,
    bullet_transform: &[f32; BULLET_TRANSFORM_FLOATS],
    made: Made,
) -> CapturedCue {
    let (id, transform, style, sound_levels) = match made {
        Made::Effect { id, handle } => {
            let transform = read_effect_transform(memory, handle)
                .ok()
                .flatten()
                .unwrap_or(*bullet_transform);
            let style = effect_instance_address(memory, handle)
                .ok()
                .flatten()
                .and_then(|instance| {
                    memory
                        .read_array::<STYLE_BYTES>(instance + EFFECT_INSTANCE_STYLE_OFFSET)
                        .ok()
                })
                .map_or([0.0; BULLET_STYLE_FLOATS], |bytes| floats_of(&bytes));
            (id, transform, style, [0.0; SHOT_SOUND_LEVELS])
        }
        Made::Sound {
            id,
            position,
            levels,
        } => {
            let transform = [
                1.0,
                0.0,
                0.0,
                position[0],
                0.0,
                1.0,
                0.0,
                position[1],
                0.0,
                0.0,
                1.0,
                position[2],
            ];
            let style = [0.0; BULLET_STYLE_FLOATS];
            (id, transform, style, levels)
        }
    };
    let offset = [
        transform[3] - bullet_transform[3],
        transform[7] - bullet_transform[7],
        transform[11] - bullet_transform[11],
    ];
    CapturedCue {
        bullet,
        kind,
        id,
        transform,
        offset,
        style,
        sound_levels,
    }
}

fn floats_of<const FLOATS: usize>(bytes: &[u8]) -> [f32; FLOATS] {
    core::array::from_fn(|index| {
        let at = index * FLOAT_BYTES;
        f32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]])
    })
}
