use std::sync::{Mutex, PoisonError};

use replicant_game::Replicant;
use replicant_game::kaine::{OwnLifeKeeper, OwnLifeStep};
use replicant_game::readiness::{classify, read_play_checks};

use crate::build_gate::VerifiedBuild;
use crate::diagnostics::log_line;
use crate::game_hooks::set_local_health_through_game;
use crate::memory::InProcessMemory;
use crate::net::net_clock_ms;

static KEEPER: Mutex<OwnLifeKeeper> = Mutex::new(OwnLifeKeeper::new());

pub(super) fn fight_began(build: &VerifiedBuild) {
    let Ok(own_hp) = Replicant::new(InProcessMemory::new(build)).health() else {
        return;
    };
    let mut keeper = KEEPER.lock().unwrap_or_else(PoisonError::into_inner);
    keeper.fight_began(own_hp);
    log_line(&format!(
        "boss drive (guest): your own life is {own_hp}, kept to be put back when the fight is over"
    ));
}

pub(crate) fn note_boss_life_seen(hp: u32, max_hp: u32) {
    KEEPER
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .boss_life_seen(hp, max_hp);
}

pub(super) fn fight_over(link_lost: bool) {
    let mut keeper = KEEPER.lock().unwrap_or_else(PoisonError::into_inner);
    let kept = keeper.fight_over(net_clock_ms(), link_lost);
    log_line(&if kept {
        format!(
            "boss drive (guest): the fight is over here ({}), your own life is put back once you are in play",
            if link_lost {
                "the link is gone"
            } else {
                "she was beaten"
            }
        )
    } else {
        "boss drive (guest): the fight is over here without a win (defeat or retry), your life is the game's own".to_owned()
    });
}

pub(super) fn restore_if_due(build: &VerifiedBuild) {
    if !KEEPER
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .restore_is_pending()
    {
        return;
    }
    let mut game = Replicant::new(InProcessMemory::new(build));
    let (Ok(own_hp), Ok(own_max)) = (game.health(), game.max_health()) else {
        return;
    };
    let in_play = classify(read_play_checks(&game)).is_in_play();
    let step = KEEPER.lock().unwrap_or_else(PoisonError::into_inner).step(
        net_clock_ms(),
        in_play,
        own_hp,
        own_max,
    );
    let OwnLifeStep::Restore { to } = step else {
        return;
    };
    let written = set_local_health_through_game(to) || game.set_health(to).is_ok();
    log_line(&format!(
        "boss drive (guest): your own life is back, {own_hp} -> {to} of {own_max}{}",
        if written { "" } else { " (the write failed)" }
    ));
}
