use crate::host_led_stretch::{HostLedStretch, StretchEnd};

pub fn started_line(stretch: &HostLedStretch, host: bool) -> String {
    let who = if host {
        "your game ran it; the guest watches you to its last step"
    } else {
        "the host ran it; you watch the host to its last step, your body hidden and held"
    };
    format!(
        "host-led stretch: started by {} in {}, it ends at {}; {who}",
        stretch.first_step, stretch.pack, stretch.last_step
    )
}

pub fn ended_line(stretch: &HostLedStretch, end: StretchEnd) -> String {
    let why = match end {
        StretchEnd::LastStep => format!("its last step {} ran", stretch.last_step),
        StretchEnd::LeftThePack => format!("this game left {}", stretch.pack),
        StretchEnd::TookTooLong => "it ran past its time limit".to_owned(),
        StretchEnd::SessionOver => "the session is over".to_owned(),
    };
    format!("host-led stretch: {} ended, {why}", stretch.first_step)
}
