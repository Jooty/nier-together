use replicant_proto::{DialogueLine, DialogueUpdate};

use crate::dialogue::{
    CHOICE_FOLLOW_GIVES_THE_GUEST_ITS_BUTTONS_AFTER_MS, FOLLOW_HOST_TALK_WAIT_MS,
    HOST_LEFT_TALK_CLOSES_A_LATE_SCRIPT_COPY_WITHIN_MS, LINE_PRESS_GIVES_UP_AFTER_MS,
    MIRROR_OPEN_RETRY_MS, MIRRORED_TALK_OWN_COPY_FINISHED_WITHIN_MS,
    OWN_ENDED_TALK_NOT_MIRRORED_MS, TALK_JOIN_RANGE_UNITS, TALK_LEAVE_EXTRA_UNITS,
};

/// What the partner's talk window must do this tick to follow the talker's line.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MirrorAction {
    /// Open the window on `talk_id` and go straight to `line` (a late joiner starts mid-talk).
    Open(DialogueLine),
    /// The talker went from one talk straight into the next in a window that stayed open: open
    /// the next talk in the same open window, as the talker's own script does, so no opening
    /// animation hides its first lines.
    SwitchTalk(DialogueLine),
    /// Move the open mirrored window to this line.
    ShowLine(u16),
    Close,
    /// This player started the same talk on their own: move their window to the talker's line.
    JumpOwnTalk(u16),
    /// The guest's own copy of a talk the host leads waits on the host: buttons do nothing.
    FollowStarted(u32),
    /// Move the guest's own copy of the host's talk to the host's line.
    FollowLine(u16),
    /// The host finished or left this talk: close the guest's copy so its script moves on.
    FinishOwnTalk(u32),
    /// The host stepped from this talk straight into another one: the guest's copy takes the same
    /// step, so its game opens the following talk itself.
    ChainOwnTalk(DialogueLine),
    /// The host never showed this talk: the guest's own talk goes on by the guest's buttons.
    FollowGaveUp(u32),
    /// The presses did not take the guest's copy on to the host's next talk in time: the copy
    /// goes on by the guest's buttons.
    ChainGaveUp(u32),
    /// The host left the talk while the guest's copy was at an answer prompt that never got the
    /// host's answer: the guest's copy goes on by the guest's buttons.
    ChoiceGaveUp(u32),
    /// The partner is still on a talk this player just finished: not opened again as a mirror.
    SkipEndedTalk(u32),
}

/// Who moves a talk both players have open on their own games. Only a partner within talk range
/// leads or follows; otherwise each player reads at their own pace.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TalkLeader {
    Local,
    Partner,
    Either,
}

/// One tick's view of the partner's talk and of this game.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MirrorInput {
    pub partner_line: Option<DialogueLine>,
    pub partner_ended_talk: Option<u32>,
    /// Same zone as the talker, in play, no menu, cutscene or loading.
    pub can_show: bool,
    pub window_open: bool,
    pub own_talk: Option<DialogueLine>,
    pub leader: TalkLeader,
    /// The own copy is at an answer prompt, or the host's answer to it has not been taken yet.
    pub answer_open: bool,
    pub now_ms: u64,
}

/// The partner's latest talk line, kept from the moment it arrives until the partner's talk ends,
/// so a guest whose own window opens a tick later still sees that the host got there first.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PartnerTalkMemory {
    line: Option<DialogueLine>,
    story: bool,
    last_ended: Option<(u32, u64)>,
}

/// A guest's own talk window that a script opened while the host was on the same talk: the
/// host leads it at any distance, through every talk that follows, until the window closes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HostLedScriptTalk {
    led: bool,
}

