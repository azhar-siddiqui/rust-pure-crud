use std::time::{Duration, Instant};

use crate::{
    models::Item,
    storage, theme,
    widgets::{
        self, BannerKind, ButtonKind, ButtonSize, DESCRIPTION_MAX, FIELD_H, Icon, NAME_MAX,
        body_text, semi_text,
    },
};
use eframe::egui::{
    self, Align, Align2, Frame, Id, Label, Layout, Margin, Pos2, Rect, Rounding, ScrollArea, Sense,
    Stroke, TextWrapMode, Vec2,
};

const SUCCESS_TOAST: Duration = Duration::from_secs(4);
const ERROR_TOAST: Duration = Duration::from_secs(5);

#[derive(Clone, Copy, PartialEq, Eq)]
enum Page {
    Dashboard,
    Records,
    Settings,
}

#[derive(Clone)]
struct Notice {
    kind: BannerKind,
    text: String,
    born: Instant,
    /// `None` stays on the page until dismissed. Action feedback uses a few seconds.
    duration: Option<Duration>,
    /// Elapsed time captured while the pointer rests on the toast, so the timer pauses.
    frozen: Option<Duration>,
}

struct Columns {
    id: f32,
    name: f32,
    description: f32,
    actions: f32,
}

pub struct CrudApp {
    items: Vec<Item>,
    next_id: u32,
    page: Page,
    dark: bool,
    search: String,
    name_input: String,
    description_input: String,
    editing_id: Option<u32>,
    show_form: bool,
    focus_name: bool,
    pending_delete: Option<u32>,
    name_error: Option<String>,
    description_error: Option<String>,
    notice: Option<Notice>,
}

