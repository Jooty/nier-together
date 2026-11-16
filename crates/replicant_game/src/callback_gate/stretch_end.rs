use crate::callback_gate::{EndArrival, EndDrop, EndFollow, end_follow};
use crate::host_led_stretch::StretchPlace;

/// The guest's verdict on an End of the host's around a host-led stretch it watches. Inside one it
/// is not run here, unless the guest is not taken along (its Ends put the scene's actors here too);
/// those and the last step run at once, as the pause found is the watched scene's.
pub fn end_follow_around_a_stretch(arrival: &EndArrival, place: StretchPlace) -> EndFollow {
    let same_phase = arrival.shared_phase == Some(arrival.message_phase)
        && arrival.local_phase == Some(arrival.message_phase);
    match place {
        StretchPlace::InsideIt if same_phase => EndFollow::Drop(EndDrop::HostLeadsTheStretch),
        StretchPlace::ItsLastStep | StretchPlace::InsideItTheGuestIsNotTakenAlong => {
            end_follow(&EndArrival {
                control_mask_settled: true,
                ..*arrival
            })
        }
        StretchPlace::InsideIt | StretchPlace::NotInOne => end_follow(arrival),
    }
}
