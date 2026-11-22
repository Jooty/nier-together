use replicant_proto::EnemyId;

pub const BOSS_FIGHT_GROUP: u16 = 1022;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BossLifeStep {
    Leave,
    Set {
        to: i32,
        host_hp: u32,
        host_max_hp: u32,
    },
}

#[derive(Debug, Default)]
pub struct BossLifeFollow {
    last_set_to: Option<i32>,
}

impl BossLifeFollow {
    pub const fn new() -> Self {
        Self { last_set_to: None }
    }

    pub fn reset(&mut self) {
        self.last_set_to = None;
    }

    /// `host` is the boss Kaine's `(hp, max_hp)` in the host's last snapshot; the step says what
    /// the guest's own player life should be written as, or `Leave` when it already is that.
    pub fn step(&mut self, host: Option<(u32, u32)>, own_hp: i32, own_max_hp: i32) -> BossLifeStep {
        let Some((host_hp, host_max_hp)) = host else {
            return BossLifeStep::Leave;
        };
        let Some(to) = guest_life_for_boss_life(host_hp, host_max_hp, own_max_hp) else {
            return BossLifeStep::Leave;
        };
        if own_hp == to {
            return BossLifeStep::Leave;
        }
        self.last_set_to = Some(to);
        BossLifeStep::Set {
            to,
            host_hp,
            host_max_hp,
        }
    }

    pub fn last_set_to(&self) -> Option<i32> {
        self.last_set_to
    }
}

pub fn is_the_boss_kaine(id: EnemyId) -> bool {
    id.group == BOSS_FIGHT_GROUP
}

/// The guest's life as the same share of its own maximum that the boss Kaine has of hers. A
/// boss at zero is dying, which the fight's own end handles, so no life is asked for then.
pub fn guest_life_for_boss_life(host_hp: u32, host_max_hp: u32, own_max_hp: i32) -> Option<i32> {
    if host_hp == 0 || host_max_hp == 0 || own_max_hp <= 0 {
        return None;
    }
    let share = f64::from(host_hp.min(host_max_hp)) / f64::from(host_max_hp);
    let wanted = (share * f64::from(own_max_hp)).ceil() as i32;
    Some(wanted.clamp(1, own_max_hp))
}