impl CrudApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::configure_fonts(&cc.egui_ctx);

        let (items, notice) = match storage::load_items() {
            Ok(items) => (items, None),
            Err(error) => (
                Vec::new(),
                Some(Notice::persistent(
                    BannerKind::Error,
                    format!("Could not load saved records: {error}"),
                )),
            ),
        };

        let next_id = items
            .iter()
            .map(|item| item.id)
            .max()
            .unwrap_or(0)
            .saturating_add(1)
            .max(1);

        Self {
            items,
            next_id,
            page: Page::Dashboard,
            dark: false,
            search: String::new(),
            name_input: String::new(),
            description_input: String::new(),
            editing_id: None,
            show_form: false,
            focus_name: false,
            pending_delete: None,
            name_error: None,
            description_error: None,
            notice,
        }
    }

    fn notify(&mut self, kind: BannerKind, text: impl Into<String>) {
        let duration = match kind {
            BannerKind::Success => SUCCESS_TOAST,
            BannerKind::Error => ERROR_TOAST,
        };
        self.notice = Some(Notice::toast(kind, text, duration));
    }

    fn persist(&mut self, success_message: &str) {
        match storage::save_items(&self.items) {
            Ok(()) => self.notify(BannerKind::Success, success_message),
            Err(error) => self.notify(BannerKind::Error, format!("Save failed: {error}")),
        }
    }

    fn open_create_form(&mut self) {
        self.name_input.clear();
        self.description_input.clear();
        self.editing_id = None;
        self.name_error = None;
        self.description_error = None;
        self.pending_delete = None;
        self.show_form = true;
        self.focus_name = true;
    }

    fn open_edit_form(&mut self, item: &Item) {
        self.name_input = item.name.clone();
        self.description_input = item.description.clone();
        self.editing_id = Some(item.id);
        self.name_error = None;
        self.description_error = None;
        self.pending_delete = None;
        self.show_form = true;
        self.focus_name = true;
    }

    fn close_overlays(&mut self) {
        self.show_form = false;
        self.pending_delete = None;
        self.name_error = None;
        self.description_error = None;
        self.focus_name = false;
    }

    fn save_form(&mut self) {
        let name = self.name_input.trim().to_string();
        let description = self.description_input.trim().to_string();
        self.name_error = None;
        self.description_error = None;

        if name.is_empty() {
            self.name_error = Some("Enter a name.".to_string());
        } else if name.chars().count() > NAME_MAX {
            self.name_error = Some(format!("Use {NAME_MAX} characters or fewer."));
        }

        if description.chars().count() > DESCRIPTION_MAX {
            self.description_error = Some(format!("Use {DESCRIPTION_MAX} characters or fewer."));
        }

        if self.name_error.is_some() || self.description_error.is_some() {
            return;
        }

        if let Some(id) = self.editing_id {
            let Some(item) = self.items.iter_mut().find(|item| item.id == id) else {
                self.show_form = false;
                self.notify(BannerKind::Error, "That record no longer exists.");
                return;
            };
            item.name = name;
            item.description = description;
            self.show_form = false;
            self.persist("Record updated.");
        } else {
            let id = self.next_id;
            self.items.push(Item {
                id,
                name,
                description,
            });
            self.next_id = self.next_id.saturating_add(1);
            self.show_form = false;
            self.persist("Record created.");
        }
    }

    fn delete_item(&mut self, id: u32) {
        let before = self.items.len();
        self.items.retain(|item| item.id != id);
        self.pending_delete = None;

        if self.items.len() != before {
            self.persist("Record deleted.");
        } else {
            self.notify(BannerKind::Error, "That record no longer exists.");
        }
    }

    fn filtered_items(&self) -> Vec<Item> {
        let query = self.search.trim().to_lowercase();
        self.items
            .iter()
            .filter(|item| {
                query.is_empty()
                    || item.name.to_lowercase().contains(&query)
                    || item.description.to_lowercase().contains(&query)
                    || item.id.to_string().contains(&query)
            })
            .cloned()
            .collect()
    }

    fn described_count(&self) -> usize {
        self.items
            .iter()
            .filter(|item| !item.description.trim().is_empty())
            .count()
    }

    fn page_copy(&self) -> (&'static str, &'static str) {
        match self.page {
            Page::Dashboard => (
                "Overview",
                "A snapshot of the records stored on this device.",
            ),
            Page::Records => ("Records", "Create, search, edit, and remove records."),
            Page::Settings => (
                "Settings",
                "Appearance and local storage for this workspace.",
            ),
        }
    }

    fn sidebar(&mut self, ctx: &egui::Context) {
        let p = theme::palette(self.dark);
        let screen_w = ctx.screen_rect().width();
        let target = if screen_w < theme::COMPACT_BREAKPOINT {
            theme::SIDEBAR_COMPACT
        } else {
            theme::SIDEBAR_WIDE
        };
        let sidebar_w = ctx.animate_value_with_time(Id::new("sidebar-width"), target, 0.16);
        let show_labels = sidebar_w >= 190.0;
        let margin = if show_labels { 16.0 } else { 10.0 };

        egui::SidePanel::left("app_sidebar")
            .exact_width(sidebar_w)
            .resizable(false)
            .frame(
                Frame::none()
                    .fill(p.sidebar)
                    .inner_margin(Margin::same(margin)),
            )
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing = Vec2::new(0.0, 4.0);

                if show_labels {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 12.0;
                        brand_mark(ui, &p);
                        ui.vertical(|ui| {
                            ui.label(semi_text("Rust Control", 15.0, p.text));
                            ui.label(body_text("Local workspace", 12.0, p.muted));
                        });
                    });
                } else {
                    ui.vertical_centered(|ui| brand_mark(ui, &p));
                }

                ui.add_space(18.0);
                let cursor_y = ui.cursor().top();
                ui.painter().hline(
                    ui.available_rect_before_wrap().x_range(),
                    cursor_y,
                    Stroke::new(1.0_f32, p.border),
                );
                ui.add_space(16.0);

                if show_labels {
                    ui.label(semi_text("MENU", 11.0, p.faint));
                    ui.add_space(8.0);
                }

                let nav = [
                    (Page::Dashboard, "Overview", Icon::Overview),
                    (Page::Records, "Records", Icon::Records),
                    (Page::Settings, "Settings", Icon::Settings),
                ];

                for (page, label, icon) in nav {
                    if widgets::nav_button(ui, &p, label, icon, self.page == page, show_labels)
                        .clicked()
                    {
                        self.page = page;
                    }
                }

                ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
                    ui.spacing_mut().item_spacing.y = 8.0;
                    if show_labels {
                        ui.label(body_text("Stored on this device", 11.0, p.faint));
                    }
                    if let Some(dark) = widgets::theme_toggle(ui, &p, self.dark, show_labels) {
                        self.dark = dark;
                    }
                });

                let rect = ui.clip_rect();
                ui.painter().vline(
                    rect.right() - 0.5,
                    rect.y_range(),
                    Stroke::new(1.0_f32, p.border),
                );
            });
    }

    fn topbar(&mut self, ctx: &egui::Context) {
        let p = theme::palette(self.dark);
        let pad = theme::page_padding(ctx.screen_rect().width());
        let compact = ctx.screen_rect().width() < theme::COMPACT_BREAKPOINT;
        let (title, subtitle) = self.page_copy();

        egui::TopBottomPanel::top("app_topbar")
            .frame(Frame::none().fill(p.topbar).inner_margin(Margin {
                left: pad,
                right: pad,
                top: 14.0,
                bottom: 0.0,
            }))
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing = Vec2::ZERO;
                let width = ui.available_width();
                let height = if compact { 40.0 } else { 52.0 };
                let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());

                let show_create = !compact && self.page == Page::Records;
                let mut trailing = widgets::pill_width(ui, "On device");
                if show_create {
                    trailing += 8.0 + widgets::button_width(ui, "New record", ButtonSize::Medium);
                }
                let title_w = (width - trailing - 16.0).max(48.0);
                let title_rect = Rect::from_min_size(rect.min, Vec2::new(title_w, height));
                let trail_rect = Rect::from_min_size(
                    Pos2::new(rect.right() - trailing, rect.top()),
                    Vec2::new(trailing, height),
                );

                widgets::place(
                    ui,
                    title_rect,
                    "top-title",
                    Layout::top_down(Align::Min),
                    |ui| {
                        ui.set_width(title_w);
                        ui.spacing_mut().item_spacing.y = 0.0;
                        ui.add_space(if compact { 8.0 } else { 4.0 });
                        ui.add(
                            Label::new(semi_text(title, 20.0, p.text))
                                .wrap_mode(TextWrapMode::Truncate),
                        );
                        if !compact {
                            ui.add(
                                Label::new(body_text(subtitle, 13.0, p.muted))
                                    .wrap_mode(TextWrapMode::Truncate),
                            );
                        }
                    },
                );

                widgets::place(
                    ui,
                    trail_rect,
                    "top-actions",
                    Layout::right_to_left(Align::Center),
                    |ui| {
                        ui.spacing_mut().item_spacing.x = 8.0;
                        if show_create
                            && widgets::button(
                                ui,
                                &p,
                                "New record",
                                ButtonKind::Primary,
                                ButtonSize::Medium,
                                None,
                            )
                            .clicked()
                        {
                            self.open_create_form();
                        }
                        widgets::pill(ui, "On device", p.success, p.success_soft);
                    },
                );

                ui.add_space(14.0);
                let y = ui.min_rect().bottom();
                ui.painter()
                    .hline(ui.clip_rect().x_range(), y, Stroke::new(1.0_f32, p.border));
            });
    }

    fn dashboard(&mut self, ui: &mut egui::Ui) {
        let p = theme::palette(self.dark);
        let total = self.items.len();
        let described = self.described_count();
        let missing = total.saturating_sub(described);
        let cards = [
            (
                "Total records",
                total.to_string(),
                "On this device",
                p.accent,
            ),
            (
                "With descriptions",
                described.to_string(),
                "Ready to search",
                p.success,
            ),
            (
                "Needs detail",
                missing.to_string(),
                "Missing a description",
                if missing == 0 { p.muted } else { p.warning },
            ),
        ];

        let width = ui.available_width();
        let gap = 16.0;
        widgets::responsive_grid(ui, 220.0, gap, cards.len(), |ui, index| {
            let (label, value, hint, tone) = &cards[index];
            stat_card(ui, &p, label, value, hint, *tone);
        });

        ui.add_space(20.0);

        let cols = widgets::grid_columns(width, 220.0, gap, cards.len());
        let card_w = widgets::grid_item_width(width, cols, gap);
        let main = width - card_w - gap;
        if cols >= 3 && main >= 240.0 {
            widgets::columns_exact(ui, &[main, card_w], gap, |columns| {
                self.recent_column(&mut columns[0]);
                self.quality_column(&mut columns[1]);
            });
        } else {
            self.recent_column(ui);
            ui.add_space(gap);
            self.quality_column(ui);
        }
    }

    fn recent_column(&mut self, ui: &mut egui::Ui) {
        self.section_bar(ui, "Recent records", Some("View all"));
        ui.add_space(10.0);
        let recent: Vec<Item> = self.items.iter().rev().take(5).cloned().collect();
        self.show_records(
            ui,
            &recent,
            false,
            "No records yet",
            "Records you create will show up here.",
            true,
        );
    }

    fn quality_column(&mut self, ui: &mut egui::Ui) {
        self.section_bar(ui, "Record quality", None);
        ui.add_space(10.0);
        self.quality_card(ui);
    }

    fn section_bar(&mut self, ui: &mut egui::Ui, title: &str, action: Option<&str>) {
        let p = theme::palette(self.dark);
        let width = ui.available_width();
        let (rect, _) = ui.allocate_exact_size(Vec2::new(width, 32.0), Sense::hover());
        ui.painter().text(
            Pos2::new(rect.left(), rect.center().y),
            Align2::LEFT_CENTER,
            title,
            theme::font_semi(16.0),
            p.text,
        );
        if let Some(label) = action {
            let button_w = widgets::button_width(ui, label, ButtonSize::Small);
            let button_rect = Rect::from_min_size(
                Pos2::new(rect.right() - button_w, rect.top()),
                Vec2::new(button_w, 32.0),
            );
            let mut clicked = false;
            widgets::place(
                ui,
                button_rect,
                ("section-action", title),
                Layout::left_to_right(Align::Center),
                |ui| {
                    clicked = widgets::button(
                        ui,
                        &p,
                        label,
                        ButtonKind::Quiet,
                        ButtonSize::Small,
                        Some(button_w),
                    )
                    .clicked();
                },
            );
            if clicked {
                self.page = Page::Records;
            }
        }
    }

    fn quality_card(&mut self, ui: &mut egui::Ui) {
        let p = theme::palette(self.dark);
        let total = self.items.len();
        let described = self.described_count();
        let ratio = if total == 0 {
            0.0
        } else {
            described as f32 / total as f32
        };
        let percent = (ratio * 100.0).round() as usize;

        widgets::surface(&p).show(ui, |ui| {
            ui.spacing_mut().item_spacing = Vec2::ZERO;
            let width = ui.available_width();
            ui.set_width(width);

            widgets::muted(
                ui,
                &p,
                &if total == 0 {
                    "Create your first record to start the workspace.".to_string()
                } else {
                    format!("{described} of {total} records include a description.")
                },
            );
            ui.add_space(14.0);

            let (row, _) = ui.allocate_exact_size(Vec2::new(width, 18.0), Sense::hover());
            ui.painter().text(
                Pos2::new(row.left(), row.center().y),
                Align2::LEFT_CENTER,
                "Descriptions",
                theme::font_body(12.0),
                p.muted,
            );
            ui.painter().text(
                Pos2::new(row.right(), row.center().y),
                Align2::RIGHT_CENTER,
                format!("{percent}%"),
                theme::font_semi(12.0),
                p.text,
            );
            ui.add_space(8.0);
            widgets::progress_bar(ui, &p, ratio);
            ui.add_space(16.0);
            if widgets::button(
                ui,
                &p,
                "New record",
                ButtonKind::Primary,
                ButtonSize::Medium,
                Some(width),
            )
            .clicked()
            {
                self.open_create_form();
            }
            ui.add_space(8.0);
            if widgets::button(
                ui,
                &p,
                "Browse records",
                ButtonKind::Secondary,
                ButtonSize::Medium,
                Some(width),
            )
            .clicked()
            {
                self.page = Page::Records;
            }
        });
    }

    fn records_page(&mut self, ui: &mut egui::Ui) {
        let p = theme::palette(self.dark);
        let compact = ui.available_width() < 720.0;

        if compact {
            if widgets::button(
                ui,
                &p,
                "New record",
                ButtonKind::Primary,
                ButtonSize::Medium,
                Some(ui.available_width()),
            )
            .clicked()
            {
                self.open_create_form();
            }
            ui.add_space(12.0);
        }

        let shown = self.filtered_items();
        let total = self.items.len();
        let filtering = !self.search.trim().is_empty();

        widgets::surface(&p)
            .inner_margin(Margin::same(12.0))
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = Vec2::ZERO;
                let width = ui.available_width();
                ui.set_width(width);
                self.search_bar(
                    ui,
                    &p,
                    width,
                    &count_label(shown.len(), total, filtering),
                    filtering,
                );
            });

        ui.add_space(12.0);

        let (title, body, offer_create) = if total == 0 {
            (
                "No records yet",
                "Create a record to keep a name and a description on this device.",
                true,
            )
        } else if shown.is_empty() {
            (
                "No matching records",
                "Try a different name, description, or ID.",
                false,
            )
        } else {
            ("", "", false)
        };

        self.show_records(ui, &shown, true, title, body, offer_create);
    }

    fn search_bar(
        &mut self,
        ui: &mut egui::Ui,
        p: &theme::Palette,
        width: f32,
        count: &str,
        filtering: bool,
    ) {
        let clear_w = if filtering {
            widgets::button_width(ui, "Clear", ButtonSize::Small)
        } else {
            0.0
        };
        let pill_w = widgets::pill_width(ui, count);
        let narrow = width < 520.0;

        if narrow {
            widgets::text_field(
                ui,
                p,
                &mut self.search,
                "Search by name, description, or ID",
                None,
                None,
            );
            ui.add_space(8.0);
            let (row, _) = ui.allocate_exact_size(Vec2::new(width, 32.0), Sense::hover());
            let pill_rect = Rect::from_min_size(row.min, Vec2::new(pill_w, 24.0));
            widgets::place(
                ui,
                pill_rect,
                "search-count",
                Layout::left_to_right(Align::Center),
                |ui| {
                    widgets::pill(ui, count, p.muted, p.surface_muted);
                },
            );
            if filtering {
                let clear_rect = Rect::from_min_size(
                    Pos2::new(row.right() - clear_w, row.top()),
                    Vec2::new(clear_w, 32.0),
                );
                let mut clear = false;
                widgets::place(
                    ui,
                    clear_rect,
                    "search-clear",
                    Layout::left_to_right(Align::Center),
                    |ui| {
                        clear = widgets::button(
                            ui,
                            p,
                            "Clear",
                            ButtonKind::Quiet,
                            ButtonSize::Small,
                            Some(clear_w),
                        )
                        .clicked();
                    },
                );
                if clear {
                    self.search.clear();
                }
            }
            return;
        }

        let gaps = if filtering { 24.0 } else { 12.0 };
        let field_w = (width - pill_w - clear_w - gaps).max(80.0);
        let (row, _) = ui.allocate_exact_size(Vec2::new(width, FIELD_H), Sense::hover());
        let field_rect = Rect::from_min_size(row.min, Vec2::new(field_w, FIELD_H));
        widgets::place(
            ui,
            field_rect,
            "search-field",
            Layout::top_down(Align::Min),
            |ui| {
                widgets::text_field(
                    ui,
                    p,
                    &mut self.search,
                    "Search by name, description, or ID",
                    None,
                    None,
                );
            },
        );

        let mut x = row.left() + field_w + 12.0;
        let pill_rect =
            Rect::from_min_size(Pos2::new(x, row.center().y - 12.0), Vec2::new(pill_w, 24.0));
        widgets::place(
            ui,
            pill_rect,
            "search-count",
            Layout::left_to_right(Align::Center),
            |ui| {
                widgets::pill(ui, count, p.muted, p.surface_muted);
            },
        );
        if filtering {
            x += pill_w + 12.0;
            let clear_rect =
                Rect::from_min_size(Pos2::new(x, row.top() + 4.0), Vec2::new(clear_w, 32.0));
            let mut clear = false;
            widgets::place(
                ui,
                clear_rect,
                "search-clear",
                Layout::left_to_right(Align::Center),
                |ui| {
                    clear = widgets::button(
                        ui,
                        p,
                        "Clear",
                        ButtonKind::Quiet,
                        ButtonSize::Small,
                        Some(clear_w),
                    )
                    .clicked();
                },
            );
            if clear {
                self.search.clear();
            }
        }
    }

    fn settings_page(&mut self, ui: &mut egui::Ui) {
        let width = ui.available_width();
        let gap = 16.0;
        let side = 320.0;
        let main = width - side - gap;
        if main >= 520.0 {
            widgets::columns_exact(ui, &[main, side], gap, |columns| {
                self.appearance_card(&mut columns[0]);
                self.storage_card(&mut columns[1]);
            });
        } else {
            self.appearance_card(ui);
            ui.add_space(gap);
            self.storage_card(ui);
        }
    }

    fn appearance_card(&mut self, ui: &mut egui::Ui) {
        let p = theme::palette(self.dark);
        widgets::surface(&p).show(ui, |ui| {
            ui.spacing_mut().item_spacing = Vec2::ZERO;
            ui.set_width(ui.available_width());
            widgets::section_title(ui, &p, "Appearance");
            ui.add_space(4.0);
            widgets::muted(ui, &p, "Choose the theme used across the workspace.");
            ui.add_space(16.0);
            widgets::responsive_grid(ui, 240.0, 12.0, 2, |ui, index| {
                let (title, body, selected) = if index == 0 {
                    (
                        "Light",
                        "Bright surfaces with an indigo accent. Suited to daytime work.",
                        !self.dark,
                    )
                } else {
                    (
                        "Dark",
                        "Dim surfaces with the same accent, spacing, and type.",
                        self.dark,
                    )
                };
                if widgets::choice_card(ui, &p, title, body, selected).clicked() {
                    self.dark = index == 1;
                }
            });
        });
    }

    fn storage_card(&mut self, ui: &mut egui::Ui) {
        let p = theme::palette(self.dark);
        widgets::surface(&p).show(ui, |ui| {
            ui.spacing_mut().item_spacing = Vec2::ZERO;
            ui.set_width(ui.available_width());
            widgets::section_title(ui, &p, "Local data");
            ui.add_space(4.0);
            let count = self.items.len();
            let noun = if count == 1 { "record" } else { "records" };
            widgets::muted(
                ui,
                &p,
                &format!("{count} {noun} saved as JSON on this device. Nothing is uploaded."),
            );
            ui.add_space(16.0);
            ui.label(semi_text("Storage path", 13.0, p.text));
            ui.add_space(6.0);
            widgets::readout(ui, &p, &storage::storage_path_label());
        });
    }

    fn show_records(
        &mut self,
        ui: &mut egui::Ui,
        records: &[Item],
        actions: bool,
        empty_title: &str,
        empty_body: &str,
        offer_create: bool,
    ) {
        let p = theme::palette(self.dark);

        if records.is_empty() {
            widgets::surface(&p)
                .inner_margin(Margin::symmetric(24.0, 36.0))
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing = Vec2::ZERO;
                    ui.set_width(ui.available_width());
                    ui.vertical_centered(|ui| {
                        ui.set_max_width(420.0);
                        widgets::empty_mark(ui, &p);
                        ui.add_space(12.0);
                        ui.label(semi_text(empty_title, 18.0, p.text));
                        ui.add_space(6.0);
                        ui.add(Label::new(body_text(empty_body, 13.0, p.muted)).wrap());
                        if offer_create {
                            ui.add_space(16.0);
                            if widgets::button(
                                ui,
                                &p,
                                "New record",
                                ButtonKind::Primary,
                                ButtonSize::Medium,
                                None,
                            )
                            .clicked()
                            {
                                self.open_create_form();
                            }
                        }
                    });
                });
            return;
        }

        if columns_for(ui.available_width(), actions).is_none() {
            self.record_cards(ui, records, actions);
            return;
        }

        widgets::surface(&p)
            .inner_margin(Margin::ZERO)
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = Vec2::ZERO;
                let width = ui.available_width();
                ui.set_width(width);
                let Some(columns) = columns_for(width, actions) else {
                    return;
                };

                self.table_band(ui, &columns, width, 40.0, true, actions, None, true);
                for (index, item) in records.iter().enumerate() {
                    let last = index + 1 == records.len();
                    self.table_band(ui, &columns, width, 56.0, false, actions, Some(item), !last);
                }
            });
    }

    fn table_band(
        &mut self,
        ui: &mut egui::Ui,
        columns: &Columns,
        width: f32,
        height: f32,
        header: bool,
        actions: bool,
        item: Option<&Item>,
        draw_rule: bool,
    ) {
        let p = theme::palette(self.dark);
        let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());

        if header {
            let fill_rect = Rect::from_min_max(
                Pos2::new(rect.left() + 1.0, rect.top() + 1.0),
                Pos2::new(rect.right() - 1.0, rect.bottom()),
            );
            ui.painter().rect_filled(
                fill_rect,
                Rounding {
                    nw: 11.0,
                    ne: 11.0,
                    sw: 0.0,
                    se: 0.0,
                },
                p.surface_muted,
            );
        } else if response.hovered() {
            let fill_rect = Rect::from_min_max(
                Pos2::new(rect.left() + 1.0, rect.top()),
                Pos2::new(
                    rect.right() - 1.0,
                    rect.bottom() - if draw_rule { 0.0 } else { 1.0 },
                ),
            );
            ui.painter()
                .rect_filled(fill_rect, Rounding::ZERO, p.surface_hover);
        }

        let cells = column_rects(rect, columns, actions);
        if header {
            let labels: &[&str] = if actions {
                &["ID", "NAME", "DESCRIPTION", "ACTIONS"]
            } else {
                &["ID", "NAME", "DESCRIPTION"]
            };
            for (index, label) in labels.iter().enumerate() {
                let cell = cells[index];
                let actions_header = actions && index + 1 == labels.len();
                let (pos, align) = if actions_header {
                    (
                        Pos2::new(cell.right(), cell.center().y),
                        Align2::RIGHT_CENTER,
                    )
                } else {
                    (Pos2::new(cell.left(), cell.center().y), Align2::LEFT_CENTER)
                };
                ui.painter()
                    .text(pos, align, *label, theme::font_semi(11.0), p.faint);
            }
        } else if let Some(item) = item {
            widgets::place(
                ui,
                cells[0],
                ("row-id", item.id),
                Layout::left_to_right(Align::Center),
                |ui| {
                    widgets::pill(ui, &format!("#{}", item.id), p.accent_text, p.accent_soft);
                },
            );
            widgets::paint_ellipsis(ui, cells[1], &item.name, theme::font_semi(14.0), p.text);
            let empty = item.description.trim().is_empty();
            let description = if empty {
                "No description"
            } else {
                item.description.as_str()
            };
            let color = if empty { p.faint } else { p.muted };
            widgets::paint_ellipsis(ui, cells[2], description, theme::font_body(13.0), color);
            if !empty {
                ui.interact(cells[2], ui.id().with(("desc", item.id)), Sense::hover())
                    .on_hover_text(&item.description);
            }
            if actions {
                widgets::place(
                    ui,
                    cells[3],
                    ("row-actions", item.id),
                    Layout::right_to_left(Align::Center),
                    |ui| {
                        ui.spacing_mut().item_spacing.x = 8.0;
                        if widgets::button(
                            ui,
                            &p,
                            "Delete",
                            ButtonKind::QuietDanger,
                            ButtonSize::Small,
                            None,
                        )
                        .clicked()
                        {
                            self.pending_delete = Some(item.id);
                        }
                        if widgets::button(
                            ui,
                            &p,
                            "Edit",
                            ButtonKind::Secondary,
                            ButtonSize::Small,
                            None,
                        )
                        .clicked()
                        {
                            self.open_edit_form(item);
                        }
                    },
                );
            }
        }

        if draw_rule {
            ui.painter().hline(
                (rect.left() + 1.0)..=(rect.right() - 1.0),
                rect.bottom(),
                Stroke::new(1.0_f32, p.border),
            );
        }
    }

    fn record_cards(&mut self, ui: &mut egui::Ui, records: &[Item], actions: bool) {
        let p = theme::palette(self.dark);
        for (index, item) in records.iter().enumerate() {
            widgets::surface(&p)
                .inner_margin(Margin::same(16.0))
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing = Vec2::ZERO;
                    let width = ui.available_width();
                    ui.set_width(width);
                    let (row, _) = ui.allocate_exact_size(Vec2::new(width, 24.0), Sense::hover());
                    let id_label = format!("#{}", item.id);
                    let pill_w = widgets::pill_width(ui, &id_label);
                    let pill_rect = Rect::from_min_size(row.min, Vec2::new(pill_w, 24.0));
                    widgets::place(
                        ui,
                        pill_rect,
                        ("card-id", item.id),
                        Layout::left_to_right(Align::Center),
                        |ui| {
                            widgets::pill(ui, &id_label, p.accent_text, p.accent_soft);
                        },
                    );
                    let name_rect = Rect::from_min_max(
                        Pos2::new((row.left() + pill_w + 8.0).min(row.right()), row.top()),
                        row.max,
                    );
                    widgets::paint_ellipsis(
                        ui,
                        name_rect,
                        &item.name,
                        theme::font_semi(15.0),
                        p.text,
                    );

                    ui.add_space(8.0);
                    let empty = item.description.trim().is_empty();
                    let description = if empty {
                        "No description".to_string()
                    } else {
                        preview(&item.description, 180)
                    };
                    let color = if empty { p.faint } else { p.muted };
                    ui.add(Label::new(body_text(description, 13.0, color)).wrap());

                    if actions {
                        ui.add_space(12.0);
                        let gap = 8.0;
                        let each = ((width - gap) / 2.0).max(32.0);
                        let (buttons, _) =
                            ui.allocate_exact_size(Vec2::new(width, FIELD_H), Sense::hover());
                        let edit_rect = Rect::from_min_size(buttons.min, Vec2::new(each, FIELD_H));
                        let delete_rect = Rect::from_min_size(
                            Pos2::new(buttons.right() - each, buttons.top()),
                            Vec2::new(each, FIELD_H),
                        );
                        let mut edit = false;
                        let mut delete = false;
                        widgets::place(
                            ui,
                            edit_rect,
                            ("card-edit", item.id),
                            Layout::left_to_right(Align::Center),
                            |ui| {
                                edit = widgets::button(
                                    ui,
                                    &p,
                                    "Edit",
                                    ButtonKind::Secondary,
                                    ButtonSize::Medium,
                                    Some(each),
                                )
                                .clicked();
                            },
                        );
                        widgets::place(
                            ui,
                            delete_rect,
                            ("card-delete", item.id),
                            Layout::left_to_right(Align::Center),
                            |ui| {
                                delete = widgets::button(
                                    ui,
                                    &p,
                                    "Delete",
                                    ButtonKind::Danger,
                                    ButtonSize::Medium,
                                    Some(each),
                                )
                                .clicked();
                            },
                        );
                        if edit {
                            self.open_edit_form(item);
                        }
                        if delete {
                            self.pending_delete = Some(item.id);
                        }
                    }
                });
            if index + 1 != records.len() {
                ui.add_space(12.0);
            }
        }
    }

    fn modals(&mut self, ctx: &egui::Context) {
        if !self.show_form && self.pending_delete.is_none() {
            return;
        }

        if ctx.input(|input| input.key_pressed(egui::Key::Escape)) {
            self.close_overlays();
            return;
        }

        let p = theme::palette(self.dark);
        if widgets::backdrop(ctx, p.overlay) {
            self.close_overlays();
            return;
        }

        if self.show_form {
            self.form_window(ctx);
        } else if self.pending_delete.is_some() {
            self.delete_window(ctx);
        }
    }

    fn form_window(&mut self, ctx: &egui::Context) {
        let p = theme::palette(self.dark);
        let editing = self.editing_id.is_some();
        let screen = ctx.screen_rect();
        let dialog_w = (screen.width() - 48.0).clamp(300.0, 480.0);
        let description_h = if screen.height() < 700.0 { 96.0 } else { 128.0 };

        egui::Window::new("record-form")
            .title_bar(false)
            .collapsible(false)
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .pivot(Align2::CENTER_CENTER)
            .order(egui::Order::Foreground)
            .frame(widgets::dialog_frame(&p, self.dark))
            .show(ctx, |ui| {
                ui.set_width(dialog_w - 48.0);
                ui.spacing_mut().item_spacing.y = 6.0;

                if self.focus_name {
                    self.focus_name = false;
                    ctx.memory_mut(|memory| memory.request_focus(Id::new("record-name")));
                }

                ui.label(semi_text(
                    if editing { "Edit record" } else { "New record" },
                    20.0,
                    p.text,
                ));
                ui.label(body_text(
                    "A name is required. A description is optional.",
                    13.0,
                    p.muted,
                ));
                ui.add_space(12.0);

                widgets::field_header(
                    ui,
                    &p,
                    "Name",
                    true,
                    self.name_input.chars().count(),
                    NAME_MAX,
                );
                let name_invalid = self.name_error.is_some();
                widgets::invalid_scope(ui, name_invalid, p.danger, |ui| {
                    if widgets::text_field(
                        ui,
                        &p,
                        &mut self.name_input,
                        "Record name",
                        Some(Id::new("record-name")),
                        Some(NAME_MAX),
                    )
                    .changed()
                    {
                        self.name_error = None;
                    }
                });
                if let Some(error) = &self.name_error {
                    widgets::error_text(ui, &p, error);
                }

                ui.add_space(10.0);
                widgets::field_header(
                    ui,
                    &p,
                    "Description",
                    false,
                    self.description_input.chars().count(),
                    DESCRIPTION_MAX,
                );
                let description_invalid = self.description_error.is_some();
                widgets::invalid_scope(ui, description_invalid, p.danger, |ui| {
                    if widgets::text_area(
                        ui,
                        &p,
                        &mut self.description_input,
                        "Add details worth finding later",
                        Some(DESCRIPTION_MAX),
                        description_h,
                    )
                    .changed()
                    {
                        self.description_error = None;
                    }
                });
                if let Some(error) = &self.description_error {
                    widgets::error_text(ui, &p, error);
                }

                ui.add_space(16.0);
                let (line, _) =
                    ui.allocate_exact_size(Vec2::new(ui.available_width(), 1.0), Sense::hover());
                ui.painter().hline(
                    line.x_range(),
                    line.center().y,
                    Stroke::new(1.0_f32, p.border),
                );
                ui.add_space(8.0);

                widgets::right_row(ui, FIELD_H, 8.0, "form-actions", |ui| {
                    if widgets::button(
                        ui,
                        &p,
                        if editing {
                            "Save changes"
                        } else {
                            "Create record"
                        },
                        ButtonKind::Primary,
                        ButtonSize::Medium,
                        None,
                    )
                    .clicked()
                    {
                        self.save_form();
                    }
                    if widgets::button(
                        ui,
                        &p,
                        "Cancel",
                        ButtonKind::Secondary,
                        ButtonSize::Medium,
                        None,
                    )
                    .clicked()
                    {
                        self.close_overlays();
                    }
                });
            });
    }

    fn delete_window(&mut self, ctx: &egui::Context) {
        let Some(id) = self.pending_delete else {
            return;
        };
        let p = theme::palette(self.dark);
        let name = self
            .items
            .iter()
            .find(|item| item.id == id)
            .map(|item| item.name.clone())
            .unwrap_or_else(|| "this record".to_string());
        let screen = ctx.screen_rect();
        let dialog_w = (screen.width() - 48.0).clamp(300.0, 440.0);

        egui::Window::new("delete-record")
            .title_bar(false)
            .collapsible(false)
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .pivot(Align2::CENTER_CENTER)
            .order(egui::Order::Foreground)
            .frame(widgets::dialog_frame(&p, self.dark))
            .show(ctx, |ui| {
                ui.set_width(dialog_w - 48.0);
                ui.label(semi_text("Delete record?", 20.0, p.text));
                ui.add_space(6.0);
                ui.add(
                    Label::new(body_text(
                        format!(
                            "\"{name}\" will be removed from this device. This cannot be undone."
                        ),
                        13.0,
                        p.muted,
                    ))
                    .wrap(),
                );
                ui.add_space(16.0);
                widgets::right_row(ui, FIELD_H, 8.0, "delete-actions", |ui| {
                    if widgets::button(
                        ui,
                        &p,
                        "Delete",
                        ButtonKind::Danger,
                        ButtonSize::Medium,
                        None,
                    )
                    .clicked()
                    {
                        self.delete_item(id);
                    }
                    if widgets::button(
                        ui,
                        &p,
                        "Cancel",
                        ButtonKind::Secondary,
                        ButtonSize::Medium,
                        None,
                    )
                    .clicked()
                    {
                        self.pending_delete = None;
                    }
                });
            });
    }
}

