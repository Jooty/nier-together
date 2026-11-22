use automata_game::outfit_look::{
    HairSwap, LOOK_UNPICKED, PartnerLookApplied, PartnerLookReading, PartnerLookStep, PlayerLook,
    partner_hair_swap, partner_look_steps,
};

const READING: PartnerLookReading = PartnerLookReading {
    outfit_body: 0,
    accessory_body: 0,
    outfit_saved: 0,
    accessory_saved: 0,
    pod_saved: -1,
};

fn look(outfit: u8, accessory: u8, hair: u8, pod: u8) -> PlayerLook {
    PlayerLook {
        outfit_variant: outfit,
        accessory_code: accessory,
        hair_colour: hair,
        pod_skin: pod,
        gun_skin: LOOK_UNPICKED,
    }
}

#[test]
fn an_unpicked_look_on_an_untouched_body_does_nothing() {
    let steps = partner_look_steps(
        &PlayerLook::UNPICKED,
        &READING,
        &PartnerLookApplied::default(),
    );
    assert!(steps.is_empty());
}

#[test]
fn a_picked_look_writes_outfit_and_accessory_and_swaps_the_pod_skin() {
    let steps = partner_look_steps(&look(2, 5, 3, 4), &READING, &PartnerLookApplied::default());
    assert_eq!(
        steps,
        vec![
            PartnerLookStep::WriteOutfit(2),
            PartnerLookStep::WriteAccessory(5),
            PartnerLookStep::SwapPodSkin {
                value: 4,
                restore: -1
            },
        ]
    );
}

#[test]
fn the_pod_swap_restores_the_value_the_local_save_held() {
    let reading = PartnerLookReading {
        pod_saved: 7,
        ..READING
    };
    let steps = partner_look_steps(
        &look(LOOK_UNPICKED, LOOK_UNPICKED, LOOK_UNPICKED, 1),
        &reading,
        &PartnerLookApplied::default(),
    );
    assert_eq!(
        steps,
        vec![PartnerLookStep::SwapPodSkin {
            value: 1,
            restore: 7
        }]
    );
}

#[test]
fn a_body_that_already_shows_the_look_gets_no_writes() {
    let reading = PartnerLookReading {
        outfit_body: 2,
        accessory_body: 5,
        ..READING
    };
    let mut applied = PartnerLookApplied::default();
    applied.note(PartnerLookStep::WriteOutfit(2));
    applied.note(PartnerLookStep::SwapPodSkin {
        value: 4,
        restore: -1,
    });
    assert!(partner_look_steps(&look(2, 5, 3, 4), &reading, &applied).is_empty());
}

#[test]
fn the_pod_skin_is_not_swapped_again_while_it_stands() {
    let mut applied = PartnerLookApplied::default();
    applied.note(PartnerLookStep::SwapPodSkin {
        value: 4,
        restore: -1,
    });
    let steps = partner_look_steps(
        &look(LOOK_UNPICKED, LOOK_UNPICKED, LOOK_UNPICKED, 4),
        &READING,
        &applied,
    );
    assert!(steps.is_empty());
}

#[test]
fn a_restore_path_that_reset_the_parts_brings_the_pod_swap_again() {
    let mut applied = PartnerLookApplied::default();
    applied.note(PartnerLookStep::WriteOutfit(2));
    applied.note(PartnerLookStep::SwapPodSkin {
        value: 4,
        restore: -1,
    });
    let steps = partner_look_steps(
        &look(2, LOOK_UNPICKED, LOOK_UNPICKED, 4),
        &READING,
        &applied,
    );
    assert_eq!(
        steps,
        vec![
            PartnerLookStep::WriteOutfit(2),
            PartnerLookStep::SwapPodSkin {
                value: 4,
                restore: -1
            },
        ]
    );
}

#[test]
fn a_look_taken_back_returns_the_body_to_the_local_saves_look() {
    let reading = PartnerLookReading {
        outfit_body: 2,
        accessory_body: 5,
        outfit_saved: 1,
        accessory_saved: 3,
        pod_saved: 8,
    };
    let mut applied = PartnerLookApplied::default();
    applied.note(PartnerLookStep::WriteOutfit(2));
    applied.note(PartnerLookStep::SwapPodSkin {
        value: 4,
        restore: 8,
    });
    let steps = partner_look_steps(&PlayerLook::UNPICKED, &reading, &applied);
    assert_eq!(
        steps,
        vec![
            PartnerLookStep::WriteOutfit(1),
            PartnerLookStep::WriteAccessory(3),
            PartnerLookStep::SwapPodSkin {
                value: 8,
                restore: 8
            },
        ]
    );
    for step in steps {
        applied.note(step);
    }
    assert_eq!(applied.pod_skin, None);
}

#[test]
fn forgetting_makes_a_new_body_start_clean() {
    let mut applied = PartnerLookApplied::default();
    applied.note(PartnerLookStep::WriteAccessory(5));
    applied.forget();
    assert_eq!(applied, PartnerLookApplied::default());
}

#[test]
fn hair_is_a_swap_only_when_picked() {
    assert_eq!(
        partner_hair_swap(&look(LOOK_UNPICKED, LOOK_UNPICKED, 9, LOOK_UNPICKED), 1),
        Some(HairSwap { slot: 1, value: 9 })
    );
    assert_eq!(partner_hair_swap(&PlayerLook::UNPICKED, 1), None);
}

#[test]
fn hair_needs_the_swap_only_when_the_global_differs() {
    let swap = HairSwap { slot: 0, value: 9 };
    assert!(swap.needs_swap(3));
    assert!(!swap.needs_swap(9));
}
