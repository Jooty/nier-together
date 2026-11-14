pub const WATCHED_BODY_MOVES_ON_KEYS_ABOVE_SPEED: f32 = 0.5;

pub fn watched_body_moves_on_keys(speed: Option<f32>, move_keys: Option<[bool; 4]>) -> bool {
    speed.is_some_and(|speed| speed > WATCHED_BODY_MOVES_ON_KEYS_ABOVE_SPEED)
        && move_keys.is_some_and(|keys| keys.contains(&true))
}
