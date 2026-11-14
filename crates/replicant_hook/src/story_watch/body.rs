use std::time::Instant;

use replicant_game::downed::{
    PLAYER_CONTROL_DISABLE_NATIVE_RVAS, PLAYER_CONTROL_ENABLE_NATIVE_RVAS,
};
use replicant_game::offsets::{ACTOR_DRAWN_FLAG_OFFSET, LIFE_FLAG_INVULNERABLE_BIT};
use replicant_game::story_watch::{PlaceStep, place_step};
use replicant_game::{GameMemory, Position, Replicant};

use crate::build_gate::VerifiedBuild;
use crate::diagnostics::log_line;
use crate::memory::InProcessMemory;
use crate::story_jump::call_native_with;
use crate::teleport::{move_beside, move_player_with_native};

pub(super) struct WatchedBody {
    invulnerable_was: Option<bool>,
    last_teleport: Option<Instant>,
}

pub(super) enum Placed {
    Left,
    Written,
    Teleported,
    NoBody,
}

impl WatchedBody {
    pub(super) const fn new() -> Self {
        Self {
            invulnerable_was: None,
            last_teleport: None,
        }
    }

    pub(super) fn hide_and_protect(&mut self, build: &VerifiedBuild) {
        let mut game = Replicant::new(InProcessMemory::new(build));
        let Ok(actor) = game.playable_actor() else {
            return;
        };
        if self.invulnerable_was.is_none()
            && let Ok(was) = game.life_flag(LIFE_FLAG_INVULNERABLE_BIT)
        {
            self.invulnerable_was = Some(was);
        }
        let _ = game.set_life_flag(LIFE_FLAG_INVULNERABLE_BIT, true);
        let _ = game
            .memory_mut()
            .write_u8(actor + ACTOR_DRAWN_FLAG_OFFSET, 0);
    }

    pub(super) fn lock_controls(&self, build: &VerifiedBuild) {
        for rva in PLAYER_CONTROL_DISABLE_NATIVE_RVAS {
            call_native_with(build, rva, &[]);
        }
    }

    pub(super) fn give_back(&mut self, build: &VerifiedBuild, host: Option<[f32; 3]>) {
        let mut game = Replicant::new(InProcessMemory::new(build));
        if let Ok(actor) = game.playable_actor() {
            let _ = game
                .memory_mut()
                .write_u8(actor + ACTOR_DRAWN_FLAG_OFFSET, 1);
        }
        if let Some(was) = self.invulnerable_was.take() {
            let _ = game.set_life_flag(LIFE_FLAG_INVULNERABLE_BIT, was);
        }
        for rva in PLAYER_CONTROL_ENABLE_NATIVE_RVAS {
            call_native_with(build, rva, &[]);
        }
        if let Some(spot) = host {
            match move_beside(build, spot) {
                Ok(()) => log_line(
                    "watching: this game's body was moved to a safe spot by the host's body",
                ),
                Err(error) => log_line(&format!(
                    "watching: cannot move this game's body to the host ({error})"
                )),
            }
        }
        self.last_teleport = None;
    }

    pub(super) fn place_at(
        &mut self,
        build: &VerifiedBuild,
        host: [f32; 3],
    ) -> (Placed, f32, [f32; 3]) {
        let mut game = Replicant::new(InProcessMemory::new(build));
        let Ok(own) = game.position() else {
            return (Placed::NoBody, 0.0, [0.0; 3]);
        };
        let spot = Position {
            x: host[0],
            y: host[1],
            z: host[2],
        };
        let error =
            ((own.x - spot.x).powi(2) + (own.y - spot.y).powi(2) + (own.z - spot.z).powi(2)).sqrt();
        let since_ms = self
            .last_teleport
            .map_or(u64::MAX, |at| at.elapsed().as_millis() as u64);
        let own = [own.x, own.y, own.z];
        match place_step(error, since_ms) {
            PlaceStep::Leave => (Placed::Left, error, own),
            PlaceStep::Write => {
                let placed = game.set_position(spot).is_ok();
                (
                    if placed {
                        Placed::Written
                    } else {
                        Placed::NoBody
                    },
                    error,
                    own,
                )
            }
            PlaceStep::Teleport => {
                move_player_with_native(build, &game, spot);
                self.last_teleport = Some(Instant::now());
                (Placed::Teleported, error, own)
            }
        }
    }
}
