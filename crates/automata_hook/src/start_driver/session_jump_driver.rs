use automata_game::address::{CURRENT_PHASE_ID_VALUE, SCENE_LOADED_FLAG_VALUE, SCENE_STATE_VALUE};
use automata_game::defeat::DefeatFlow;
use automata_game::partner::not_free_reason;
use automata_game::session_jump::{
    SessionJumpAction, SessionJumpOrder, SessionJumpOrigin, SessionJumpReadings,
    decide_session_jump, landing_company_line, record_of_start_index,
};
use automata_game::travel::JumpRole;
use automata_proto::Message;
use nt_hook::diagnostics::log_line;

use crate::actions::{Action, enqueue};
use crate::net::handle;
use crate::partner_driver::{held_partner_body, hero_is_down};
use crate::start_driver::NOT_READ_SENTINEL;
use crate::start_driver::chapter_table_read::read_chapter_table;
use crate::start_driver::driver::log_all;
use crate::start_driver::driver_state::{DRIVER, Driver};
use crate::start_driver::link_reading::link_reading;

const SESSION_JUMP_QUEUED_TEXT: &str = "asked the game to jump to that chapter start";
const SESSION_JUMP_GUEST_TEXT: &str = "not asked: only the host starts a jump in a session";

struct SceneReading {
    state: i32,
    phase_id: i32,
    loaded: i32,
}

pub fn ask_session_jump(start_index: usize) -> Result<String, String> {
    if link_reading().role == JumpRole::Guest {
        return Err(SESSION_JUMP_GUEST_TEXT.to_owned());
    }
    enqueue(Action::SessionJump(start_index))
        .map(|()| SESSION_JUMP_QUEUED_TEXT.to_owned())
        .map_err(|error| format!("not queued: {error}"))
}

pub fn run_session_jump(start_index: usize) {
    let Ok(mut guard) = DRIVER.try_lock() else {
        log_line("session jump: not begun, the start driver was busy");
        return;
    };
    if let Some(driver) = guard.as_mut() {
        driver.begin_session_jump(start_index, SessionJumpOrigin::ThisPlayer);
    }
}

impl Driver {
    pub(super) fn step_session_jump(&mut self) {
        self.hear_the_hosts_order();
        let now_ms = self.started.elapsed().as_millis() as u64;
        let arrival = self.session_jump.wants_arrival_readings().then(|| {
            let scene = self.read_scene();
            self.read_arrival(scene.state, scene.phase_id, scene.loaded)
        });
        let step = self.session_jump.step(now_ms, arrival.as_ref());
        log_all(step.logs);
        match step.action {
            SessionJumpAction::Nothing => {}
            SessionJumpAction::RunPlan(plan) => {
                self.after_phase_start = plan.after_phase_start.clone();
                for call in &plan.before_phase_start {
                    if let Err(reason) = self.run_call(call) {
                        log_all(self.session_jump.call_failed(&reason));
                        return;
                    }
                }
                self.write_the_watched_hero(&automata_game::load_gate::StoryPlace::new(
                    plan.phase_start.phase_id,
                    &plan.phase_start.phase_name,
                ));
            }
            SessionJumpAction::TryPhaseStart(call) => {
                let accepted = self.call_phase_start(&call);
                log_all(self.session_jump.phase_start_returned(now_ms, accepted));
                if accepted {
                    self.tell_the_guest();
                }
            }
        }
        if step.landed {
            self.log_the_landing_company();
        }
    }

    pub(super) fn begin_session_jump(&mut self, start_index: usize, origin: SessionJumpOrigin) {
        if let Err(reason) = self.try_begin_session_jump(start_index, origin) {
            log_line(&format!(
                "session jump: not begun ({}): {reason}",
                origin.label()
            ));
        }
    }

    fn try_begin_session_jump(
        &mut self,
        start_index: usize,
        origin: SessionJumpOrigin,
    ) -> Result<(), String> {
        let readings = self.read_session_jump_readings();
        decide_session_jump(origin, &readings).map_err(|refusal| refusal.to_string())?;
        let records = read_chapter_table(&self.calls)?;
        let record = record_of_start_index(&records, start_index)?;
        let order = SessionJumpOrder::of_start_index(start_index)
            .ok_or_else(|| format!("start index {start_index} cannot be sent"))?;
        log_all(
            self.session_jump
                .begin(&record, self.session_jump_characters, origin),
        );
        self.session_jump_to_tell = (origin == SessionJumpOrigin::ThisPlayer).then_some(order);
        Ok(())
    }

    pub(super) fn read_session_jump_readings(&self) -> SessionJumpReadings {
        let calls = &self.calls;
        SessionJumpReadings {
            play: calls.read_play(calls.current_hero_present()),
            role: link_reading().role,
            not_free_reason: not_free_reason(&calls.read_free()).map(|reason| reason.to_string()),
            defeat_flow_active: calls.read_defeat_flow().is_some_and(DefeatFlow::is_active),
            hero_is_down: hero_is_down(),
            load_under_way: self.load_watch.is_loading() || self.gate.is_holding(),
            jump_under_way: self.session_jump.is_active(),
        }
    }

    fn read_scene(&self) -> SceneReading {
        let value = |address| self.calls.read_i32(address).unwrap_or(NOT_READ_SENTINEL);
        SceneReading {
            state: value(SCENE_STATE_VALUE),
            phase_id: value(CURRENT_PHASE_ID_VALUE),
            loaded: value(SCENE_LOADED_FLAG_VALUE),
        }
    }

    fn hear_the_hosts_order(&mut self) {
        let Some(net) = handle() else {
            return;
        };
        let heard = net.take_relayed_from(|message| matches!(message, Message::SessionJump(_)));
        let Some(Message::SessionJump(order)) =
            heard.into_iter().map(|(_, message)| message).next_back()
        else {
            return;
        };
        log_line(&format!(
            "session jump: the host's order heard, start index {}",
            order.start_index
        ));
        self.begin_session_jump(
            usize::from(order.start_index),
            SessionJumpOrigin::TheHostsOrder,
        );
    }

    fn tell_the_guest(&mut self) {
        let Some(order) = self.session_jump_to_tell.take() else {
            return;
        };
        if link_reading().role != JumpRole::Host {
            return;
        }
        if let Some(net) = handle() {
            net.send_relayed(Message::SessionJump(order));
            log_line(&format!(
                "session jump #{}: the scene is changing, told the guest's game to do the same jump, start index {} (a reliable message)",
                self.session_jump.number(),
                order.start_index
            ));
        }
    }

    fn log_the_landing_company(&self) {
        let link = link_reading();
        log_line(&landing_company_line(
            self.session_jump.number(),
            link.partner_present,
            link.partner_watching,
            held_partner_body().is_some(),
        ));
    }
}