impl eframe::App for CrudApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        theme::apply(ctx, self.dark);
        self.sidebar(ctx);
        self.topbar(ctx);

        let p = theme::palette(self.dark);
        let pad = theme::page_padding(ctx.screen_rect().width());

        egui::CentralPanel::default()
            .frame(
                Frame::none()
                    .fill(p.background)
                    .inner_margin(Margin::same(pad)),
            )
            .show(ctx, |ui| {
                ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing = Vec2::ZERO;
                        ui.set_width(ui.available_width());
                        if let Some(notice) = self
                            .notice
                            .clone()
                            .filter(|notice| notice.duration.is_none())
                        {
                            if widgets::notice_banner(ui, &p, notice.kind, &notice.text) {
                                self.notice = None;
                            }
                            ui.add_space(16.0);
                        }

                        match self.page {
                            Page::Dashboard => self.dashboard(ui),
                            Page::Records => self.records_page(ui),
                            Page::Settings => self.settings_page(ui),
                        }
                    });
            });

        self.modals(ctx);
        self.show_toast(ctx);
    }
}

impl Notice {
    fn persistent(kind: BannerKind, text: impl Into<String>) -> Self {
        Self {
            kind,
            text: text.into(),
            born: Instant::now(),
            duration: None,
            frozen: None,
        }
    }

