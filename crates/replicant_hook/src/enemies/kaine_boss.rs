use std::collections::{BTreeMap, HashMap};
use std::ffi::CString;
use std::sync::{LazyLock, Mutex, PoisonError};
use std::time::{Duration, Instant};

use replicant_game::effects::projectile_owner;
use replicant_game::enemies::{AttackRecord, HitReaction, MirroredEnemy, shadow_matrix};
use replicant_game::kaine::{
    BossDriveMisses, BossLifeFollow, BossLifeStep, BossNierHits, HerMotionChoice,
    PLAYER_SWING_TARGET_KIND_BIT, PVP_DAMAGE_PERCENT, SentHitVerdict, her_motion_for,
    is_the_boss_kaine, pvp_scaled_damage, scaled_frame,
};
use replicant_game::npcs::read_enemy_body_layer;
use replicant_game::offsets::{
    ACTOR_KIND_MASK_OFFSET, ACTOR_TRANSFORM_BLOCK_BYTES, ACTOR_TRANSFORM_BLOCK_OFFSET,
    ATTACK_RECORD_HP_DAMAGE_OFFSET, BODY_MOTION_LAYER, PARTNER_KAINE_ACTOR_RVA,
    PLAYER_MESSAGE_DIRECT_HIT,
};
use replicant_game::puppet::{
    MotionEntry, MotionStep, follow_step, frame_to_seek, motion_entries, motion_entry_for_hash,
    motion_name, read_layer_motion, read_puppet_layer, seek_layer_frame,
};
use replicant_game::{GameMemory, PhaseEntry, Replicant};
use replicant_proto::{
    BossHitOnNier, EnemyId, Message, NO_MOTION_NAME_HASH, TrackSample, motion_name_hash,
};

use crate::build_gate::VerifiedBuild;
use crate::diagnostics::log_line;
use crate::enemies::combat_view::{applying_remote, as_remote_with_reaction};
use crate::enemies::game_calls::EnemyCalls;
use crate::enemies::sync::LocalView;
use crate::game_hooks::{set_local_health_through_game, while_menu_free};
use crate::memory::InProcessMemory;
use crate::net::{self, net_clock_ms};
use crate::paused_hits::paused_players_can_be_hit;
use crate::stay_together::player_in_menu;

const BOSS_DRIVE_LINE_INTERVAL: Duration = Duration::from_secs(1);

static FOLLOW: LazyLock<Mutex<Follow>> = LazyLock::new(Mutex::default);
static NIER_HITS: LazyLock<Mutex<BossNierHits>> = LazyLock::new(Mutex::default);
static LIFE: LazyLock<Mutex<BossLifeFollow>> = LazyLock::new(Mutex::default);
static GHOST_KIND: Mutex<Option<(u64, i32)>> = Mutex::new(None);

#[derive(Default)]
struct Follow {
    guest_table: Option<(u64, Vec<MotionEntry>)>,
    guest_names: HashMap<u32, Option<String>>,
    her_indices: HashMap<&'static str, Option<i32>>,
    last_started: Option<i32>,
    her_motion: Option<&'static str>,
    misses: BossDriveMisses,
    unreadable_logged: bool,
    no_name_logged: bool,
    placed: u32,
    place_failures: u32,
    seeks: u32,
    starts: u32,
    last_line: Option<Instant>,
}

pub(crate) fn boss_kaine_life(build: &VerifiedBuild) -> Option<(i32, i32)> {
    let memory = InProcessMemory::new(build);
    let her = memory.module_base() + u64::from(PARTNER_KAINE_ACTOR_RVA);
    EnemyCalls::new(build).hp_of(&memory, her)
}

pub(crate) fn reset_boss_pvp() {
    NIER_HITS
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .reset();
    let mut follow = FOLLOW.lock().unwrap_or_else(PoisonError::into_inner);
    let her_indices = std::mem::take(&mut follow.her_indices);
    *follow = Follow {
        her_indices,
        ..Follow::default()
    };
}

