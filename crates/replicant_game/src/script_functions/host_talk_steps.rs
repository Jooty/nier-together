pub static STEPS_THE_HOSTS_OWN_TALK_STARTS: [(&str, &str); 1] =
    [("A_CENTER_VILLAGE_01", "EID_1500_a0005_020_Select")];

/// A step the end of the host's own talk with a villager starts. The guest's game never reaches it
/// on its own, so the host's copy runs here at once and the guest's own prompt opens with the
/// host's instead of three seconds later, after the host has answered.
pub fn is_started_by_the_hosts_own_talk(phase_label: &str, name: &str) -> bool {
    STEPS_THE_HOSTS_OWN_TALK_STARTS.contains(&(phase_label, name))
}