    fn toast(kind: BannerKind, text: impl Into<String>, duration: Duration) -> Self {
        Self {
            kind,
            text: text.into(),
            born: Instant::now(),
            duration: Some(duration),
            frozen: None,
        }
    }
}

impl CrudApp {
    fn show_toast(&mut self, ctx: &egui::Context) {
        let Some(snapshot) = self
            .notice
            .clone()
            .filter(|notice| notice.duration.is_some())
        else {
            return;
        };

        let elapsed = snapshot.frozen.unwrap_or_else(|| snapshot.born.elapsed());
        let age = elapsed.as_secs_f32();
        let limit = snapshot.duration.map(|duration| duration.as_secs_f32());

        if snapshot.frozen.is_none() {
            let keep_animating = limit.is_none_or(|limit| age < limit);
            if keep_animating {
                ctx.request_repaint_after(Duration::from_millis(16));
            }
        }

        let (opacity, slide) = if snapshot.frozen.is_some() {
            (1.0, 0.0)
        } else {
            toast_motion(age, limit)
        };

        let hit = widgets::toast(
            ctx,
            &theme::palette(self.dark),
            self.dark,
            snapshot.kind,
            &snapshot.text,
            opacity,
            slide,
        );

        if hit.dismiss {
            self.notice = None;
            return;
        }

        let expired = {
            let Some(notice) = &mut self.notice else {
                return;
            };
            if hit.hovered {
                if notice.frozen.is_none() {
                    notice.frozen = Some(notice.born.elapsed());
                }
            } else if let Some(frozen) = notice.frozen.take() {
                notice.born = Instant::now() - frozen;
            }
            notice.duration.is_some_and(|duration| {
                notice.frozen.unwrap_or_else(|| notice.born.elapsed()) >= duration
            })
        };

        if expired {
            self.notice = None;
        }
    }
}

