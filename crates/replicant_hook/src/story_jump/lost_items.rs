use replicant_game::story_jump::{SET_ITEM_NUM_NATIVE_RVA, item_shortfalls};

use crate::build_gate::VerifiedBuild;
use crate::diagnostics::log_line;
use crate::pickups::{add_item_for_partner, announce_local_count, local_count};
use crate::story_jump::natives::call_native;

pub(super) fn give_back_lost_items(build: &VerifiedBuild, wanted: &[(u16, u8)]) {
    for shortfall in item_shortfalls(wanted, local_count) {
        let item_id = shortfall.item_id;
        let added = add_item_for_partner(item_id, shortfall.missing());
        let mut now = local_count(item_id).unwrap_or(0);
        if now < shortfall.wanted {
            call_native(
                build,
                SET_ITEM_NUM_NATIVE_RVA,
                &[i32::from(item_id), i32::from(shortfall.wanted)],
            );
            now = local_count(item_id).unwrap_or(0);
        }
        announce_local_count(item_id, now);
        log_line(&format!(
            "story jump: item {item_id} was {} after the load, wanted {}; added {} (add returned {added:?}), now {now}",
            shortfall.held,
            shortfall.wanted,
            shortfall.missing(),
        ));
    }
}
