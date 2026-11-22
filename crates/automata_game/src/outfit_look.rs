mod apply_guard;
mod look_item;
mod look_kind;
mod look_names;
mod look_picker;
mod look_picks;
mod look_rule;
mod look_send;
mod look_slot;
mod look_steps;
mod partner_look;
mod player_look;

pub use apply_guard::{ApplyGuard, MOST_TRIES_PER_VALUE, REAPPLY_COOLDOWN_TICKS};
pub use look_item::{
    CLOTH_STATE_ITEM_ID, CLOTH_STATE_OFF, CLOTH_STATE_ON, LOOK_ITEM_COUNT, LOOK_ITEMS, LookItem,
    look_items,
};
pub use look_kind::{LOOK_KIND_COUNT, LookKind};
pub use look_names::{LookNames, NAME_READ_ATTEMPTS};
pub use look_picker::{
    CAMPAIGN_LOOK_ALLOWED_TEXT, CAMPAIGN_LOOK_NOT_ALLOWED_TEXT, CLOTH_STATE_OFF_LABEL,
    CLOTH_STATE_ON_LABEL, FREE_ROAM_LOOK_TEXT, GAME_DEFAULT_LABEL, GUN_SKIN_ONLY_YOU_TEXT,
    HAIR_NOT_OFFERED_TEXT, NO_CHARACTER_TEXT, PickerEntry, PickerRow, picker_is_usable,
    picker_note, picker_rows,
};
pub use look_picks::{LookPicks, SavedLook};
pub use look_rule::looks_allowed;
pub use look_send::{look_message_due, look_send_due};
pub use look_slot::{LOOK_SLOT_COUNT, look_slot_of};
pub use look_steps::{LookReading, LookStep, look_steps};
pub use partner_look::{
    HairSwap, PartnerLookApplied, PartnerLookReading, PartnerLookStep, partner_hair_swap,
    partner_look_steps,
};
pub use player_look::PlayerLook;

pub const LOOK_UNPICKED: u8 = u8::MAX;
pub const HIGHEST_OUTFIT_VARIANT: u8 = 3;
pub const HIGHEST_ACCESSORY_CODE: u8 = 19;
pub const HIGHEST_HAIR_COLOUR: u8 = 18;
pub const HIGHEST_POD_SKIN: u8 = 9;
pub const HIGHEST_GUN_SKIN: u8 = 2;
pub const HOST_SLOT: u8 = 0;
