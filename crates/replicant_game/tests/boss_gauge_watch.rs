use replicant_game::enemies::{BossGaugeReading, BossGaugeWatch};

fn reading(state: Option<i32>, bound: Option<&str>) -> BossGaugeReading {
    BossGaugeReading {
        state,
        bound_enemy: bound.map(str::to_owned),
        boss: Some("100:0".to_owned()),
    }
}

#[test]
fn run_1119g_a_gauge_that_never_opens_on_the_guest_shows_in_the_first_line_and_not_again() {
    let mut watch = BossGaugeWatch::new();
    let line = watch.observe("guest", &reading(Some(0), None)).unwrap();
    assert!(line.contains("not open (state 0)"));
    assert!(line.contains("no enemy of the roster"));
    assert_eq!(watch.observe("guest", &reading(Some(0), None)), None);
}

#[test]
fn an_opening_and_the_enemy_it_binds_to_are_each_a_new_line() {
    let mut watch = BossGaugeWatch::new();
    watch.observe("host", &reading(Some(0), None));
    let opened = watch
        .observe("host", &reading(Some(1), Some("100:0")))
        .unwrap();
    assert!(opened.contains("open (state 1)"));
    assert!(opened.contains("bound to 100:0"));
    let closed = watch
        .observe("host", &reading(Some(3), Some("100:0")))
        .unwrap();
    assert!(closed.contains("not open (state 3)"));
}

#[test]
fn leaving_the_fight_starts_the_watch_over() {
    let mut watch = BossGaugeWatch::new();
    watch.observe("guest", &reading(Some(0), None));
    watch.forget();
    assert!(watch.observe("guest", &reading(Some(0), None)).is_some());
}
