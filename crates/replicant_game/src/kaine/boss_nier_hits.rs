pub const NATIVE_HIT_WINDOW_MS: u64 = 600;
pub const SENT_HIT_REPEAT_WINDOW_MS: u64 = 1_000;
pub const BOSS_HIT_DEFAULT_PERCENT_OF_NIER_MAX_HP: i32 = 5;
pub const BOSS_HIT_PERCENT_BY_MOTION_PREFIX: &[(&str, i32)] = &[
    ("kaine010_at_sword^t", 12),
    ("kaine010_at_sword^c", 6),
    ("kaine010_at_magic", 15),
];
pub const PVP_DAMAGE_PERCENT: i32 = 35;
pub const PLAYER_SWING_TARGET_KIND_BIT: i32 = 0x8;
const MOST_SENT_HITS_REMEMBERED: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SentHitVerdict {
    Apply { damage: i32, learned: bool },
    CountedNatively,
    Repeat,
}

#[derive(Debug, Default)]
pub struct BossNierHits {
    last_native_ms: Option<u64>,
    learned: Vec<(String, i32)>,
    sent: Vec<(u32, u64)>,
}

impl BossNierHits {
    pub const fn new() -> Self {
        Self {
            last_native_ms: None,
            learned: Vec::new(),
            sent: Vec::new(),
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Her own attack landed on the host's Nier through the game's damage path: its damage is
    /// what her motion deals, and a guest hit sent for the same swing must not count again.
    pub fn native_hit(&mut self, now_ms: u64, her_motion: &str, damage: i32) {
        self.last_native_ms = Some(now_ms);
        if damage <= 0 {
            return;
        }
        if let Some((_, known)) = self
            .learned
            .iter_mut()
            .find(|(motion, _)| motion == her_motion)
        {
            *known = damage;
        } else {
            self.learned.push((her_motion.to_owned(), damage));
        }
    }

    pub fn sent_hit(
        &mut self,
        now_ms: u64,
        hit_id: u32,
        her_motion: &str,
        nier_max_hp: i32,
    ) -> SentHitVerdict {
        self.sent
            .retain(|(_, at)| now_ms.saturating_sub(*at) < SENT_HIT_REPEAT_WINDOW_MS);
        if self.sent.iter().any(|(id, _)| *id == hit_id) {
            return SentHitVerdict::Repeat;
        }
        if self.sent.len() >= MOST_SENT_HITS_REMEMBERED {
            self.sent.remove(0);
        }
        self.sent.push((hit_id, now_ms));
        if self
            .last_native_ms
            .is_some_and(|at| now_ms.saturating_sub(at) < NATIVE_HIT_WINDOW_MS)
        {
            return SentHitVerdict::CountedNatively;
        }
        if let Some((_, damage)) = self.learned.iter().find(|(motion, _)| motion == her_motion) {
            return SentHitVerdict::Apply {
                damage: pvp_scaled_damage(*damage),
                learned: true,
            };
        }
        SentHitVerdict::Apply {
            damage: pvp_scaled_damage((nier_max_hp * percent_for_motion(her_motion) / 100).max(1)),
            learned: false,
        }
    }
}

pub fn percent_for_motion(her_motion: &str) -> i32 {
    BOSS_HIT_PERCENT_BY_MOTION_PREFIX
        .iter()
        .find(|(prefix, _)| her_motion.starts_with(prefix))
        .map_or(BOSS_HIT_DEFAULT_PERCENT_OF_NIER_MAX_HP, |(_, percent)| {
            *percent
        })
}

pub fn pvp_scaled_damage(damage: i32) -> i32 {
    if damage <= 0 {
        return damage;
    }
    (damage * PVP_DAMAGE_PERCENT / 100).max(1)
}
