use crate::enemies::{EMPTY_ENTITY_HANDLES, ENTITY_BEHAVIOR_POINTER_OFFSET};
use crate::late_join::{
    BOSS_GAUGE_FIRST_HANDLE_OFFSET, BOSS_GAUGE_READ_LEN, BOSS_GAUGE_SECOND_HANDLE_OFFSET,
    BOSS_GAUGE_SECOND_HANDLE_USED_OFFSET, ENTITY_DEAD_OR_DYING_BITS, ENTITY_HEADER_READ_LEN,
    ENTITY_LIVENESS_WORD_OFFSET, ENTITY_SLOT_HANDLE_GENERATION_MASK, ENTITY_SLOT_HANDLE_INDEX_MASK,
    ENTITY_SLOT_HANDLE_INDEX_SHIFT, UI_OBJECT_FLAGS_OFFSET, UI_OBJECT_HIGHEST_ALIVE_STATE,
    UI_OBJECT_LOWEST_ALIVE_STATE, UI_OBJECT_SHOWN_FLAG, UI_OBJECT_STATE_OFFSET,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BossGaugeKind {
    PerEnemy,
    HudManager,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BossGaugeObject {
    pub kind: BossGaugeKind,
    pub state: u32,
    pub flags: u32,
    pub handles: [u32; 2],
    pub second_handle_word: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetReading {
    Unreadable,
    Unresolved,
    Health(i32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BossFightConditions {
    pub state_is_alive: bool,
    pub is_shown: bool,
    pub target_resolves: bool,
    pub target_has_health: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BossFight {
    pub kind: BossGaugeKind,
    pub target_handle: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BossReading {
    NoBoss,
    Fight(BossFight),
    Unreadable(String),
}

impl BossGaugeKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::PerEnemy => "per-enemy gauge",
            Self::HudManager => "HUD gauge",
        }
    }
}

impl BossGaugeObject {
    pub fn from_bytes(kind: BossGaugeKind, bytes: &[u8]) -> Option<Self> {
        if bytes.len() < BOSS_GAUGE_READ_LEN {
            return None;
        }
        Some(Self {
            kind,
            state: u32_at(bytes, UI_OBJECT_STATE_OFFSET),
            flags: u32_at(bytes, UI_OBJECT_FLAGS_OFFSET),
            handles: [
                u32_at(bytes, BOSS_GAUGE_FIRST_HANDLE_OFFSET),
                u32_at(bytes, BOSS_GAUGE_SECOND_HANDLE_OFFSET),
            ],
            second_handle_word: u32_at(bytes, BOSS_GAUGE_SECOND_HANDLE_USED_OFFSET),
        })
    }

    pub fn state_is_alive(&self) -> bool {
        (UI_OBJECT_LOWEST_ALIVE_STATE..=UI_OBJECT_HIGHEST_ALIVE_STATE).contains(&self.state)
    }

    pub fn is_shown(&self) -> bool {
        self.flags & UI_OBJECT_SHOWN_FLAG != 0
    }

    pub fn is_worth_reading_targets(&self) -> bool {
        self.state_is_alive() && self.is_shown()
    }

    pub fn target_handles(&self) -> Vec<u32> {
        let second_is_used = self.kind == BossGaugeKind::HudManager && self.second_handle_word == 1;
        let used = if second_is_used {
            &self.handles[..]
        } else {
            &self.handles[..1]
        };
        used.iter()
            .copied()
            .filter(|handle| !EMPTY_ENTITY_HANDLES.contains(handle))
            .collect()
    }
}

impl BossFightConditions {
    pub fn all_hold(self) -> bool {
        self.state_is_alive && self.is_shown && self.target_resolves && self.target_has_health
    }
}

pub fn conditions_of(gauge: &BossGaugeObject, targets: &[TargetReading]) -> BossFightConditions {
    BossFightConditions {
        state_is_alive: gauge.state_is_alive(),
        is_shown: gauge.is_shown(),
        target_resolves: targets
            .iter()
            .any(|target| matches!(target, TargetReading::Health(_))),
        target_has_health: targets
            .iter()
            .any(|target| matches!(target, TargetReading::Health(health) if *health > 0)),
    }
}

pub fn boss_reading_of(gauges: &[(BossGaugeObject, Vec<TargetReading>)]) -> BossReading {
    let mut unreadable = None;
    for (gauge, targets) in gauges {
        if conditions_of(gauge, targets).all_hold() {
            let handles = gauge.target_handles();
            let target_handle = handles
                .iter()
                .zip(targets)
                .find(|(_, target)| matches!(target, TargetReading::Health(health) if *health > 0))
                .map_or(0, |(handle, _)| *handle);
            return BossReading::Fight(BossFight {
                kind: gauge.kind,
                target_handle,
            });
        }
        if gauge.is_worth_reading_targets()
            && targets.contains(&TargetReading::Unreadable)
            && unreadable.is_none()
        {
            unreadable = Some(format!(
                "a shown {} could not read its target",
                gauge.kind.label()
            ));
        }
    }
    unreadable.map_or(BossReading::NoBoss, BossReading::Unreadable)
}

pub fn handle_slot_index(handle: u32) -> u32 {
    (handle >> ENTITY_SLOT_HANDLE_INDEX_SHIFT) & ENTITY_SLOT_HANDLE_INDEX_MASK
}

pub fn slot_holds_handle(handle: u32, slot_first_word: u32) -> bool {
    (slot_first_word ^ handle) & ENTITY_SLOT_HANDLE_GENERATION_MASK == 0
}

pub fn entity_body_pointer(header: &[u8]) -> Option<u64> {
    if header.len() < ENTITY_HEADER_READ_LEN {
        return None;
    }
    let liveness = u32_at(header, ENTITY_LIVENESS_WORD_OFFSET);
    if liveness & ENTITY_DEAD_OR_DYING_BITS != 0 {
        return None;
    }
    let offset = ENTITY_BEHAVIOR_POINTER_OFFSET as usize;
    let mut bytes = [0_u8; 8];
    bytes.copy_from_slice(&header[offset..offset + 8]);
    Some(u64::from_le_bytes(bytes)).filter(|body| *body != 0)
}

fn u32_at(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}
