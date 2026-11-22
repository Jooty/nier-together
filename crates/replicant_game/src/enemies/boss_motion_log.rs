use std::collections::HashMap;

use replicant_proto::EnemyId;

use crate::puppet::LayerMotion;

pub const BOSS_MOTION_LINES_PER_TICK: usize = 4;

#[derive(Debug, Default)]
pub struct BossMotionLog {
    last: HashMap<EnemyId, Option<i32>>,
}

impl BossMotionLog {
    pub fn observe(
        &mut self,
        side: &str,
        bosses: impl IntoIterator<Item = (EnemyId, Option<LayerMotion>)>,
    ) -> Vec<String> {
        let mut lines = Vec::new();
        for (id, motion) in bosses {
            let index = motion.map(|motion| motion.index);
            let before = self.last.insert(id, index);
            if before.is_some_and(|before| before != index)
                && lines.len() < BOSS_MOTION_LINES_PER_TICK
            {
                lines.push(boss_motion_changed_text(side, id, before.flatten(), motion));
            }
        }
        lines
    }
}

pub fn boss_motion_changed_text(
    side: &str,
    id: EnemyId,
    from: Option<i32>,
    to: Option<LayerMotion>,
) -> String {
    let show = |index: Option<i32>| index.map_or_else(|| "none".to_owned(), |i| i.to_string());
    let detail = to.map_or_else(String::new, |motion| {
        format!(
            ", at frame {:.1} speed {:.2}{}",
            motion.frame,
            motion.speed,
            if motion.looping { " looping" } else { "" }
        )
    });
    format!(
        "boss body ({side}): {}:{} motion {} -> {}{detail}",
        id.group,
        id.ordinal,
        show(from),
        show(to.map(|motion| motion.index))
    )
}
