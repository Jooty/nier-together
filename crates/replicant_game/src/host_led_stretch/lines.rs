use crate::host_led_stretch::{HostLedStretch, StretchEnd};

pub fn started_line(stretch: &HostLedStretch, host: bool) -> String {
    let who = if host {
        "your game ran it; the guest watches you to its last step"
    } else {
        "the host ran it; you watch the host to its last step, your body hidden and held"
    };
    format!(
        "host-led stretch: started by {} in {}, it ends at {} in {}; {who}",
        stretch.first_step, stretch.pack, stretch.last_step, stretch.last_pack
    )
}

pub fn ended_line(stretch: &HostLedStretch, end: StretchEnd) -> String {
    format!(
        "host-led stretch: {} ended, {}",
        stretch.first_step,
        end_reason(stretch, end)
    )
}

pub fn armed_line(stretch: &HostLedStretch, armed_by: &str) -> String {
    let doors = if stretch.refuses_guest_doors {
        "; the guest's door use is refused from now until it ends"
    } else {
        ""
    };
    format!(
        "host-led stretch: {} is armed by {armed_by} in {}, it starts at {}{doors}",
        stretch.first_step, stretch.pack, stretch.first_step
    )
}

pub fn disarmed_line(stretch: &HostLedStretch, end: StretchEnd) -> String {
    format!(
        "host-led stretch: {} was armed and is dropped, {}",
        stretch.first_step,
        end_reason(stretch, end)
    )
}

pub fn door_refused_line(stretch: &HostLedStretch, door: &str) -> String {
    format!(
        "host-led stretch: refused your use of {door}, doors stay shut for the guest until {} in {}",
        stretch.last_step, stretch.last_pack
    )
}

pub fn host_run_not_replayed_line(stretch: &HostLedStretch, function: &str) -> String {
    format!(
        "host-led stretch: the host's {function} is not replayed here, you watch the host to {}",
        stretch.last_step
    )
}

fn end_reason(stretch: &HostLedStretch, end: StretchEnd) -> String {
    match end {
        StretchEnd::LastStep => format!("its last step {} ran", stretch.last_step),
        StretchEnd::LeftThePack => format!("this game left {}", stretch.pack),
        StretchEnd::TookTooLong => "it ran past its time limit".to_owned(),
        StretchEnd::SessionOver => "the session is over".to_owned(),
    }
}
