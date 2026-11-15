use crate::callback_gate::{EndArrival, EndDrop, EndFollow, end_follow};
use crate::host_led_stretch::StretchPlace;

/// The guest's verdict on an End of the host's around a host-led stretch it watches: inside the
/// stretch the host's game runs every step, so the End is not run here; the stretch's own last
/// step runs here at once, as the pause it finds is the watched scene's, not this player's.
pub fn end_follow_around_a_stretch(arrival: &EndArrival, place: StretchPlace) -> EndFollow {
    let same_phase = arrival.shared_phase == Some(arrival.message_phase)
        && arrival.local_phase == Some(arrival.message_phase);
    match place {
        StretchPlace::InsideIt if same_phase => EndFollow::Drop(EndDrop::HostLeadsTheStretch),
        StretchPlace::ItsLastStep => end_follow(&EndArrival {
            control_mask_settled: true,
            ..*arrival
        }),
        StretchPlace::InsideIt | StretchPlace::NotInOne => end_follow(arrival),
    }
}
