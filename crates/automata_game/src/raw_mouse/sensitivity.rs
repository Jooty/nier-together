use crate::raw_mouse::{
    DEFAULT_X_SENSITIVITY_VALUE, DEGREES_PER_COUNT_AT_DEFAULT_X_SENSITIVITY,
    SENSITIVITY_LAST_LINEAR_VALUE, SENSITIVITY_LINEAR_START, SENSITIVITY_LINEAR_STEP,
    SENSITIVITY_MULTIPLIER_AT_LAST_LINEAR_VALUE, SENSITIVITY_STEP_ABOVE_LAST_LINEAR_VALUE,
    X_SENSITIVITY_FLOOR_FLAG_BIT, X_SENSITIVITY_FLOOR_VALUE,
};

pub fn sensitivity_multiplier(value: i32) -> f32 {
    if value > SENSITIVITY_LAST_LINEAR_VALUE {
        (value - SENSITIVITY_LAST_LINEAR_VALUE) as f32 * SENSITIVITY_STEP_ABOVE_LAST_LINEAR_VALUE
            + SENSITIVITY_MULTIPLIER_AT_LAST_LINEAR_VALUE
    } else {
        value as f32 * SENSITIVITY_LINEAR_STEP + SENSITIVITY_LINEAR_START
    }
}

pub fn x_sensitivity_value(value: i32, option_flags: u32) -> i32 {
    if value <= SENSITIVITY_LAST_LINEAR_VALUE && option_flags & X_SENSITIVITY_FLOOR_FLAG_BIT != 0 {
        value.max(X_SENSITIVITY_FLOOR_VALUE)
    } else {
        value
    }
}

pub fn radians_per_count(multiplier: f32) -> f32 {
    let at_default = sensitivity_multiplier(DEFAULT_X_SENSITIVITY_VALUE);
    DEGREES_PER_COUNT_AT_DEFAULT_X_SENSITIVITY.to_radians() / at_default * multiplier
}
