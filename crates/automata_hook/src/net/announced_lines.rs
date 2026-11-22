use std::sync::{Mutex, PoisonError};

const MOST_ANNOUNCED_LINES_WAITING: usize = 32;

static ANNOUNCED_LINES_WAITING: Mutex<Vec<String>> = Mutex::new(Vec::new());

pub fn push_announced_line(line: String) {
    let mut waiting = ANNOUNCED_LINES_WAITING
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    if waiting.len() >= MOST_ANNOUNCED_LINES_WAITING {
        waiting.remove(0);
    }
    waiting.push(line);
}

pub fn take_postable_announced_lines(local_in_play: bool, local_free: bool) -> Vec<String> {
    take_postable_announced_lines_where(local_in_play, local_free, |_| true)
}

pub fn take_postable_announced_lines_where(
    local_in_play: bool,
    local_free: bool,
    wanted: impl Fn(&str) -> bool,
) -> Vec<String> {
    if !local_in_play || !local_free {
        return Vec::new();
    }
    take_announced_lines_where(wanted)
}

pub fn take_announced_lines_where(wanted: impl Fn(&str) -> bool) -> Vec<String> {
    let mut waiting = ANNOUNCED_LINES_WAITING
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    let (taken, kept): (Vec<String>, Vec<String>) = std::mem::take(&mut *waiting)
        .into_iter()
        .partition(|line| wanted(line));
    *waiting = kept;
    taken
}
