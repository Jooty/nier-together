use crate::event_follow::EventId;

pub fn own_pause_told_line(id: EventId, paused: bool) -> String {
    let what = if paused { "opened" } else { "closed" };
    format!(
        "event: the event pause {what} here for {}, told the partner",
        id.describe()
    )
}

pub fn own_pause_outside_a_cutscene_line() -> String {
    "event: the pause opened here while no cutscene plays, the partner is not told: only a cutscene pause is shared".to_owned()
}

pub fn own_pause_outside_the_rule_line(id: EventId) -> String {
    format!(
        "event: the pause opened here during {}, but the partner is not told: the link is down or a copy is out of play",
        id.describe()
    )
}

pub fn partner_pause_heard_line(id: EventId, paused: bool) -> String {
    let what = if paused { "opened" } else { "closed" };
    format!(
        "event: the partner's event pause for {} {what}",
        id.describe()
    )
}

pub fn partner_pause_ended_line(id: EventId) -> String {
    format!(
        "event: the partner's event pause for {} is dropped, another event plays here",
        id.describe()
    )
}

pub fn partner_pause_event_ended_here_line(id: EventId) -> String {
    format!(
        "event: the partner's event pause for {} is dropped, the event ended here",
        id.describe()
    )
}

pub fn partner_pause_rule_ended_line(id: EventId) -> String {
    format!(
        "event: the partner's event pause for {} is dropped, the link is down or a copy is out of play",
        id.describe()
    )
}

pub fn pause_clock_line(id: EventId) -> String {
    format!(
        "event: the event clock is paused here for the partner's pause of {}",
        id.describe()
    )
}

pub fn resume_clock_line() -> String {
    "event: the event clock is resumed here, the partner's pause is over".to_owned()
}
