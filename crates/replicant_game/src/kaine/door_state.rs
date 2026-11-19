use replicant_proto::PartnerBody;

pub const BODIES_WITH_A_CAPPED_OPEN_DOOR_STATE: [PartnerBody; 1] = [PartnerBody::SeedKaineE];
pub const SWAPPED_BODY_OPEN_DOOR_STATE_MOST_MS: u64 = 3_000;
pub const OPEN_DOOR_STATE_UPDATE_GAP_THAT_STARTS_A_NEW_STATE_MS: u64 = 1_000;
pub const OPEN_DOOR_STATE_STEP_WAITING_FOR_THE_DOOR_MOTION: i32 = 0x288;
pub const DOOR_CARRY_FARTHEST_ALLOWED_METRES: f32 = 6.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenDoorStateVerdict {
    Run,
    EndNow { ran_ms: u64 },
}

#[derive(Debug, Default)]
pub struct OpenDoorStateClock {
    state: u64,
    first_update_ms: u64,
    last_update_ms: u64,
    door_closed_ms: Option<u64>,
    carried: bool,
}

impl OpenDoorStateClock {
    pub const fn new() -> Self {
        Self {
            state: 0,
            first_update_ms: 0,
            last_update_ms: 0,
            door_closed_ms: None,
            carried: false,
        }
    }

    pub fn note_door_closed(&mut self, now_ms: u64) {
        self.door_closed_ms = Some(now_ms);
    }

    pub fn carry_is_due(&mut self, capped_body: bool, step: i32) -> bool {
        if !capped_body || self.carried || step != OPEN_DOOR_STATE_STEP_WAITING_FOR_THE_DOOR_MOTION
        {
            return false;
        }
        self.carried = true;
        true
    }

    pub fn observe(&mut self, state: u64, capped_body: bool, now_ms: u64) -> OpenDoorStateVerdict {
        if !capped_body {
            *self = Self::new();
            return OpenDoorStateVerdict::Run;
        }
        let a_new_state = self.state != state
            || now_ms.saturating_sub(self.last_update_ms)
                >= OPEN_DOOR_STATE_UPDATE_GAP_THAT_STARTS_A_NEW_STATE_MS;
        if a_new_state {
            self.state = state;
            self.first_update_ms = now_ms;
            self.carried = false;
        }
        self.last_update_ms = now_ms;
        let door_closed_during_this_state = self
            .door_closed_ms
            .is_some_and(|closed_ms| closed_ms >= self.first_update_ms);
        let ran_ms = now_ms.saturating_sub(self.first_update_ms);
        if ran_ms < SWAPPED_BODY_OPEN_DOOR_STATE_MOST_MS && !door_closed_during_this_state {
            return OpenDoorStateVerdict::Run;
        }
        *self = Self::new();
        OpenDoorStateVerdict::EndNow { ran_ms }
    }
}

pub fn body_has_a_capped_open_door_state(body: PartnerBody, by_swap: bool) -> bool {
    by_swap && BODIES_WITH_A_CAPPED_OPEN_DOOR_STATE.contains(&body)
}

pub fn open_door_state_ended_text(ran_ms: u64) -> String {
    format!(
        "doors: the swapped body's door state ran {ran_ms} ms with no door motion of hers to end it, ended it now; the door's own scripts carry on"
    )
}

pub fn other_side_of_a_door(side: i32) -> Option<i32> {
    match side {
        0 => Some(1),
        1 => Some(0),
        _ => None,
    }
}

pub fn door_carry_is_sane(from: Option<[f32; 3]>, to: [f32; 3]) -> bool {
    from.is_some_and(|from| {
        let [dx, dy, dz] = [to[0] - from[0], to[1] - from[1], to[2] - from[2]];
        (dx * dx + dy * dy + dz * dz).sqrt() <= DOOR_CARRY_FARTHEST_ALLOWED_METRES
    })
}

pub fn door_carry_text(
    side: i32,
    from: Option<[f32; 3]>,
    to: [f32; 3],
    heading_degrees: f32,
    carried: bool,
) -> String {
    let from_text = from.map_or_else(
        || "an unreadable position".to_owned(),
        |at| format!("({:.2}, {:.2}, {:.2})", at[0], at[1], at[2]),
    );
    let what = if carried {
        "carried her through"
    } else {
        "did not carry her through (the spot is too far from where she stands)"
    };
    format!(
        "doors: the swapped body has no walk-through motion; {what} from the door's side {side}: she is at {from_text}, the other side's stand point is ({:.2}, {:.2}, {:.2}), facing {heading_degrees:.1}",
        to[0], to[1], to[2]
    )
}
