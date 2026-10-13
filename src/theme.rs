use eframe::egui::{
    Color32, Context, FontData, FontDefinitions, FontFamily, FontId, Margin, Rounding, Shadow,
    Stroke, TextStyle, Vec2, Visuals, style::WidgetVisuals,
};

pub const CONTROL_RADIUS: f32 = 8.0;
pub const CARD_RADIUS: f32 = 12.0;
pub const SIDEBAR_WIDE: f32 = 248.0;
pub const SIDEBAR_COMPACT: f32 = 76.0;
pub const COMPACT_BREAKPOINT: f32 = 980.0;

#[derive(Clone, Copy)]
pub struct Palette {
    pub background: Color32,
    pub sidebar: Color32,
    pub topbar: Color32,
    pub surface: Color32,
    pub surface_muted: Color32,
    pub surface_hover: Color32,
    pub sidebar_hover: Color32,
    pub track: Color32,
    pub input: Color32,
    pub border: Color32,
    pub border_strong: Color32,
    pub text: Color32,
    pub muted: Color32,
    pub faint: Color32,
    pub accent: Color32,
    pub accent_hover: Color32,
    pub accent_text: Color32,
    pub accent_soft: Color32,
    pub on_accent: Color32,
    pub danger: Color32,
    pub danger_hover: Color32,
    pub danger_soft: Color32,
    pub success: Color32,
    pub success_soft: Color32,
    pub warning: Color32,
    pub overlay: Color32,
    pub hover_tint: Color32,
}

pub fn palette(dark: bool) -> Palette {
    if dark {
        Palette {
            background: Color32::from_rgb(11, 13, 18),
            sidebar: Color32::from_rgb(15, 17, 24),
            topbar: Color32::from_rgb(15, 17, 24),
            surface: Color32::from_rgb(22, 26, 36),
            surface_muted: Color32::from_rgb(30, 36, 50),
            surface_hover: Color32::from_rgb(36, 43, 59),
            sidebar_hover: Color32::from_rgb(30, 36, 50),
            track: Color32::from_rgb(22, 26, 36),
            input: Color32::from_rgb(14, 16, 23),
            border: Color32::from_rgb(46, 54, 72),
            border_strong: Color32::from_rgb(78, 90, 116),
            text: Color32::from_rgb(243, 244, 246),
            muted: Color32::from_rgb(164, 172, 188),
            faint: Color32::from_rgb(113, 121, 138),
            accent: Color32::from_rgb(79, 70, 229),
            accent_hover: Color32::from_rgb(99, 91, 237),
            accent_text: Color32::from_rgb(199, 210, 254),
            accent_soft: Color32::from_rgba_unmultiplied(129, 140, 248, 48),
            on_accent: Color32::WHITE,
            danger: Color32::from_rgb(248, 113, 113),
            danger_hover: Color32::from_rgb(252, 165, 165),
            danger_soft: Color32::from_rgba_unmultiplied(248, 113, 113, 36),
            success: Color32::from_rgb(52, 211, 153),
            success_soft: Color32::from_rgba_unmultiplied(52, 211, 153, 36),
            warning: Color32::from_rgb(251, 191, 36),
            overlay: Color32::from_black_alpha(168),
            hover_tint: Color32::from_white_alpha(14),
        }
    } else {
        Palette {
            background: Color32::from_rgb(243, 245, 249),
            sidebar: Color32::WHITE,
            topbar: Color32::WHITE,
            surface: Color32::WHITE,
            surface_muted: Color32::from_rgb(247, 248, 252),
            surface_hover: Color32::from_rgb(241, 243, 248),
            sidebar_hover: Color32::from_rgb(243, 245, 249),
            track: Color32::from_rgb(238, 240, 246),
            input: Color32::from_rgb(247, 248, 252),
            border: Color32::from_rgb(226, 229, 237),
            border_strong: Color32::from_rgb(196, 202, 216),
            text: Color32::from_rgb(17, 24, 39),
            muted: Color32::from_rgb(100, 108, 128),
            faint: Color32::from_rgb(148, 156, 175),
            accent: Color32::from_rgb(79, 70, 229),
            accent_hover: Color32::from_rgb(67, 56, 202),
            accent_text: Color32::from_rgb(67, 56, 202),
            accent_soft: Color32::from_rgb(238, 240, 255),
            on_accent: Color32::WHITE,
            danger: Color32::from_rgb(220, 38, 38),
            danger_hover: Color32::from_rgb(185, 28, 28),
            danger_soft: Color32::from_rgb(254, 242, 242),
            success: Color32::from_rgb(5, 150, 105),
            success_soft: Color32::from_rgb(236, 253, 245),
            warning: Color32::from_rgb(180, 83, 9),
            overlay: Color32::from_black_alpha(128),
            hover_tint: Color32::from_black_alpha(10),
        }
    }
}

