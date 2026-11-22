use std::collections::VecDeque;
use std::sync::{LazyLock, Mutex, MutexGuard, PoisonError};

use automata_game::character::Character;
use automata_game::mode::SessionMode;
use automata_game::outfit_look::{LookKind, LookNames, LookPicks};

use crate::game_calls::GameCalls;

static LOOK: LazyLock<Mutex<LookState>> = LazyLock::new(|| Mutex::new(LookState::default()));

#[derive(Debug, Clone, Copy)]
pub(super) struct PickRequest {
    pub slot: usize,
    pub kind: LookKind,
    pub value: Option<i32>,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct PanelView {
    pub slot: Option<usize>,
    pub character: Option<Character>,
    pub mode: SessionMode,
    pub host_allows: bool,
}

#[derive(Debug)]
pub(super) struct LookState {
    pub picks: LookPicks,
    pub names: LookNames,
    pub requests: VecDeque<PickRequest>,
    pub view: PanelView,
    pub calls: Option<GameCalls>,
}

impl Default for LookState {
    fn default() -> Self {
        Self {
            picks: LookPicks::default(),
            names: LookNames::default(),
            requests: VecDeque::new(),
            view: PanelView {
                slot: None,
                character: None,
                mode: SessionMode::Campaign,
                host_allows: false,
            },
            calls: None,
        }
    }
}

pub(super) fn state() -> MutexGuard<'static, LookState> {
    LOOK.lock().unwrap_or_else(PoisonError::into_inner)
}

pub(super) fn try_state() -> Option<MutexGuard<'static, LookState>> {
    match LOOK.try_lock() {
        Ok(guard) => Some(guard),
        Err(std::sync::TryLockError::Poisoned(poisoned)) => Some(poisoned.into_inner()),
        Err(std::sync::TryLockError::WouldBlock) => None,
    }
}
