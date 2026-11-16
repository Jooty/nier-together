use replicant_proto::EnemyId;

pub const MAX_LASTING_CUE_SOUNDS: usize = 32;

/// The host's lasting sounds that went out as actor cues, by the host's own sound handle, so
/// the game's later stop of that handle can be sent too.
#[derive(Debug, Default)]
pub struct SentLastingSounds {
    sent: Vec<(u64, EnemyId, u32)>,
}

/// One host lasting sound the guest plays: the host's handle, the guest's own, and where the
/// sound sat from the copy when it started.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReplayedLastingSound {
    pub actor: EnemyId,
    pub id: u32,
    pub sender_handle: u64,
    pub own_handle: u64,
    pub offset_from_copy: [f32; 3],
}

/// The guest's replays of the host's lasting sounds, until the host stops them or the copy
/// stops being held here.
#[derive(Debug, Default)]
pub struct ReplayedLastingSounds {
    playing: Vec<ReplayedLastingSound>,
}

impl SentLastingSounds {
    pub fn note_sent(&mut self, handle: u64, actor: EnemyId, id: u32) {
        self.sent.retain(|&(sent, _, _)| sent != handle);
        if self.sent.len() >= MAX_LASTING_CUE_SOUNDS {
            self.sent.remove(0);
        }
        self.sent.push((handle, actor, id));
    }

    /// The actor and sound id of a sent lasting sound the game just stopped, forgotten here.
    pub fn take_stopped(&mut self, handle: u64) -> Option<(EnemyId, u32)> {
        let position = self.sent.iter().position(|&(sent, _, _)| sent == handle)?;
        let (_, actor, id) = self.sent.remove(position);
        Some((actor, id))
    }

    pub fn is_empty(&self) -> bool {
        self.sent.is_empty()
    }
}

impl ReplayedLastingSounds {
    /// Remembers a replayed sound; returns the own handles it replaces (the same host handle
    /// again, or the oldest when the list is full), which the caller stops.
    pub fn track(&mut self, sound: ReplayedLastingSound) -> Vec<u64> {
        let (replaced, kept): (Vec<_>, Vec<_>) = self
            .playing
            .iter()
            .partition(|playing| playing.sender_handle == sound.sender_handle);
        self.playing = kept;
        let mut stopped: Vec<u64> = replaced.iter().map(|old| old.own_handle).collect();
        if self.playing.len() >= MAX_LASTING_CUE_SOUNDS {
            stopped.push(self.playing.remove(0).own_handle);
        }
        self.playing.push(sound);
        stopped
    }

    /// The own handle that plays the host's sound `id` of `actor` under `sender_handle`.
    pub fn take_stopped(&mut self, actor: EnemyId, id: u32, sender_handle: u64) -> Option<u64> {
        let position = self.playing.iter().position(|playing| {
            playing.actor == actor && playing.id == id && playing.sender_handle == sender_handle
        })?;
        Some(self.playing.remove(position).own_handle)
    }

    /// Takes every replay whose copy is no longer held here, for the caller to stop.
    pub fn take_unheld(
        &mut self,
        mut held: impl FnMut(EnemyId) -> bool,
    ) -> Vec<ReplayedLastingSound> {
        let (kept, gone): (Vec<_>, Vec<_>) =
            self.playing.iter().partition(|playing| held(playing.actor));
        self.playing = kept;
        gone
    }

    /// Forgets the replay whose own sound the game no longer plays.
    pub fn forget(&mut self, own_handle: u64) {
        self.playing
            .retain(|playing| playing.own_handle != own_handle);
    }

    pub fn playing(&self) -> &[ReplayedLastingSound] {
        &self.playing
    }
}
