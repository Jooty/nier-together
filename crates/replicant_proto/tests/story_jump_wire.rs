use replicant_proto::{Message, StoryJump, StoryJumpPoint, decode, encode};

const ID_CODE_NAME_ALIAS: [(u16, &str, &str, &str); 28] = [
    (
        30,
        "VillageAfterPrologue",
        "Village, after prologue",
        "AfterPrologue",
    ),
    (
        40,
        "LibraryPopolaTalk",
        "Library, Popola talk",
        "AtPopolaTalk",
    ),
    (
        60,
        "VillageBeforeNextDay",
        "Village, before next day",
        "BeforeNextDay",
    ),
    (70, "ShrineEntrance", "Shrine, entrance", "AtShrineEntrance"),
    (80, "ShrineTop", "Shrine, top", "AtShrineTop"),
    (100, "ShrineBoss2", "Shrine, boss 2", "AtShrineBoss"),
    (120, "AerieBoss3", "Aerie, boss 3", "BossFight3"),
    (180, "JunkHeapB2", "Junk Heap, floor B2", "JunkHeapFloorB2"),
    (
        190,
        "JunkHeapBoss5",
        "Junk Heap, boss 5",
        "JunkHeapBossFight5",
    ),
    (200, "AerieBoss6", "Aerie, boss 6", "BossFight6"),
    (
        210,
        "AerieAfterBoss6",
        "Aerie, after boss 6",
        "AfterBossFight6",
    ),
    (
        240,
        "DesertAfterFacade",
        "Desert, after Facade",
        "AfterFacade",
    ),
    (250, "TempleEntrance", "Temple, entrance", "TempleEntrance"),
    (280, "TempleBoss7", "Temple, boss 7", "TempleBeforeBoss"),
    (
        290,
        "VillageAfterTemple",
        "Village, after temple",
        "VillageBackFromTemple",
    ),
    (310, "ForestArrival", "Forest, arrival", "ForestInside"),
    (
        340,
        "SouthPlainsRedBag",
        "South plains, red bag",
        "SeafrontWithRedBag",
    ),
    (350, "ManorFront", "Manor, front", "ManorFront"),
    (
        370,
        "ManorEmilFollows",
        "Manor, Emil follows",
        "ManorEmilFollows",
    ),
    (380, "ManorBoss8", "Manor, boss 8", "ManorBeforeBossFight8"),
    (400, "VillageBoss9", "Village, boss 9", "BossFight9"),
    (410, "LibraryBoss10", "Library, boss 10", "LibraryBoss10"),
    (420, "LibraryBoss11", "Library, boss 11", "LibraryBoss11"),
    (430, "Part2Start", "Part 2, start", "Part2Start"),
    (
        440,
        "Part2ManorCourtyard",
        "Part 2, manor courtyard",
        "Part2ManorCourtyard",
    ),
    (
        450,
        "Part2Boss12",
        "Boss fight 12",
        "Part2FacilitySublevel3",
    ),
    (460, "Part2Boss13", "Boss fight 13", "Part2Boss13"),
    (
        470,
        "Part2OutsideHouse",
        "Outside the house (part 2)",
        "Part2OutsideHouse",
    ),
];

#[test]
fn a_story_jump_round_trips_and_is_relayed() {
    let message = Message::StoryJump(StoryJump {
        point: StoryJumpPoint::VillageAfterPrologue,
    });
    assert!(message.is_relayed());
    assert_eq!(message.validate(), Ok(()));
    assert_eq!(decode(&encode(&message).unwrap()), Ok(message));
}

#[test]
fn the_popola_talk_jump_round_trips_and_is_relayed() {
    let message = Message::StoryJump(StoryJump {
        point: StoryJumpPoint::LibraryPopolaTalk,
    });
    assert!(message.is_relayed());
    assert_eq!(message.validate(), Ok(()));
    assert_eq!(decode(&encode(&message).unwrap()), Ok(message));
}

#[test]
fn the_next_day_jump_round_trips_and_is_relayed() {
    let message = Message::StoryJump(StoryJump {
        point: StoryJumpPoint::VillageBeforeNextDay,
    });
    assert!(message.is_relayed());
    assert_eq!(message.validate(), Ok(()));
    assert_eq!(decode(&encode(&message).unwrap()), Ok(message));
}