pub fn font_semi(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("semibold".into()))
}

pub const fn font_body(size: f32) -> FontId {
    FontId::new(size, FontFamily::Proportional)
}

pub fn configure_fonts(ctx: &Context) {
    let mut fonts = FontDefinitions::default();

    fonts.font_data.insert(
        "Inter".to_owned(),
        FontData::from_static(include_bytes!("../assets/Inter-Regular.ttf")),
    );
    fonts.font_data.insert(
        "InterSemiBold".to_owned(),
        FontData::from_static(include_bytes!("../assets/Inter-SemiBold.ttf")),
    );

    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, "Inter".to_owned());

    fonts.families.insert(
        FontFamily::Name("semibold".into()),
        vec!["InterSemiBold".to_owned(), "Inter".to_owned()],
    );

    ctx.set_fonts(fonts);
}

pub fn apply(ctx: &Context, dark: bool) {
    let p = palette(dark);
    let mut visuals = if dark {
        Visuals::dark()
    } else {
        Visuals::light()
    };

    visuals.panel_fill = p.background;
    visuals.window_fill = p.surface;
    visuals.extreme_bg_color = p.input;
    visuals.faint_bg_color = p.surface_muted;
    visuals.code_bg_color = p.surface_muted;
    visuals.override_text_color = Some(p.text);
    visuals.window_stroke = Stroke::new(1.0_f32, p.border);
    visuals.window_rounding = Rounding::same(16.0);
    visuals.menu_rounding = Rounding::same(CONTROL_RADIUS);
    visuals.window_shadow = if dark {
        Shadow {
            offset: Vec2::new(0.0, 16.0),
            blur: 40.0,
            spread: 0.0,
            color: Color32::from_black_alpha(120),
        }
    } else {
        Shadow {
            offset: Vec2::new(0.0, 12.0),
            blur: 32.0,
            spread: 0.0,
            color: Color32::from_black_alpha(36),
        }
    };

    style_widget(
        &mut visuals.widgets.noninteractive,
        p.surface,
        p.surface,
        p.text,
        p.border,
    );
    style_widget(
        &mut visuals.widgets.inactive,
        p.input,
        p.surface,
        p.text,
        p.border,
    );
    style_widget(
        &mut visuals.widgets.hovered,
        p.input,
        p.surface_hover,
        p.text,
        p.border_strong,
    );
    style_widget(
        &mut visuals.widgets.active,
        p.input,
        p.surface_hover,
        p.text,
        p.accent,
    );
    style_widget(
        &mut visuals.widgets.open,
        p.input,
        p.surface_hover,
        p.text,
        p.accent,
    );

    visuals.selection.bg_fill =
        Color32::from_rgba_unmultiplied(p.accent.r(), p.accent.g(), p.accent.b(), 72);
    visuals.selection.stroke = Stroke::new(1.5_f32, p.accent);
    visuals.hyperlink_color = p.accent_text;

    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = Vec2::new(8.0, 8.0);
    style.spacing.button_padding = Vec2::new(14.0, 8.0);
    style.spacing.interact_size = Vec2::new(40.0, 40.0);
    style.spacing.window_margin = Margin::ZERO;
    style.spacing.menu_margin = Margin::same(8.0);
    style.spacing.text_edit_width = f32::INFINITY;
    style.spacing.scroll.floating = true;
    style.spacing.scroll.bar_width = 8.0;
    style.interaction.selectable_labels = false;
    style.interaction.multi_widget_text_select = false;
    style.text_styles = [
        (TextStyle::Small, font_body(12.0)),
        (TextStyle::Body, font_body(14.0)),
        (
            TextStyle::Monospace,
            FontId::new(13.0, FontFamily::Monospace),
        ),
        (TextStyle::Button, font_semi(13.5)),
        (TextStyle::Heading, font_semi(22.0)),
    ]
    .into();

    ctx.set_style(style);
}

fn style_widget(
    widget: &mut WidgetVisuals,
    fill: Color32,
    weak: Color32,
    fg: Color32,
    stroke: Color32,
) {
    widget.bg_fill = fill;
    widget.weak_bg_fill = weak;
    widget.fg_stroke = Stroke::new(1.0_f32, fg);
    widget.bg_stroke = Stroke::new(1.0_f32, stroke);
    widget.rounding = Rounding::same(CONTROL_RADIUS);
    widget.expansion = 0.0;
}

pub fn page_padding(screen_width: f32) -> f32 {
    if screen_width < COMPACT_BREAKPOINT {
        16.0
    } else {
        28.0
    }
}
