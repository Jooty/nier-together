use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, PoisonError};

use replicant_game::kaine::{
    BOSS_FIGHT_KAINE_MODE, BOSS_FIGHT_PACK, BossDrive, BossDriveEvent, BossDriveFacts, GuestSeat,
    KAINE_MODE_OFFSET, StandInSide, boss_drive_event_text,
};
use replicant_game::marker::read_world_flags;
use replicant_game::offsets::{
    PARTNER_ACTIVE_FLAG_OFFSET, PARTNER_KAINE_ACTOR_RVA, PARTNER_SCRIPT_MODE_OFFSET,
};
use replicant_game::{GameMemory, Replicant};

use crate::build_gate::VerifiedBuild;
use crate::diagnostics::log_line;
use crate::enemies::set_driven_boss;
use crate::kaine::stand_in::session_side;
use crate::memory::InProcessMemory;
use crate::net::{GhostWish, net_clock_ms};

static DRIVE: Mutex<BossDrive> = Mutex::new(BossDrive::new());
static BOSS_FIGHT_LIVE: AtomicBool = AtomicBool::new(false);
static DRIVING: AtomicBool = AtomicBool::new(false);
static GUEST_IN_BOSS_FIGHT: AtomicBool = AtomicBool::new(false);

/// Once per ghost tick, before anything that deals with her: reads whether the fight where she
/// is the boss is live here, who sits at the guest's seat, and switches her AI off or back on.
pub(crate) fn tick_boss_drive(build: &VerifiedBuild, wish: GhostWish, guest_is_watched: bool) {
    let memory = InProcessMemory::new(build);
    let her = memory.module_base() + u64::from(PARTNER_KAINE_ACTOR_RVA);
    let in_boss_fight_pack = Replicant::new(InProcessMemory::new(build))
        .current_phase()
        .is_ok_and(|pack| pack == BOSS_FIGHT_PACK);
    let side = session_side().map(|(side, _)| side);
    let on_the_host = side == Some(StandInSide::Host);
    let fight_here = in_boss_fight_pack
        && memory.read_i32(her + KAINE_MODE_OFFSET).ok() == Some(BOSS_FIGHT_KAINE_MODE);
    let guest_fight_here = fight_here && side == Some(StandInSide::Guest);
    if GUEST_IN_BOSS_FIGHT.swap(guest_fight_here, Ordering::Relaxed) != guest_fight_here {
        if guest_fight_here {
            log_line(
                "boss drive (guest): the fight is live here, your own AI Kaine stays off and out of your scan; your life follows hers on the host",
            );
            super::boss_own_life::fight_began(build);
        } else {
            super::boss_own_life::fight_over(side.is_none());
            crate::enemies::reset_boss_pvp();
        }
    }
    super::boss_own_life::restore_if_due(build);
    let facts = BossDriveFacts {
        in_boss_fight_pack: in_boss_fight_pack && on_the_host,
        her_mode: memory.read_i32(her + KAINE_MODE_OFFSET).ok(),
        her_active: memory
            .read_u8(her + PARTNER_ACTIVE_FLAG_OFFSET)
            .is_ok_and(|byte| byte != 0),
        guest: match (wish, guest_is_watched) {
            (GhostWish::Wanted, false) => GuestSeat::Playing,
            (GhostWish::Wanted, true) => GuestSeat::Watching,
            _ => GuestSeat::Absent,
        },
        game_poses_her: read_world_flags(&memory).is_ok_and(|flags| flags.event_scene)
            || memory
                .read_i32(her + PARTNER_SCRIPT_MODE_OFFSET)
                .is_ok_and(|mode| mode != 0),
        now_ms: net_clock_ms(),
    };
    let mut drive = DRIVE.lock().unwrap_or_else(PoisonError::into_inner);
    let events = drive.step(&facts);
    BOSS_FIGHT_LIVE.store(drive.boss_fight_is_live(), Ordering::Relaxed);
    let driving = drive.is_driving();
    drop(drive);
    DRIVING.store(driving, Ordering::Relaxed);
    set_driven_boss(driving.then_some(her));
    for event in events {
        log_line(&boss_drive_event_text(event));
        if matches!(event, BossDriveEvent::FightEnded { .. }) {
            crate::enemies::reset_boss_pvp();
        }
    }
    super::boss_phase::tick(
        build,
        on_the_host && BOSS_FIGHT_LIVE.load(Ordering::Relaxed),
    );
    if on_the_host {
        crate::enemies::apply_guest_boss_hits(build);
    }
}

pub(crate) fn guest_in_boss_fight() -> bool {
    GUEST_IN_BOSS_FIGHT.load(Ordering::Relaxed)
}

/// Her mode is 2 in the fight's pack: she is the boss, with a guest or without.
pub(crate) fn boss_fight_is_live() -> bool {
    BOSS_FIGHT_LIVE.load(Ordering::Relaxed)
}

pub(crate) fn boss_drive_is_on() -> bool {
    DRIVING.load(Ordering::Relaxed)
}
