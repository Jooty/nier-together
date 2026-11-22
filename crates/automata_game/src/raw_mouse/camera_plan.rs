#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RawCameraPlan {
    pub replace_game_mouse: bool,
    pub counts: (i32, i32),
}

pub fn raw_camera_plan(
    setting_on: bool,
    menu_open: bool,
    frame_counts: (i32, i32),
) -> RawCameraPlan {
    if !setting_on {
        return RawCameraPlan {
            replace_game_mouse: false,
            counts: (0, 0),
        };
    }
    RawCameraPlan {
        replace_game_mouse: true,
        counts: if menu_open { (0, 0) } else { frame_counts },
    }
}
