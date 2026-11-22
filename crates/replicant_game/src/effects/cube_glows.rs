use replicant_proto::{AttachedEffect, EnemyId};

use crate::effects::REMOVE_ATTACHED_ANY_BONE_AT_OR_BELOW;

pub const MAX_CUBE_GLOWS: usize = 256;
pub const CUBE_GLOW_RESEND_INTERVAL_MS: u64 = 2000;
pub const CUBE_GLOW_REPLAY_EXPIRES_AFTER_MS: u64 = 7000;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SentCubeGlow {
    pub actor: u64,
    pub enemy: EnemyId,
    pub id: u32,
    pub attached: AttachedEffect,
    pub last_sent_ms: u64,
}

#[derive(Debug, Default)]
pub struct SentCubeGlows {
    glows: Vec<SentCubeGlow>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReplayedCubeGlow {
    pub actor: u64,
    pub enemy: EnemyId,
    pub id: u32,
    pub bone: i16,
    pub refreshed_ms: u64,
}

#[derive(Debug, Default)]
pub struct ReplayedCubeGlows {
    glows: Vec<ReplayedCubeGlow>,
}

impl SentCubeGlows {
    pub fn note_attached(&mut self, glow: SentCubeGlow) {
        let same_glow = |known: &SentCubeGlow| {
            known.actor == glow.actor
                && known.id == glow.id
                && known.attached.bone == glow.attached.bone
        };
        if let Some(known) = self.glows.iter_mut().find(|known| same_glow(known)) {
            *known = glow;
            return;
        }
        if self.glows.len() >= MAX_CUBE_GLOWS {
            self.glows.remove(0);
        }
        self.glows.push(glow);
    }

    pub fn take_removed(&mut self, actor: u64, id: u32, bone: i32) -> Option<EnemyId> {
        let matches = |glow: &SentCubeGlow| {
            glow.actor == actor && glow.id == id && bone_matches(glow.attached.bone, bone)
        };
        let enemy = self.glows.iter().find(|glow| matches(glow))?.enemy;
        self.glows.retain(|glow| !matches(glow));
        Some(enemy)
    }

    pub fn take_gone(
        &mut self,
        mut still_it: impl FnMut(u64, EnemyId) -> bool,
    ) -> Vec<SentCubeGlow> {
        let (kept, gone): (Vec<_>, Vec<_>) = self
            .glows
            .iter()
            .partition(|glow| still_it(glow.actor, glow.enemy));
        self.glows = kept;
        gone
    }

    pub fn take_due_resends(&mut self, now_ms: u64) -> Vec<SentCubeGlow> {
        let mut due = Vec::new();
        for glow in &mut self.glows {
            if now_ms.saturating_sub(glow.last_sent_ms) >= CUBE_GLOW_RESEND_INTERVAL_MS {
                glow.last_sent_ms = now_ms;
                due.push(*glow);
            }
        }
        due
    }

    pub fn is_empty(&self) -> bool {
        self.glows.is_empty()
    }

    pub fn len(&self) -> usize {
        self.glows.len()
    }
}

impl ReplayedCubeGlows {
    pub fn refresh(&mut self, actor: u64, id: u32, bone: i16, now_ms: u64) -> bool {
        let Some(known) = self
            .glows
            .iter_mut()
            .find(|known| known.actor == actor && known.id == id && known.bone == bone)
        else {
            return false;
        };
        known.refreshed_ms = now_ms;
        true
    }

    pub fn note_attached(&mut self, glow: ReplayedCubeGlow) {
        if self.glows.len() >= MAX_CUBE_GLOWS {
            self.glows.remove(0);
        }
        self.glows.push(glow);
    }

    pub fn take_removed(&mut self, actor: u64, id: u32, bone: i32) -> bool {
        let before = self.glows.len();
        self.glows.retain(|glow| {
            !(glow.actor == actor && glow.id == id && bone_matches(glow.bone, bone))
        });
        self.glows.len() != before
    }

    pub fn take_gone(
        &mut self,
        mut still_it: impl FnMut(u64, EnemyId) -> bool,
    ) -> Vec<ReplayedCubeGlow> {
        let (kept, gone): (Vec<_>, Vec<_>) = self
            .glows
            .iter()
            .partition(|glow| still_it(glow.actor, glow.enemy));
        self.glows = kept;
        gone
    }

    pub fn take_expired(&mut self, now_ms: u64) -> Vec<ReplayedCubeGlow> {
        let (kept, expired): (Vec<_>, Vec<_>) = self.glows.iter().partition(|glow| {
            now_ms.saturating_sub(glow.refreshed_ms) < CUBE_GLOW_REPLAY_EXPIRES_AFTER_MS
        });
        self.glows = kept;
        expired
    }

    pub fn is_empty(&self) -> bool {
        self.glows.is_empty()
    }

    pub fn len(&self) -> usize {
        self.glows.len()
    }
}

fn bone_matches(bone: i16, removed_bone: i32) -> bool {
    removed_bone <= REMOVE_ATTACHED_ANY_BONE_AT_OR_BELOW || i32::from(bone) == removed_bone
}
