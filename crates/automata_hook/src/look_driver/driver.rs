use std::panic;
use std::sync::Mutex;

use automata_game::build::variant_of_image_size;
use automata_game::character::Character;
use automata_game::outfit_look::{
    ApplyGuard, PlayerLook, look_message_due, look_slot_of, look_steps,
};
use automata_game::partner::BODY_MODEL_ID_OFFSET;
use automata_proto::Message;
use nt_hook::build_gate::VerifiedBuild;
use nt_hook::diagnostics::log_line;

use crate::game_calls::GameCalls;
use crate::look_driver::load_hooks::{self, take_load_pass_seen};
use crate::look_driver::net_reading::{NetReading, net_reading};
use crate::look_driver::partner_looks::PartnerLooks;
use crate::look_driver::shared::{PanelView, state, try_state};
use crate::net::{handle, local_in_play};

const NAMES_READ_PER_TICK: usize = 6;

static CONTEXT: Mutex<Option<LookContext>> = Mutex::new(None);

#[derive(Debug, Clone, Copy)]
struct Hero {
    body: usize,
    character: Character,
    slot: usize,
}

struct LookContext {
    calls: GameCalls,
    tick: u64,
    guard: ApplyGuard,
    last_body: usize,
    gave_up_logged: bool,
    last_sent: Option<PlayerLook>,
    was_session_open: bool,
    last_peers: usize,
    partners: PartnerLooks,
}

impl LookContext {
    fn new(calls: GameCalls) -> Self {
        Self {
            calls,
            tick: 0,
            guard: ApplyGuard::default(),
            last_body: 0,
            gave_up_logged: false,
            last_sent: None,
            was_session_open: false,
            last_peers: 0,
            partners: PartnerLooks::default(),
        }
    }

    fn step(&mut self) {
        self.tick += 1;
        let net = net_reading();
        let in_play = local_in_play();
        self.read_names(in_play);
        let hero = self.read_hero().filter(|_| in_play);
        if self.was_session_open && !net.session_open {
            self.end_session(hero.is_some());
        }
        self.was_session_open = net.session_open;
        publish_view(&net, hero);
        let Some(hero) = hero else {
            self.partners.clear();
            return;
        };
        if net.connected {
            self.partners.tick(&self.calls, net.allowed(), self.tick);
        } else {
            self.partners.clear();
        }
        if hero.body != self.last_body {
            self.last_body = hero.body;
            self.guard.reset();
            self.gave_up_logged = false;
        }
        let Some(reading) = self.calls.read_look(hero.body, hero.slot) else {
            return;
        };
        let (targets, wanted) = self.take_requests_and_targets(hero, &net);
        for step in look_steps(&targets, &reading) {
            self.run_step(hero, step);
        }
        self.send_look(&net, wanted);
    }

    fn take_requests_and_targets(
        &mut self,
        hero: Hero,
        net: &NetReading,
    ) -> (
        [Option<i32>; automata_game::outfit_look::LOOK_KIND_COUNT],
        PlayerLook,
    ) {
        let allowed = net.allowed();
        let mut shared = state();
        while let Some(request) = shared.requests.pop_front() {
            if !allowed {
                continue;
            }
            let current = self
                .calls
                .read_look_global(request.kind, request.slot)
                .unwrap_or_default();
            shared
                .picks
                .pick(request.slot, request.kind, request.value, current);
            log_line(&format!(
                "looks: picked {:?} {:?} for slot {}",
                request.kind, request.value, request.slot
            ));
        }
        (
            shared.picks.take_targets(hero.slot, allowed),
            shared.picks.wanted_look(hero.slot, allowed),
        )
    }

    fn run_step(&mut self, hero: Hero, step: automata_game::outfit_look::LookStep) {
        if self.guard.allow(step.kind(), step.value(), self.tick) {
            if !self.calls.apply_look_step(hero.body, hero.slot, step) {
                log_line(&format!("looks: {step:?} was not written"));
            }
        } else if self.guard.gave_up(step.kind(), step.value()) && !self.gave_up_logged {
            self.gave_up_logged = true;
            log_line(&format!(
                "looks: the game did not take {step:?}; it is not tried again for this body"
            ));
        }
    }

    fn send_look(&mut self, net: &NetReading, wanted: PlayerLook) {
        let peer_joined = net.peers > self.last_peers;
        self.last_peers = net.peers;
        let scene_loaded = take_load_pass_seen();
        if !net.connected || !look_message_due(self.last_sent, wanted, peer_joined, scene_loaded) {
            return;
        }
        if let Some(net_thread) = handle() {
            net_thread.send_relayed(Message::Look(wanted));
            self.last_sent = Some(wanted);
            log_line("looks: told the other player our look");
        }
    }

    fn end_session(&mut self, can_restore_on_the_body: bool) {
        let mut shared = state();
        shared.picks.end_session();
        let saved = if can_restore_on_the_body {
            Vec::new()
        } else {
            shared.picks.take_saved_looks()
        };
        drop(shared);
        for look in saved {
            self.calls
                .write_look_global(look.kind, look.slot, look.value);
        }
        self.last_sent = None;
        log_line("looks: the session ended, the saved looks come back");
    }

    fn read_hero(&self) -> Option<Hero> {
        let body = self.calls.current_hero();
        if body == 0 {
            return None;
        }
        let model = self
            .calls
            .read_u32_at(self.calls.body_field(body, BODY_MODEL_ID_OFFSET))
            .ok()?;
        let character = Character::from_model_id(model)?;
        Some(Hero {
            body,
            character,
            slot: look_slot_of(character),
        })
    }

    fn read_names(&self, in_play: bool) {
        if !in_play {
            return;
        }
        let ids = state().names.unread_ids(NAMES_READ_PER_TICK);
        for id in ids {
            let name = self.calls.item_name(id);
            state().names.record(id, name);
        }
    }
}

fn publish_view(net: &NetReading, hero: Option<Hero>) {
    state().view = PanelView {
        slot: hero.map(|hero| hero.slot),
        character: hero.map(|hero| hero.character),
        mode: net.mode,
        host_allows: net.host_allows,
    };
}

pub fn install(build: VerifiedBuild) {
    let Some(variant) = variant_of_image_size(build.size_of_image()) else {
        return;
    };
    let calls = GameCalls::new(build, variant);
    state().calls = Some(calls);
    if let Ok(mut context) = CONTEXT.lock() {
        *context = Some(LookContext::new(calls));
    }
    load_hooks::install(build, &calls);
}

pub fn tick() {
    let _ = panic::catch_unwind(|| {
        let Ok(mut context) = CONTEXT.try_lock() else {
            return;
        };
        if let Some(context) = context.as_mut() {
            context.step();
        }
    });
}

pub fn restore_on_detach() {
    let _ = panic::catch_unwind(|| {
        let Some(shared) = try_state() else {
            return;
        };
        let Some(calls) = shared.calls else {
            return;
        };
        for look in shared.picks.saved_looks() {
            calls.write_look_global(look.kind, look.slot, look.value);
        }
    });
}
