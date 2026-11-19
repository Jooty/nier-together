use crate::game::replicant::text_up_to_nul;
use crate::game::{GameError, Replicant};
use crate::memory::GameMemory;
use crate::offsets::{
    ENTITY_ARRIVAL_POINT_OFFSET, ENTITY_ZONE_OFFSET, MAP_MANAGER_ACTIVE_CONTEXT_POINTER_OFFSET,
    MAP_MANAGER_RVA, PHASE_NAME_BUFFER_LEN,
};
use crate::phase_request::{
    LOADED_TALK_SET_NAME_READ_LEN, NEXT_PHASE_NAME_RVA, TALK_WINDOW_LOADED_SET_NAME_OFFSET,
    TALK_WINDOW_RVA, loaded_talk_set_letter,
};

impl<M: GameMemory> Replicant<M> {
    pub fn current_phase(&self) -> Result<String, GameError> {
        let bytes = self
            .memory()
            .read_array::<PHASE_NAME_BUFFER_LEN>(self.entity_field(ENTITY_ZONE_OFFSET))?;
        Ok(text_up_to_nul(&bytes))
    }

    pub fn arrival_point(&self) -> Result<i32, GameError> {
        Ok(self
            .memory()
            .read_i32(self.entity_field(ENTITY_ARRIVAL_POINT_OFFSET))?)
    }

    pub fn map_manager_address(&self) -> u64 {
        self.module_address(MAP_MANAGER_RVA)
    }

    pub fn map_manager_is_active(&self) -> Result<bool, GameError> {
        let pointer_address =
            self.map_manager_address() + MAP_MANAGER_ACTIVE_CONTEXT_POINTER_OFFSET;
        Ok(self.memory().read_u64(pointer_address)? != 0)
    }

    pub fn put_loaded_talk_set_letter_in_next_phase_name(&mut self) -> Result<(), GameError> {
        let set_name_bytes = self.memory().read_array::<LOADED_TALK_SET_NAME_READ_LEN>(
            self.module_address(TALK_WINDOW_RVA) + TALK_WINDOW_LOADED_SET_NAME_OFFSET,
        )?;
        let set_name = text_up_to_nul(&set_name_bytes);
        let Some(letter) = loaded_talk_set_letter(&set_name) else {
            return Ok(());
        };
        let address = self.module_address(NEXT_PHASE_NAME_RVA);
        let mut buffer = self.memory().read_array::<PHASE_NAME_BUFFER_LEN>(address)?;
        if buffer[0] == 0 {
            buffer[1] = b'_';
        }
        buffer[0] = letter;
        self.memory_mut().write(address, &buffer)?;
        Ok(())
    }
}
