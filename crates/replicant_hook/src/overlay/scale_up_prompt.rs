use hudhook::imgui::{MouseButton, Ui};
use replicant_game::scale_up_prompt::{
    KEYBOARD_NOT_NOW_HINT, KEYBOARD_SCALE_UP_HINT, MOUSE_HINT, NOT_NOW_ANSWERED_LINE_SUFFIX,
    PAD_NOT_NOW_HINT, PAD_SCALE_UP_HINT, PressSample, SCALE_UP_ANSWERED_LINE_SUFFIX,
};
use replicant_proto::{SCALE_UP_NO_LABEL, SCALE_UP_YES_LABEL, scale_up_body, scale_up_title};

use crate::diagnostics::log_line;
use crate::event_log::post_event;
use crate::game_hooks;
use crate::net::{self, net_clock_ms};
use crate::overlay::hud_assets;
use crate::proxy::pad_press;

const PANEL_COLOR: [f32; 4] = [0.0, 0.0, 0.0, 0.72];
const BUTTON_COLOR: [f32; 4] = [0.18, 0.16, 0.12, 0.9];
const BUTTON_HOVER_COLOR: [f32; 4] = [0.34, 0.3, 0.2, 0.95];
const HOLD_FILL_COLOR: [f32; 4] = [0.95, 0.85, 0.5, 0.55];
const TITLE_COLOR: [f32; 4] = [1.0, 0.92, 0.55, 1.0];
const TEXT_COLOR: [f32; 4] = [0.9, 0.87, 0.78, 1.0];
const HINT_COLOR: [f32; 4] = [0.7, 0.68, 0.6, 1.0];
const SHADOW_COLOR: [f32; 4] = [0.0, 0.0, 0.0, 0.85];
const SHADOW_OFFSET_PIXELS: f32 = 1.0;
const TITLE_TEXT_SIZE_PIXELS: f32 = 24.0;
const BODY_TEXT_SIZE_PIXELS: f32 = 18.0;
const LABEL_TEXT_SIZE_PIXELS: f32 = 22.0;
const HINT_TEXT_SIZE_PIXELS: f32 = 14.0;
const PANEL_WIDTH_FRACTION_OF_SCREEN: f32 = 0.42;
const PANEL_TOP_FRACTION_OF_SCREEN_HEIGHT: f32 = 0.12;
const PANEL_PADDING_PIXELS: f32 = 16.0;
const PANEL_ROUNDING_PIXELS: f32 = 6.0;
const LINE_GAP_PIXELS: f32 = 6.0;
const BUTTON_HEIGHT_PIXELS: f32 = 74.0;
const BUTTON_GAP_PIXELS: f32 = 12.0;

struct ButtonRects {
    scale_up: ([f32; 2], [f32; 2]),
    not_now: ([f32; 2], [f32; 2]),
}

struct PanelLayout {
    min: [f32; 2],
    max: [f32; 2],
    body_lines: Vec<String>,
    buttons: ButtonRects,
}

pub(super) fn pointer_wanted() -> bool {
    net::handle().is_some_and(|net| net.shown_scale_up_ask().is_some())
}

pub(super) fn draw_scale_up_prompt(ui: &Ui) {
    let Some(net) = net::handle() else {
        return;
    };
    let shown = net.shown_scale_up_ask();
    let layout = shown
        .as_ref()
        .map(|ask| panel_layout(ui, &scale_up_body(&ask.name)));
    let click = layout
        .as_ref()
        .and_then(|layout| clicked_button(ui, &layout.buttons));
    let step = net.step_scale_up_prompt(
        net_clock_ms(),
        PressSample {
            keyboard: game_hooks::read_answer_gesture(),
            pad: pad_press::current_gesture(),
            click,
        },
    );
    let hiding = net.scale_up_input_masked();
    pad_press::set_prompt_up(hiding);
    if let (Some(answer), Some(ask)) = (step.answer, shown.as_ref()) {
        announce_answer(&ask.name, answer.yes);
    }
    let (Some(ask), Some(layout)) = (shown, layout) else {
        return;
    };
    if step.answer.is_none() {
        draw_panel(ui, &ask.name, &layout, step.hold);
    }
}

fn panel_layout(ui: &Ui, body: &str) -> PanelLayout {
    let [screen_width, screen_height] = ui.io().display_size;
    let width = screen_width * PANEL_WIDTH_FRACTION_OF_SCREEN;
    let left = (screen_width - width) * 0.5;
    let top = screen_height * PANEL_TOP_FRACTION_OF_SCREEN_HEIGHT;
    let inner_width = width - 2.0 * PANEL_PADDING_PIXELS;
    let body_lines = wrap_words(ui, body, inner_width);
    let body_height = body_lines.len() as f32 * (BODY_TEXT_SIZE_PIXELS + LINE_GAP_PIXELS);
    let buttons_top = top
        + PANEL_PADDING_PIXELS
        + TITLE_TEXT_SIZE_PIXELS
        + LINE_GAP_PIXELS * 2.0
        + body_height
        + LINE_GAP_PIXELS;
    let button_width = (inner_width - BUTTON_GAP_PIXELS) * 0.5;
    let first_left = left + PANEL_PADDING_PIXELS;
    let second_left = first_left + button_width + BUTTON_GAP_PIXELS;
    let buttons = ButtonRects {
        scale_up: (
            [first_left, buttons_top],
            [
                first_left + button_width,
                buttons_top + BUTTON_HEIGHT_PIXELS,
            ],
        ),
        not_now: (
            [second_left, buttons_top],
            [
                second_left + button_width,
                buttons_top + BUTTON_HEIGHT_PIXELS,
            ],
        ),
    };
    PanelLayout {
        min: [left, top],
        max: [
            left + width,
            buttons_top + BUTTON_HEIGHT_PIXELS + PANEL_PADDING_PIXELS,
        ],
        body_lines,
        buttons,
    }
}

