use nt_launcher_core::{LauncherEvent, LauncherNet};
use replicant_proto::{LOBBY_MODE_CONTINUE, ScaleUpAsk, scale_up_no_save_yet_line};

use crate::app::Launcher;
use crate::lobby::{QuestionStep, SeatQuestion};
use crate::saves::SlotSummary;
use crate::seat_round::{HOST_MEMBER_SLOT, LateJoinSeat};

impl Launcher {
    pub fn guest_waits_for_the_hosts_decision(&self) -> bool {
        self.host_deciding_on_your_save
    }

    pub(super) fn catch_up_late_member(
        &mut self,
        member_slot: u8,
        name: &str,
        net: &dyn LauncherNet,
    ) {
        self.drop_seat_question(net);
        let Some(mode) = self.host_started else {
            net.release_start(member_slot, None);
            return;
        };
        let Some(played) = self.host_played_slot(mode) else {
            net.release_start(member_slot, None);
            return;
        };
        let listed_before = self.listed_save(played);
        self.refresh_saves();
        let Some(slot) = self.listed_save(played).or(listed_before) else {
            self.log.push(format!(
                "a player joined, but the save being played (slot {played}) has no finished save yet; no save was sent"
            ));
            self.tell_the_game_no_save(net, member_slot, name);
            net.release_start(member_slot, None);
            return;
        };
        let seat_start_mode = (mode != LOBBY_MODE_CONTINUE).then_some(LOBBY_MODE_CONTINUE);
        let Some(outcome) = self
            .host_round
            .as_mut()
            .map(|round| round.late_join(net, member_slot, &slot))
        else {
            net.release_start(member_slot, None);
            return;
        };
        self.take_round_logs();
        match outcome {
            LateJoinSeat::Sent => net.release_start(member_slot, seat_start_mode),
            LateJoinSeat::NotReadable => net.release_start(member_slot, None),
            LateJoinSeat::Empty => {
                self.seat_question = Some(SeatQuestion::new(
                    member_slot,
                    name,
                    played,
                    seat_start_mode,
                ));
            }
        }
    }

    pub(super) fn step_seat_question(&mut self, net: &dyn LauncherNet, now_seconds: f64) {
        let Some(question) = self.seat_question.as_ref() else {
            return;
        };
        if self.link.guest || self.host_started.is_none() {
            self.log.push(
                "the question about the guest's save was withdrawn: your game closed".to_owned(),
            );
            self.drop_seat_question(net);
            return;
        }
        if !question.needs_the_save_file() {
            return;
        }
        let Some(path) = self
            .host_round
            .as_ref()
            .map(|round| round.coop_file().to_path_buf())
        else {
            return;
        };
        let file = std::fs::read(&path);
        let step = self
            .seat_question
            .as_mut()
            .map(|question| question.read_file(file, now_seconds));
        if let Some(step) = step {
            self.act_on_question_step(step, net);
        }
    }

    pub(super) fn feed_seat_question(&mut self, event: &LauncherEvent, net: &dyn LauncherNet) {
        match event {
            LauncherEvent::ScaleUpAnswered { slot, yes } => {
                self.on_scale_up_answered(*slot, *yes, net)
            }
            LauncherEvent::MemberLeft { slot, .. }
                if self.seat_question.as_ref().map(SeatQuestion::member_slot) == Some(*slot) =>
            {
                self.log.push(
                    "the guest left, so the question about their save was withdrawn".to_owned(),
                );
                self.drop_seat_question(net);
            }
            LauncherEvent::SessionEnded { .. }
            | LauncherEvent::Stopped
            | LauncherEvent::Failed { .. } => {
                self.drop_seat_question(net);
                self.host_deciding_on_your_save = false;
            }
            LauncherEvent::MemberLeft { slot, .. } if *slot == HOST_MEMBER_SLOT => {
                self.host_deciding_on_your_save = false;
            }
            LauncherEvent::HostAsked { ask } if self.link.guest => match ask {
                ScaleUpAsk::Open { .. } => self.host_deciding_on_your_save = true,
                ScaleUpAsk::Withdrawn { .. } | ScaleUpAsk::NoSaveYet { .. } => {
                    self.host_deciding_on_your_save = false;
                }
            },
            LauncherEvent::LobbyStartReceived { .. } => self.host_deciding_on_your_save = false,
            _ => {}
        }
    }

