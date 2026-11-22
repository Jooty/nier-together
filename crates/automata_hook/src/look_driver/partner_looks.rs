use std::collections::BTreeMap;
use std::sync::{Mutex, PoisonError};

use automata_game::character::Character;
use automata_game::outfit_look::{
    ApplyGuard, HairSwap, LookKind, PartnerLookApplied, PartnerLookStep, PlayerLook, look_slot_of,
    partner_hair_swap, partner_look_steps,
};
use automata_game::partner::BODY_MODEL_ID_OFFSET;
use nt_hook::diagnostics::log_line;

use crate::extra_bodies::body_of_player;
use crate::game_calls::GameCalls;
use crate::look_driver::net_reading::other_players_looks;

static HAIR_SWAPS: Mutex<Option<HairSwaps>> = Mutex::new(None);

struct HairSwaps {
    calls: GameCalls,
    by_body: Vec<(usize, HairSwap)>,
}

struct PartnerBody {
    body: usize,
    applied: PartnerLookApplied,
    guard: ApplyGuard,
    gave_up_logged: bool,
}

#[derive(Default)]
pub(super) struct PartnerLooks {
    bodies: BTreeMap<u8, PartnerBody>,
}

impl PartnerBody {
    fn new(body: usize) -> Self {
        Self {
            body,
            applied: PartnerLookApplied::default(),
            guard: ApplyGuard::default(),
            gave_up_logged: false,
        }
    }
}

impl PartnerLooks {
    pub fn tick(&mut self, calls: &GameCalls, allowed: bool, tick: u64) {
        let mut looks: BTreeMap<u8, PlayerLook> = self
            .bodies
            .keys()
            .map(|slot| (*slot, PlayerLook::UNPICKED))
            .collect();
        if allowed {
            for (slot, look) in other_players_looks() {
                if look.is_valid() {
                    looks.insert(slot, look);
                }
            }
        }
        let mut swaps = Vec::new();
        for (slot, look) in looks {
            if let Some(swap) = self.tick_one(calls, slot, look, tick) {
                swaps.push(swap);
            }
        }
        publish_hair_swaps(calls, swaps);
    }

    pub fn clear(&mut self) {
        self.bodies.clear();
        clear_hair_swaps();
    }

    fn tick_one(
        &mut self,
        calls: &GameCalls,
        slot: u8,
        look: PlayerLook,
        tick: u64,
    ) -> Option<(usize, HairSwap)> {
        let Some((body, character)) = live_body_of(calls, slot) else {
            self.bodies.remove(&slot);
            return None;
        };
        let look_slot = look_slot_of(character);
        let entry = self
            .bodies
            .entry(slot)
            .or_insert_with(|| PartnerBody::new(body));
        if entry.body != body {
            *entry = PartnerBody::new(body);
        }
        let reading = calls.read_partner_look(body, look_slot)?;
        for step in partner_look_steps(&look, &reading, &entry.applied) {
            run_step(calls, entry, body, look_slot, step, tick);
        }
        partner_hair_swap(&look, look_slot).map(|swap| (body, swap))
    }
}

fn run_step(
    calls: &GameCalls,
    entry: &mut PartnerBody,
    body: usize,
    look_slot: usize,
    step: PartnerLookStep,
    tick: u64,
) {
    let allowed = match step {
        PartnerLookStep::WriteOutfit(value) => entry.guard.allow(LookKind::Outfit, value, tick),
        PartnerLookStep::WriteAccessory(value) => {
            entry.guard.allow(LookKind::Accessory, value, tick)
        }
        PartnerLookStep::SwapPodSkin { .. } => true,
    };
    if !allowed {
        let gave_up = match step {
            PartnerLookStep::WriteOutfit(value) => entry.guard.gave_up(LookKind::Outfit, value),
            PartnerLookStep::WriteAccessory(value) => {
                entry.guard.gave_up(LookKind::Accessory, value)
            }
            PartnerLookStep::SwapPodSkin { .. } => false,
        };
        if gave_up && !entry.gave_up_logged {
            entry.gave_up_logged = true;
            log_line(&format!(
                "looks: the partner body did not keep {step:?}; it is not tried again for this body"
            ));
        }
        return;
    }
    if !calls.apply_partner_look_step(body, look_slot, step) {
        log_line(&format!(
            "looks: {step:?} was not written on the partner body"
        ));
    }
    entry.applied.note(step);
}

fn live_body_of(calls: &GameCalls, slot: u8) -> Option<(usize, Character)> {
    let body = body_of_player(slot)?;
    let model = calls
        .read_u32_at(calls.body_field(body, BODY_MODEL_ID_OFFSET))
        .ok()?;
    Some((body, Character::from_model_id(model)?))
}

fn publish_hair_swaps(calls: &GameCalls, by_body: Vec<(usize, HairSwap)>) {
    let mut guard = HAIR_SWAPS.lock().unwrap_or_else(PoisonError::into_inner);
    *guard = Some(HairSwaps {
        calls: *calls,
        by_body,
    });
}

fn clear_hair_swaps() {
    let mut guard = HAIR_SWAPS.lock().unwrap_or_else(PoisonError::into_inner);
    *guard = None;
}

/// Runs a body's update; a partner body with a picked hair colour sees it in the global for the length of the update only.
pub fn with_partner_hair<T>(body: usize, update: impl FnOnce() -> T) -> T {
    let swap = HAIR_SWAPS.try_lock().ok().and_then(|guard| {
        let swaps = guard.as_ref()?;
        let (_, swap) = swaps.by_body.iter().find(|(held, _)| *held == body)?;
        Some((swaps.calls, *swap))
    });
    match swap {
        Some((calls, swap)) => calls.with_hair_swapped(swap.slot, swap.value, update),
        None => update(),
    }
}
