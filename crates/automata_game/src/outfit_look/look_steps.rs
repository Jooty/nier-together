use crate::outfit_look::{LOOK_KIND_COUNT, LookKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LookReading {
    pub outfit_global: i32,
    pub outfit_body: i32,
    pub accessory_global: i32,
    pub accessory_body: i32,
    pub hair: i32,
    pub pod_skin: i32,
    pub gun_skin: i32,
    pub cloth_state: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LookStep {
    SetOutfit(i32),
    SetAccessory(i32),
    SetHair(i32),
    SetPodSkin(i32),
    SetGunSkin(i32),
    SetClothState(i32),
}

impl LookStep {
    pub const fn kind(self) -> LookKind {
        match self {
            Self::SetOutfit(_) => LookKind::Outfit,
            Self::SetAccessory(_) => LookKind::Accessory,
            Self::SetHair(_) => LookKind::Hair,
            Self::SetPodSkin(_) => LookKind::PodSkin,
            Self::SetGunSkin(_) => LookKind::GunSkin,
            Self::SetClothState(_) => LookKind::ClothState,
        }
    }

    pub const fn value(self) -> i32 {
        match self {
            Self::SetOutfit(value)
            | Self::SetAccessory(value)
            | Self::SetHair(value)
            | Self::SetPodSkin(value)
            | Self::SetGunSkin(value)
            | Self::SetClothState(value) => value,
        }
    }
}

pub fn look_steps(
    targets: &[Option<i32>; LOOK_KIND_COUNT],
    reading: &LookReading,
) -> Vec<LookStep> {
    let mut steps = Vec::new();
    for kind in LookKind::ALL {
        let Some(target) = targets[kind.index()] else {
            continue;
        };
        let differs = match kind {
            LookKind::Outfit => reading.outfit_global != target || reading.outfit_body != target,
            LookKind::Accessory => {
                reading.accessory_global != target || reading.accessory_body != target
            }
            LookKind::Hair => reading.hair != target,
            LookKind::PodSkin => reading.pod_skin != target,
            LookKind::GunSkin => reading.gun_skin != target,
            LookKind::ClothState => reading.cloth_state != target,
        };
        if differs {
            steps.push(step_for(kind, target));
        }
    }
    steps
}

const fn step_for(kind: LookKind, value: i32) -> LookStep {
    match kind {
        LookKind::Outfit => LookStep::SetOutfit(value),
        LookKind::Accessory => LookStep::SetAccessory(value),
        LookKind::Hair => LookStep::SetHair(value),
        LookKind::PodSkin => LookStep::SetPodSkin(value),
        LookKind::GunSkin => LookStep::SetGunSkin(value),
        LookKind::ClothState => LookStep::SetClothState(value),
    }
}