    pub(super) fn drop_seat_question(&mut self, net: &dyn LauncherNet) {
        if let Some(question) = self.seat_question.take() {
            net.send_scale_up_ask(ScaleUpAsk::Withdrawn {
                slot: question.member_slot(),
            });
        }
    }

    fn listed_save(&self, index: u8) -> Option<SlotSummary> {
        self.slots
            .iter()
            .find(|slot| slot.index == index && slot.used)
            .cloned()
    }

    fn host_played_slot(&self, mode: u8) -> Option<u8> {
        if mode == LOBBY_MODE_CONTINUE {
            return self
                .host_round
                .as_ref()?
                .playing()
                .map(|playing| playing.host_slot);
        }
        self.last_launch
            .filter(|last| !last.guest)
            .and_then(|last| last.slot)
    }

    fn on_scale_up_answered(&mut self, slot: u8, yes: bool, net: &dyn LauncherNet) {
        let Some(question) = self
            .seat_question
            .as_mut()
            .filter(|question| question.member_slot() == slot && question.is_asked())
        else {
            self.log.push(
                "an answer about a guest's save arrived with nothing asked; it did nothing"
                    .to_owned(),
            );
            return;
        };
        let name = question.guest_name().to_owned();
        let step = question.answered(yes);
        if yes {
            self.log
                .push(format!("you chose to give {name} a copy of your save"));
        }
        self.act_on_question_step(step, net);
    }

    fn act_on_question_step(&mut self, step: QuestionStep, net: &dyn LauncherNet) {
        let Some(question) = self.seat_question.as_ref() else {
            return;
        };
        let member_slot = question.member_slot();
        let name = question.guest_name().to_owned();
        let seat_start_mode = question.seat_start_mode();
        match step {
            QuestionStep::Waiting => {}
            QuestionStep::Ask => {
                self.log.push(format!(
                    "asking your game whether to give {name} a copy of your save"
                ));
                net.send_scale_up_ask(ScaleUpAsk::Open {
                    slot: member_slot,
                    name,
                });
            }
            QuestionStep::NoCompletedSave(reason) => {
                self.log.push(format!(
                    "{name} has no save and was not offered a copy of yours: {reason}"
                ));
                self.tell_the_game_no_save(net, member_slot, &name);
                self.finish_seat_question(net, member_slot, None);
            }
            QuestionStep::CopyFailed(reason) => {
                self.log.push(format!(
                    "could not copy your save for {name}: {reason}; they start a new game"
                ));
                self.tell_the_game_no_save(net, member_slot, &name);
                self.finish_seat_question(net, member_slot, None);
            }
            QuestionStep::GuestStartsOwnGame => {
                self.log.push(format!(
                    "you chose not to copy your save for {name}; they start a new game"
                ));
                self.finish_seat_question(net, member_slot, None);
            }
            QuestionStep::Copy(host_seat) => {
                let given = self
                    .host_round
                    .as_mut()
                    .map(|round| round.give_seat_copy_of(net, &host_seat, &name));
                self.take_round_logs();
                let release_mode = match given {
                    Some(Ok(seat)) => match net.send_seat(member_slot, seat) {
                        Some(transfer) => {
                            self.log.push(format!(
                                "gave {name} a copy of your save and sent it (transfer {transfer})"
                            ));
                            seat_start_mode
                        }
                        None => {
                            self.log.push(format!(
                                "gave {name} a copy of your save but the network is not running, so it was not sent"
                            ));
                            None
                        }
                    },
                    Some(Err(reason)) => {
                        self.log.push(format!(
                            "could not give {name} a copy of your save: {reason}"
                        ));
                        None
                    }
                    None => None,
                };
                self.finish_seat_question(net, member_slot, release_mode);
            }
        }
    }

    fn finish_seat_question(&mut self, net: &dyn LauncherNet, member_slot: u8, mode: Option<u8>) {
        self.seat_question = None;
        net.release_start(member_slot, mode);
    }

    fn tell_the_game_no_save(&mut self, net: &dyn LauncherNet, member_slot: u8, name: &str) {
        self.log.push(scale_up_no_save_yet_line(name));
        net.send_scale_up_ask(ScaleUpAsk::NoSaveYet {
            slot: member_slot,
            name: name.to_owned(),
        });
    }
}
