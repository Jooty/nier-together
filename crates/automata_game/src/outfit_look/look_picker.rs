use crate::mode::SessionMode;
use crate::outfit_look::{
    CLOTH_STATE_ITEM_ID, CLOTH_STATE_OFF, CLOTH_STATE_ON, LookKind, LookNames, LookPicks,
    look_items, looks_allowed,
};

pub const GAME_DEFAULT_LABEL: &str = "game default";
pub const CLOTH_STATE_OFF_LABEL: &str = "off";
pub const CLOTH_STATE_ON_LABEL: &str = "on";
pub const FREE_ROAM_LOOK_TEXT: &str = "Free roam: any outfit and accessory can be worn.";
pub const CAMPAIGN_LOOK_ALLOWED_TEXT: &str = "Outfits and accessories are meant to be unlocked by playing. The host's switch makes them all available here.";
pub const CAMPAIGN_LOOK_NOT_ALLOWED_TEXT: &str = "Outfits and accessories are meant to be unlocked by playing. The host has not turned on \"Any outfit and accessory\" for this campaign.";
pub const GUN_SKIN_ONLY_YOU_TEXT: &str = "Only you see your gun skin.";
pub const HAIR_NOT_OFFERED_TEXT: &str = "Hair colour is not offered for this character.";
pub const NO_CHARACTER_TEXT: &str = "Start playing to choose a look.";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PickerEntry {
    pub label: String,
    pub value: Option<i32>,
    pub chosen: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PickerRow {
    pub kind: LookKind,
    pub heading: String,
    pub note: Option<&'static str>,
    pub entries: Vec<PickerEntry>,
}

pub fn picker_note(mode: SessionMode, host_allows_any_look: bool) -> &'static str {
    match (mode.story_rules_apply(), host_allows_any_look) {
        (false, _) => FREE_ROAM_LOOK_TEXT,
        (true, true) => CAMPAIGN_LOOK_ALLOWED_TEXT,
        (true, false) => CAMPAIGN_LOOK_NOT_ALLOWED_TEXT,
    }
}

pub fn picker_is_usable(mode: SessionMode, host_allows_any_look: bool) -> bool {
    looks_allowed(mode, host_allows_any_look)
}

pub fn picker_rows(slot: usize, picks: &LookPicks, names: &LookNames) -> Vec<PickerRow> {
    LookKind::ALL
        .into_iter()
        .map(|kind| picker_row(slot, kind, picks, names))
        .collect()
}

fn picker_row(slot: usize, kind: LookKind, picks: &LookPicks, names: &LookNames) -> PickerRow {
    let picked = picks.picked(slot, kind);
    let mut entries = vec![PickerEntry {
        label: GAME_DEFAULT_LABEL.to_owned(),
        value: None,
        chosen: picked.is_none(),
    }];
    let offered: Vec<(String, i32)> = if kind == LookKind::ClothState {
        vec![
            (CLOTH_STATE_OFF_LABEL.to_owned(), CLOTH_STATE_OFF),
            (CLOTH_STATE_ON_LABEL.to_owned(), CLOTH_STATE_ON),
        ]
    } else {
        look_items(slot, kind)
            .into_iter()
            .map(|item| (names.label(item.id), item.value))
            .collect()
    };
    entries.extend(offered.into_iter().map(|(label, value)| PickerEntry {
        label,
        value: Some(value),
        chosen: picked == Some(value),
    }));
    PickerRow {
        kind,
        heading: heading_of(kind, names),
        note: note_of(kind, entries.len()),
        entries,
    }
}

fn heading_of(kind: LookKind, names: &LookNames) -> String {
    if kind == LookKind::ClothState
        && let Some(name) = names.name_of(CLOTH_STATE_ITEM_ID)
    {
        return name.to_owned();
    }
    kind.heading().to_owned()
}

fn note_of(kind: LookKind, entry_count: usize) -> Option<&'static str> {
    match kind {
        LookKind::GunSkin => Some(GUN_SKIN_ONLY_YOU_TEXT),
        LookKind::Hair if entry_count == 1 => Some(HAIR_NOT_OFFERED_TEXT),
        _ => None,
    }
}
