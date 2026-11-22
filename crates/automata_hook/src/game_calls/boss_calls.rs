use automata_game::address::{
    ENTITY_TABLE_POINTER_VALUE, ENTITY_TABLE_SLOT_COUNT_VALUE, UI_BOSS_GAUGE_HUD_VFTABLE,
    UI_BOSS_GAUGE_PER_ENEMY_VFTABLE, UI_REGISTRY_HEADER_VALUE, UI_REGISTRY_IS_KEPT_FLAG_VALUE,
};
use automata_game::enemies::ENTITY_TABLE_SLOT_LEN;
use automata_game::late_join::{
    BOSS_GAUGE_READ_LEN, BossGaugeKind, BossGaugeObject, BossReading, ENTITY_HEADER_READ_LEN,
    MOST_UI_OBJECTS_SCANNED, TargetReading, UI_REGISTRY_COUNT_OFFSET, UI_REGISTRY_HEADER_LEN,
    boss_reading_of, entity_body_pointer, handle_slot_index, slot_holds_handle,
};

use crate::game_calls::GameCalls;

const REGISTRY_WORD_LEN: usize = 8;

struct EntityTableRead {
    slot_count: u32,
    table: u64,
}

impl GameCalls {
    /// Guessed boss-fight reading: a boss gauge in the UI registry whose target is alive. Reads only.
    /// Reads: 3 fixed, one per registry object and per gauge found, then 2 shared and 3 per target.
    pub fn read_boss_fight(&self) -> BossReading {
        match self.read_boss_gauges() {
            Ok(gauges) => boss_reading_of(&gauges),
            Err(reason) => BossReading::Unreadable(reason.to_owned()),
        }
    }

    fn read_boss_gauges(&self) -> Result<Vec<(BossGaugeObject, Vec<TargetReading>)>, &'static str> {
        let kept = self
            .read_u32(UI_REGISTRY_IS_KEPT_FLAG_VALUE)
            .map_err(|_| "the UI registry flag is unreadable")?;
        if kept == 0 {
            return Err("the game keeps no UI registry");
        }
        let header = self
            .read_bytes(UI_REGISTRY_HEADER_VALUE, UI_REGISTRY_HEADER_LEN)
            .map_err(|_| "the UI registry header is unreadable")?;
        let array = u64_of(&header, 0);
        let count = u64_of(&header, UI_REGISTRY_COUNT_OFFSET) as u32 as usize;
        if array == 0 || count > MOST_UI_OBJECTS_SCANNED {
            return Err("the UI registry header holds no sane array");
        }
        let objects = self
            .read_bytes_at(array, count * REGISTRY_WORD_LEN)
            .map_err(|_| "the UI registry array is unreadable")?;
        let per_enemy = self.address_of(UI_BOSS_GAUGE_PER_ENEMY_VFTABLE) as u64;
        let hud = self.address_of(UI_BOSS_GAUGE_HUD_VFTABLE) as u64;
        let mut entity_table = None;
        let mut gauges = Vec::new();
        for index in 0..count {
            let object = u64_of(&objects, index * REGISTRY_WORD_LEN);
            let Some(kind) = self.gauge_kind_of(object, per_enemy, hud) else {
                continue;
            };
            let Some(gauge) = self
                .read_bytes_at(object, BOSS_GAUGE_READ_LEN)
                .ok()
                .and_then(|bytes| BossGaugeObject::from_bytes(kind, &bytes))
            else {
                continue;
            };
            let targets = if gauge.is_worth_reading_targets() {
                self.read_gauge_targets(&gauge, &mut entity_table)
            } else {
                Vec::new()
            };
            gauges.push((gauge, targets));
        }
        Ok(gauges)
    }

    fn gauge_kind_of(&self, object: u64, per_enemy: u64, hud: u64) -> Option<BossGaugeKind> {
        if object == 0 {
            return None;
        }
        let vftable = self.read_u64_at(object).ok()?;
        if vftable == per_enemy {
            Some(BossGaugeKind::PerEnemy)
        } else if vftable == hud {
            Some(BossGaugeKind::HudManager)
        } else {
            None
        }
    }

    fn read_gauge_targets(
        &self,
        gauge: &BossGaugeObject,
        entity_table: &mut Option<Option<EntityTableRead>>,
    ) -> Vec<TargetReading> {
        let table = entity_table
            .get_or_insert_with(|| {
                Some(EntityTableRead {
                    slot_count: self.read_u32(ENTITY_TABLE_SLOT_COUNT_VALUE).ok()?,
                    table: self.read_u64(ENTITY_TABLE_POINTER_VALUE).ok()?,
                })
            })
            .as_ref();
        gauge
            .target_handles()
            .into_iter()
            .map(|handle| match table {
                Some(table) => self.read_target_health(handle, table),
                None => TargetReading::Unreadable,
            })
            .collect()
    }

    fn read_target_health(&self, handle: u32, table: &EntityTableRead) -> TargetReading {
        let index = handle_slot_index(handle);
        if index >= table.slot_count {
            return TargetReading::Unresolved;
        }
        let slot_address = table.table + u64::from(index) * ENTITY_TABLE_SLOT_LEN as u64;
        let Ok(slot) = self.read_bytes_at(slot_address, ENTITY_TABLE_SLOT_LEN) else {
            return TargetReading::Unreadable;
        };
        if !slot_holds_handle(handle, u64_of(&slot, 0) as u32) {
            return TargetReading::Unresolved;
        }
        let entity = u64_of(&slot, 8);
        if entity == 0 {
            return TargetReading::Unresolved;
        }
        let Ok(header) = self.read_bytes_at(entity, ENTITY_HEADER_READ_LEN) else {
            return TargetReading::Unreadable;
        };
        let Some(body) = entity_body_pointer(&header) else {
            return TargetReading::Unresolved;
        };
        self.read_enemy_health(body as usize)
            .map_or(TargetReading::Unreadable, TargetReading::Health)
    }
}

fn u64_of(bytes: &[u8], offset: usize) -> u64 {
    let mut word = [0_u8; 8];
    word.copy_from_slice(&bytes[offset..offset + 8]);
    u64::from_le_bytes(word)
}
