use replicant_proto::PickupSource;

use crate::pickups::is_key_item;

pub fn world_pickup_refused_while_watching(
    this_game_is_watching: bool,
    source: PickupSource,
    item_id: i32,
) -> bool {
    if !this_game_is_watching || source != PickupSource::World {
        return false;
    }
    u16::try_from(item_id).is_ok_and(|item_id| !is_key_item(item_id))
}
