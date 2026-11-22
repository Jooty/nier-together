use crate::character::Character;

pub const LOOK_SLOT_COUNT: usize = 3;

pub const fn look_slot_of(character: Character) -> usize {
    match character {
        Character::TwoB => 0,
        Character::NineS => 1,
        Character::A2 => 2,
    }
}