pub(crate) fn drive_kaine_boss(build: &VerifiedBuild, sample: &TrackSample) {
    let mut memory = InProcessMemory::new(build);
    let calls = EnemyCalls::new(build);
    let her = memory.module_base() + u64::from(PARTNER_KAINE_ACTOR_RVA);
    let mut follow = FOLLOW.lock().unwrap_or_else(PoisonError::into_inner);
    let placed = place(&memory, &calls, her, sample);
    if placed {
        follow.placed += 1;
    } else {
        follow.place_failures += 1;
    }
    follow_guest_motion(&mut follow, &mut memory, &calls, her, sample);
    if follow
        .last_line
        .is_some_and(|at| at.elapsed() < BOSS_DRIVE_LINE_INTERVAL)
    {
        return;
    }
    follow.last_line = Some(Instant::now());
    log_line(&format!(
        "boss drive: last second she was placed {} times ({} refused), {} motions started, {} frame seeks; guest at ({:.1}, {:.1}, {:.1}) yaw {:.2}, her life {}, her motion {}",
        std::mem::take(&mut follow.placed),
        std::mem::take(&mut follow.place_failures),
        std::mem::take(&mut follow.starts),
        std::mem::take(&mut follow.seeks),
        sample.position[0],
        sample.position[1],
        sample.position[2],
        sample.yaw_radians,
        calls.hp_of(&memory, her).map_or_else(
            || "unreadable".to_owned(),
            |(hp, max_hp)| format!("{hp} of {max_hp}")
        ),
        read_enemy_body_layer(&memory, her)
            .ok()
            .flatten()
            .map_or_else(
                || "unreadable".to_owned(),
                |layer| format!(
                    "{} frame {:.1} of {}",
                    layer.motion.index, layer.motion.frame, layer.frame_count
                )
            ),
    ));
}

fn place(memory: &InProcessMemory, calls: &EnemyCalls, her: u64, sample: &TrackSample) -> bool {
    let Ok(block) =
        memory.read_array::<ACTOR_TRANSFORM_BLOCK_BYTES>(her + ACTOR_TRANSFORM_BLOCK_OFFSET)
    else {
        return false;
    };
    let matrix = shadow_matrix(&block, sample.position, sample.yaw_radians);
    calls.set_transform(memory, her, &matrix)
}

fn follow_guest_motion(
    follow: &mut Follow,
    memory: &mut InProcessMemory,
    calls: &EnemyCalls,
    her: u64,
    sample: &TrackSample,
) {
    let Some(motion) = sample.motion else {
        return;
    };
    if motion.name_hash == NO_MOTION_NAME_HASH {
        if !follow.no_name_logged {
            follow.no_name_logged = true;
            log_line("boss drive: the guest's motion carries no name, she keeps her last motion");
        }
        return;
    }
    let Some(puppet) = crate::puppet::first_actor_address() else {
        return;
    };
    let Some(guest_name) = guest_motion_name(follow, memory, puppet, motion.name_hash) else {
        return;
    };
    let choice = her_motion_for(&guest_name, |wanted| her_index(follow, calls, her, wanted));
    if let Some(text) = follow.misses.observe(&guest_name, choice) {
        log_line(&text);
    }
    let HerMotionChoice::Play { wanted, index } = choice else {
        return;
    };
    let guest_frames = read_puppet_layer(&*memory, puppet, BODY_MOTION_LAYER)
        .ok()
        .flatten()
        .map_or(0, |layer| layer.frame_count);
    let layer = read_enemy_body_layer(&*memory, her).ok().flatten();
    let her_frames = layer.map_or(0, |layer| layer.frame_count);
    let wanted_frame = scaled_frame(motion.frame, guest_frames, her_frames);
    match follow_step(layer.map(|layer| layer.motion), follow.last_started, index) {
        MotionStep::Start { blend } => {
            calls.play_body_motion(her, index, blend, motion.looping, 1.0, wanted_frame);
            follow.last_started = Some(index);
            follow.her_motion = Some(wanted);
            follow.starts += 1;
            log_line(&format!(
                "boss drive: the guest's {guest_name} plays as {wanted} on her (index {index})"
            ));
        }
        MotionStep::Keep => {
            let Some(layer) = layer.filter(|layer| layer.motion.index == index) else {
                return;
            };
            if let Some(frame) = frame_to_seek(
                layer.motion.frame,
                wanted_frame,
                layer.frame_count,
                layer.motion.looping,
            ) && seek_layer_frame(memory, &layer, frame).is_ok()
            {
                follow.seeks += 1;
            }
        }
    }
}

