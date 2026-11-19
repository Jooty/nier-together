use replicant_proto::StoryJumpPoint;

use crate::pickups::is_shared_item;
use crate::story_jump::{JumpStep, loadout_steps};

pub const ITEM_AUDIT_AFTER_RESTORE_POINT_MS: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemShortfall {
    pub item_id: u16,
    pub held: u8,
    pub wanted: u8,
}

impl ItemShortfall {
    pub const fn missing(self) -> u8 {
        self.wanted.saturating_sub(self.held)
    }
}

pub fn shared_items_the_jump_gives(point: StoryJumpPoint) -> Vec<(u16, u8)> {
    let mut wanted: Vec<(u16, u8)> = Vec::new();
    for step in loadout_steps(point) {
        let JumpStep::SetItem { id, count } = step else {
            continue;
        };
        let (Ok(item_id), Ok(count)) = (u16::try_from(id), u8::try_from(count)) else {
            continue;
        };
        if !is_shared_item(item_id) {
            continue;
        }
        wanted.retain(|(known, _)| *known != item_id);
        if count > 0 {
            wanted.push((item_id, count));
        }
    }
    wanted
}

pub fn unreadable_items(wanted: &[(u16, u8)], held: impl Fn(u16) -> Option<u8>) -> Vec<u16> {
    wanted
        .iter()
        .filter(|&&(item_id, _)| held(item_id).is_none())
        .map(|&(item_id, _)| item_id)
        .collect()
}

pub fn item_audit_is_due(ms_since_restore_point: u64) -> bool {
    ms_since_restore_point >= ITEM_AUDIT_AFTER_RESTORE_POINT_MS
}

pub fn item_shortfalls(
    wanted: &[(u16, u8)],
    held: impl Fn(u16) -> Option<u8>,
) -> Vec<ItemShortfall> {
    wanted
        .iter()
        .filter_map(|&(item_id, wanted)| {
            let held = held(item_id)?;
            (held < wanted).then_some(ItemShortfall {
                item_id,
                held,
                wanted,
            })
        })
        .collect()
}