#[test]
fn a_teleport_here_round_trips_and_is_relayed() {
    let message = Message::TeleportHere(replicant_proto::TeleportHere {
        phase: 7,
        position: [1.5, 2.0, -3.25],
    });
    assert!(message.is_relayed());
    assert_eq!(message.validate(), Ok(()));
    assert_eq!(decode(&encode(&message).unwrap()), Ok(message));
}

#[test]
fn a_teleport_here_with_a_non_finite_position_is_refused() {
    let message = Message::TeleportHere(replicant_proto::TeleportHere {
        phase: 7,
        position: [f32::NAN, 0.0, 0.0],
    });
    assert!(message.validate().is_err());
}

#[test]
fn the_boss_fight_3_jump_round_trips_and_is_relayed() {
    let message = Message::StoryJump(StoryJump {
        point: StoryJumpPoint::AerieBoss3,
    });
    assert!(message.is_relayed());
    assert_eq!(message.validate(), Ok(()));
    assert_eq!(decode(&encode(&message).unwrap()), Ok(message));
}

#[test]
fn the_junk_heap_jump_round_trips_and_is_relayed() {
    let message = Message::StoryJump(StoryJump {
        point: StoryJumpPoint::JunkHeapB2,
    });
    assert!(message.is_relayed());
    assert_eq!(message.validate(), Ok(()));
    assert_eq!(decode(&encode(&message).unwrap()), Ok(message));
}

#[test]
fn the_boss_fight_5_jump_round_trips_and_is_relayed() {
    let message = Message::StoryJump(StoryJump {
        point: StoryJumpPoint::JunkHeapBoss5,
    });
    assert!(message.is_relayed());
    assert_eq!(message.validate(), Ok(()));
    assert_eq!(decode(&encode(&message).unwrap()), Ok(message));
}

#[test]
fn the_after_boss_fight_6_jump_round_trips_and_is_relayed() {
    let message = Message::StoryJump(StoryJump {
        point: StoryJumpPoint::AerieAfterBoss6,
    });
    assert!(message.is_relayed());
    assert_eq!(message.validate(), Ok(()));
    assert_eq!(decode(&encode(&message).unwrap()), Ok(message));
}

#[test]
fn the_after_the_facade_jump_round_trips_and_is_relayed() {
    let message = Message::StoryJump(StoryJump {
        point: StoryJumpPoint::DesertAfterFacade,
    });
    assert!(message.is_relayed());
    assert_eq!(message.validate(), Ok(()));
    assert_eq!(decode(&encode(&message).unwrap()), Ok(message));
}

#[test]
fn a_gather_for_scene_round_trips_and_is_relayed() {
    let message = Message::GatherForScene(replicant_proto::GatherForScene {
        phase: 7,
        position: [1.5, 2.0, -3.25],
        in_side_view: true,
        side_view_door: 3,
    });
    assert!(message.is_relayed());
    assert_eq!(message.validate(), Ok(()));
    assert_eq!(decode(&encode(&message).unwrap()), Ok(message));
}

#[test]
fn a_gather_for_scene_with_a_non_finite_position_is_refused() {
    let message = Message::GatherForScene(replicant_proto::GatherForScene {
        phase: 7,
        position: [0.0, f32::INFINITY, 0.0],
        in_side_view: false,
        side_view_door: 0,
    });
    assert!(message.validate().is_err());
}

#[test]
fn every_gather_answer_round_trips_and_is_relayed() {
    for reply in [
        replicant_proto::GatherReply::Ready,
        replicant_proto::GatherReply::InAnotherArea,
        replicant_proto::GatherReply::InOwnTalk,
        replicant_proto::GatherReply::InOwnScene,
        replicant_proto::GatherReply::Downed,
        replicant_proto::GatherReply::SideViewNotGiven,
        replicant_proto::GatherReply::DidNotArrive,
    ] {
        let message = Message::GatherAnswer(replicant_proto::GatherAnswer { phase: 7, reply });
        assert!(message.is_relayed());
        assert_eq!(message.validate(), Ok(()));
        assert_eq!(decode(&encode(&message).unwrap()), Ok(message));
    }
}

