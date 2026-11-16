use std::sync::{Mutex, PoisonError};

use replicant_game::Replicant;
use replicant_game::downed::BACKUP_SAVE_DATA_RVA;
use replicant_game::story_jump::restore_point_is_due;

use crate::build_gate::VerifiedBuild;
use crate::diagnostics::log_line;
use crate::memory::InProcessMemory;
use crate::net::net_clock_ms;

const RESTORE_POINT_GIVE_UP_MS: u64 = 60_000;

static WAITING: Mutex<Option<Waiting>> = Mutex::new(None);

struct Waiting {
    phase_at_apply: String,
    applied_ms: u64,
    phase_changed_ms: Option<u64>,
    field_was_left: bool,
}

type BackupSaveData = unsafe extern "system" fn();

pub(super) fn expect_restore_point(build: &VerifiedBuild) {
    let game = Replicant::new(InProcessMemory::new(build));
    *WAITING.lock().unwrap_or_else(PoisonError::into_inner) = Some(Waiting {
        phase_at_apply: game.current_phase().unwrap_or_default(),
        applied_ms: net_clock_ms(),
        phase_changed_ms: None,
        field_was_left: false,
    });
}

pub(super) fn tick_restore_point(build: &VerifiedBuild) {
    let mut guard = WAITING.lock().unwrap_or_else(PoisonError::into_inner);
    let Some(waiting) = guard.as_mut() else {
        return;
    };
    let now = net_clock_ms();
    if now.saturating_sub(waiting.applied_ms) > RESTORE_POINT_GIVE_UP_MS {
        log_line("story jump: restore point not made, the map never reloaded");
        *guard = None;
        return;
    }
    let game = Replicant::new(InProcessMemory::new(build));
    let phase_now = game.current_phase().unwrap_or_default();
    let field_ready = game.map_manager_is_active().unwrap_or(false);
    if !field_ready {
        waiting.field_was_left = true;
        waiting.phase_changed_ms = None;
        return;
    }
    if phase_now == waiting.phase_at_apply && !waiting.field_was_left {
        return;
    }
    let changed_ms = *waiting.phase_changed_ms.get_or_insert(now);
    if !restore_point_is_due(
        &waiting.phase_at_apply,
        &phase_now,
        waiting.field_was_left,
        field_ready,
        now.saturating_sub(changed_ms),
    ) {
        return;
    }
    *guard = None;
    drop(guard);
    make_restore_point(build);
    log_line(&format!("story jump: restore point made ({phase_now})"));
}

fn make_restore_point(build: &VerifiedBuild) {
    let backup = (build.module_base() + u64::from(BACKUP_SAVE_DATA_RVA)) as usize;
    #[expect(
        unsafe_code,
        reason = "calls the game's save block backup, the body of the _BackupSaveData native"
    )]
    // SAFETY: the RVA starts SaveBlock::BackupToShadow (no arguments) in the verified build; it only
    // copies the save block to its shadow in memory (docs off-hand/story-events.md). Calling the
    // hooked entry runs the rewind detour, so the host also tells the guest. Runs on the game thread.
    unsafe {
        core::mem::transmute::<usize, BackupSaveData>(backup)();
    }
}
