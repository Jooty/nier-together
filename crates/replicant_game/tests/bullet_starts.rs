use replicant_game::bullet_flights::read_bullet;
use replicant_game::enemy_bullets::{
    BulletStartMade, bullet_start_cue_kind, read_not_drawn_normal_bullet,
};
use replicant_game::offsets::{
    ACTOR_TRANSFORM_BLOCK_BYTES, ACTOR_TRANSFORM_BLOCK_OFFSET, EFFECT_INVALID_HANDLE,
    NORMAL_BULLET_ATTACK_SETTINGS_POINTER_OFFSET, NORMAL_BULLET_EFFECT_HANDLES_OFFSET,
    NORMAL_BULLET_HIT_ID_OFFSET, NORMAL_BULLET_RECORD_EFFECT_IDS_OFFSET,
    NORMAL_BULLET_RECORD_POINTER_OFFSET, NORMAL_BULLET_RECORD_START_EFFECT_ID_OFFSET,
    NORMAL_BULLET_SETTINGS_GROUND_EFFECT_ID_OFFSET, NORMAL_BULLET_SETTINGS_LASTING_SOUND_ID_OFFSET,
    NORMAL_BULLET_SETTINGS_START_SOUND_ID_OFFSET,
};
use replicant_proto::{EnemyId, ShotCueKind};

use crate::fake_memory::FakeMemory;

const BULLET: u64 = 0x1000_0000;
const RECORD: u64 = 0x2000_0000;
const SETTINGS: u64 = 0x3000_0000;
const FIGHT_9_SHOCKWAVE_START_EFFECT_ID: i32 = 3501;
const FIGHT_6_SHOCKWAVE_SLOT_EFFECT_ID: i32 = 1300;
const FIGHT_6_SHOCKWAVE_START_EFFECT_ID: i32 = 1301;
const SHOCKWAVE_START_SOUND_ID: i32 = 77006;
const START_EFFECT_HANDLE: u32 = 0x0002_A123;
const SLOT_EFFECT_HANDLE: u32 = 0x0002_A124;

struct BulletData {
    slot_effect_ids: [i32; 4],
    start_effect_id: i32,
    start_sound_id: i32,
    lasting_sound_id: i32,
}

fn fight_9_shockwave() -> BulletData {
    // Settings 271 of BulletAbilityOrigin.binl (struct_2_bin) and its record in
    // BulletAbilityEffectOrigin.binl: radius 2 to 17 over 0.7 s, 21.43 a second.
    BulletData {
        slot_effect_ids: [0; 4],
        start_effect_id: FIGHT_9_SHOCKWAVE_START_EFFECT_ID,
        start_sound_id: SHOCKWAVE_START_SOUND_ID,
        lasting_sound_id: 0,
    }
}

fn fight_6_shockwave() -> BulletData {
    // Settings 261: radius 1 to 25 over 1 s, ring 1300 in slot 0.
    BulletData {
        slot_effect_ids: [FIGHT_6_SHOCKWAVE_SLOT_EFFECT_ID, 0, 0, 0],
        start_effect_id: FIGHT_6_SHOCKWAVE_START_EFFECT_ID,
        start_sound_id: SHOCKWAVE_START_SOUND_ID,
        lasting_sound_id: 0,
    }
}

fn bullet_after_its_start(data: &BulletData, slot_handles: [u32; 4]) -> FakeMemory {
    let mut memory = FakeMemory::new();
    memory.put(
        BULLET + NORMAL_BULLET_RECORD_POINTER_OFFSET,
        &RECORD.to_le_bytes(),
    );
    memory.put(
        BULLET + NORMAL_BULLET_ATTACK_SETTINGS_POINTER_OFFSET,
        &SETTINGS.to_le_bytes(),
    );
    memory.put(BULLET + NORMAL_BULLET_HIT_ID_OFFSET, &7_i32.to_le_bytes());
    memory.put(
        BULLET + ACTOR_TRANSFORM_BLOCK_OFFSET,
        &[0; ACTOR_TRANSFORM_BLOCK_BYTES],
    );
    for (slot, (id, handle)) in (0_u64..).zip(data.slot_effect_ids.iter().zip(slot_handles)) {
        memory.put(
            RECORD + NORMAL_BULLET_RECORD_EFFECT_IDS_OFFSET + slot * 4,
            &id.to_le_bytes(),
        );
        memory.put(
            BULLET + NORMAL_BULLET_EFFECT_HANDLES_OFFSET + slot * 4,
            &handle.to_le_bytes(),
        );
    }
    memory.put(
        RECORD + NORMAL_BULLET_RECORD_START_EFFECT_ID_OFFSET,
        &data.start_effect_id.to_le_bytes(),
    );
    memory.put(
        SETTINGS + NORMAL_BULLET_SETTINGS_GROUND_EFFECT_ID_OFFSET,
        &0_i32.to_le_bytes(),
    );
    memory.put(
        SETTINGS + NORMAL_BULLET_SETTINGS_START_SOUND_ID_OFFSET,
        &data.start_sound_id.to_le_bytes(),
    );
    memory.put(
        SETTINGS + NORMAL_BULLET_SETTINGS_LASTING_SOUND_ID_OFFSET,
        &data.lasting_sound_id.to_le_bytes(),
    );
    memory
}

