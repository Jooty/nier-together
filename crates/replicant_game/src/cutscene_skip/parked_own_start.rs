use crate::cutscene_skip::GUEST_OWN_START_HOLD_CAP_WHEN_HOST_NEVER_STARTS_MS;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParkedOwnStart {
    args: u64,
    since_ms: u64,
    replaced: bool,
}

impl ParkedOwnStart {
    pub const fn new(args: u64, since_ms: u64) -> Self {
        Self {
            args,
            since_ms,
            replaced: false,
        }
    }

    pub const fn is_call(&self, args: u64) -> bool {
        self.args == args
    }

    pub const fn replace_with_partner_copy(&mut self) {
        self.replaced = true;
    }

    pub const fn is_replaced(&self) -> bool {
        self.replaced
    }

    pub const fn host_never_started(&self, now_ms: u64) -> bool {
        !self.replaced
            && now_ms.saturating_sub(self.since_ms)
                >= GUEST_OWN_START_HOLD_CAP_WHEN_HOST_NEVER_STARTS_MS
    }
}