fn guest_motion_name(
    follow: &mut Follow,
    memory: &InProcessMemory,
    puppet: u64,
    hash: u32,
) -> Option<String> {
    if let Some(name) = follow.guest_names.get(&hash) {
        return name.clone();
    }
    if follow
        .guest_table
        .as_ref()
        .is_none_or(|(actor, table)| *actor != puppet || table.is_empty())
    {
        follow.guest_names.clear();
        let table = motion_entries(memory, puppet).unwrap_or_default();
        if table.is_empty() {
            if !follow.unreadable_logged {
                follow.unreadable_logged = true;
                log_line("boss drive: the guest's puppet has no motion names to read yet");
            }
            return None;
        }
        follow.guest_table = Some((puppet, table));
    }
    let (_, table) = follow.guest_table.as_ref()?;
    let name = motion_entry_for_hash(table, hash).map(|entry| entry.name.clone());
    follow.guest_names.insert(hash, name.clone());
    name
}

fn her_index(
    follow: &mut Follow,
    calls: &EnemyCalls,
    her: u64,
    wanted: &'static str,
) -> Option<i32> {
    if let Some(resolved) = follow.her_indices.get(wanted) {
        return *resolved;
    }
    let resolved = [wanted.to_owned(), wanted.to_ascii_uppercase()]
        .into_iter()
        .filter_map(|spelling| CString::new(spelling).ok())
        .map(|name| calls.motion_index_by_name(her, &name))
        .find(|index| *index >= 0);
    follow.her_indices.insert(wanted, resolved);
    log_line(&format!(
        "boss drive: her pack answers {wanted} with {}",
        resolved.map_or_else(|| "no motion".to_owned(), |index| format!("index {index}"))
    ));
    resolved
}

pub(super) fn follow_boss_life(local: &LocalView<'_>, hosts: &BTreeMap<EnemyId, MirroredEnemy>) {
    let in_the_fight = crate::kaine::guest_in_boss_fight();
    tune_host_ghost_kind(local.build, in_the_fight);
    let mut life = LIFE.lock().unwrap_or_else(PoisonError::into_inner);
    if !in_the_fight {
        life.reset();
        return;
    }
    super::kaine_boss_gauge::hide_boss_gauge(local.build);
    let boss = hosts
        .values()
        .find(|enemy| is_the_boss_kaine(enemy.id))
        .map(|enemy| (enemy.hp, enemy.max_hp));
    if let Some((hp, max_hp)) = boss {
        crate::kaine::note_boss_life_seen(hp, max_hp);
    }
    let (Ok(own_hp), Ok(own_max)) = (local.game.health(), local.game.max_health()) else {
        return;
    };
    let BossLifeStep::Set {
        to,
        host_hp,
        host_max_hp,
    } = life.step(boss, own_hp, own_max)
    else {
        return;
    };
    let written = set_local_health_through_game(to)
        || Replicant::new(InProcessMemory::new(local.build))
            .set_health(to)
            .is_ok();
    log_line(&format!(
        "boss drive (guest): your life follows hers, the host has her at {host_hp} of {host_max_hp}, you were at {own_hp} of {own_max}, now {to}{}",
        if written { "" } else { " (the write failed)" }
    ));
}

