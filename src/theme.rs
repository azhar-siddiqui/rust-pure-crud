use eframe::egui;

pub const BG: egui::Color32 = egui::Color32::from_rgb(15, 17, 23);
pub const PANEL: egui::Color32 = egui::Color32::from_rgb(22, 25, 33);
pub const PANEL_ALT: egui::Color32 = egui::Color32::from_rgb(29, 33, 43);
pub const BORDER: egui::Color32 = egui::Color32::from_rgb(48, 54, 68);
pub const TEXT: egui::Color32 = egui::Color32::from_rgb(238, 241, 248);
pub const MUTED: egui::Color32 = egui::Color32::from_rgb(166, 175, 192);
pub const BLUE: egui::Color32 = egui::Color32::from_rgb(76, 132, 255);
pub const GREEN: egui::Color32 = egui::Color32::from_rgb(65, 205, 151);
pub const RED: egui::Color32 = egui::Color32::from_rgb(242, 100, 116);

pub fn configure_fonts(ctx: &egui::Context) {
    let fonts = egui::FontDefinitions::default();
    ctx.set_fonts(fonts);
}

pub fn apply_production_styling(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();

    style.text_styles = [
        (egui::TextStyle::Small, egui::FontId::proportional(12.0)),
        (egui::TextStyle::Body, egui::FontId::proportional(14.0)),
        (egui::TextStyle::Button, egui::FontId::proportional(14.0)),
        (egui::TextStyle::Heading, egui::FontId::proportional(23.0)),
        (egui::TextStyle::Monospace, egui::FontId::monospace(13.0)),
    ]
    .into();

    style.spacing.item_spacing = egui::vec2(12.0, 14.0);
    style.spacing.button_padding = egui::vec2(16.0, 10.0);
    style.spacing.interact_size = egui::vec2(40.0, 38.0);
    style.spacing.window_margin = egui::Margin::same(20.0);

    style.visuals = egui::Visuals::dark();
    style.visuals.override_text_color = Some(TEXT);
    style.visuals.panel_fill = BG;
    style.visuals.window_fill = PANEL;
    style.visuals.extreme_bg_color = egui::Color32::from_rgb(12, 14, 19);
    style.visuals.faint_bg_color = PANEL_ALT;
    style.visuals.window_rounding = egui::Rounding::same(12.0);

    style.visuals.widgets.inactive.bg_fill = PANEL_ALT;
    style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(42, 48, 62);
    style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(35, 67, 125);

    style.visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0_f32, BORDER);
    style.visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0_f32, MUTED);
    style.visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0_f32, BLUE);

    style.visuals.widgets.inactive.rounding = egui::Rounding::same(7.0);
    style.visuals.widgets.hovered.rounding = egui::Rounding::same(7.0);
    style.visuals.widgets.active.rounding = egui::Rounding::same(7.0);

    style.visuals.selection.bg_fill = BLUE;
    style.visuals.selection.stroke = egui::Stroke::new(1.0_f32, TEXT);

    ctx.set_style(style);
}
