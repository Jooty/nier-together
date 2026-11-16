use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StoryJumpView {
    pub host: bool,
    pub map_manager_active: bool,
    pub event_scene: bool,
    pub talk_window_open: bool,
    pub ui_dialog_state: i32,
    pub pause_menu_open: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoryJumpRefusal {
    NotHost,
    NotInTheField,
    EventScene,
    TalkOpen,
    DialogOpen(i32),
}

impl fmt::Display for StoryJumpRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::NotHost => "only the host can jump",
            Self::NotInTheField => "not in the field yet",
            Self::EventScene => "an event scene is playing, wait for it to end",
            Self::TalkOpen => "a talk is open, wait for it to end",
            Self::DialogOpen(state) => {
                return write!(formatter, "a dialog is open (state {state:#x}), close it");
            }
        };
        formatter.write_str(text)
    }
}

pub fn check_story_jump(view: &StoryJumpView) -> Result<(), StoryJumpRefusal> {
    if !view.host {
        return Err(StoryJumpRefusal::NotHost);
    }
    if !view.map_manager_active {
        return Err(StoryJumpRefusal::NotInTheField);
    }
    if view.event_scene {
        return Err(StoryJumpRefusal::EventScene);
    }
    if view.talk_window_open {
        return Err(StoryJumpRefusal::TalkOpen);
    }
    if view.ui_dialog_state != 0 {
        return Err(StoryJumpRefusal::DialogOpen(view.ui_dialog_state));
    }
    Ok(())
}

pub fn jump_waits_for_pause_menu(view: &StoryJumpView) -> bool {
    view.pause_menu_open
}

/// The host's jump reloads its phase in place when it is already there, and its stage start runs
/// again with the jump's flags. A guest already in that phase must reload too, or its game keeps
/// the actors of the old flags; a guest elsewhere follows the host's phase change as usual.
pub fn guest_reloads_in_place(current_phase: Option<&str>, destination_phase: &str) -> bool {
    current_phase == Some(destination_phase)
}
