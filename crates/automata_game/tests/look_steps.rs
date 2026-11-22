use automata_game::outfit_look::{
    ApplyGuard, LOOK_KIND_COUNT, LookKind, LookReading, LookStep, MOST_TRIES_PER_VALUE, PlayerLook,
    REAPPLY_COOLDOWN_TICKS, look_message_due, look_steps,
};

const MATCHING: LookReading = LookReading {
    outfit_global: 2,
    outfit_body: 2,
    accessory_global: 5,
    accessory_body: 5,
    hair: 3,
    pod_skin: -1,
    gun_skin: 0,
    cloth_state: 1,
};

fn targets(set: &[(LookKind, i32)]) -> [Option<i32>; LOOK_KIND_COUNT] {
    let mut targets = [None; LOOK_KIND_COUNT];
    for (kind, value) in set {
        targets[kind.index()] = Some(*value);
    }
    targets
}

#[test]
fn nothing_is_done_when_the_game_already_shows_the_target() {
    let all = targets(&[
        (LookKind::Outfit, 2),
        (LookKind::Accessory, 5),
        (LookKind::Hair, 3),
        (LookKind::PodSkin, -1),
        (LookKind::GunSkin, 0),
        (LookKind::ClothState, 1),
    ]);
    assert!(look_steps(&all, &MATCHING).is_empty());
}

#[test]
fn a_kind_without_a_target_is_left_alone() {
    assert!(look_steps(&targets(&[]), &MATCHING).is_empty());
}

#[test]
fn a_differing_global_gives_the_step_of_its_kind() {
    let steps = look_steps(
        &targets(&[(LookKind::Hair, 9), (LookKind::GunSkin, 2)]),
        &MATCHING,
    );
    assert_eq!(steps, vec![LookStep::SetHair(9), LookStep::SetGunSkin(2)]);
}

#[test]
fn a_body_the_game_rebuilt_without_the_pick_gets_it_again() {
    let rebuilt = LookReading {
        accessory_body: 0,
        outfit_body: 0,
        ..MATCHING
    };
    let steps = look_steps(
        &targets(&[(LookKind::Outfit, 2), (LookKind::Accessory, 5)]),
        &rebuilt,
    );
    assert_eq!(
        steps,
        vec![LookStep::SetOutfit(2), LookStep::SetAccessory(5)]
    );
}

#[test]
fn a_global_the_sanitizer_reset_is_written_back() {
    let reset = LookReading {
        pod_skin: -1,
        accessory_global: 0,
        ..MATCHING
    };
    let steps = look_steps(
        &targets(&[(LookKind::PodSkin, 4), (LookKind::Accessory, 5)]),
        &reset,
    );
    assert_eq!(
        steps,
        vec![LookStep::SetAccessory(5), LookStep::SetPodSkin(4)]
    );
}

#[test]
fn a_step_reports_its_kind_and_value() {
    assert_eq!(LookStep::SetPodSkin(7).kind(), LookKind::PodSkin);
    assert_eq!(LookStep::SetPodSkin(7).value(), 7);
}

#[test]
fn the_first_try_is_allowed_and_a_repeat_waits_for_the_cooldown() {
    let mut guard = ApplyGuard::default();
    assert!(guard.allow(LookKind::Accessory, 4, 100));
    assert!(!guard.allow(LookKind::Accessory, 4, 101));
    assert!(guard.allow(LookKind::Accessory, 4, 100 + REAPPLY_COOLDOWN_TICKS));
}

#[test]
fn a_step_the_game_never_takes_is_given_up_after_three_tries() {
    let mut guard = ApplyGuard::default();
    let mut tick = 0;
    let mut allowed = 0;
    for _ in 0..10 {
        if guard.allow(LookKind::Accessory, 4, tick) {
            allowed += 1;
        }
        tick += REAPPLY_COOLDOWN_TICKS;
    }
    assert_eq!(allowed, MOST_TRIES_PER_VALUE);
    assert!(guard.gave_up(LookKind::Accessory, 4));
}

#[test]
fn another_value_or_a_new_body_starts_the_tries_over() {
    let mut guard = ApplyGuard::default();
    assert!(guard.allow(LookKind::Hair, 1, 0));
    assert!(guard.allow(LookKind::Hair, 2, 1));
    guard.reset();
    assert!(guard.allow(LookKind::Hair, 2, 2));
}

#[test]
fn a_player_who_never_picked_sends_nothing() {
    assert!(!look_message_due(None, PlayerLook::UNPICKED, true, true));
}

#[test]
fn a_cleared_pick_is_sent_once_and_then_nothing() {
    let picked = PlayerLook {
        hair_colour: 3,
        ..PlayerLook::UNPICKED
    };
    assert!(look_message_due(
        Some(picked),
        PlayerLook::UNPICKED,
        false,
        false
    ));
    assert!(!look_message_due(
        Some(PlayerLook::UNPICKED),
        PlayerLook::UNPICKED,
        true,
        false
    ));
}

#[test]
fn a_pick_goes_out_on_change_on_join_and_after_a_load() {
    let picked = PlayerLook {
        hair_colour: 3,
        ..PlayerLook::UNPICKED
    };
    assert!(look_message_due(None, picked, false, false));
    assert!(!look_message_due(Some(picked), picked, false, false));
    assert!(look_message_due(Some(picked), picked, true, false));
    assert!(look_message_due(Some(picked), picked, false, true));
}
