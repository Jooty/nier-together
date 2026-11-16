use replicant_proto::EnemyId;

pub const MAX_ATTACHED_CUE_EFFECTS: usize = 256;
pub const REMOVE_ATTACHED_ANY_BONE_AT_OR_BELOW: i32 = -99;

/// An effect attached by id through an actor cue: on the host the one it sent, on the guest
/// the one it made on its copy. `actor` is this game's own actor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttachedCueEffect {
    pub actor: u64,
    pub enemy: EnemyId,
    pub id: u32,
    pub bone: i16,
}

/// The attaches that crossed, so their removal crosses too, wherever the game removes them.
#[derive(Debug, Default)]
pub struct AttachedCueEffects {
    attached: Vec<AttachedCueEffect>,
}

impl AttachedCueEffects {
    pub fn note_attached(&mut self, effect: AttachedCueEffect) {
        if self.attached.contains(&effect) {
            return;
        }
        if self.attached.len() >= MAX_ATTACHED_CUE_EFFECTS {
            self.attached.remove(0);
        }
        self.attached.push(effect);
    }

    /// The network id of `actor` when an attach of effect `id` on `bone` crossed (a bone of -99
    /// or less is any bone); the matching attaches are forgotten.
    pub fn take_removed(&mut self, actor: u64, id: u32, bone: i32) -> Option<EnemyId> {
        let matches = |effect: &AttachedCueEffect| {
            effect.actor == actor
                && effect.id == id
                && (bone <= REMOVE_ATTACHED_ANY_BONE_AT_OR_BELOW || i32::from(effect.bone) == bone)
        };
        let enemy = self.attached.iter().find(|effect| matches(effect))?.enemy;
        self.attached.retain(|effect| !matches(effect));
        Some(enemy)
    }

    /// Takes every attach whose actor no longer stands for its network id here (destroyed,
    /// no longer shared, or no longer held), for the caller to remove.
    pub fn take_gone(
        &mut self,
        mut still_it: impl FnMut(u64, EnemyId) -> bool,
    ) -> Vec<AttachedCueEffect> {
        let (kept, gone): (Vec<_>, Vec<_>) = self
            .attached
            .iter()
            .partition(|effect| still_it(effect.actor, effect.enemy));
        self.attached = kept;
        gone
    }

    pub fn is_empty(&self) -> bool {
        self.attached.is_empty()
    }
}