fn toast_motion(age: f32, limit: Option<f32>) -> (f32, f32) {
    let enter = smoothstep((age / 0.22).clamp(0.0, 1.0));
    let exit = match limit {
        Some(limit) => {
            let start = (limit - 0.28).max(0.0);
            if age <= start {
                0.0
            } else {
                smoothstep(((age - start) / (limit - start).max(0.05)).clamp(0.0, 1.0))
            }
        }
        None => 0.0,
    };
    let opacity = enter * (1.0 - exit);
    let slide = (1.0 - enter) * 18.0 + exit * 14.0;
    (opacity, slide)
}

fn smoothstep(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

fn brand_mark(ui: &mut egui::Ui, p: &theme::Palette) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(36.0), Sense::hover());
    ui.painter()
        .rect_filled(rect, Rounding::same(10.0), p.accent);
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        "RC",
        theme::font_semi(13.0),
        p.on_accent,
    );
}

fn stat_card(
    ui: &mut egui::Ui,
    p: &theme::Palette,
    label: &str,
    value: &str,
    hint: &str,
    tone: egui::Color32,
) {
    widgets::surface(p)
        .inner_margin(Margin::same(18.0))
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing = Vec2::ZERO;
            ui.set_width(ui.available_width());
            ui.set_min_height(88.0);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 8.0;
                widgets::dot(ui, tone);
                ui.label(semi_text(label, 12.0, p.muted));
            });
            ui.add_space(8.0);
            ui.label(semi_text(value, 30.0, p.text));
            ui.add_space(4.0);
            ui.label(body_text(hint, 12.0, p.muted));
        });
}