/// Follows the partner's talk in this game's own talk window, line by line.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PartnerTalkMirror {
    shown: Option<DialogueLine>,
    retry_open_at_ms: u64,
    joined_own_talk: Option<u32>,
    following: Option<FollowedTalk>,
    last_own_talk: Option<u32>,
    own_ended: Option<EndedTalk>,
    mirrored: Vec<MirroredTalk>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct MirroredTalk {
    talk_id: u32,
    at_ms: u64,
    own_copy_finished: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct EndedTalk {
    talk_id: u32,
    at_ms: u64,
    skip_told: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FollowedTalk {
    talk_id: u32,
    since_ms: u64,
    host_seen_on_it: bool,
    gave_up: bool,
    /// The host went on to this talk at this time; the copy stays led while its presses take it there too.
    chained_to: Option<(u32, u64)>,
    finish_deferred_since_ms: Option<u64>,
    answer_seen: bool,
}

impl HostLedScriptTalk {
    pub const fn new() -> Self {
        Self { led: false }
    }

    pub const fn is_led(&self) -> bool {
        self.led
    }

    /// Once per guest tick; true while the host leads the open window from any distance.
    pub fn observe(
        &mut self,
        own_talk: Option<DialogueLine>,
        started_by_player_command: bool,
        host_on_same_talk: bool,
    ) -> bool {
        self.led =
            own_talk.is_some() && !started_by_player_command && (self.led || host_on_same_talk);
        self.led
    }
}

impl PartnerTalkMemory {
    pub const fn new() -> Self {
        Self {
            line: None,
            story: false,
            last_ended: None,
        }
    }

    pub fn receive(&mut self, update: DialogueUpdate, now_ms: u64) {
        match update {
            DialogueUpdate::Line(line) | DialogueUpdate::StoryLine(line) => {
                self.line = Some(line);
                self.story = matches!(update, DialogueUpdate::StoryLine(_));
                self.last_ended = None;
            }
            DialogueUpdate::Ended => {
                self.story = false;
                if let Some(line) = self.line.take() {
                    self.last_ended = Some((line.talk_id, now_ms));
                }
            }
        }
    }

    pub fn forget(&mut self) {
        self.line = None;
        self.story = false;
        self.last_ended = None;
    }

    pub fn current_line(&self) -> Option<DialogueLine> {
        self.line
    }

    /// The partner is on `talk_id` now, or left it within the follow wait before `now_ms`.
    pub fn is_or_just_was_on(&self, talk_id: u32, now_ms: u64) -> bool {
        self.line.is_some_and(|line| line.talk_id == talk_id)
            || self.last_ended.is_some_and(|(ended, ended_ms)| {
                ended == talk_id && now_ms.saturating_sub(ended_ms) <= FOLLOW_HOST_TALK_WAIT_MS
            })
    }

    pub fn is_on(&self, talk_id: u32) -> bool {
        self.line.is_some_and(|line| line.talk_id == talk_id)
    }

    pub fn any_talk_on(&self) -> bool {
        self.line.is_some()
    }

    pub fn any_talk_ended_within_follow_wait(&self, now_ms: u64) -> bool {
        self.last_ended.is_some_and(|(_, ended_ms)| {
            now_ms.saturating_sub(ended_ms) <= FOLLOW_HOST_TALK_WAIT_MS
        })
    }

    pub fn ended_within_follow_wait(&self, talk_id: u32, now_ms: u64) -> bool {
        self.last_ended.is_some_and(|(ended, ended_ms)| {
            ended == talk_id && now_ms.saturating_sub(ended_ms) <= FOLLOW_HOST_TALK_WAIT_MS
        })
    }

    /// The talk the host left a while before this tick, when the guest's own copy is that same
    /// talk: the host already left, so the guest's copy closes at once. The guest's scene copy can
    /// reach the talk seconds after the host's, so the memory outlasts the follow wait.
    pub fn ended_for_follow(
        &self,
        own_talk: Option<DialogueLine>,
        leader: TalkLeader,
        now_ms: u64,
    ) -> Option<u32> {
        let own = own_talk.filter(|_| leader == TalkLeader::Partner)?;
        self.last_ended
            .filter(|&(talk_id, ended_ms)| {
                talk_id == own.talk_id
                    && now_ms.saturating_sub(ended_ms)
                        <= HOST_LEFT_TALK_CLOSES_A_LATE_SCRIPT_COPY_WITHIN_MS
            })
            .map(|(talk_id, _)| talk_id)
    }

    /// The line the mirror may see. Only a guest with its own copy of a talk the host leads
    /// follows; a partner talk this player does not have open stays unshown.
    pub fn for_follow(
        &self,
        own_talk: Option<DialogueLine>,
        leader: TalkLeader,
    ) -> Option<DialogueLine> {
        self.line
            .filter(|_| own_talk.is_some() && leader == TalkLeader::Partner)
    }

    /// The partner's line while their talk is a story step.
    pub fn story_line(&self) -> Option<DialogueLine> {
        self.line.filter(|_| self.story)
    }

    /// The line the mirror may see: the followed line of an own copy, or, on a game that shows
    /// the partner's story talks, a story step's line while no talk of this player's is open.
    pub fn for_mirror(
        &self,
        own_talk: Option<DialogueLine>,
        leader: TalkLeader,
        shows_partner_story_talks: bool,
    ) -> Option<DialogueLine> {
        self.for_follow(own_talk, leader).or_else(|| {
            self.story_line()
                .filter(|_| own_talk.is_none() && shows_partner_story_talks)
        })
    }
}

impl PartnerTalkMirror {
    pub const fn new() -> Self {
        Self {
            shown: None,
            retry_open_at_ms: 0,
            joined_own_talk: None,
            following: None,
            last_own_talk: None,
            own_ended: None,
            mirrored: Vec::new(),
        }
    }

    pub fn is_showing(&self) -> bool {
        self.shown.is_some()
    }

    /// The guest's own talk window waits on the host's lines, so its buttons must not move it.
    pub fn holds_own_talk(&self) -> bool {
        self.following.is_some_and(|followed| !followed.gave_up)
    }

    pub fn step(&mut self, input: MirrorInput) -> Option<MirrorAction> {
        if let Some(own) = input.own_talk {
            self.shown = None;
            self.last_own_talk = Some(own.talk_id);
            self.own_ended = None;
            let host_still_on_it = input
                .partner_line
                .is_some_and(|partner| partner.talk_id == own.talk_id);
            if input.leader == TalkLeader::Partner
                && !host_still_on_it
                && let Some(talk_id) = self.finish_mirrored_own_copy(own.talk_id, input.now_ms)
            {
                return Some(MirrorAction::FinishOwnTalk(talk_id));
            }
            return match input.leader {
                TalkLeader::Partner => self.follow_host(
                    own,
                    input.partner_line,
                    input.partner_ended_talk,
                    input.answer_open,
                    input.now_ms,
                ),
                TalkLeader::Local => {
                    self.following = None;
                    None
                }
                TalkLeader::Either => {
                    self.following = None;
                    self.join_own_talk(own, input.partner_line)
                }
            };
        }
        self.joined_own_talk = None;
        self.following = None;
        if let Some(talk_id) = self.last_own_talk.take() {
            self.mirrored.retain(|talk| talk.talk_id != talk_id);
            self.own_ended = Some(EndedTalk {
                talk_id,
                at_ms: input.now_ms,
                skip_told: false,
            });
        }
        if let Some(skip) = self.skip_ended_talk(input.partner_line, input.now_ms) {
            return skip;
        }
        if self.shown.is_some() && !input.window_open {
            self.shown = None;
        }
        let wanted = input.partner_line.filter(|_| input.can_show);
        match (self.shown, wanted) {
            (None, None) => None,
            (None, Some(line)) => {
                if input.window_open || input.now_ms < self.retry_open_at_ms {
                    return None;
                }
                self.shown = Some(line);
                self.remember_mirrored(line.talk_id, input.now_ms);
                self.retry_open_at_ms = input.now_ms + MIRROR_OPEN_RETRY_MS;
                Some(MirrorAction::Open(line))
            }
            (Some(shown), Some(line)) if shown.talk_id == line.talk_id => {
                if shown.line == line.line {
                    return None;
                }
                self.shown = Some(line);
                Some(MirrorAction::ShowLine(line.line))
            }
            (Some(_), Some(line)) => {
                self.shown = Some(line);
                self.remember_mirrored(line.talk_id, input.now_ms);
                Some(MirrorAction::SwitchTalk(line))
            }
            (Some(_), None) => {
                self.shown = None;
                Some(MirrorAction::Close)
            }
        }
    }

    fn remember_mirrored(&mut self, talk_id: u32, now_ms: u64) {
        self.mirrored.retain(|talk| {
            now_ms.saturating_sub(talk.at_ms) < MIRRORED_TALK_OWN_COPY_FINISHED_WITHIN_MS
        });
        match self
            .mirrored
            .iter_mut()
            .find(|talk| talk.talk_id == talk_id)
        {
            Some(talk) => talk.at_ms = now_ms,
            None => self.mirrored.push(MirroredTalk {
                talk_id,
                at_ms: now_ms,
                own_copy_finished: false,
            }),
        }
    }

    fn finish_mirrored_own_copy(&mut self, talk_id: u32, now_ms: u64) -> Option<u32> {
        let talk = self
            .mirrored
            .iter_mut()
            .find(|talk| talk.talk_id == talk_id && !talk.own_copy_finished)?;
        if now_ms.saturating_sub(talk.at_ms) >= MIRRORED_TALK_OWN_COPY_FINISHED_WITHIN_MS {
            return None;
        }
        talk.own_copy_finished = true;
        Some(talk_id)
    }

    fn skip_ended_talk(
        &mut self,
        partner_line: Option<DialogueLine>,
        now_ms: u64,
    ) -> Option<Option<MirrorAction>> {
        let ended = self.own_ended.as_mut()?;
        let partner_still_on_it = partner_line.is_some_and(|line| line.talk_id == ended.talk_id);
        let fresh = now_ms.saturating_sub(ended.at_ms) < OWN_ENDED_TALK_NOT_MIRRORED_MS;
        if !partner_still_on_it || !fresh || self.shown.is_some() {
            self.own_ended = None;
            return None;
        }
        if ended.skip_told {
            return Some(None);
        }
        ended.skip_told = true;
        Some(Some(MirrorAction::SkipEndedTalk(ended.talk_id)))
    }

    fn follow_host(
        &mut self,
        own: DialogueLine,
        partner_line: Option<DialogueLine>,
        partner_ended_talk: Option<u32>,
        answer_open: bool,
        now_ms: u64,
    ) -> Option<MirrorAction> {
        let fresh = self
            .following
            .is_none_or(|followed| followed.talk_id != own.talk_id);
        if fresh {
            // The copy arrived on the talk the host chained to: the host was on it, even if it
            // has moved on since, so the copy chains on instead of waiting for the host to show it.
            let host_was_on_it = self.following.is_some_and(|followed| {
                followed
                    .chained_to
                    .is_some_and(|(next, _)| next == own.talk_id)
            });
            self.following = Some(FollowedTalk {
                talk_id: own.talk_id,
                since_ms: now_ms,
                host_seen_on_it: host_was_on_it,
                gave_up: false,
                chained_to: None,
                finish_deferred_since_ms: None,
                answer_seen: false,
            });
        }
        let followed = self.following.as_mut()?;
        followed.answer_seen |= answer_open;
        if followed.gave_up {
            return None;
        }
        if let Some((_, since_ms)) = followed.chained_to {
            if now_ms.saturating_sub(since_ms) < LINE_PRESS_GIVES_UP_AFTER_MS {
                return None;
            }
            followed.gave_up = true;
            return Some(MirrorAction::ChainGaveUp(own.talk_id));
        }
        let host_left_before_open =
            fresh && partner_line.is_none() && partner_ended_talk == Some(own.talk_id);
        if host_left_before_open {
            return Self::finish_own_talk(followed, own.talk_id, None, answer_open, now_ms);
        }
        let host_on_it = partner_line.filter(|partner| partner.talk_id == own.talk_id);
        if let Some(host) = host_on_it {
            followed.host_seen_on_it = true;
            followed.finish_deferred_since_ms = None;
            if fresh {
                return Some(MirrorAction::FollowStarted(own.talk_id));
            }
            return (host.line != own.line).then_some(MirrorAction::FollowLine(host.line));
        }
        if followed.host_seen_on_it {
            let chained_to = partner_line.filter(|partner| partner.talk_id != own.talk_id);
            return Self::finish_own_talk(followed, own.talk_id, chained_to, answer_open, now_ms);
        }
        if fresh {
            return Some(MirrorAction::FollowStarted(own.talk_id));
        }
        if now_ms.saturating_sub(followed.since_ms) >= FOLLOW_HOST_TALK_WAIT_MS {
            followed.gave_up = true;
            return Some(MirrorAction::FollowGaveUp(own.talk_id));
        }
        None
    }

    fn finish_own_talk(
        followed: &mut FollowedTalk,
        talk_id: u32,
        chained_to: Option<DialogueLine>,
        answer_open: bool,
        now_ms: u64,
    ) -> Option<MirrorAction> {
        if !answer_open {
            return Some(match chained_to.filter(|_| !followed.answer_seen) {
                Some(next) => {
                    // The copy stays led, so its buttons stay off while the presses take it on.
                    followed.chained_to = Some((next.talk_id, now_ms));
                    MirrorAction::ChainOwnTalk(next)
                }
                None => {
                    followed.gave_up = true;
                    MirrorAction::FinishOwnTalk(talk_id)
                }
            });
        }
        let since_ms = *followed.finish_deferred_since_ms.get_or_insert(now_ms);
        if now_ms.saturating_sub(since_ms) >= CHOICE_FOLLOW_GIVES_THE_GUEST_ITS_BUTTONS_AFTER_MS {
            followed.gave_up = true;
            return Some(MirrorAction::ChoiceGaveUp(talk_id));
        }
        None
    }

    /// The guest's copy goes on by its own buttons from now on.
    pub fn release_followed_talk(&mut self) {
        if let Some(followed) = self.following.as_mut() {
            followed.gave_up = true;
        }
    }

    fn join_own_talk(
        &mut self,
        own: DialogueLine,
        partner_line: Option<DialogueLine>,
    ) -> Option<MirrorAction> {
        if self.joined_own_talk == Some(own.talk_id) {
            return None;
        }
        self.joined_own_talk = Some(own.talk_id);
        let partner = partner_line.filter(|partner| partner.talk_id == own.talk_id)?;
        (partner.line > own.line).then_some(MirrorAction::JumpOwnTalk(partner.line))
    }
}

/// Whether this player is close enough to the talker to join their talk. A window already
/// showing stays a little longer, so standing on the edge does not open and close it.
pub fn within_talk_range(distance_units: f32, already_showing: bool) -> bool {
    let range = if already_showing {
        TALK_JOIN_RANGE_UNITS + TALK_LEAVE_EXTRA_UNITS
    } else {
        TALK_JOIN_RANGE_UNITS
    };
    distance_units.is_finite() && distance_units <= range
}

/// Whether a talk line's tag callbacks may run on this machine. Only the talker runs them.
pub fn talk_tags_run_here(mirroring: bool, mirror_action_running: bool) -> bool {
    !mirroring && !mirror_action_running
}

/// A talk the local player started with the talk command is theirs alone: it is never led by
/// the partner. A copy a script opened (stream entry, relayed call, shared event) follows `base`.
pub fn leader_for_own_talk(base: TalkLeader, started_by_player_command: bool) -> TalkLeader {
    if started_by_player_command && base == TalkLeader::Partner {
        TalkLeader::Local
    } else {
        base
    }
}