fn wrap_words(ui: &Ui, text: &str, max_width: f32) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for word in text.split_whitespace() {
        let candidate = match lines.last() {
            Some(line) => format!("{line} {word}"),
            None => word.to_owned(),
        };
        let fits = hud_assets::text_size(ui, &candidate, BODY_TEXT_SIZE_PIXELS)[0] <= max_width;
        match lines.last_mut() {
            Some(line) if fits => *line = candidate,
            _ => lines.push(word.to_owned()),
        }
    }
    lines
}

fn clicked_button(ui: &Ui, buttons: &ButtonRects) -> Option<bool> {
    if !game_hooks::cursor_mode_active() || !ui.is_mouse_clicked(MouseButton::Left) {
        return None;
    }
    let at = ui.io().mouse_pos;
    if inside(at, buttons.scale_up) {
        Some(true)
    } else if inside(at, buttons.not_now) {
        Some(false)
    } else {
        None
    }
}

fn inside(point: [f32; 2], (min, max): ([f32; 2], [f32; 2])) -> bool {
    point[0] >= min[0] && point[0] <= max[0] && point[1] >= min[1] && point[1] <= max[1]
}

fn draw_panel(ui: &Ui, name: &str, layout: &PanelLayout, hold: Option<(bool, f32)>) {
    let draw_list = ui.get_background_draw_list();
    draw_list
        .add_rect(layout.min, layout.max, PANEL_COLOR)
        .filled(true)
        .rounding(PANEL_ROUNDING_PIXELS)
        .build();
    let text_left = layout.min[0] + PANEL_PADDING_PIXELS;
    let mut y = layout.min[1] + PANEL_PADDING_PIXELS;
    let mut line = |text: &str, size: f32, color: [f32; 4], gap: f32| {
        hud_assets::draw_text(
            &draw_list,
            text,
            [text_left, y],
            size,
            color,
            (SHADOW_COLOR, SHADOW_OFFSET_PIXELS),
        );
        y += size + gap;
    };
    line(
        &scale_up_title(name),
        TITLE_TEXT_SIZE_PIXELS,
        TITLE_COLOR,
        LINE_GAP_PIXELS * 2.0,
    );
    for body_line in &layout.body_lines {
        line(
            body_line,
            BODY_TEXT_SIZE_PIXELS,
            TEXT_COLOR,
            LINE_GAP_PIXELS,
        );
    }
    let pointer_in_use = game_hooks::cursor_mode_active();
    let mouse = ui.io().mouse_pos;
    let buttons = [
        (
            layout.buttons.scale_up,
            SCALE_UP_YES_LABEL,
            format!("{KEYBOARD_SCALE_UP_HINT}, {PAD_SCALE_UP_HINT}"),
            true,
        ),
        (
            layout.buttons.not_now,
            SCALE_UP_NO_LABEL,
            format!("{KEYBOARD_NOT_NOW_HINT}, {PAD_NOT_NOW_HINT}"),
            false,
        ),
    ];
    for (rect, label, hint, is_yes) in buttons {
        let hovered = pointer_in_use && inside(mouse, rect);
        let color = if hovered {
            BUTTON_HOVER_COLOR
        } else {
            BUTTON_COLOR
        };
        draw_list
            .add_rect(rect.0, rect.1, color)
            .filled(true)
            .rounding(PANEL_ROUNDING_PIXELS)
            .build();
        if let Some((yes, fraction)) = hold
            && yes == is_yes
        {
            let filled_right = rect.0[0] + (rect.1[0] - rect.0[0]) * fraction.clamp(0.0, 1.0);
            draw_list
                .add_rect(rect.0, [filled_right, rect.1[1]], HOLD_FILL_COLOR)
                .filled(true)
                .rounding(PANEL_ROUNDING_PIXELS)
                .build();
        }
        let inner_left = rect.0[0] + PANEL_PADDING_PIXELS * 0.5;
        hud_assets::draw_text(
            &draw_list,
            label,
            [inner_left, rect.0[1] + 8.0],
            LABEL_TEXT_SIZE_PIXELS,
            TITLE_COLOR,
            (SHADOW_COLOR, SHADOW_OFFSET_PIXELS),
        );
        hud_assets::draw_text(
            &draw_list,
            &hint,
            [
                inner_left,
                rect.0[1] + 8.0 + LABEL_TEXT_SIZE_PIXELS + LINE_GAP_PIXELS,
            ],
            HINT_TEXT_SIZE_PIXELS,
            HINT_COLOR,
            (SHADOW_COLOR, SHADOW_OFFSET_PIXELS),
        );
        hud_assets::draw_text(
            &draw_list,
            MOUSE_HINT,
            [
                inner_left,
                rect.0[1]
                    + 8.0
                    + LABEL_TEXT_SIZE_PIXELS
                    + HINT_TEXT_SIZE_PIXELS
                    + LINE_GAP_PIXELS * 2.0,
            ],
            HINT_TEXT_SIZE_PIXELS,
            HINT_COLOR,
            (SHADOW_COLOR, SHADOW_OFFSET_PIXELS),
        );
    }
}

fn announce_answer(name: &str, yes: bool) {
    let suffix = if yes {
        SCALE_UP_ANSWERED_LINE_SUFFIX
    } else {
        NOT_NOW_ANSWERED_LINE_SUFFIX
    };
    let line = format!("{name} {suffix}");
    log_line(&format!(
        "scale up: answered {}, {line}",
        if yes { "yes" } else { "no" }
    ));
    post_event(&line);
}