const TABLE_INSET: f32 = 16.0;
const TABLE_GAP: f32 = 16.0;
const TABLE_ID: f32 = 72.0;
const TABLE_ACTIONS: f32 = 156.0;

fn columns_for(width: f32, actions: bool) -> Option<Columns> {
    let actions_w = if actions { TABLE_ACTIONS } else { 0.0 };
    let gaps = TABLE_GAP * if actions { 3.0 } else { 2.0 };
    let rest = width - TABLE_INSET * 2.0 - TABLE_ID - actions_w - gaps;
    if rest < 280.0 {
        return None;
    }
    let name = (rest * 0.38).clamp(150.0, 280.0);
    let description = rest - name;
    if description < 120.0 {
        return None;
    }
    Some(Columns {
        id: TABLE_ID,
        name,
        description,
        actions: actions_w,
    })
}

fn column_rects(row: Rect, columns: &Columns, actions: bool) -> Vec<Rect> {
    let mut widths = vec![columns.id, columns.name, columns.description];
    if actions {
        widths.push(columns.actions);
    }
    let mut x = row.left() + TABLE_INSET;
    let mut rects = Vec::with_capacity(widths.len());
    for width in widths {
        rects.push(Rect::from_min_size(
            Pos2::new(x, row.top()),
            Vec2::new(width, row.height()),
        ));
        x += width + TABLE_GAP;
    }
    rects
}

fn count_label(shown: usize, total: usize, filtering: bool) -> String {
    if filtering {
        format!("{shown} of {total}")
    } else if total == 1 {
        "1 record".to_string()
    } else {
        format!("{total} records")
    }
}

fn preview(text: &str, max_chars: usize) -> String {
    let trimmed = text.trim();
    if trimmed.chars().count() <= max_chars {
        trimmed.to_string()
    } else {
        let clipped: String = trimmed.chars().take(max_chars).collect();
        format!("{clipped}…")
    }
}
