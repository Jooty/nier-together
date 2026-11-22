use crate::downed::WatchSide;

pub fn boss_fight_watch_side(
    host_drive_is_on: bool,
    guest_is_in_the_fight: bool,
) -> Option<WatchSide> {
    if host_drive_is_on {
        Some(WatchSide::Partner)
    } else if guest_is_in_the_fight {
        Some(WatchSide::ThisGame)
    } else {
        None
    }
}
