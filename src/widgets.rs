use std::hash::Hash;

use crate::theme::{CARD_RADIUS, CONTROL_RADIUS, Palette, font_body, font_semi};
use eframe::egui::text::{LayoutJob, TextWrapping};
use eframe::egui::{
    Align, Align2, Color32, Context, CursorIcon, FontId, Frame, Id, Label, Layout, Margin, Order,
    Pos2, Rect, Response, RichText, Rounding, Sense, Shadow, Stroke, TextEdit, Ui, UiBuilder, Vec2,
};

pub const FIELD_H: f32 = 40.0;
pub const NAME_MAX: usize = 100;
pub const DESCRIPTION_MAX: usize = 1000;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ButtonKind {
    Primary,
    Secondary,
    Danger,
    Quiet,
    QuietDanger,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ButtonSize {
    Medium,
    Small,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BannerKind {
    Success,
    Error,
}

#[derive(Clone, Copy)]
pub enum Icon {
    Overview,
    Records,
    Settings,
}

pub fn semi_text(text: impl AsRef<str>, size: f32, color: Color32) -> RichText {
    RichText::new(text.as_ref())
        .font(font_semi(size))
        .color(color)
}

pub fn body_text(text: impl AsRef<str>, size: f32, color: Color32) -> RichText {
    RichText::new(text.as_ref())
        .font(font_body(size))
        .color(color)
}

pub fn surface(p: &Palette) -> Frame {
    Frame::none()
        .fill(p.surface)
        .stroke(Stroke::new(1.0_f32, p.border))
        .rounding(Rounding::same(CARD_RADIUS))
        .inner_margin(Margin::same(20.0))
}

pub fn dialog_frame(p: &Palette, dark: bool) -> Frame {
    Frame::none()
        .fill(p.surface)
        .stroke(Stroke::new(1.0_f32, p.border))
        .rounding(Rounding::same(16.0))
        .inner_margin(Margin::same(24.0))
        .shadow(Shadow {
            offset: Vec2::new(0.0, 16.0),
            blur: 40.0,
            spread: 0.0,
            color: Color32::from_black_alpha(if dark { 140 } else { 48 }),
        })
}

const fn button_metrics(size: ButtonSize) -> (f32, f32, f32) {
    match size {
        ButtonSize::Medium => (FIELD_H, 14.0, 16.0),
        ButtonSize::Small => (32.0, 13.0, 12.0),
    }
}

pub fn button_width(ui: &Ui, label: &str, size: ButtonSize) -> f32 {
    let (_, font_size, pad_x) = button_metrics(size);
    let text_width = ui.fonts(|fonts| {
        fonts
            .layout_no_wrap(label.to_owned(), font_semi(font_size), Color32::WHITE)
            .size()
            .x
    });
    (text_width + pad_x * 2.0).max(32.0)
}

pub fn pill_width(ui: &Ui, text: &str) -> f32 {
    let text_width = ui.fonts(|fonts| {
        fonts
            .layout_no_wrap(text.to_owned(), font_body(12.0), Color32::WHITE)
            .size()
            .x
    });
    text_width + 16.0
}

pub fn button(
    ui: &mut Ui,
    p: &Palette,
    label: &str,
    kind: ButtonKind,
    size: ButtonSize,
    width: Option<f32>,
) -> Response {
    let (height, font_size, _) = button_metrics(size);
    let font = font_semi(font_size);
    let width = match width {
        Some(requested) => requested.max(32.0),
        None => button_width(ui, label, size),
    };
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::click());
    let hot = response.hovered() || response.is_pointer_button_down_on();

    let (fill, text_color, stroke) = match kind {
        ButtonKind::Primary => (
            if hot { p.accent_hover } else { p.accent },
            p.on_accent,
            Stroke::NONE,
        ),
        ButtonKind::Secondary => (
            if hot { p.surface_hover } else { p.surface },
            p.text,
            Stroke::new(1.0_f32, if hot { p.border_strong } else { p.border }),
        ),
        ButtonKind::Danger => (
            if hot { p.danger_hover } else { p.danger },
            Color32::WHITE,
            Stroke::NONE,
        ),
        ButtonKind::Quiet => (
            if hot {
                p.hover_tint
            } else {
                Color32::TRANSPARENT
            },
            p.muted,
            Stroke::NONE,
        ),
        ButtonKind::QuietDanger => (
            if hot {
                p.danger_soft
            } else {
                Color32::TRANSPARENT
            },
            p.danger,
            Stroke::NONE,
        ),
    };

    if fill != Color32::TRANSPARENT || stroke != Stroke::NONE {
        ui.painter()
            .rect(rect, Rounding::same(CONTROL_RADIUS), fill, stroke);
    }

    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        label,
        font,
        text_color,
    );

    response.on_hover_cursor(CursorIcon::PointingHand)
}

