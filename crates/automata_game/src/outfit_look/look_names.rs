use std::collections::BTreeMap;

use crate::outfit_look::{CLOTH_STATE_ITEM_ID, LOOK_ITEMS};

pub const NAME_READ_ATTEMPTS: u8 = 3;

/// The game's own names of the look items, read once on the game thread.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LookNames {
    names: BTreeMap<i32, String>,
    misses: BTreeMap<i32, u8>,
}

impl LookNames {
    pub fn unread_ids(&self, limit: usize) -> Vec<i32> {
        LOOK_ITEMS
            .iter()
            .map(|item| item.id)
            .chain([CLOTH_STATE_ITEM_ID])
            .filter(|id| {
                !self.names.contains_key(id)
                    && self.misses.get(id).copied().unwrap_or(0) < NAME_READ_ATTEMPTS
            })
            .take(limit)
            .collect()
    }

    pub fn record(&mut self, id: i32, name: Option<String>) {
        match name.map(|name| name.trim().to_owned()) {
            Some(name) if !name.is_empty() => {
                self.names.insert(id, name);
            }
            _ => *self.misses.entry(id).or_insert(0) += 1,
        }
    }

    pub fn name_of(&self, id: i32) -> Option<&str> {
        self.names.get(&id).map(String::as_str)
    }

    pub fn label(&self, id: i32) -> String {
        self.name_of(id)
            .map_or_else(|| id.to_string(), str::to_owned)
    }
}
