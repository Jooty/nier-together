use eframe::egui::{Align, FontId, Layout, Rect, Ui, UiBuilder, pos2, vec2};

use crate::app::pages::dialog_frame::show_dialog;
use crate::app::palette::{INK, INK_MUTED};
use crate::app::parts::{LINE_BUTTON_HEIGHT_PX, LineTone, command_button, line_button};
use crate::app::style::{body_font, garamond_family};
use crate::app::{Launcher, NetForms};

const DIALOG_SIZE_PX: (f32, f32) = (560.0, 248.0);
const DIALOG_PADDING_PX: f32 = 28.0;
const TITLE_PX: f32 = 30.0;
const TITLE_TO_BODY_PX: f32 = 18.0;
const COMMAND_SIZE_PX: (f32, f32) = (176.0, 48.0);
const BUTTON_GAP_PX: f32 = 22.0;
const DIALOG_LAYER: &str = "scale-up-dialog";
const SCALE_UP_LABEL: &str = "Scale up";
const NOT_NOW_LABEL: &str = "Not now";

fn scale_up_title(guest_name: &str) -> String {
    format!("No save for {guest_name}")
}

fn scale_up_body(guest_name: &str) -> String {
    format!(
        "{guest_name} has no save in this campaign. Give them a copy of your save under their name?"
    )
}

pub(super) fn draw_scale_up_dialog(ui: &mut Ui, launcher: &mut Launcher, forms: &mut NetForms) {
    let Some(guest_name) = launcher.scale_up_guest_name() else {
        return;
    };
    let mut answer = None;
    let close = show_dialog(ui, DIALOG_SIZE_PX, DIALOG_LAYER, |ui, dialog| {
        answer = draw_dialog(ui, dialog, &guest_name);
    });
    if answer == Some(true) {
        launcher.scale_up_partner(&*forms.net);
    } else if close || answer == Some(false) {
        launcher.decline_scale_up();
    }
}

fn draw_dialog(ui: &mut Ui, dialog: Rect, guest_name: &str) -> Option<bool> {
    let painter = ui.painter().clone();
    let inner = dialog.shrink(DIALOG_PADDING_PX);
    let title = painter.layout_no_wrap(
        scale_up_title(guest_name),
        FontId::new(TITLE_PX, garamond_family()),
        INK,
    );
    let title_height = title.size().y;
    painter.galley(inner.min, title, INK);
    let body_top = inner.top() + title_height + TITLE_TO_BODY_PX;
    let body = painter.layout(
        scale_up_body(guest_name),
        body_font(),
        INK_MUTED,
        inner.width(),
    );
    painter.galley(pos2(inner.left(), body_top), body, INK_MUTED);
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
            if command_button(ui, SCALE_UP_LABEL, COMMAND_SIZE_PX, true, true).clicked() {
                return Some(true);
            }
            let not_now = line_button(
                ui,
                NOT_NOW_LABEL,
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