struct FieldEditor<'a> {
    text: &'a mut String,
    hint: &'a str,
    id: Option<Id>,
    limit: Option<usize>,
    multiline: bool,
    height: f32,
}

pub fn text_field(
    ui: &mut Ui,
    p: &Palette,
    text: &mut String,
    hint: &str,
    id: Option<Id>,
    limit: Option<usize>,
) -> Response {
    field_editor(
        ui,
        p,
        FieldEditor {
            text,
            hint,
            id,
            limit,
            multiline: false,
            height: FIELD_H,
        },
    )
}

pub fn text_area(
    ui: &mut Ui,
    p: &Palette,
    text: &mut String,
    hint: &str,
    limit: Option<usize>,
    height: f32,
) -> Response {
    field_editor(
        ui,
        p,
        FieldEditor {
            text,
            hint,
            id: None,
            limit,
            multiline: true,
            height,
        },
    )
}

fn field_editor(ui: &mut Ui, p: &Palette, field: FieldEditor<'_>) -> Response {
    let FieldEditor {
        text,
        hint,
        id,
        limit,
        multiline,
        height,
    } = field;
    let font = font_body(14.0);
    let mut editor = if multiline {
        TextEdit::multiline(text)
            .desired_rows(4)
            .vertical_align(Align::Min)
    } else {
        TextEdit::singleline(text).vertical_align(Align::Center)
    };
    editor = editor
        .hint_text(body_text(hint, 14.0, p.faint))
        .font(font)
        .margin(Margin::symmetric(12.0, if multiline { 10.0 } else { 0.0 }))
        .min_size(Vec2::new(0.0, height));
    if let Some(id) = id {
        editor = editor.id(id);
    }
    if let Some(limit) = limit {
        editor = editor.char_limit(limit);
    }
    ui.add(editor)
}

pub fn invalid_scope(ui: &mut Ui, invalid: bool, danger: Color32, body: impl FnOnce(&mut Ui)) {
    if !invalid {
        body(ui);
        return;
    }

    ui.scope(|ui| {
        let stroke = Stroke::new(1.5_f32, danger);
        ui.visuals_mut().selection.stroke = stroke;
        ui.visuals_mut().widgets.inactive.bg_stroke = Stroke::new(1.0_f32, danger);
        ui.visuals_mut().widgets.hovered.bg_stroke = Stroke::new(1.0_f32, danger);
        ui.visuals_mut().widgets.active.bg_stroke = stroke;
        body(ui);
    });
}

pub fn field_header(
    ui: &mut Ui,
    p: &Palette,
    label: &str,
    required: bool,
    count: usize,
    max: usize,
) {
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, 20.0), Sense::hover());
    let font = font_semi(13.0);
    ui.painter().text(
        Pos2::new(rect.left(), rect.center().y),
        Align2::LEFT_CENTER,
        label,
        font.clone(),
        p.text,
    );
    if required {
        let label_w = ui.fonts(|fonts| {
            fonts
                .layout_no_wrap(label.to_owned(), font, p.text)
                .size()
                .x
        });
        ui.painter().text(
            Pos2::new(rect.left() + label_w + 4.0, rect.center().y),
            Align2::LEFT_CENTER,
            "*",
            font_semi(13.0),
            p.danger,
        );
    }
    let count_color = if count > max { p.danger } else { p.muted };
    ui.painter().text(
        Pos2::new(rect.right(), rect.center().y),
        Align2::RIGHT_CENTER,
        format!("{count} / {max}"),
        font_body(12.0),
        count_color,
    );
}

