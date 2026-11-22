use crate::outfit_look::{LOOK_UNPICKED, PlayerLook};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PartnerLookReading {
    pub outfit_body: i32,
    pub accessory_body: i32,
    pub outfit_saved: i32,
    pub accessory_saved: i32,
    pub pod_saved: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartnerLookStep {
    WriteOutfit(i32),
    WriteAccessory(i32),
    SwapPodSkin { value: i32, restore: i32 },
}

/// What this game has already put on one partner body, so a look that cannot be read back is not repeated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PartnerLookApplied {
    pub parts_touched: bool,
    pub pod_skin: Option<i32>,
}

/// The hair colour the partner body's update must see, and the character slot whose global it replaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HairSwap {
    pub slot: usize,
    pub value: i32,
}

impl PartnerLookApplied {
    pub fn note(&mut self, step: PartnerLookStep) {
        match step {
            PartnerLookStep::WriteOutfit(_) | PartnerLookStep::WriteAccessory(_) => {
                self.parts_touched = true;
            }
            PartnerLookStep::SwapPodSkin { value, restore } => {
                self.pod_skin = (value != restore).then_some(value);
            }
        }
    }

    pub fn forget(&mut self) {
        *self = Self::default();
    }
}

impl HairSwap {
    pub const fn needs_swap(&self, current_global: i32) -> bool {
        self.value != current_global
    }
}

pub fn partner_hair_swap(look: &PlayerLook, slot: usize) -> Option<HairSwap> {
    picked(look.hair_colour).map(|value| HairSwap { slot, value })
}

pub fn partner_look_steps(
    look: &PlayerLook,
    reading: &PartnerLookReading,
    applied: &PartnerLookApplied,
) -> Vec<PartnerLookStep> {
    let mut steps = Vec::new();
    let outfit = wanted_part(look.outfit_variant, reading.outfit_saved, applied);
    if let Some(outfit) = outfit.filter(|wanted| *wanted != reading.outfit_body) {
        steps.push(PartnerLookStep::WriteOutfit(outfit));
    }
    let accessory = wanted_part(look.accessory_code, reading.accessory_saved, applied);
    if let Some(accessory) = accessory.filter(|wanted| *wanted != reading.accessory_body) {
        steps.push(PartnerLookStep::WriteAccessory(accessory));
    }
    let parts_came_back = applied.parts_touched && !steps.is_empty();
    if let Some(pod) = pod_wanted(look, reading, applied, parts_came_back) {
        steps.push(PartnerLookStep::SwapPodSkin {
            value: pod,
            restore: reading.pod_saved,
        });
    }
    steps
}

fn picked(value: u8) -> Option<i32> {
    (value != LOOK_UNPICKED).then_some(i32::from(value))
}

fn wanted_part(wire: u8, saved: i32, applied: &PartnerLookApplied) -> Option<i32> {
    picked(wire).or_else(|| applied.parts_touched.then_some(saved))
}

fn pod_wanted(
    look: &PlayerLook,
    reading: &PartnerLookReading,
    applied: &PartnerLookApplied,
    parts_came_back: bool,
) -> Option<i32> {
    match (picked(look.pod_skin), applied.pod_skin) {
        (Some(pod), applied_pod) => (applied_pod != Some(pod) || parts_came_back).then_some(pod),
        (None, Some(_)) => Some(reading.pod_saved),
        (None, None) => None,
    }
}