fn tune_host_ghost_kind(build: &VerifiedBuild, in_the_fight: bool) {
    let mut memory = InProcessMemory::new(build);
    let mut remembered = GHOST_KIND.lock().unwrap_or_else(PoisonError::into_inner);
    let ghost = crate::puppet::first_actor_address();
    if let Some((actor, original)) = *remembered
        && (!in_the_fight || ghost != Some(actor))
    {
        if ghost == Some(actor) {
            let _ = memory.write_i32(actor + ACTOR_KIND_MASK_OFFSET, original);
        }
        *remembered = None;
        log_line("boss drive (guest): the host's Nier ghost is no longer a target");
    }
    let Some(ghost) = ghost.filter(|_| in_the_fight) else {
        return;
    };
    let Ok(current) = memory.read_i32(ghost + ACTOR_KIND_MASK_OFFSET) else {
        return;
    };
    let original = remembered.map_or(current, |(_, original)| original);
    let wanted = original | PLAYER_SWING_TARGET_KIND_BIT;
    if current != wanted
        && memory
            .write_i32(ghost + ACTOR_KIND_MASK_OFFSET, wanted)
            .is_ok()
    {
        log_line(&format!(
            "boss drive (guest): the host's Nier ghost can be hit now (kind {original:#x} -> {wanted:#x})"
        ));
    }
    *remembered = Some((ghost, original));
}

pub(crate) fn guest_hit_on_host_ghost(build: &VerifiedBuild, victim: u64, record: u64) -> bool {
    if !crate::kaine::guest_in_boss_fight() || crate::puppet::first_actor_address() != Some(victim)
    {
        return false;
    }
    let game = Replicant::new(InProcessMemory::new(build));
    let memory = game.memory();
    let (Ok(player), Some(attack), Some(reaction)) = (
        game.playable_actor(),
        AttackRecord::read(memory, record),
        HitReaction::read(memory, record),
    ) else {
        return false;
    };
    if !attack.sources(memory).contains(&Some(player)) {
        return false;
    }
    let motion_hash = read_layer_motion(memory, player, BODY_MOTION_LAYER)
        .ok()
        .flatten()
        .and_then(|motion| motion_name(memory, player, motion.index).ok().flatten())
        .map_or(NO_MOTION_NAME_HASH, |name| motion_name_hash(&name));
    let Some(phase) = game
        .current_phase()
        .ok()
        .and_then(|name| PhaseEntry::index_of(&name))
        .and_then(|index| u16::try_from(index).ok())
    else {
        return true;
    };
    let Some(net) = net::handle() else {
        return true;
    };
    net.send_relayed(Message::BossHitOnNier(BossHitOnNier {
        phase,
        hit_id: attack.hit_id,
        motion_name_hash: motion_hash,
        attack_kind: attack.attack_kind.cast_unsigned(),
        flags: attack.flags,
        push: reaction.push,
        direction: reaction.direction,
    }));
    log_line(&format!(
        "boss drive (guest): your attack connected with the host's Nier (hit id {}, flags {:#x}, kind {}, your motion hash {motion_hash:#010x}), sent",
        attack.hit_id, attack.flags, attack.attack_kind
    ));
    true
}

pub(crate) fn note_native_boss_hit(build: &VerifiedBuild, victim: u64, record: u64) {
    if !crate::kaine::boss_fight_is_live() || applying_remote() {
        return;
    }
    let game = Replicant::new(InProcessMemory::new(build));
    if game.playable_actor().ok() != Some(victim) {
        return;
    }
    let memory = game.memory();
    let her = memory.module_base() + u64::from(PARTNER_KAINE_ACTOR_RVA);
    let Some(attack) = AttackRecord::read(memory, record) else {
        return;
    };
    if !attack.sources(memory).contains(&Some(her)) {
        return;
    }
    let motion = FOLLOW
        .try_lock()
        .ok()
        .and_then(|follow| follow.her_motion)
        .unwrap_or("unknown");
    NIER_HITS
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .native_hit(net_clock_ms(), motion, attack.hp_damage);
    log_line(&format!(
        "boss drive: her attack hit the host's Nier natively (hp {}, flags {:#x}, kind {}, hit id {}, her motion {motion})",
        attack.hp_damage, attack.flags, attack.attack_kind, attack.hit_id
    ));
}

