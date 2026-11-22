use automata_game::character::Character;
use automata_game::outfit_look::{
    LookKind, NO_CHARACTER_TEXT, PickerRow, picker_is_usable, picker_note, picker_rows,
};
use hudhook::imgui::Ui;
use nt_hook::menu_view::PANEL_COLUMN_LEFT_PX;
use nt_hook::menu_view::parts::{
    fact_line, line_buttons_when, note_line, panel_window, sub_heading,
};
use nt_ui_style::{INK, INK_MUTED};

use crate::look_driver::shared::{PickRequest, state};
use crate::menu::LOOK_PANEL_LABEL;

const PANEL_TOP_PX: f32 = 120.0;
const PANEL_SIZE_PX: [f32; 2] = [470.0, 560.0];
const PLAYING_KEY: &str = "playing";

struct PanelSnapshot {
    slot: Option<usize>,
    character: Option<Character>,
    usable: bool,
    note: &'static str,
    rows: Vec<PickerRow>,
}

pub fn draw_look_panel(ui: &Ui, open: &mut bool) {
    let shown = snapshot();
    let mut clicked = None;
    panel_window(
        ui,
        LOOK_PANEL_LABEL,
        open,
        [PANEL_COLUMN_LEFT_PX, PANEL_TOP_PX],
        PANEL_SIZE_PX,
        || clicked = draw_body(ui, &shown),
    );
    if let (Some(slot), Some((kind, value))) = (shown.slot, clicked) {
        state()
            .requests
            .push_back(PickRequest { slot, kind, value });
    }
}

fn snapshot() -> PanelSnapshot {
    let shared = state();
    let view = shared.view;
    let note = picker_note(view.mode, view.host_allows);
    let rows = view
        .slot
        .map(|slot| picker_rows(slot, &shared.picks, &shared.names))
        .unwrap_or_default();
    PanelSnapshot {
        slot: view.slot,
        character: view.character,
        usable: picker_is_usable(view.mode, view.host_allows),
        note,
        rows,
    }
}

fn draw_body(ui: &Ui, shown: &PanelSnapshot) -> Option<(LookKind, Option<i32>)> {
    note_line(ui, shown.note, INK_MUTED);
    let Some(character) = shown.character.filter(|_| shown.slot.is_some()) else {
        note_line(ui, NO_CHARACTER_TEXT, INK_MUTED);
        return None;
    };
    fact_line(
        ui,
        PLAYING_KEY,
        &character.launcher_id().to_ascii_uppercase(),
        INK,
    );
    let mut clicked = None;
    for (index, row) in shown.rows.iter().enumerate() {
        let _id = ui.push_id_usize(index);
        sub_heading(ui, &row.heading);
        let labels: Vec<&str> = row
            .entries
            .iter()
            .map(|entry| entry.label.as_str())
            .collect();
        let held: Vec<bool> = row.entries.iter().map(|entry| entry.chosen).collect();
        if let Some(entry) = line_buttons_when(ui, &labels, &held, shown.usable)
            .and_then(|entry_index| row.entries.get(entry_index))
        {
            clicked = Some((row.kind, entry.value));
        }
        if let Some(note) = row.note {
            note_line(ui, note, INK_MUTED);
        }
    }
    clicked
}
