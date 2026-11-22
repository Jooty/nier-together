use replicant_proto::EnemyId;

use crate::effects::MAX_LASTING_SENT_EFFECTS;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SentPlacedCueEffect {
    pub handle: u32,
    pub enemy: EnemyId,
    pub id: u32,
}

#[derive(Debug, Clone, Default)]
pub struct SentPlacedCueEffects {
    sent: Vec<SentPlacedCueEffect>,
}

impl SentPlacedCueEffects {
    /// Remembers a placed effect the host sent; the oldest is handed back when the list is full.
    pub fn note_sent(&mut self, effect: SentPlacedCueEffect) -> Option<SentPlacedCueEffect> {
        let evicted = (self.sent.len() >= MAX_LASTING_SENT_EFFECTS).then(|| self.sent.remove(0));
        self.sent.push(effect);
        evicted
    }

    /// Drops and returns the effects `has_ended` reports as ended.
    pub fn take_ended(
        &mut self,
        mut has_ended: impl FnMut(u32) -> bool,
    ) -> Vec<SentPlacedCueEffect> {
        let (ended, live): (Vec<_>, Vec<_>) = self
            .sent
            .iter()
            .copied()
            .partition(|effect| has_ended(effect.handle));
        self.sent = live;
        ended
    }

    pub fn take_all(&mut self) -> Vec<SentPlacedCueEffect> {
        std::mem::take(&mut self.sent)
    }

    pub fn is_empty(&self) -> bool {
        self.sent.is_empty()
    }
}
