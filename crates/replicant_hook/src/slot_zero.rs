use core::ffi::c_void;

use replicant_game::GameMemory;
use replicant_game::dialogue::{CONTROL_MASK_SET_SLOT_ZERO_RVA, GAME_STATE_RVA};
use replicant_game::marker::CONTROL_MASK_FIRST_RVA;

use crate::build_gate::VerifiedBuild;
use crate::memory::InProcessMemory;

type SetMaskSlot = unsafe extern "system" fn(*mut c_void, u32);

pub(crate) fn read_slot_zero(build: &VerifiedBuild) -> Option<u32> {
    let memory = InProcessMemory::new(build);
    memory
        .read_i32(memory.module_base() + u64::from(CONTROL_MASK_FIRST_RVA))
        .ok()
        .map(i32::cast_unsigned)
}

#[expect(
    unsafe_code,
    reason = "calls the control mask slot 0 setter by address"
)]
pub(crate) fn write_slot_zero(build: &VerifiedBuild, value: u32) {
    let base = InProcessMemory::new(build).module_base();
    // SAFETY: the slot 0 setter (game state, value) in the verified build, called on the game
    // thread as the stuck menu mask recovery does.
    unsafe {
        core::mem::transmute::<usize, SetMaskSlot>(
            (base + u64::from(CONTROL_MASK_SET_SLOT_ZERO_RVA)) as usize,
        )((base + u64::from(GAME_STATE_RVA)) as *mut c_void, value);
    }
}