pub fn error_text(ui: &mut Ui, p: &Palette, message: &str) {
    ui.label(body_text(message, 12.0, p.danger));
}

pub fn section_title(ui: &mut Ui, p: &Palette, title: &str) {
    ui.label(semi_text(title, 16.0, p.text));
}

pub fn muted(ui: &mut Ui, p: &Palette, text: &str) {
    ui.add(Label::new(body_text(text, 13.0, p.muted)).wrap());
}

pub fn pill(ui: &mut Ui, text: &str, fg: Color32, bg: Color32) -> Response {
    let font = font_body(12.0);
    let width = pill_width(ui, text);
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, 24.0), Sense::hover());
    ui.painter().rect_filled(rect, 12.0, bg);
    ui.painter()
        .text(rect.center(), Align2::CENTER_CENTER, text, font, fg);
    response
}

pub fn readout(ui: &mut Ui, p: &Palette, text: &str) {
    Frame::none()
        .fill(p.input)
        .stroke(Stroke::new(1.0_f32, p.border))
        .rounding(Rounding::same(CONTROL_RADIUS))
        .inner_margin(Margin::symmetric(12.0, 10.0))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.add(Label::new(body_text(text, 13.0, p.text)).wrap());
        });
}

pub fn progress_bar(ui: &mut Ui, p: &Palette, value: f32) {
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, 8.0), Sense::hover());
    ui.painter().rect_filled(rect, 4.0, p.track);
    let clamped = value.clamp(0.0, 1.0);
    if clamped > 0.0 {
        let mut fill = rect;
        fill.max.x = rect.min.x + rect.width() * clamped;
        ui.painter().rect_filled(fill, 4.0, p.accent);
    }
}

pub fn notice_banner(ui: &mut Ui, p: &Palette, kind: BannerKind, text: &str) -> bool {
    let (fill, fg, border) = match kind {
        BannerKind::Success => (p.success_soft, p.success, p.success),
        BannerKind::Error => (p.danger_soft, p.danger, p.danger),
    };
    let mut dismiss = false;
    Frame::none()
        .fill(fill)
        .stroke(Stroke::new(
            1.0_f32,
            Color32::from_rgba_unmultiplied(border.r(), border.g(), border.b(), 70),
        ))
        .rounding(Rounding::same(CONTROL_RADIUS))
        .inner_margin(Margin::symmetric(14.0, 10.0))
        .show(ui, |ui| {
            let width = ui.available_width();
            ui.set_width(width);
            let button_w = button_width(ui, "Dismiss", ButtonSize::Small);
            let text_w = (width - button_w - 12.0).max(40.0);
            let galley = ui.fonts(|fonts| {
                fonts.layout_job(LayoutJob::simple(
                    text.to_owned(),
                    font_body(13.0),
                    fg,
                    text_w,
                ))
            });
            let height = galley.size().y.max(32.0);
            let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
            ui.painter().galley(
                Pos2::new(rect.left(), rect.center().y - galley.size().y * 0.5),
                galley,
                fg,
            );
            let button_rect = Rect::from_min_size(
                Pos2::new(rect.right() - button_w, rect.center().y - 16.0),
                Vec2::new(button_w, 32.0),
            );
            place(
                ui,
                button_rect,
                "notice-dismiss",
                Layout::left_to_right(Align::Center),
                |ui| {
                    if button(
                        ui,
                        p,
                        "Dismiss",
                        ButtonKind::Quiet,
                        ButtonSize::Small,
                        Some(button_w),
                    )
                    .clicked()
                    {
                        dismiss = true;
                    }
                },
            );
        });
    dismiss
}

