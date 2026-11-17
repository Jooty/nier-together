use automata_game::load_gate::wall_limit_passed;

#[test]
fn nothing_kept_yet_never_passes_the_limit() {
    assert!(!wall_limit_passed(None, u64::MAX, 1));
}

#[test]
fn the_limit_passes_exactly_at_the_limit_by_the_clock_alone() {
    assert!(!wall_limit_passed(Some(1_000), 60_999, 60_000));
    assert!(wall_limit_passed(Some(1_000), 61_000, 60_000));
}

#[test]
fn a_clock_that_went_backwards_does_not_pass() {
    assert!(!wall_limit_passed(Some(5_000), 4_000, 1));
}
