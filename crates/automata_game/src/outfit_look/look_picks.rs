use crate::outfit_look::{LOOK_KIND_COUNT, LOOK_SLOT_COUNT, LOOK_UNPICKED, LookKind, PlayerLook};

type SlotValues = [[Option<i32>; LOOK_KIND_COUNT]; LOOK_SLOT_COUNT];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SavedLook {
    pub slot: usize,
    pub kind: LookKind,
    pub value: i32,
}

/// What the player picked for each character, and the value the save had before the first pick of each kind.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LookPicks {
    picked: SlotValues,
    saved: SlotValues,
}

impl LookPicks {
    pub fn pick(&mut self, slot: usize, kind: LookKind, value: Option<i32>, current_global: i32) {
        let Some(cell) = self.picked.get_mut(slot) else {
            return;
        };
        cell[kind.index()] = value;
        if value.is_some()
            && let Some(saved) = self.saved.get_mut(slot)
        {
            saved[kind.index()].get_or_insert(current_global);
        }
    }

    pub fn picked(&self, slot: usize, kind: LookKind) -> Option<i32> {
        self.picked.get(slot)?[kind.index()]
    }

    pub fn is_saved(&self, slot: usize, kind: LookKind) -> bool {
        self.saved
            .get(slot)
            .is_some_and(|saved| saved[kind.index()].is_some())
    }

    pub fn wanted(&self, slot: usize, kind: LookKind, allowed: bool) -> Option<i32> {
        if allowed {
            self.picked(slot, kind)
        } else {
            None
        }
    }

    pub fn wanted_look(&self, slot: usize, allowed: bool) -> PlayerLook {
        let value = |kind| {
            self.wanted(slot, kind, allowed)
                .and_then(|value| u8::try_from(value).ok())
                .unwrap_or(LOOK_UNPICKED)
        };
        PlayerLook {
            outfit_variant: value(LookKind::Outfit),
            accessory_code: value(LookKind::Accessory),
            hair_colour: value(LookKind::Hair),
            pod_skin: value(LookKind::PodSkin),
            gun_skin: value(LookKind::GunSkin),
        }
    }

    pub fn take_targets(&mut self, slot: usize, allowed: bool) -> [Option<i32>; LOOK_KIND_COUNT] {
        LookKind::ALL.map(|kind| {
            if let Some(wanted) = self.wanted(slot, kind, allowed) {
                return Some(wanted);
            }
            self.saved.get_mut(slot)?[kind.index()].take()
        })
    }

    pub fn globals_after_load(&self, allowed: bool) -> Vec<(usize, LookKind, i32)> {
        let mut writes = Vec::new();
        for slot in 0..LOOK_SLOT_COUNT {
            for kind in LookKind::ALL {
                if let Some(value) = self.wanted(slot, kind, allowed) {
                    writes.push((slot, kind, value));
                }
            }
        }
        writes
    }

    pub fn saved_looks(&self) -> Vec<SavedLook> {
        let mut looks = Vec::new();
        for (slot, saved) in self.saved.iter().enumerate() {
            for kind in LookKind::ALL {
                if let Some(value) = saved[kind.index()] {
                    looks.push(SavedLook { slot, kind, value });
                }
            }
        }
        looks
    }

    pub fn end_session(&mut self) {
        self.picked = SlotValues::default();
    }

    pub fn take_saved_looks(&mut self) -> Vec<SavedLook> {
        let looks = self.saved_looks();
        self.saved = SlotValues::default();
        looks
    }
}