pub fn nav_button(
    ui: &mut Ui,
    p: &Palette,
    label: &str,
    icon: Icon,
    selected: bool,
    show_label: bool,
) -> Response {
    let width = ui.available_width();
    let (rect, mut response) = ui.allocate_exact_size(Vec2::new(width, FIELD_H), Sense::click());
    let fill = if selected {
        p.accent_soft
    } else if response.hovered() {
        p.sidebar_hover
    } else {
        Color32::TRANSPARENT
    };
    if fill != Color32::TRANSPARENT {
        ui.painter()
            .rect_filled(rect, Rounding::same(CONTROL_RADIUS), fill);
    }
    if selected && show_label {
        let bar = Rect::from_center_size(
            Pos2::new(rect.left() + 6.0, rect.center().y),
            Vec2::new(3.0, 16.0),
        );
        ui.painter().rect_filled(bar, 2.0, p.accent);
    }

    let icon_color = if selected { p.accent_text } else { p.muted };
    let text_color = if selected { p.text } else { p.muted };
    let icon_center = if show_label {
        Pos2::new(rect.left() + 28.0, rect.center().y)
    } else {
        rect.center()
    };
    paint_icon(ui.painter(), icon, icon_center, icon_color);
    if show_label {
        ui.painter().text(
            Pos2::new(rect.left() + 46.0, rect.center().y),
            Align2::LEFT_CENTER,
            label,
            font_semi(14.0),
            text_color,
        );
    } else {
        response = response.on_hover_text(label);
    }
    response.on_hover_cursor(CursorIcon::PointingHand)
}

pub fn theme_toggle(ui: &mut Ui, p: &Palette, dark: bool, wide: bool) -> Option<bool> {
    if !wide {
        let label = if dark { "Light" } else { "Dark" };
        let response = button(
            ui,
            p,
            label,
            ButtonKind::Secondary,
            ButtonSize::Medium,
            Some(ui.available_width()),
        );
        return if response.clicked() {
            Some(!dark)
        } else {
            None
        };
    }

    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, FIELD_H), Sense::hover());
    ui.painter()
        .rect_filled(rect, Rounding::same(CONTROL_RADIUS), p.track);

    let inner = rect.shrink(3.0);
    let half = inner.width() / 2.0;
    let left = Rect::from_min_size(inner.min, Vec2::new(half, inner.height()));
    let right = Rect::from_min_size(
        Pos2::new(inner.left() + half, inner.top()),
        Vec2::new(half, inner.height()),
    );
    let left_response = ui.interact(left, ui.id().with("theme-light"), Sense::click());
    let right_response = ui.interact(right, ui.id().with("theme-dark"), Sense::click());

    paint_segment(ui, p, left, "Light", !dark, left_response.hovered());
    paint_segment(ui, p, right, "Dark", dark, right_response.hovered());

    if left_response
        .on_hover_cursor(CursorIcon::PointingHand)
        .clicked()
    {
        Some(false)
    } else if right_response
        .on_hover_cursor(CursorIcon::PointingHand)
        .clicked()
    {
        Some(true)
    } else {
        None
    }
}

fn paint_segment(ui: &Ui, p: &Palette, rect: Rect, label: &str, selected: bool, hovered: bool) {
    if selected {
        ui.painter().rect_filled(rect, 6.0, p.accent);
    } else if hovered {
        ui.painter().rect_filled(rect, 6.0, p.hover_tint);
    }
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        label,
        font_semi(13.0),
        if selected { p.on_accent } else { p.muted },
    );
}

pub fn grid_columns(width: f32, min_item: f32, gap: f32, count: usize) -> usize {
    if count == 0 || width <= 0.0 {
        return 1;
    }
    let cols = ((width + gap) / (min_item + gap)).floor().max(1.0) as usize;
    cols.min(count).max(1)
}

pub fn grid_item_width(width: f32, cols: usize, gap: f32) -> f32 {
    if cols <= 1 {
        width.max(0.0)
    } else {
        (width - gap * (cols - 1) as f32) / cols as f32
    }
}

