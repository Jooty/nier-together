use replicant_proto::StoryJumpPoint;

use crate::pickups::is_shared_item;
use crate::story_jump::{JumpStep, loadout_steps};

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