#[test]
fn fight_9s_shockwave_has_no_slot_effect_so_the_flights_never_drew_it() {
    let memory = bullet_after_its_start(&fight_9_shockwave(), [EFFECT_INVALID_HANDLE; 4]);
    assert_eq!(read_bullet(&memory, BULLET, 3), None);
}

#[test]
fn fight_9s_shockwave_start_effect_and_sound_become_one_shot_cues() {
    let memory = bullet_after_its_start(&fight_9_shockwave(), [EFFECT_INVALID_HANDLE; 4]);
    let effect = BulletStartMade::Effect {
        handle: START_EFFECT_HANDLE,
    };
    let sound = BulletStartMade::Sound {
        id: SHOCKWAVE_START_SOUND_ID.cast_unsigned(),
    };
    assert_eq!(
        bullet_start_cue_kind(&memory, BULLET, effect),
        Some(ShotCueKind::OneShotEffect)
    );
    assert_eq!(
        bullet_start_cue_kind(&memory, BULLET, sound),
        Some(ShotCueKind::OneShotSound)
    );
}

#[test]
fn fight_6s_shockwave_ring_stays_with_the_flights_and_its_start_effect_is_a_cue() {
    let mut handles = [EFFECT_INVALID_HANDLE; 4];
    handles[0] = SLOT_EFFECT_HANDLE;
    let memory = bullet_after_its_start(&fight_6_shockwave(), handles);
    let ring = BulletStartMade::Effect {
        handle: SLOT_EFFECT_HANDLE,
    };
    let start = BulletStartMade::Effect {
        handle: START_EFFECT_HANDLE,
    };
    assert_eq!(bullet_start_cue_kind(&memory, BULLET, ring), None);
    assert_eq!(
        bullet_start_cue_kind(&memory, BULLET, start),
        Some(ShotCueKind::OneShotEffect)
    );
}

#[test]
fn a_bullets_lasting_sound_follows_its_copy_and_a_failed_effect_is_no_cue() {
    let data = BulletData {
        lasting_sound_id: SHOCKWAVE_START_SOUND_ID,
        ..fight_9_shockwave()
    };
    let memory = bullet_after_its_start(&data, [EFFECT_INVALID_HANDLE; 4]);
    let lasting = BulletStartMade::Sound {
        id: SHOCKWAVE_START_SOUND_ID.cast_unsigned(),
    };
    let failed = BulletStartMade::Effect {
        handle: EFFECT_INVALID_HANDLE,
    };
    assert_eq!(
        bullet_start_cue_kind(&memory, BULLET, lasting),
        Some(ShotCueKind::LastingSound)
    );
    assert_eq!(bullet_start_cue_kind(&memory, BULLET, failed), None);
}

#[test]
fn a_widening_ordinary_bullet_with_no_live_effect_is_read_for_the_log() {
    // L34 run0-newer 08:37:36.351: "99:0's ordinary bullet ... width grows 21.43, ... ground effect 0".
    let memory = bullet_after_its_start(&fight_9_shockwave(), [EFFECT_INVALID_HANDLE; 4]);
    let boss = EnemyId {
        group: 99,
        ordinal: 0,
    };
    let read = read_not_drawn_normal_bullet(&memory, BULLET, boss).expect("readable");
    assert_eq!(read.enemy, boss);
    assert_eq!(read.record_effect_ids, [0; 4]);
    assert_eq!(read.effect_handles, [EFFECT_INVALID_HANDLE; 4]);
    assert_eq!(read.ground_effect_id, 0);
    assert_eq!(read.start_effect_id, FIGHT_9_SHOCKWAVE_START_EFFECT_ID);
    assert_eq!(read.start_sound_id, SHOCKWAVE_START_SOUND_ID);
    assert_eq!(read.lasting_sound_id, 0);
}
