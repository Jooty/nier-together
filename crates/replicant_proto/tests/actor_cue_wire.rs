use replicant_proto::{
    ActorCue, ActorCueKind, AttachedEffect, EnemyId, MAX_ACTOR_CUE_ID, MAX_MESSAGE_BYTES, Message,
    StoneHandler, StoneTurn, WireError, decode, encode,
};

fn emil() -> EnemyId {
    EnemyId {
        group: 0xFFFF,
        ordinal: 0xFFFE,
    }
}

fn magic_array() -> ActorCue {
    ActorCue {
        actor: emil(),
        id: 458,
        kind: ActorCueKind::AttachEffect(AttachedEffect {
            bone: -1,
            unique: false,
            colour: [1.0; 4],
            style: [1.0; 4],
            mode: 0,
            flags: 0,
            extra: 0,
        }),
    }
}

fn shield_sound() -> ActorCue {
    ActorCue {
        actor: EnemyId {
            group: 308,
            ordinal: 1,
        },
        id: 15075,
        kind: ActorCueKind::Sound {
            position: [-74.3, 3.3, 20.5],
            levels: [1.0, 0.0],
        },
    }
}

fn emil_loop_sound() -> ActorCue {
    ActorCue {
        actor: emil(),
        id: 56038,
        kind: ActorCueKind::LastingSound {
            position: [-4.13, 1.2, 0.54],
            levels: [1.0, 0.0],
            handle: 0x2_9F10_4C00,
        },
    }
}

fn emil_loop_sound_stop() -> ActorCue {
    ActorCue {
        actor: emil(),
        id: 56038,
        kind: ActorCueKind::StopSound {
            handle: 0x2_9F10_4C00,
        },
    }
}

#[test]
fn every_kind_round_trips_fits_a_message_and_is_relayed() {
    let mut transform = [0.0_f32; 12];
    transform[0] = 1.0;
    transform[5] = 1.0;
    transform[10] = 1.0;
    transform[3] = 150.0;
    transform[11] = 135.0;
    let stone = ActorCue {
        actor: EnemyId {
            group: 9,
            ordinal: 1,
        },
        id: 0,
        kind: ActorCueKind::TurnedToStone(StoneTurn {
            handler: StoneHandler::Leader,
            state_bytes: [1, 0],
        }),
    };
    let cues = [
        magic_array(),
        shield_sound(),
        stone,
        ActorCue {
            kind: ActorCueKind::RemoveEffect {
                bone: -1,
                all_matching: true,
            },
            ..magic_array()
        },
        ActorCue {
            id: 5001,
            kind: ActorCueKind::PlaceEffect {
                transform,
                handle: 0x1234,
            },
            ..magic_array()
        },
        emil_loop_sound(),
        emil_loop_sound_stop(),
        ActorCue {
            id: 213,
            kind: ActorCueKind::PlacedEffectEnded { handle: 0x1234 },
            ..magic_array()
        },
        ActorCue {
            kind: ActorCueKind::StoneBroken,
            ..stone
        },
    ];
    for cue in cues {
        let message = Message::ActorCue(cue);
        assert!(message.is_relayed());
        assert_eq!(message.kind_name(), "ActorCue");
        assert_eq!(message.validate(), Ok(()));
        let bytes = encode(&message).unwrap();
        assert!(bytes.len() <= MAX_MESSAGE_BYTES);
        assert_eq!(decode(&bytes), Ok(message));
    }
}

#[test]
fn an_actor_cue_out_of_range_is_refused() {
    let bad_id = ActorCue {
        id: MAX_ACTOR_CUE_ID + 1,
        ..magic_array()
    };
    let bad_sound = ActorCue {
        kind: ActorCueKind::Sound {
            position: [f32::NAN, 0.0, 0.0],
            levels: [1.0, 0.0],
        },
        ..shield_sound()
    };
    let bad_colour = ActorCue {
        kind: ActorCueKind::AttachEffect(AttachedEffect {
            bone: 0,
            unique: false,
            colour: [f32::INFINITY, 1.0, 1.0, 1.0],
            style: [1.0; 4],
            mode: 0,
            flags: 0,
            extra: 0,
        }),
        ..magic_array()
    };
    assert_eq!(
        bad_id.validate(),
        Err(WireError::OutOfRange {
            field: "actor cue id"
        })
    );
    let bad_lasting = ActorCue {
        kind: ActorCueKind::LastingSound {
            position: [0.0, f32::INFINITY, 0.0],
            levels: [1.0, 0.0],
            handle: 1,
        },
        ..emil_loop_sound()
    };
    for bad in [bad_sound, bad_colour, bad_lasting] {
        assert_eq!(
            bad.validate(),
            Err(WireError::OutOfRange { field: "actor cue" })
        );
    }
}

#[test]
fn a_lasting_sound_stop_has_its_fixed_bytes() {
    let bytes = encode(&Message::ActorCue(emil_loop_sound_stop())).unwrap();
    let hex: String = bytes.iter().map(|byte| format!("{byte:02x}")).collect();
    assert_eq!(hex, "7fffff03feff03e6b503068098c1f829");
}

#[test]
fn a_stone_break_has_its_fixed_bytes() {
    let broken = ActorCue {
        actor: EnemyId {
            group: 9,
            ordinal: 0,
        },
        id: 0,
        kind: ActorCueKind::StoneBroken,
    };
    let bytes = encode(&Message::ActorCue(broken)).unwrap();
    let hex: String = bytes.iter().map(|byte| format!("{byte:02x}")).collect();
    assert_eq!(hex, "7f09000007");
}
