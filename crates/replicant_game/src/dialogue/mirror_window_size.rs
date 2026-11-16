use crate::dialogue::TALK_WINDOW_FULL_PAGE_LAYOUT_SIZES;

/// The `_SetTalkWindowSize` argument a guest's mirrored window opens with. `TalkWindow::Open`
/// lays the window out by the size last stored at window+0x4A4168, so a mirror opened at the
/// guest's own size shows the host's full-page talk (sizes 9, 11, 12) as a plain window.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MirrorWindowSize {
    host_size: Option<i32>,
    own_size: i32,
    applied: i32,
}

impl MirrorWindowSize {
    pub const fn new() -> Self {
        Self {
            host_size: None,
            own_size: 0,
            applied: 0,
        }
    }

    pub fn note_host_size(&mut self, size: i32) {
        self.host_size = Some(size);
    }

    /// This game's own script set the size: it is the size from now on, also after the mirror.
    pub fn note_own_size(&mut self, size: i32) {
        self.own_size = size;
        self.applied = size;
    }

    /// Right before the mirror opens the host's story talk: the size to set first, if any.
    pub fn size_for_the_mirror(&mut self) -> Option<i32> {
        let host = self.host_size?;
        if host == self.applied {
            return None;
        }
        self.applied = host;
        Some(host)
    }

    /// Once no talk window is open: the size to give back to this game, if the mirror took it.
    pub fn size_to_give_back(&mut self) -> Option<i32> {
        if self.applied == self.own_size {
            return None;
        }
        self.applied = self.own_size;
        Some(self.own_size)
    }
}

/// The window's stored layout size (window+0x4A4168, the internal number the native maps a
/// script size to) as a log text.
pub fn layout_size_text(layout: i32) -> String {
    let kind = if TALK_WINDOW_FULL_PAGE_LAYOUT_SIZES.contains(&layout) {
        "full page"
    } else {
        "plain window"
    };
    format!("{layout:#x}, {kind}")
}
