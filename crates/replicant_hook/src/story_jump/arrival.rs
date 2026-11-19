use std::sync::{Mutex, PoisonError};

use replicant_game::story_jump::{
    ArrivalSpot, LEVEL_CHECK_SETTLE_MS, PLACEMENT_READ_BACK_AFTER_MS,
    PLACEMENTS_REPEATED_AFTER_AN_ENTRY_SCRIPT, ReadBackStep, SET_PLAYER_POSITION_NATIVE_RVA,
    arrival_is_due, arrival_spot, read_back_step,
};
use replicant_game::{Position, Replicant};
use replicant_proto::StoryJumpPoint;

use crate::build_gate::VerifiedBuild;
use crate::diagnostics::log_line;
use crate::memory::InProcessMemory;
use crate::net::net_clock_ms;
use crate::stay_together::game_says_free_to_act;
use crate::story_jump::natives::{NativeArg, call_native_with};

const ARRIVAL_GIVE_UP_MS: u64 = 60_000;

static WAITING: Mutex<Option<Waiting>> = Mutex::new(None);

struct Waiting {
    spot: ArrivalSpot,
    phase_at_apply: String,
    applied_ms: u64,
    phase_changed_ms: Option<u64>,
    field_was_left: bool,
    placed_ms: Option<u64>,
    placements_left: u8,
}

pub(super) fn expect_arrival(build: &VerifiedBuild, point: StoryJumpPoint, host: bool) {
    let Some(spot) = arrival_spot(point, host) else {
        return;
    };
    let game = Replicant::new(InProcessMemory::new(build));
    *WAITING.lock().unwrap_or_else(PoisonError::into_inner) = Some(Waiting {
        spot,
        phase_at_apply: game.current_phase().unwrap_or_default(),
        applied_ms: net_clock_ms(),
        phase_changed_ms: None,
        field_was_left: false,
        placed_ms: None,
        placements_left: PLACEMENTS_REPEATED_AFTER_AN_ENTRY_SCRIPT,
    });
}

pub(super) fn tick_arrival(build: &VerifiedBuild) {
    let mut guard = WAITING.lock().unwrap_or_else(PoisonError::into_inner);
    let Some(waiting) = guard.as_mut() else {
        return;
    };
    let now = net_clock_ms();
    let game = Replicant::new(InProcessMemory::new(build));
    if let Some(placed_ms) = waiting.placed_ms {
        if now.saturating_sub(placed_ms) < PLACEMENT_READ_BACK_AFTER_MS {
            return;
        }
        let spot = waiting.spot;
        let Ok(here) = game.position() else {
            *guard = None;
            drop(guard);
            log_line("story jump: the player position is unreadable after placing it");
            return;
        };
        let distance = (here.x - spot.x).hypot(here.z - spot.z);
        let free_to_act = game_says_free_to_act(build).unwrap_or(false);
        let step = read_back_step(
            distance,
            free_to_act,
            waiting.placements_left,
            now.saturating_sub(placed_ms),
        );
        match step {
            ReadBackStep::WaitForControl => {}
            ReadBackStep::PlaceAgain => {
                waiting.placements_left -= 1;
                waiting.placed_ms = Some(now);
                drop(guard);
                log_line(&format!(
                    "story jump: the floor's own script moved the player to {:.1}, {:.1}, {:.1}, placing again now that it is free",
                    here.x, here.y, here.z
                ));
                place(build, &game, spot, &phase_name(&game));
            }
            ReadBackStep::Arrived | ReadBackStep::DidNotArrive => {
                *guard = None;
                drop(guard);
                log_read_back(here, spot, step == ReadBackStep::Arrived);
            }
        }
        return;
    }
    if now.saturating_sub(waiting.applied_ms) > ARRIVAL_GIVE_UP_MS {
        log_line("story jump: placing the player gave up, the map never reloaded");
        *guard = None;
        return;
    }
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
    if !arrival_is_due(
        &waiting.phase_at_apply,
        &phase_now,
        waiting.field_was_left,
        field_ready,
        now.saturating_sub(changed_ms),
    ) {
        return;
    }
    waiting.placed_ms = Some(now);
    let spot = waiting.spot;
    drop(guard);
    place(build, &game, spot, &phase_now);
}

fn place(build: &VerifiedBuild, game: &Replicant<InProcessMemory>, spot: ArrivalSpot, phase: &str) {
    let heading_degrees = game.yaw_radians().map_or(0.0, f32::to_degrees);
    call_native_with(
        build,
        SET_PLAYER_POSITION_NATIVE_RVA,
        &[
            NativeArg::Real(f64::from(spot.x)),
            NativeArg::Real(f64::from(spot.y)),
            NativeArg::Real(f64::from(spot.z)),
            NativeArg::Real(f64::from(heading_degrees)),
        ],
    );
    log_line(&format!(
        "story jump: placed the player in {phase} with _SetPlayerPosition({:.1}, {:.1}, {:.1}, heading {heading_degrees:.0}) after {LEVEL_CHECK_SETTLE_MS} ms settle",
        spot.x, spot.y, spot.z
    ));
}

fn phase_name(game: &Replicant<InProcessMemory>) -> String {
    game.current_phase().unwrap_or_default()
}

fn log_read_back(here: Position, spot: ArrivalSpot, arrived: bool) {
    let verdict = if arrived { "arrived" } else { "did not arrive" };
    log_line(&format!(
        "story jump: {verdict} (now at {:.1}, {:.1}, {:.1}, wanted {:.1}, {:.1}, {:.1})",
        here.x, here.y, here.z, spot.x, spot.y, spot.z
    ));
}
