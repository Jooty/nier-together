#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartButton {
    Start,
    WaitingForHost,
    Rejoin,
}

impl StartButton {
    pub fn label(self) -> &'static str {
        match self {
            Self::Start => "Start",
            Self::WaitingForHost => "Waiting for host",
            Self::Rejoin => "Rejoin",
        }
    }
}

pub fn start_button(viewer_is_guest: bool, can_rejoin: bool) -> StartButton {
    match (viewer_is_guest, can_rejoin) {
        (false, _) => StartButton::Start,
        (true, true) => StartButton::Rejoin,
        (true, false) => StartButton::WaitingForHost,
    }
}
