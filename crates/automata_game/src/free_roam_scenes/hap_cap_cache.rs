use std::collections::HashMap;

use crate::free_roam_scenes::{HapWalkCap, MOST_HAPS_REMEMBERED, hap_walk_cap};

#[derive(Debug, Clone, Copy)]
struct HapJudgement {
    list: u64,
    action_count: u32,
    cap: Option<HapWalkCap>,
}

#[derive(Debug, Default)]
pub struct HapCapCache {
    judged: HashMap<u64, HapJudgement>,
}

impl HapCapCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cap_for(
        &mut self,
        hap: u64,
        list: u64,
        action_count: u32,
        cursor: u32,
        action_id_at: impl FnMut(u32) -> Option<u32>,
    ) -> Option<HapWalkCap> {
        let known = self
            .judged
            .get(&hap)
            .filter(|judgement| judgement.list == list && judgement.action_count == action_count)
            .copied();
        let judgement = match known {
            Some(judgement) => judgement,
            None => {
                if self.judged.len() >= MOST_HAPS_REMEMBERED {
                    self.judged.clear();
                }
                let judgement = HapJudgement {
                    list,
                    action_count,
                    cap: hap_walk_cap(true, action_count, 0, action_id_at),
                };
                self.judged.insert(hap, judgement);
                judgement
            }
        };
        judgement.cap.filter(|cap| cap.index >= cursor)
    }
}
