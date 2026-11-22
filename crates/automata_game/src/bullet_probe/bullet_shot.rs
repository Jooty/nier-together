use crate::bullet_probe::PLAYER_BODY_OBJECT_ID;
use crate::enemy_lead::BOSS_CLASS_HEALTH_FROM;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BulletShot {
    pub owner_object: u32,
    pub owner_handle: u32,
    pub owner_health: i32,
    pub owner_max_health: i32,
    pub shot_kind: i32,
    pub behavior_kind: i32,
    pub attack_group: i32,
    pub attack_index: i32,
    pub position: [f32; 3],
}

impl BulletShot {
    pub fn is_by_a_boss_class_owner(&self) -> bool {
        self.owner_object != PLAYER_BODY_OBJECT_ID
            && self.owner_max_health >= BOSS_CLASS_HEALTH_FROM
    }

    pub fn line(&self) -> String {
        format!(
            "bullets: spawn by object {:#X} (handle {:#X}, health {}/{}): shot kind {}, behavior kind {}, attack {:#X}/{:#X}, at {:.2} {:.2} {:.2}",
            self.owner_object,
            self.owner_handle,
            self.owner_health,
            self.owner_max_health,
            self.shot_kind,
            self.behavior_kind,
            self.attack_group,
            self.attack_index,
            self.position[0],
            self.position[1],
            self.position[2],
        )
    }
}
