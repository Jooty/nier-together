use serde::{Deserialize, Serialize};

use crate::start::MAX_CHAPTER_RECORDS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionJumpOrder {
    pub start_index: u16,
}

impl SessionJumpOrder {
    pub fn of_start_index(start_index: usize) -> Option<Self> {
        let start_index = u16::try_from(start_index).ok()?;
        let order = Self { start_index };
        order.is_valid().then_some(order)
    }

    pub fn is_valid(&self) -> bool {
        usize::from(self.start_index) < MAX_CHAPTER_RECORDS
    }
}
