use replicant_game::effects::{
    ActorCueCall, MAX_LASTING_CUE_SOUNDS, ReplayedLastingSound, ReplayedLastingSounds,
    SentLastingSounds, by_id_sound_cue,
};
use replicant_game::offsets::SOUND_BY_ID_WRAPPER_BODY_RVAS;
use replicant_proto::{ActorCueKind, EnemyId};

const LOOP_SE_OBJECT_PLAY_CALLER_RVA: u32 = 0x18_A472;
const EMIL_LOOP_SOUND_ID: u32 = 56038;
const EMIL_SECOND_LOOP_SOUND_ID: u32 = 56031;
const HOST_SOUND_HANDLE: u64 = 0x2_9F10_4C00;
const GUEST_SOUND_HANDLE: u64 = 0x3_1A22_0800;

fn emil() -> EnemyId {
    EnemyId {
        group: 0xFFFF,
        ordinal: 0xFFFE,
    }
}

fn emil_loop_sound(sender_handle: u64, own_handle: u64) -> ReplayedLastingSound {
    ReplayedLastingSound {
        actor: emil(),
        id: EMIL_LOOP_SOUND_ID,
        sender_handle,
        own_handle,
        offset_from_copy: [0.0, 1.2, 0.0],
    }
}

#[test]
fn emils_loop_sound_from_the_loop_sound_object_is_sent_as_a_lasting_sound() {
    // L33 run1 host 01:56:09.224: "by-id sound 56038 at (-4.13, 1.20, 0.54) from 0x18a472",
    // while the variant companion was shared (01:56:03.144 to 01:58:07.153).
    let cue = by_id_sound_cue(
        LOOP_SE_OBJECT_PLAY_CALLER_RVA,
        [-4.13, 1.20, 0.54],
        [1.0, 0.0],
        HOST_SOUND_HANDLE,
    );
    assert_eq!(
        cue,
        Some(ActorCueKind::LastingSound {
            position: [-4.13, 1.20, 0.54],
            levels: [1.0, 0.0],
            handle: HOST_SOUND_HANDLE,
        })
    );
    assert_eq!(
        ActorCueCall::of_kind(&cue.unwrap()),
        ActorCueCall::LastingSoundById
    );
}

#[test]
fn a_wrapper_sound_stays_a_one_shot_and_a_lasting_sound_with_no_handle_is_not_sent() {
    let wrapper = SOUND_BY_ID_WRAPPER_BODY_RVAS.start + 4;
    assert!(matches!(
        by_id_sound_cue(wrapper, [0.0; 3], [1.0, 0.0], HOST_SOUND_HANDLE),
        Some(ActorCueKind::Sound { .. })
    ));
    assert_eq!(
        by_id_sound_cue(LOOP_SE_OBJECT_PLAY_CALLER_RVA, [0.0; 3], [1.0, 0.0], 0),
        None
    );
}

#[test]
fn the_hosts_stop_of_a_sent_handle_names_its_actor_and_sound_once() {
    let mut sent = SentLastingSounds::default();
    sent.note_sent(HOST_SOUND_HANDLE, emil(), EMIL_LOOP_SOUND_ID);
    assert_eq!(sent.take_stopped(0x1234), None);
    assert_eq!(
        sent.take_stopped(HOST_SOUND_HANDLE),
        Some((emil(), EMIL_LOOP_SOUND_ID))
    );
    assert_eq!(sent.take_stopped(HOST_SOUND_HANDLE), None);
    assert!(sent.is_empty());
}

#[test]
fn a_reused_host_handle_names_the_newer_sound() {
    let mut sent = SentLastingSounds::default();
    sent.note_sent(HOST_SOUND_HANDLE, emil(), EMIL_LOOP_SOUND_ID);
    sent.note_sent(HOST_SOUND_HANDLE, emil(), EMIL_SECOND_LOOP_SOUND_ID);
    assert_eq!(
        sent.take_stopped(HOST_SOUND_HANDLE),
        Some((emil(), EMIL_SECOND_LOOP_SOUND_ID))
    );
    assert!(sent.is_empty());
}

#[test]
fn the_guest_stops_its_own_handle_when_the_hosts_stop_arrives() {
    let mut replayed = ReplayedLastingSounds::default();
    assert!(
        replayed
            .track(emil_loop_sound(HOST_SOUND_HANDLE, GUEST_SOUND_HANDLE))
            .is_empty()
    );
    assert_eq!(
        replayed.take_stopped(emil(), EMIL_SECOND_LOOP_SOUND_ID, HOST_SOUND_HANDLE),
        None
    );
    assert_eq!(
        replayed.take_stopped(emil(), EMIL_LOOP_SOUND_ID, HOST_SOUND_HANDLE),
        Some(GUEST_SOUND_HANDLE)
    );
    assert!(replayed.playing().is_empty());
}

#[test]
fn the_guest_stops_a_replay_once_its_copy_is_no_longer_held() {
    // L34 run1 host 08:35:11.833: "the variant companion is no longer sent", so the guest's
    // copy is let go and its replayed sound must stop with it.
    let mut replayed = ReplayedLastingSounds::default();
    replayed.track(emil_loop_sound(HOST_SOUND_HANDLE, GUEST_SOUND_HANDLE));
    assert!(replayed.take_unheld(|_| true).is_empty());
    let stopped = replayed.take_unheld(|_| false);
    assert_eq!(stopped.len(), 1);
    assert_eq!(stopped[0].own_handle, GUEST_SOUND_HANDLE);
    assert!(replayed.playing().is_empty());
}

#[test]
fn the_same_host_handle_again_stops_the_older_replay_and_a_full_list_stops_the_oldest() {
    let mut replayed = ReplayedLastingSounds::default();
    replayed.track(emil_loop_sound(HOST_SOUND_HANDLE, GUEST_SOUND_HANDLE));
    assert_eq!(
        replayed.track(emil_loop_sound(HOST_SOUND_HANDLE, GUEST_SOUND_HANDLE + 8)),
        vec![GUEST_SOUND_HANDLE]
    );
    for index in 1..MAX_LASTING_CUE_SOUNDS as u64 {
        assert!(
            replayed
                .track(emil_loop_sound(index, GUEST_SOUND_HANDLE + 0x100 * index))
                .is_empty()
        );
    }
    assert_eq!(
        replayed.track(emil_loop_sound(0x9999, 0x9999)),
        vec![GUEST_SOUND_HANDLE + 8]
    );
}
