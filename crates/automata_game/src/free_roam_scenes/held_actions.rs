use crate::free_roam_scenes::MOST_HELD_ACTION_IDS_REMEMBERED;

#[derive(Debug, Clone, Default)]
pub struct HeldActions {
    ids: Vec<u32>,
}

impl HeldActions {
    pub const fn new() -> Self {
        Self { ids: Vec::new() }
    }

    pub fn is_first_hold_of(&mut self, id: u32) -> bool {
        if self.ids.contains(&id) || self.ids.len() >= MOST_HELD_ACTION_IDS_REMEMBERED {
            return false;
        }
        self.ids.push(id);
        true
    }

    pub fn actions_held(&self) -> usize {
        self.ids.len()
    }
}
