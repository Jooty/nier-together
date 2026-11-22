mod bullet_shot;

pub use bullet_shot::BulletShot;

pub const BULLET_SHOT_SETUP_FUNCTION_START: [u8; 24] = [
    0x40, 0x55, 0x53, 0x56, 0x57, 0x41, 0x56, 0x41, 0x57, 0x48, 0x8D, 0xAC, 0x24, 0xF8, 0xFE, 0xFF,
    0xFF, 0x48, 0x81, 0xEC, 0x08, 0x02, 0x00, 0x00,
];

pub const PLAYER_BODY_OBJECT_ID: u32 = 0x1_0000;
pub const BULLET_SHOT_LINES_LOGGED_PER_MINUTE: u32 = 90;

pub const OWNER_OBJECT_ID_OFFSET: u32 = 0x28;
pub const SHOT_INFO_KIND_OFFSET: u64 = 0;
pub const SHOT_INFO_ATTACK_GROUP_OFFSET: u64 = 8;
pub const SHOT_INFO_ATTACK_INDEX_OFFSET: u64 = 12;
pub const BULLET_BEHAVIOR_KIND_OFFSET: u64 = 0xC84;
pub const BULLET_POSITION_OFFSET: u64 = 0x50;