/// Columns with explicit widths. Each column is top-down and justified to its width.
/// The parent cursor advances by the full available width and the tallest column.
pub fn columns_exact<R>(
    ui: &mut Ui,
    widths: &[f32],
    gap: f32,
    add_contents: impl FnOnce(&mut [Ui]) -> R,
) -> R {
    let count = widths.len();
    assert!(count > 0, "columns_exact needs at least one column");

    let row_width = ui.available_width().max(0.0);
    let top_left = ui.cursor().min;
    let bottom = top_left.y + ui.available_height().max(0.0);
    let batch = ui.next_auto_id();
    ui.skip_ahead_auto_ids(1);

    let mut columns = Vec::with_capacity(count);
    let mut x = top_left.x;
    for (index, width) in widths.iter().copied().enumerate() {
        let width = width.max(0.0);
        let rect = Rect::from_min_max(Pos2::new(x, top_left.y), Pos2::new(x + width, bottom));
        let mut column = ui.new_child(
            UiBuilder::new()
                .id_salt((batch, index))
                .max_rect(rect)
                .layout(Layout::top_down_justified(Align::Min)),
        );
        column.set_width(width);
        columns.push(column);
        x += width + gap;
    }

    let result = add_contents(&mut columns);
    let height = columns
        .iter()
        .map(|column| column.min_size().y)
        .fold(0.0_f32, f32::max);
    ui.advance_cursor_after_rect(Rect::from_min_size(top_left, Vec2::new(row_width, height)));
    result
}

pub fn place<R>(
    ui: &mut Ui,
    rect: Rect,
    salt: impl Hash,
    layout: Layout,
    add: impl FnOnce(&mut Ui) -> R,
) -> R {
    let mut child = ui.new_child(UiBuilder::new().id_salt(salt).max_rect(rect).layout(layout));
    child.set_width(rect.width().max(0.0));
    add(&mut child)
}

pub fn right_row<R>(
    ui: &mut Ui,
    height: f32,
    gap: f32,
    salt: impl Hash,
    add: impl FnOnce(&mut Ui) -> R,
) -> R {
    let width = ui.available_width().max(0.0);
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
    let mut child = ui.new_child(
        UiBuilder::new()
            .id_salt(salt)
            .max_rect(rect)
            .layout(Layout::right_to_left(Align::Center)),
    );
    child.spacing_mut().item_spacing.x = gap;
    child.set_width(width);
    add(&mut child)
}

pub fn paint_ellipsis(ui: &Ui, rect: Rect, text: &str, font: FontId, color: Color32) {
    let width = rect.width().max(0.0);
    let mut job = LayoutJob::simple(text.to_owned(), font, color, width);
    job.break_on_newline = false;
    job.wrap = TextWrapping::truncate_at_width(width);
    let galley = ui.fonts(|fonts| fonts.layout_job(job));
    ui.painter().galley(
        Pos2::new(rect.left(), rect.center().y - galley.size().y * 0.5),
        galley,
        color,
    );
}

pub fn responsive_grid(
    ui: &mut Ui,
    min_item_width: f32,
    gap: f32,
    count: usize,
    mut item: impl FnMut(&mut Ui, usize),
) {
    if count == 0 {
        return;
    }

    let width = ui.available_width().max(0.0);
    let cols = grid_columns(width, min_item_width, gap, count);
    let card_w = grid_item_width(width, cols, gap);
    let previous_spacing = ui.spacing().item_spacing;
    ui.spacing_mut().item_spacing = Vec2::ZERO;

    let mut index = 0;
    while index < count {
        let n = (count - index).min(cols);
        let widths = vec![card_w; n];
        let row_start = index;
        columns_exact(ui, &widths, gap, |columns| {
            for (offset, column) in columns.iter_mut().enumerate() {
                item(column, row_start + offset);
            }
        });
        index += n;
        if index < count {
            ui.add_space(gap);
        }
    }

    ui.spacing_mut().item_spacing = previous_spacing;
}

