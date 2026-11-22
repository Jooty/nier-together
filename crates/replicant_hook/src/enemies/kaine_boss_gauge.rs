use core::ffi::c_void;

use replicant_game::GameMemory;
use replicant_game::build::SUPPORTED_PE_IDENTITY;
use replicant_game::enemies::{
    BOSS_BAR_HUD_OBJECT_RVA, boss_bar_state_is_open, read_boss_bar_state,
};

use crate::build_gate::VerifiedBuild;
use crate::diagnostics::log_line;
use crate::memory::InProcessMemory;

const HUD_CLOSE_BOSS_GAUGE_RVA: u32 = 0xC_C340;
const CLOSE_WITHOUT_ANIMATION: i32 = 0;

type CloseBossGauge = unsafe extern "system" fn(*mut c_void, i32);

pub(super) fn hide_boss_gauge(build: &VerifiedBuild) {
    let memory = InProcessMemory::new(build);
    let Some(state) = read_boss_bar_state(&memory) else {
        return;
    };
    if !boss_bar_state_is_open(state) {
        return;
    }
    let module_base = memory.module_base();
    let image_end = module_base + u64::from(SUPPORTED_PE_IDENTITY.size_of_image);
    let function = module_base + u64::from(HUD_CLOSE_BOSS_GAUGE_RVA);
    if !(module_base..image_end).contains(&function) {
        return;
    }
    let hud = module_base + u64::from(BOSS_BAR_HUD_OBJECT_RVA);
    #[expect(
        unsafe_code,
        reason = "calls HudObject::CloseBossGauge at its verified address"
    )]
    // SAFETY: the build gate passed, so the RVA starts HudObject::CloseBossGauge (hud, int),
    // which closes the boss life gauge only if it is open; the HUD object is the game's static
    // one; this runs on the game thread inside the enemy sync step.
    unsafe {
        core::mem::transmute::<usize, CloseBossGauge>(function as usize)(
            hud as usize as *mut c_void,
            CLOSE_WITHOUT_ANIMATION,
        );
    }
    log_line(&format!(
        "boss drive (guest): the boss life gauge was open (state {state}) on its inactive AI Kaine, it is closed here"
    ));
}
