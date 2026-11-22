use automata_game::late_join::{
    BossReading, LandingReadings, arrival_beside_line, boss_fight_refusal, landing_begins_line,
    landing_no_single_chapter_line, safe_moment_refusal, start_index_of_place,
};
use automata_game::load_gate::{AloneReason, Release};
use automata_game::mode::SessionMode;
use automata_game::session_jump::SessionJumpOrigin;
use automata_game::travel::JumpRole;
use automata_proto::Message;
use nt_hook::diagnostics::log_line;

use crate::actions::{Action, enqueue};
use crate::net::{
    NetSharedExt, current_session_mode, handle, held_arrival_joiners, held_arrivals_of,
    local_story_place, push_announced_line, take_in_held_players,
};
use crate::partner_driver::held_partner_body;
use crate::start_driver::chapter_table_read::read_chapter_table;
use crate::start_driver::driver::log_all;
use crate::start_driver::driver_state::Driver;
use crate::start_driver::link_reading::{LinkReading, link_reading};

impl Driver {
    pub(super) fn step_late_join(&mut self) {
        self.arm_the_landing_once();
        let Some(net) = handle() else {
            return;
        };
        let waiting = net.update(|shared| held_arrivals_of(shared) > 0);
        let landing_armed = self.landing.is_armed();
        let now_ms = self.started.elapsed().as_millis() as u64;
        if !waiting {
            self.boss_wait.nobody_waits();
        }
        if !waiting && !landing_armed {
            self.joiner.step(now_ms, false, None);
            return;
        }
        let jump_refusal = safe_moment_refusal(&self.read_session_jump_readings());
        let boss_holds = waiting && jump_refusal.is_none() && self.boss_fight_holds(now_ms);
        let refusal_text = match &jump_refusal {
            Some(refusal) => Some(refusal.to_string()),
            None => boss_holds.then(boss_fight_refusal),
        };
        let step = self.joiner.step(now_ms, waiting, refusal_text);
        log_all(step.logs);
        if step.take_in {
            for name in net.update(take_in_held_players) {
                log_line(&format!("late join: {name} is taken in"));
            }
        }
        if landing_armed {
            self.step_landing(&link_reading(), jump_refusal.is_none(), now_ms);
        }
    }

    fn boss_fight_holds(&mut self, now_ms: u64) -> bool {
        let Some(net) = handle() else {
            return false;
        };
        if !self.boss_wait.scan_due(now_ms) {
            return self.boss_wait.holding();
        }
        let joiners = net.update(|shared| held_arrival_joiners(shared));
        let reading = if joiners.is_empty() {
            BossReading::NoBoss
        } else {
            self.calls.read_boss_fight()
        };
        let step = self.boss_wait.step(now_ms, &joiners, reading);
        log_all(step.logs);
        for line in step.in_game_lines {
            push_announced_line(line);
        }
        for notice in step.notices {
            net.send_relayed(Message::BossWait {
                slot: notice.slot,
                waiting: notice.waiting,
            });
        }
        self.boss_wait.holding()
    }

    fn arm_the_landing_once(&mut self) {
        if self.late_arming_checked || !self.machine.is_finished() {
            return;
        }
        self.late_arming_checked = true;
        let linked_after_the_host_played = handle()
            .is_some_and(|net| net.update(|shared| shared.host_was_in_play_when_first_heard()));
        if let Some(Release::Alone { reason, .. }) = self.gate.release()
            && linked_after_the_host_played
            && matches!(
                reason,
                AloneReason::PartnerAlreadyInPlay | AloneReason::PartnerStayedInAnotherPlace
            )
        {
            self.landing.arm();
            log_line(
                "late join: the host was already playing when this game's start ended, this game lands beside it",
            );
        }
    }

    fn step_landing(&mut self, link: &LinkReading, safe: bool, now_ms: u64) {
        let own_place = local_story_place();
        let free_roam = current_session_mode() == SessionMode::FreeRoam;
        let step = self.landing.step(&LandingReadings {
            now_ms,
            applies: link.load_role == JumpRole::Guest,
            can_jump: free_roam,
            own_place,
            host_place: link.partner_place,
            host_in_play: link.partner.in_play,
            safe_moment: safe,
            host_free: link.partner_free,
            host_body_present: held_partner_body().is_some(),
        });
        if step.go_to_partner {
            log_line(&arrival_beside_line(&link.partner_place));
            if let Err(error) = enqueue(Action::GoToPartner) {
                log_line(&format!(
                    "late join: the walk to the host was not queued: {error}"
                ));
            }
        }
        let Some(host_place) = step.land_at else {
            return;
        };
        let start_index = read_chapter_table(&self.calls)
            .ok()
            .and_then(|records| start_index_of_place(&records, &host_place));
        let Some(start_index) = start_index else {
            let line = landing_no_single_chapter_line(&host_place);
            log_line(&line);
            push_announced_line(line);
            self.landing.give_up();
            return;
        };
        log_line(&landing_begins_line(&own_place, &host_place, start_index));
        self.begin_session_jump(start_index, SessionJumpOrigin::LandingAtTheHost);
    }
}
