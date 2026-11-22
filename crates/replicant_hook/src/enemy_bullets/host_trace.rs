use replicant_game::enemy_bullets::{
    EnemyBullet, HostBulletTrace, HostSighting, read_bullet_motion_fields,
};

use crate::diagnostics::log_line;
use crate::memory::InProcessMemory;

const TRANSLATION_COLUMN: usize = 3;
const TRANSFORM_ROW_FLOATS: usize = 4;

pub(super) struct ScanFlights<'a> {
    pub started: &'a [u16],
    pub sent: &'a [u16],
    pub ended: &'a [u16],
}

pub(super) fn log_host_trace(
    trace: &mut HostBulletTrace,
    memory: &InProcessMemory,
    bullets: &[EnemyBullet],
    flights: &ScanFlights,
    now_ms: u64,
) {
    for slot in flights.ended {
        trace.stop(*slot);
    }
    for slot in flights.started {
        if let Some(seen) = bullet_in_slot(bullets, *slot) {
            trace.start(*slot, position_of(seen), now_ms);
        }
    }
    for slot in trace.slots() {
        let Some(seen) = bullet_in_slot(bullets, slot) else {
            continue;
        };
        let sighting = HostSighting {
            slot,
            hit_id: seen.bullet.hit_id,
            position: position_of(seen),
            fields: read_bullet_motion_fields(memory, seen.bullet.actor),
        };
        if let Some(line) = trace.line(&sighting, flights.sent.contains(&slot), now_ms) {
            log_line(&line);
        }
    }
}

fn bullet_in_slot(bullets: &[EnemyBullet], slot: u16) -> Option<&EnemyBullet> {
    bullets.iter().find(|seen| seen.bullet.slot == slot)
}

fn position_of(seen: &EnemyBullet) -> [f32; 3] {
    core::array::from_fn(|axis| {
        seen.bullet.transform[axis * TRANSFORM_ROW_FLOATS + TRANSLATION_COLUMN]
    })
}
