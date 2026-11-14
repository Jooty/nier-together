use crate::story_watch::WatchEnd;

pub fn started_line(host_name: &str, pack: &str) -> String {
    format!(
        "watching: the story took the companion out of play (script id -2 deactivated); this game's \
         body is hidden, locked and kept at {host_name}'s body in {pack}"
    )
}

pub fn ended_line(end: WatchEnd) -> String {
    let why = match end {
        WatchEnd::StoryBroughtHerBack => {
            "the story brought the companion back (script id -2 activated)"
        }
        WatchEnd::LeftTheSession => "the session is over",
        WatchEnd::SettingOff => "guest_watches_when_the_story_takes_her is off",
        WatchEnd::PackIsOneWhereTheGuestPlays => "this pack is one where the guest plays",
    };
    format!("watching: ended, {why}; this game's body is shown and free again at the host's place")
}

pub fn camera_line(own: [f32; 3], host: [f32; 3], error: f32, placed: &str) -> String {
    format!(
        "watching: the camera follows this game's body at ({:.1}, {:.1}, {:.1}), the host's body is at ({:.1}, {:.1}, {:.1}), error {error:.2}; {placed}",
        own[0], own[1], own[2], host[0], host[1], host[2]
    )
}

pub fn watching_host_text(host_name: &str) -> String {
    format!("Watching {host_name}")
}

pub fn guest_is_watching_text(guest_name: &str) -> String {
    format!("{guest_name} is watching")
}
