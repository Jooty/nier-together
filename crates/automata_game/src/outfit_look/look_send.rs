use crate::outfit_look::PlayerLook;

/// A look goes out when it changed, and again for a player who just joined or after a scene load.
pub fn look_send_due(
    last_sent: Option<PlayerLook>,
    current: PlayerLook,
    peer_joined: bool,
    scene_loaded: bool,
) -> bool {
    last_sent != Some(current) || peer_joined || scene_loaded
}

/// As `look_send_due`, but a player who never picked anything has nothing to say.
pub fn look_message_due(
    last_sent: Option<PlayerLook>,
    current: PlayerLook,
    peer_joined: bool,
    scene_loaded: bool,
) -> bool {
    let nothing_to_say = current.is_unpicked() && last_sent.is_none_or(|sent| sent.is_unpicked());
    !nothing_to_say && look_send_due(last_sent, current, peer_joined, scene_loaded)
}
