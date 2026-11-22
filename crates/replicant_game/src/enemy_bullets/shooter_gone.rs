use std::collections::HashMap;

use replicant_proto::EnemyId;

use crate::enemy_bullets::MAX_SHOOTER_GONE_LINES_LOGGED;

#[derive(Debug, Clone, Copy)]
struct OrphanedBullet {
    enemy: EnemyId,
    since_ms: u64,
}

/// The host's log of bullets that keep flying after their shooter died or left the shared
/// enemies, and of when the game ends each one.
#[derive(Debug, Default)]
pub struct ShooterGoneTally {
    orphaned: HashMap<u16, OrphanedBullet>,
    lines: u32,
    lost_shooter: u32,
    ended_after: u32,
}

impl ShooterGoneTally {
    pub fn note_scan(&mut self, kept: &[(u16, EnemyId)], now_ms: u64) -> Vec<String> {
        let mut lines = Vec::new();
        let mut ended: Vec<(u16, OrphanedBullet)> = self
            .orphaned
            .iter()
            .filter(|(slot, _)| !kept.iter().any(|(kept_slot, _)| kept_slot == *slot))
            .map(|(slot, orphan)| (*slot, *orphan))
            .collect();
        ended.sort_by_key(|(slot, _)| *slot);
        for (slot, orphan) in ended {
            self.orphaned.remove(&slot);
            self.ended_after += 1;
            if self.admit_line() {
                lines.push(format!(
                    "enemy bullets: bullet {slot} of enemy {}:{} ended by the game {} ms after its shooter went away",
                    orphan.enemy.group,
                    orphan.enemy.ordinal,
                    now_ms.saturating_sub(orphan.since_ms)
                ));
            }
        }
        for (slot, enemy) in kept {
            if self.orphaned.contains_key(slot) {
                continue;
            }
            self.orphaned.insert(
                *slot,
                OrphanedBullet {
                    enemy: *enemy,
                    since_ms: now_ms,
                },
            );
            self.lost_shooter += 1;
            if self.admit_line() {
                lines.push(format!(
                    "enemy bullets: bullet {slot} of enemy {}:{} lost its shooter and keeps flying, followed until the game ends it",
                    enemy.group, enemy.ordinal
                ));
            }
        }
        lines
    }

    pub fn restart_line_budget(&mut self) {
        self.lines = 0;
    }

    pub fn take_summary(&mut self) -> Option<String> {
        if self.lost_shooter == 0 && self.ended_after == 0 {
            return None;
        }
        let line = format!(
            "enemy bullets: last 5 s {} bullets lost their shooter and kept flying, {} of those ended by the game, {} still flying",
            self.lost_shooter,
            self.ended_after,
            self.orphaned.len()
        );
        self.lost_shooter = 0;
        self.ended_after = 0;
        Some(line)
    }

    fn admit_line(&mut self) -> bool {
        if self.lines >= MAX_SHOOTER_GONE_LINES_LOGGED {
            return false;
        }
        self.lines += 1;
        true
    }
}
