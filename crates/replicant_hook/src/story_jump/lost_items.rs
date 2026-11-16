use std::sync::{Mutex, PoisonError};

use replicant_game::story_jump::{
    SET_ITEM_NUM_NATIVE_RVA, item_audit_is_due, item_shortfalls, unreadable_items,
};

use crate::build_gate::VerifiedBuild;
use crate::diagnostics::log_line;
use crate::net::net_clock_ms;
use crate::pickups::{add_item_for_partner, announce_local_count, local_count};
use crate::story_jump::natives::call_native;

static AUDIT: Mutex<Option<ItemAudit>> = Mutex::new(None);

struct ItemAudit {
    wanted: Vec<(u16, u8)>,
    started_ms: u64,
}

pub(super) fn schedule_item_audit(wanted: Vec<(u16, u8)>) {
    *AUDIT.lock().unwrap_or_else(PoisonError::into_inner) = Some(ItemAudit {
        wanted,
        started_ms: net_clock_ms(),
    });
}

pub(super) fn tick_item_audit(build: &VerifiedBuild) {
    let mut guard = AUDIT.lock().unwrap_or_else(PoisonError::into_inner);
    let Some(audit) = guard.as_ref() else {
        return;
    };
    if !item_audit_is_due(net_clock_ms().saturating_sub(audit.started_ms)) {
        return;
    }
    let wanted = audit.wanted.clone();
    *guard = None;
    drop(guard);
    log_line("story jump: auditing the jump's shared items after the restore point");
    give_back_lost_items(build, &wanted);
}

pub(super) fn give_back_lost_items(build: &VerifiedBuild, wanted: &[(u16, u8)]) {
    for item_id in unreadable_items(wanted, local_count) {
        log_line(&format!(
            "story jump: item {item_id} count is unreadable, nothing can be checked or given back"
        ));
    }
    for &(item_id, wanted_count) in wanted {
        if let Some(held) = local_count(item_id) {
            log_line(&format!(
                "story jump: item {item_id} held {held}, wanted {wanted_count}"
            ));
        }
    }
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