pub fn backdrop(ctx: &Context, overlay: Color32) -> bool {
    let screen = ctx.screen_rect();
    let mut clicked = false;
    eframe::egui::Area::new(Id::new("modal-backdrop"))
        .order(Order::Foreground)
        .movable(false)
        .interactable(true)
        .fixed_pos(screen.min)
        .show(ctx, |ui| {
            ui.set_min_size(screen.size());
            ui.painter().rect_filled(screen, 0.0, overlay);
            clicked = ui
                .interact(screen, Id::new("modal-backdrop-hit"), Sense::click())
                .clicked();
        });
    clicked
}

pub fn choice_card(ui: &mut Ui, p: &Palette, title: &str, body: &str, selected: bool) -> Response {
    let width = ui.available_width();
    let height = 112.0;
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::click());
    let fill = if selected {
        p.accent_soft
    } else if response.hovered() {
        p.surface_hover
    } else {
        p.surface
    };
    let stroke = if selected {
        Stroke::new(1.5_f32, p.accent)
    } else {
        Stroke::new(1.0_f32, p.border)
    };
    ui.painter()
        .rect(rect, Rounding::same(CARD_RADIUS), fill, stroke);
    ui.painter().text(
        Pos2::new(rect.left() + 16.0, rect.top() + 16.0),
        Align2::LEFT_TOP,
        title,
        font_semi(16.0),
        p.text,
    );

    let body_width = (rect.width() - 32.0).max(40.0);
    let mut job = LayoutJob::simple(body.to_owned(), font_body(13.0), p.muted, body_width);
    job.wrap = TextWrapping {
        max_width: body_width,
        max_rows: 3,
        break_anywhere: false,
        overflow_character: Some('…'),
    };
    let galley = ui.fonts(|fonts| fonts.layout_job(job));
    ui.painter().galley(
        Pos2::new(rect.left() + 16.0, rect.top() + 44.0),
        galley,
        p.muted,
    );

    response.on_hover_cursor(CursorIcon::PointingHand)
}

pub fn paint_icon(painter: &eframe::egui::Painter, icon: Icon, center: Pos2, color: Color32) {
    match icon {
        Icon::Overview => {
            let size = 5.5;
            let gap = 2.5;
            let origin = center - Vec2::splat((size * 2.0 + gap) / 2.0);
            for index in 0..4 {
                let column = index % 2;
                let row = index / 2;
                let rect = Rect::from_min_size(
                    origin + Vec2::new(column as f32 * (size + gap), row as f32 * (size + gap)),
                    Vec2::splat(size),
                );
                painter.rect_filled(rect, 1.5, color);
            }
        }
        Icon::Records => {
            let width = 14.0;
            let height = 2.0;
            let gap = 3.0;
            let top = center.y - (height * 3.0 + gap * 2.0) / 2.0;
            let left = center.x - width / 2.0;
            for index in 0..3 {
                let rect = Rect::from_min_size(
                    Pos2::new(left, top + index as f32 * (height + gap)),
                    Vec2::new(width, height),
                );
                painter.rect_filled(rect, 1.0, color);
            }
        }
        Icon::Settings => {
            let width = 14.0;
            let left = center.x - width / 2.0;
            let knobs = [0.28, 0.72, 0.46];
            for (index, knob) in knobs.iter().enumerate() {
                let y = center.y - 5.0 + index as f32 * 5.0;
                painter.hline(left..=(left + width), y, Stroke::new(1.5_f32, color));
                painter.circle_filled(Pos2::new(left + width * knob, y), 2.1, color);
            }
        }
    }
}

pub fn empty_mark(ui: &mut Ui, p: &Palette) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(48.0), Sense::hover());
    ui.painter()
        .circle_filled(rect.center(), 24.0, p.accent_soft);
    paint_icon(ui.painter(), Icon::Records, rect.center(), p.accent_text);
}

pub fn dot(ui: &mut Ui, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(8.0), Sense::hover());
    ui.painter().circle_filled(rect.center(), 4.0, color);
}