#[test]
fn every_jump_point_round_trips_by_its_id_and_is_relayed() {
    for (id, _, _, _) in ID_CODE_NAME_ALIAS {
        let point = StoryJumpPoint::from_id(id).expect("the id is a point");
        assert_eq!(point.id(), id);
        let message = Message::StoryJump(StoryJump { point });
        assert!(message.is_relayed());
        assert_eq!(message.validate(), Ok(()));
        assert_eq!(decode(&encode(&message).unwrap()), Ok(message));
    }
}

#[test]
fn a_jump_point_is_sent_as_its_id_number_not_its_enum_index() {
    for (id, _, _, _) in ID_CODE_NAME_ALIAS {
        let point = StoryJumpPoint::from_id(id).unwrap();
        let bytes = postcard::to_allocvec(&point).unwrap();
        assert_eq!(bytes, postcard::to_allocvec(&id).unwrap());
    }
    let temple_boss = StoryJumpPoint::from_id(280).unwrap();
    assert_eq!(
        postcard::to_allocvec(&temple_boss).unwrap(),
        vec![0x98, 0x02]
    );
}

#[test]
fn an_id_that_is_no_point_is_refused_on_the_wire() {
    for id in [0u16, 10, 50, 399, 401, u16::MAX] {
        let bytes = postcard::to_allocvec(&id).unwrap();
        assert!(postcard::from_bytes::<StoryJumpPoint>(&bytes).is_err());
        assert_eq!(StoryJumpPoint::from_id(id), None);
    }
}

#[test]
fn the_ids_codes_and_names_match_the_list() {
    for (id, code, name, _) in ID_CODE_NAME_ALIAS {
        let point = StoryJumpPoint::from_id(id).unwrap();
        assert_eq!(point.code(), code);
        assert_eq!(point.name(), name);
        assert_eq!(StoryJumpPoint::from_code(code), Some(point));
    }
}

#[test]
fn the_points_are_in_id_order_with_no_repeats() {
    let ids: Vec<u16> = StoryJumpPoint::in_id_order()
        .map(StoryJumpPoint::id)
        .collect();
    let mut sorted = ids.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(ids, sorted);
    assert_eq!(ids.len(), 28);
}

#[test]
fn the_old_variant_names_are_still_accepted_as_aliases() {
    for (id, _, _, alias) in ID_CODE_NAME_ALIAS {
        assert_eq!(
            StoryJumpPoint::from_code(alias).map(StoryJumpPoint::id),
            Some(id)
        );
    }
    assert_eq!(StoryJumpPoint::from_code("NoSuchPoint"), None);
}

#[test]
fn a_harness_command_names_a_point_by_old_name_new_code_or_id() {
    let by_old_name: StoryJumpPoint = serde_json::from_str("\"BossFight9\"").unwrap();
    let by_new_code: StoryJumpPoint = serde_json::from_str("\"VillageBoss9\"").unwrap();
    let by_id: StoryJumpPoint = serde_json::from_str("400").unwrap();
    assert_eq!(by_old_name, StoryJumpPoint::VillageBoss9);
    assert_eq!(by_new_code, by_old_name);
    assert_eq!(by_id, by_old_name);
    assert!(serde_json::from_str::<StoryJumpPoint>("\"NoSuchPoint\"").is_err());
    assert!(serde_json::from_str::<StoryJumpPoint>("401").is_err());
    assert_eq!(serde_json::to_string(&by_id).unwrap(), "\"VillageBoss9\"");
}

#[test]
fn the_menu_lists_the_points_in_id_order_by_name() {
    let listed: Vec<(u16, &str)> = StoryJumpPoint::in_id_order()
        .map(|point| (point.id(), point.name()))
        .collect();
    let expected: Vec<(u16, &str)> = ID_CODE_NAME_ALIAS
        .iter()
        .map(|(id, _, name, _)| (*id, *name))
        .collect();
    assert_eq!(listed, expected);
    assert!(listed.windows(2).all(|pair| pair[0].0 < pair[1].0));
}
