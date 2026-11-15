/// A stretch of the story the guest watches from its first step to its last: the host's game runs
/// every step, the guest's body is hidden and kept at the host's, and it sees the host's talks
/// and screen fades.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HostLedStretch {
    pub pack: &'static str,
    pub first_step: &'static str,
    pub last_step: &'static str,
}

pub static HOST_LED_STRETCHES: [HostLedStretch; 2] = [
    HostLedStretch {
        pack: "A_CENTER_VILLAGE_01",
        first_step: "EID_1500_a0005_010_01_Start",
        last_step: "EID_1500_a0005_010_01_End",
    },
    HostLedStretch {
        pack: "C_FOREST_FIELD_01",
        first_step: "EID_1500_a0020g_0010_Start",
        last_step: "EID_1500_a0020g_0010_End",
    },
];

pub fn stretch_starting_at(pack: &str, step: &str) -> Option<&'static HostLedStretch> {
    HOST_LED_STRETCHES
        .iter()
        .find(|stretch| stretch.pack == pack && stretch.first_step == step)
}
