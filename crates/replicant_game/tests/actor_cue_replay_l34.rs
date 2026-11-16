use replicant_game::effects::{
    ActorCueMoment, AttachedCueEffect, AttachedCueEffects, CopyForCue, actor_cue_is_sent,
    actor_cue_replay_target,
};
use replicant_proto::EnemyId;

const HOST_LINES: &[&str] = &[
    "08:33:02.898 companion (host): unique actor -2 is sent to the guest, whose own follows it",
    "08:33:22.482 companion (host): unique actor -2 is no longer sent (not active here, or a scene plays here)",
    "08:33:26.444 companion (host): the variant companion is sent to the guest, whose own follows it",
    "08:35:11.833 companion (host): the variant companion is no longer sent (not active here, or a scene plays here)",
];

const GUEST_LINES: &[&str] = &[
    "08:32:58.933 companion (guest): unique actor -2 is left to this game: a script drives it here",
    "08:33:04.354 companion (guest): unique actor -2 is left to this game: a script drives it here",
    "08:33:26.416 companion (guest): the variant companion is left to this game: the host sends none (not active there, or in a scene there)",
    "08:33:26.464 companion (guest): the variant companion is held here and follows the host's (place, motion and life)",
];

const KAINE_COPY: u64 = 0x5100_0000;
const EMIL_COPY: u64 = 0x5200_0000;
const HOST_EMIL: u64 = 0x6200_0000;
const EMIL_MAGIC_ARRAY_EFFECT_ID: u32 = 458;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Companion {
    Kaine,
    Emil,
}

fn companion_id(companion: Companion) -> EnemyId {
    match companion {
        Companion::Kaine => EnemyId {
            group: 0xFFFF,
            ordinal: 0xFFFE,
        },
        Companion::Emil => EnemyId {
            group: 0xFFFF,
            ordinal: 0xFFFD,
        },
    }
}

fn ms_of(line: &str) -> u64 {
    let clock: Vec<f64> = line[..12]
        .split(':')
        .map(|part| part.parse().unwrap())
        .collect();
    ((clock[0] * 3600.0 + clock[1] * 60.0 + clock[2]) * 1000.0).round() as u64
}

fn companion_of(line: &str) -> Companion {
    if line.contains("unique actor -2") {
        Companion::Kaine
    } else {
        assert!(line.contains("the variant companion"), "{line}");
        Companion::Emil
    }
}

fn last_line_before<'a>(lines: &[&'a str], companion: Companion, at_ms: u64) -> Option<&'a str> {
    lines
        .iter()
        .copied()
        .rfind(|line| companion_of(line) == companion && ms_of(line) <= at_ms)
}

fn host_shares(companion: Companion, at_ms: u64) -> bool {
    last_line_before(HOST_LINES, companion, at_ms).is_some_and(|line| line.contains(" is sent "))
}

fn guest_copy(companion: Companion, at_ms: u64) -> Option<CopyForCue> {
    let line = last_line_before(GUEST_LINES, companion, at_ms)?;
    Some(CopyForCue {
        actor: match companion {
            Companion::Kaine => KAINE_COPY,
            Companion::Emil => EMIL_COPY,
        },
        held: line.contains("is held here"),
        live: true,
    })
}

fn hosts_ai_step(companion: Companion, at_ms: u64) -> ActorCueMoment {
    ActorCueMoment {
        hosting: true,
        shared_actor: host_shares(companion, at_ms),
        in_ai_step: true,
        ..ActorCueMoment::default()
    }
}

fn plays_on_the_guest(companion: Companion, at: &str) -> Option<u64> {
    let at_ms = ms_of(at);
    actor_cue_is_sent(hosts_ai_step(companion, at_ms))
        .then(|| actor_cue_replay_target(guest_copy(companion, at_ms)))
        .flatten()
}

#[test]
fn kaines_cues_while_a_script_drives_her_on_the_guest_are_sent_but_not_replayed() {
    // R2c-D2: the host shares her from 08:33:02.898, the guest leaves her to her script from
    // 08:33:04.354; both games ran her AI, so a replay would have doubled every effect.
    let at = "08:33:10.000";
    assert!(actor_cue_is_sent(hosts_ai_step(
        Companion::Kaine,
        ms_of(at)
    )));
    assert_eq!(plays_on_the_guest(Companion::Kaine, at), None);
}

#[test]
fn emils_cues_while_the_guest_holds_his_copy_are_replayed_on_it() {
    assert_eq!(
        plays_on_the_guest(Companion::Emil, "08:33:30.000"),
        Some(EMIL_COPY)
    );
}

#[test]
fn emils_cues_in_the_48_ms_before_the_guest_held_him_are_not_replayed() {
    // The host shares him at 08:33:26.444; the guest holds its copy at 08:33:26.464.
    assert_eq!(plays_on_the_guest(Companion::Emil, "08:33:26.450"), None);
}

#[test]
fn nothing_is_sent_once_the_host_no_longer_shares_emil_and_his_attaches_are_handed_back() {
    let end = "08:35:11.833";
    assert!(!actor_cue_is_sent(hosts_ai_step(
        Companion::Emil,
        ms_of(end)
    )));
    let mut sent = AttachedCueEffects::default();
    sent.note_attached(AttachedCueEffect {
        actor: HOST_EMIL,
        enemy: companion_id(Companion::Emil),
        id: EMIL_MAGIC_ARRAY_EFFECT_ID,
        bone: -1,
    });
    let before_end = ms_of("08:35:11.800");
    assert!(
        sent.take_gone(|_, _| host_shares(Companion::Emil, before_end))
            .is_empty()
    );
    let gone = sent.take_gone(|_, _| host_shares(Companion::Emil, ms_of(end)));
    assert_eq!(gone.len(), 1);
    assert_eq!(gone[0].id, EMIL_MAGIC_ARRAY_EFFECT_ID);
}
