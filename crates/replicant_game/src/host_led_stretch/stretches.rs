use crate::host_led_stretch::{
    HOST_LED_LONG_STRETCH_ENDS_ITSELF_AFTER_MS, HOST_LED_STRETCH_ENDS_ITSELF_AFTER_MS,
    HOST_LED_STRETCH_NEVER_ENDS_ITSELF_AFTER_MS,
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
    pub guest_waits_for_the_hosts_first_step: bool,
    pub ends_itself_after_ms: u64,
    /// The story state inside the stretch: this game flag is on and `story_off_flag` is not.
    /// A save loaded or a story jump landed in that state is inside the stretch.
    pub story_on_flag: Option<u16>,
    pub story_off_flag: Option<u16>,
    /// The story's own removal of the companion ends with the stretch's last step here, so the
    /// guest plays again from that step, not from the later call that brings her back.
    pub the_guest_plays_from_its_last_step: bool,
    /// The stretch goes on through every pack the host walks into: the guest leaving the first
    /// pack does not end it, and the story state alone says whether this game is inside it. It
    /// ends at its last step, or when the story state is no longer inside it.
    pub spans_every_pack: bool,
}

pub static HOST_LED_STRETCHES: [HostLedStretch; 4] = [
    HostLedStretch {
        pack: "C_FOREST_FIELD_01",
        first_step: "EID_1500_a0020g_0010_Start",
        last_pack: "C_FOREST_FIELD_01",
        last_step: "EID_1500_a0020g_0010_End",
        armed_by: None,
        refuses_guest_doors: false,
        guest_is_not_taken_along: false,
        guest_waits_for_the_hosts_first_step: true,
        ends_itself_after_ms: HOST_LED_STRETCH_ENDS_ITSELF_AFTER_MS,
        story_on_flag: None,
        story_off_flag: None,
        the_guest_plays_from_its_last_step: false,
        spans_every_pack: false,
    },
    HostLedStretch {
        pack: "A_SOUTH_MANSION_01",
        first_step: "Open_Door_01",
        last_pack: "A_SOUTH_MANSION_02",
        last_step: "EID_1100_a0080_01_Start",
        armed_by: Some("ScriptMode_to_Kaine_03_04"),
        refuses_guest_doors: true,
        guest_is_not_taken_along: true,
        guest_waits_for_the_hosts_first_step: false,
        ends_itself_after_ms: HOST_LED_LONG_STRETCH_ENDS_ITSELF_AFTER_MS,
        story_on_flag: Some(504),
        story_off_flag: Some(511),
        the_guest_plays_from_its_last_step: true,
        spans_every_pack: false,
    },
    HostLedStretch {
        pack: "A_SEASIDE_TOWN_01",
        first_step: "EID_9300_a0050_0010_Start",
        last_pack: "A_SEASIDE_TOWN_01",
        last_step: "EID_9300_a0050_0240_End",
        armed_by: None,
        refuses_guest_doors: false,
        guest_is_not_taken_along: false,
        guest_waits_for_the_hosts_first_step: true,
        ends_itself_after_ms: HOST_LED_STRETCH_ENDS_ITSELF_AFTER_MS,
        story_on_flag: None,
        story_off_flag: None,
        the_guest_plays_from_its_last_step: false,
        spans_every_pack: false,
    },
    HostLedStretch {
        pack: "B_CENTER_VILLAGE_01",
        first_step: "EID_3000_a0012_Start",
        last_pack: "B_CENTER_LIBRARY_01",
        last_step: "EID_3200_a0052_End",
        armed_by: None,
        refuses_guest_doors: false,
        guest_is_not_taken_along: false,
        guest_waits_for_the_hosts_first_step: true,
        ends_itself_after_ms: HOST_LED_STRETCH_NEVER_ENDS_ITSELF_AFTER_MS,
        story_on_flag: Some(556),
        story_off_flag: Some(125),
        the_guest_plays_from_its_last_step: true,
        spans_every_pack: true,
    },
];

pub fn stretch_starting_at(pack: &str, step: &str) -> Option<&'static HostLedStretch> {
    HOST_LED_STRETCHES
        .iter()
        .find(|stretch| stretch.pack == pack && stretch.first_step == step)
}

pub fn stretch_the_guest_waits_to_start(pack: &str, step: &str) -> Option<&'static HostLedStretch> {
    stretch_starting_at(pack, step).filter(|stretch| stretch.guest_waits_for_the_hosts_first_step)
}

/// The stretch the story state puts this game inside: one with story flags, in one of its packs,
/// whose on flag is set and whose off flag is not.
pub fn stretch_the_story_is_inside(
    pack: &str,
    flag_on: impl Fn(u16) -> bool,
) -> Option<&'static HostLedStretch> {
    HOST_LED_STRETCHES.iter().find(|stretch| {
        let (Some(on), Some(off)) = (stretch.story_on_flag, stretch.story_off_flag) else {
            return false;
        };
        let in_its_packs =
            stretch.spans_every_pack || stretch.pack == pack || stretch.last_pack == pack;
        in_its_packs && flag_on(on) && !flag_on(off)
    })
}

pub fn stretch_armed_by(pack: &str, step: &str) -> Option<&'static HostLedStretch> {
    HOST_LED_STRETCHES
        .iter()
        .find(|stretch| stretch.pack == pack && stretch.armed_by == Some(step))
}
