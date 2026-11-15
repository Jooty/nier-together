use crate::host_led_stretch::{
    HOST_LED_LONG_STRETCH_ENDS_ITSELF_AFTER_MS, HOST_LED_STRETCH_ENDS_ITSELF_AFTER_MS,
};

/// A stretch of the story the guest watches from its first step to its last: the host's game runs
/// every step, the guest's body is hidden and kept at the host's, and it sees the host's talks
/// and screen fades.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HostLedStretch {
    pub pack: &'static str,
    pub first_step: &'static str,
    pub last_pack: &'static str,
    pub last_step: &'static str,
    pub armed_by: Option<&'static str>,
    pub refuses_guest_doors: bool,
    pub guest_is_not_taken_along: bool,
    pub ends_itself_after_ms: u64,
}

pub static HOST_LED_STRETCHES: [HostLedStretch; 3] = [
    HostLedStretch {
        pack: "A_CENTER_VILLAGE_01",
        first_step: "EID_1500_a0005_010_01_Start",
        last_pack: "A_CENTER_VILLAGE_01",
        last_step: "EID_1500_a0005_010_01_End",
        armed_by: None,
        refuses_guest_doors: false,
        guest_is_not_taken_along: false,
        ends_itself_after_ms: HOST_LED_STRETCH_ENDS_ITSELF_AFTER_MS,
    },
    HostLedStretch {
        pack: "C_FOREST_FIELD_01",
        first_step: "EID_1500_a0020g_0010_Start",
        last_pack: "C_FOREST_FIELD_01",
        last_step: "EID_1500_a0020g_0010_End",
        armed_by: None,
        refuses_guest_doors: false,
        guest_is_not_taken_along: false,
        ends_itself_after_ms: HOST_LED_STRETCH_ENDS_ITSELF_AFTER_MS,
    },
    HostLedStretch {
        pack: "A_SOUTH_MANSION_01",
        first_step: "Open_Door_01",
        last_pack: "A_SOUTH_MANSION_02",
        last_step: "EID_1100_a0080_01_End",
        armed_by: Some("ScriptMode_to_Kaine_03_04"),
        refuses_guest_doors: true,
        guest_is_not_taken_along: true,
        ends_itself_after_ms: HOST_LED_LONG_STRETCH_ENDS_ITSELF_AFTER_MS,
    },
];

pub fn stretch_starting_at(pack: &str, step: &str) -> Option<&'static HostLedStretch> {
    HOST_LED_STRETCHES
        .iter()
        .find(|stretch| stretch.pack == pack && stretch.first_step == step)
}

pub fn stretch_armed_by(pack: &str, step: &str) -> Option<&'static HostLedStretch> {
    HOST_LED_STRETCHES
        .iter()
        .find(|stretch| stretch.pack == pack && stretch.armed_by == Some(step))
}
