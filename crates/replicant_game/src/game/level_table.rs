use crate::game::{GameError, Replicant};
use crate::level_up::{
    LEVEL_TABLE_EXPERIENCE_TO_LEAVE_LEVEL_OFFSET, LEVEL_TABLE_ROW_SIZE,
    LEVEL_TABLE_ROWS_RELATIVE_OFFSET_FIELD, PLAYER_PARAM_LEVEL_TABLE_POINTER_OFFSET,
};
use crate::memory::GameMemory;
use crate::offsets::PLAYER_PARAM_ARRAY_RVA;

impl<M: GameMemory> Replicant<M> {
    /// What the game's level-up redo (0x140407070) compares against to leave `level`: row `level` of
    /// the local player param's table (param+0x58), field +0x6C. `None` where the game uses 9999999.
    pub fn experience_to_leave_level(&self, level: i32) -> Result<Option<i32>, GameError> {
        let player_param = self.module_address(PLAYER_PARAM_ARRAY_RVA);
        let table = self
            .memory()
            .read_u64(player_param + PLAYER_PARAM_LEVEL_TABLE_POINTER_OFFSET)?;
        if table == 0 || level < 0 {
            return Ok(None);
        }
        let rows_field = table + LEVEL_TABLE_ROWS_RELATIVE_OFFSET_FIELD;
        let rows_offset = self.memory().read_i32(rows_field)?;
        if rows_offset == 0 {
            return Ok(None);
        }
        let rows = rows_field.wrapping_add_signed(i64::from(rows_offset));
        let row = rows.wrapping_add_signed(i64::from(level) * LEVEL_TABLE_ROW_SIZE);
        let field = row.wrapping_add_signed(LEVEL_TABLE_EXPERIENCE_TO_LEAVE_LEVEL_OFFSET);
        Ok(Some(self.memory().read_i32(field)?))
    }
}
