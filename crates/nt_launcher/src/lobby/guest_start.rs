pub const GUEST_WAIT_FOR_SEAT_SECONDS: f64 = 45.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuestStartStep {
    KeepWaiting,
    Begin,
    GiveUpOnSeat,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GuestStartWait {
    seat_waited_since: Option<f64>,
}

impl GuestStartWait {
    pub fn new() -> Self {
        Self {
            seat_waited_since: None,
        }
    }

    pub fn step(
        &mut self,
        own_ready: bool,
        seat_pending: bool,
        now_seconds: f64,
    ) -> GuestStartStep {
        if !own_ready {
            self.seat_waited_since = None;
            return GuestStartStep::KeepWaiting;
        }
        if !seat_pending {
            return GuestStartStep::Begin;
        }
        let since = *self.seat_waited_since.get_or_insert(now_seconds);
        if now_seconds - since >= GUEST_WAIT_FOR_SEAT_SECONDS {
            return GuestStartStep::GiveUpOnSeat;
        }
        GuestStartStep::KeepWaiting
    }
}

impl Default for GuestStartWait {
    fn default() -> Self {
        Self::new()
    }
}