pub(crate) fn scale_host_hit_on_driven_kaine(build: &VerifiedBuild, victim: u64, record: u64) {
    if !crate::kaine::boss_drive_is_on() || applying_remote() {
        return;
    }
    let game = Replicant::new(InProcessMemory::new(build));
    let her = game.memory().module_base() + u64::from(PARTNER_KAINE_ACTOR_RVA);
    let Ok(player) = game.playable_actor() else {
        return;
    };
    if victim != her {
        return;
    }
    let mut memory = InProcessMemory::new(build);
    let Some(attack) = AttackRecord::read(&memory, record) else {
        return;
    };
    let owner = attack.sources(&memory)[0].and_then(|source| projectile_owner(&memory, source));
    if !attack.sources(&memory).contains(&Some(player)) && owner != Some(player) {
        return;
    }
    let scaled = pvp_scaled_damage(attack.hp_damage);
    if scaled == attack.hp_damage
        || memory
            .write_i32(record + ATTACK_RECORD_HP_DAMAGE_OFFSET, scaled)
            .is_err()
    {
        return;
    }
    log_line(&format!(
        "boss drive: your hit on Kaine deals hp {scaled} (was {}, {PVP_DAMAGE_PERCENT}% in the PvP fight), hit id {}",
        attack.hp_damage, attack.hit_id
    ));
}

pub(crate) fn apply_guest_boss_hits(build: &VerifiedBuild) {
    let Some(net) = net::handle() else {
        return;
    };
    let hits = net.take_relayed(|message| matches!(message, Message::BossHitOnNier(_)));
    if hits.is_empty() {
        return;
    }
    let game = Replicant::new(InProcessMemory::new(build));
    let memory = game.memory();
    let calls = EnemyCalls::new(build);
    let her = memory.module_base() + u64::from(PARTNER_KAINE_ACTOR_RVA);
    let phase = game
        .current_phase()
        .ok()
        .and_then(|name| PhaseEntry::index_of(&name))
        .and_then(|index| u16::try_from(index).ok());
    let motion = FOLLOW
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .her_motion
        .unwrap_or("unknown");
    for message in hits {
        let Message::BossHitOnNier(hit) = message else {
            continue;
        };
        let Ok(player) = game.playable_actor() else {
            continue;
        };
        if !crate::kaine::boss_drive_is_on() || phase != Some(hit.phase) {
            log_line(&format!(
                "boss drive: the guest's hit {} on your Nier is not applied (drive on {}, phase {phase:?} here, {} sent)",
                hit.hit_id,
                crate::kaine::boss_drive_is_on(),
                hit.phase
            ));
            continue;
        }
        let held_in_menu = player_in_menu(build) && !paused_players_can_be_hit();
        if held_in_menu {
            log_line(&format!(
                "boss drive: the guest's hit {} on your Nier is held, your Nier is in the menu and paused players can be hit is off, so the game ignores it (no hp change)",
                hit.hit_id
            ));
            continue;
        }
        let verdict = NIER_HITS
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .sent_hit(
                net_clock_ms(),
                hit.hit_id,
                motion,
                game.max_health().unwrap_or(0),
            );
        match verdict {
            SentHitVerdict::Repeat => {}
            SentHitVerdict::CountedNatively => log_line(&format!(
                "boss drive: the guest's hit {} on your Nier was already counted by her own attack, not applied again",
                hit.hit_id
            )),
            SentHitVerdict::Apply { damage, learned } => {
                let before = game.health().unwrap_or(-1);
                let reaction = HitReaction {
                    flags: hit.flags,
                    push: hit.push,
                    direction: hit.direction,
                };
                let sent = as_remote_with_reaction(reaction, || {
                    while_menu_free(|| {
                        calls.send_actor_message(
                            memory,
                            player,
                            PLAYER_MESSAGE_DIRECT_HIT,
                            her,
                            u64::try_from(damage).unwrap_or(0),
                        )
                    })
                });
                log_line(&format!(
                    "boss drive: the guest's hit {} lands on your Nier: hp {damage} ({} for her {motion}), flags {:#x}, push {:.2}, sent {sent}, your hp {before} -> {}",
                    hit.hit_id,
                    if learned {
                        "her native strength"
                    } else {
                        "table strength"
                    },
                    hit.flags,
                    hit.push,
                    game.health().unwrap_or(-1)
                ));
            }
        }
    }
}
