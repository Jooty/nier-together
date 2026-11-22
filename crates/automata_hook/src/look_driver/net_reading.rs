use automata_game::mode::SessionMode;
use automata_game::outfit_look::{PlayerLook, looks_allowed};
use nt_hook::net::NetState;

use crate::net::{SessionModeExt, handle};

#[derive(Debug, Clone, Copy)]
pub(super) struct NetReading {
    pub connected: bool,
    pub session_open: bool,
    pub mode: SessionMode,
    pub host_allows: bool,
    pub peers: usize,
}

impl NetReading {
    pub fn allowed(&self) -> bool {
        looks_allowed(self.mode, self.host_allows)
    }
}

pub(super) fn net_reading() -> NetReading {
    let Some(net) = handle() else {
        return NetReading {
            connected: false,
            session_open: false,
            mode: SessionMode::Campaign,
            host_allows: false,
            peers: 0,
        };
    };
    net.update(|shared| NetReading {
        connected: shared.state == NetState::Connected,
        session_open: shared.extras.session_open,
        mode: shared.session_mode(),
        host_allows: shared.extras.any_look_allowed,
        peers: shared.counted_others().count(),
    })
}

pub(super) fn other_players_looks() -> Vec<(u8, PlayerLook)> {
    let Some(net) = handle() else {
        return Vec::new();
    };
    net.update(|shared| {
        shared
            .others()
            .filter_map(|player| player.watch.look.map(|look| (player.slot, look)))
            .collect()
    })
}
