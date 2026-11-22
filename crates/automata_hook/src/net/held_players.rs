use crate::net::link_state::NetShared;
use automata_game::late_join::WaitingJoiner;

/// How many late joiners have a game that has sent a body and wait to be taken in.
pub fn held_arrivals_of(shared: &NetShared) -> usize {
    shared
        .players
        .values()
        .filter(|player| player.watch.held_back && player.watch.arrival_waiting)
        .count()
}

/// The late joiners that wait to be taken in.
pub fn held_arrival_joiners(shared: &NetShared) -> Vec<WaitingJoiner> {
    shared
        .players
        .values()
        .filter(|player| player.watch.held_back && player.watch.arrival_waiting)
        .map(|player| WaitingJoiner {
            slot: player.slot,
            name: player.name.clone(),
        })
        .collect()
}

/// Takes in every held player whose game has arrived; returns their names.
pub fn take_in_held_players(shared: &mut NetShared) -> Vec<String> {
    shared
        .players
        .values_mut()
        .filter(|player| player.watch.held_back && player.watch.arrival_waiting)
        .map(|player| {
            player.watch.held_back = false;
            player.name.clone()
        })
        .collect()
}
