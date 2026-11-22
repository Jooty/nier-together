use eframe::egui::{Align, FontId, Layout, Rect, Ui, UiBuilder, pos2, vec2};
use nt_launcher_core::{
    KEYBOARD_MOUSE_FIXES_EXPLAINER_LINES, KEYBOARD_MOUSE_OFFER_CLOSING_LINE,
    KEYBOARD_MOUSE_OFFER_INTRO, KEYBOARD_MOUSE_OFFER_NOT_NOW_LABEL, KEYBOARD_MOUSE_OFFER_TITLE,
    KEYBOARD_MOUSE_OFFER_TURN_ON_LABEL,
};

use crate::app::Launcher;
use crate::app::pages::dialog_frame::show_dialog;
use crate::app::palette::{INK, INK_MUTED};
use crate::app::parts::{LINE_BUTTON_HEIGHT_PX, LineTone, command_button, line_button};
use crate::app::style::{body_font, garamond_family};

const DIALOG_SIZE_PX: (f32, f32) = (600.0, 372.0);
const DIALOG_PADDING_PX: f32 = 28.0;
const TITLE_PX: f32 = 30.0;
const TITLE_TO_BODY_PX: f32 = 18.0;
const PARAGRAPH_GAP_PX: f32 = 12.0;
const COMMAND_SIZE_PX: (f32, f32) = (176.0, 48.0);
const BUTTON_GAP_PX: f32 = 22.0;
const DIALOG_LAYER: &str = "keyboard-mouse-dialog";

pub(super) fn draw_keyboard_mouse_dialog(ui: &mut Ui, launcher: &mut Launcher) {
    let mut answer = None;
    show_dialog(ui, DIALOG_SIZE_PX, DIALOG_LAYER, |ui, dialog| {
        answer = draw_dialog(ui, dialog);
    });
    if let Some(turn_on) = answer {
        launcher.answer_keyboard_mouse_offer(turn_on);
    }
}

fn draw_dialog(ui: &mut Ui, dialog: Rect) -> Option<bool> {
    let painter = ui.painter().clone();
    let inner = dialog.shrink(DIALOG_PADDING_PX);
    let title = painter.layout_no_wrap(
        KEYBOARD_MOUSE_OFFER_TITLE.to_owned(),
        FontId::new(TITLE_PX, garamond_family()),
        INK,
    );
    let mut top = inner.top() + title.size().y + TITLE_TO_BODY_PX;
    painter.galley(inner.min, title, INK);
    let paragraphs = std::iter::once(KEYBOARD_MOUSE_OFFER_INTRO)
        .chain(KEYBOARD_MOUSE_FIXES_EXPLAINER_LINES)
        .chain(std::iter::once(KEYBOARD_MOUSE_OFFER_CLOSING_LINE));
    for paragraph in paragraphs {
        let galley = painter.layout(paragraph.to_owned(), body_font(), INK_MUTED, inner.width());
        let height = galley.size().y;
        painter.galley(pos2(inner.left(), top), galley, INK_MUTED);
        top += height + PARAGRAPH_GAP_PX;
    }
    let command_row = Rect::from_min_max(
        pos2(inner.left(), inner.bottom() - COMMAND_SIZE_PX.1),
        inner.right_bottom(),
    );
    ui.scope_builder(
        UiBuilder::new()
            .max_rect(command_row)
            .layout(Layout::right_to_left(Align::Center)),
        |ui| {
            ui.spacing_mut().item_spacing = vec2(BUTTON_GAP_PX, 0.0);
            if command_button(
                ui,
                KEYBOARD_MOUSE_OFFER_TURN_ON_LABEL,
                COMMAND_SIZE_PX,
                true,
                true,
            )
            .clicked()
            {
                return Some(true);
            }
            let not_now = line_button(
                ui,
                KEYBOARD_MOUSE_OFFER_NOT_NOW_LABEL,
                LineTone::Plain,
                LINE_BUTTON_HEIGHT_PX,
                None,
                true,
            );
            not_now.clicked().then_some(false)
        },
    )
    .inner
}
