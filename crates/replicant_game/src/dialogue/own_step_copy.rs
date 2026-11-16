use crate::dialogue::OWN_STEP_COPY_AWAITED_MS;

/// One tick of the guest's side when the host shows a story talk.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OwnStepCopyReading<'a> {
    pub host_story_talk: Option<u32>,
    pub story_step_running_here: Option<&'a str>,
    pub own_copy_open: bool,
    pub mirror_showing: bool,
    pub now_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OwnStepCopyEvent {
    Waits {
        talk_id: u32,
        step: String,
    },
    OwnCopyCame {
        talk_id: u32,
        waited_ms: u64,
    },
    GaveUp {
        talk_id: u32,
        step: String,
        waited_ms: u64,
    },
}

/// Holds the host's story talk out of the guest's window while the guest's own game runs a story
/// step: that step opens its own copy (with its own window size), and a mirrored window would
/// stop the step's `_Wait` (it counts down only while no control slot has the talk bit 0x20).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OwnStepCopyWait {
    waiting: Option<WaitedTalk>,
    given_up_on: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct WaitedTalk {
    talk_id: u32,
    step: String,
    since_ms: u64,
}

impl OwnStepCopyWait {
    pub const fn new() -> Self {
        Self {
            waiting: None,
            given_up_on: None,
        }
    }

    /// Once per tick, on the guest: true while the host's story talk must not be opened here.
    pub fn step(&mut self, reading: &OwnStepCopyReading<'_>) -> (bool, Option<OwnStepCopyEvent>) {
        let Some(talk_id) = reading.host_story_talk else {
            self.given_up_on = None;
            return (false, self.take_waiting_as_came(reading));
        };
        if reading.own_copy_open || reading.mirror_showing {
            return (false, self.take_waiting_as_came(reading));
        }
        if self.given_up_on == Some(talk_id) {
            return (false, None);
        }
        if let Some(waited) = self.waiting.as_mut() {
            // The host chained on to its next talk while this game's copy is still coming: the
            // same wait goes on, with its first clock.
            waited.talk_id = talk_id;
            let waited_ms = reading.now_ms.saturating_sub(waited.since_ms);
            if waited_ms < OWN_STEP_COPY_AWAITED_MS {
                return (true, None);
            }
            let step = waited.step.clone();
            self.waiting = None;
            self.given_up_on = Some(talk_id);
            return (
                false,
                Some(OwnStepCopyEvent::GaveUp {
                    talk_id,
                    step,
                    waited_ms,
                }),
            );
        }
        let Some(step) = reading.story_step_running_here else {
            return (false, None);
        };
        self.waiting = Some(WaitedTalk {
            talk_id,
            step: step.to_owned(),
            since_ms: reading.now_ms,
        });
        (
            true,
            Some(OwnStepCopyEvent::Waits {
                talk_id,
                step: step.to_owned(),
            }),
        )
    }

    fn take_waiting_as_came(
        &mut self,
        reading: &OwnStepCopyReading<'_>,
    ) -> Option<OwnStepCopyEvent> {
        let waited = self.waiting.take()?;
        reading
            .own_copy_open
            .then(|| OwnStepCopyEvent::OwnCopyCame {
                talk_id: waited.talk_id,
                waited_ms: reading.now_ms.saturating_sub(waited.since_ms),
            })
    }
}

impl OwnStepCopyEvent {
    pub fn describe(&self) -> String {
        match self {
            Self::Waits { talk_id, step } => format!(
                "dialogue: the host's story talk {talk_id} is not opened in your window yet: your game runs its own {step}, which opens its own copy (waits up to {OWN_STEP_COPY_AWAITED_MS} ms)"
            ),
            Self::OwnCopyCame { talk_id, waited_ms } => format!(
                "dialogue: your game's own copy opened {waited_ms} ms after the host's story talk {talk_id} came, it follows the host's lines"
            ),
            Self::GaveUp {
                talk_id,
                step,
                waited_ms,
            } => format!(
                "dialogue: your game's {step} did not open its own copy within {waited_ms} ms, the host's story talk {talk_id} opens in your window"
            ),
        }
    }
}
