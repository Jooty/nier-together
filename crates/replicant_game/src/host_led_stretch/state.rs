use crate::host_led_stretch::{
    HOST_LED_STRETCH_ENDS_ITSELF_AFTER_MS, HostLedStretch, stretch_starting_at,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StretchEnd {
    LastStep,
    LeftThePack,
    TookTooLong,
    SessionOver,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StretchChange {
    Started(&'static HostLedStretch),
    Ended(&'static HostLedStretch, StretchEnd),
}

/// Which host-led stretch is on, fed with the host's steps: on the host its own pops, on the guest
/// the steps the host sends.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HostLedStretchState {
    on: Option<(&'static HostLedStretch, u64)>,
}

impl HostLedStretchState {
    pub const fn new() -> Self {
        Self { on: None }
    }

    pub fn current(&self) -> Option<&'static HostLedStretch> {
        self.on.map(|(stretch, _)| stretch)
    }

    pub fn host_step(&mut self, pack: &str, step: &str, now_ms: u64) -> Option<StretchChange> {
        if let Some((stretch, _)) = self.on {
            if stretch.pack == pack && stretch.last_step == step {
                self.on = None;
                return Some(StretchChange::Ended(stretch, StretchEnd::LastStep));
            }
            return None;
        }
        let stretch = stretch_starting_at(pack, step)?;
        self.on = Some((stretch, now_ms));
        Some(StretchChange::Started(stretch))
    }

    /// Once per tick with the pack this game is in (`None` while it loads or is in a menu).
    pub fn check(&mut self, pack: Option<&str>, now_ms: u64) -> Option<StretchChange> {
        let (stretch, since_ms) = self.on?;
        let end = if pack.is_some_and(|pack| pack != stretch.pack) {
            StretchEnd::LeftThePack
        } else if now_ms.saturating_sub(since_ms) >= HOST_LED_STRETCH_ENDS_ITSELF_AFTER_MS {
            StretchEnd::TookTooLong
        } else {
            return None;
        };
        self.on = None;
        Some(StretchChange::Ended(stretch, end))
    }

    pub fn end_with_the_session(&mut self) -> Option<StretchChange> {
        let (stretch, _) = self.on.take()?;
        Some(StretchChange::Ended(stretch, StretchEnd::SessionOver))
    }
}
