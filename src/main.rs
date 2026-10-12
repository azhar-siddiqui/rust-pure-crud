#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod models;
mod storage;
mod theme;

use eframe::egui;
use models::CrudApp;
use theme::*;

impl eframe::App for CrudApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        theme::apply_production_styling(ctx);

        egui::SidePanel::left("navigation")
            .resizable(false)
            .default_width(190.0)
            .frame(
                egui::Frame::none()
                    .fill(PANEL)
                    .inner_margin(egui::Margin::same(18.0)),
            )
            .show(ctx, |ui| {
                ui.label(
                    egui::RichText::new("◆  DATA CONSOLE")
                        .strong()
                        .size(17.0)
                        .color(TEXT),
                );

                ui.add_space(34.0);

                ui.label(
                    egui::RichText::new("WORKSPACE")
                        .strong()
                        .size(10.0)
                        .color(MUTED),
                );

                ui.add_space(10.0);

                ui.label(egui::RichText::new("▦  Dashboard").size(14.0).color(BLUE));

                ui.add_space(12.0);

                ui.label(egui::RichText::new("▤  Resources").size(14.0).color(TEXT));

                ui.add_space(12.0);

                ui.label(egui::RichText::new("⚙  Settings").size(14.0).color(MUTED));

                ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                    ui.separator();

                    ui.label(
                        egui::RichText::new("Local workspace")
                            .size(12.0)
                            .color(MUTED),
                    );

                    ui.label(egui::RichText::new("v1.0.0").size(11.0).color(MUTED));
                });
            });

        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(BG)
                    .inner_margin(egui::Margin::same(24.0)),
            )
            .show(ctx, |ui| {
                // Header
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new("Dashboard")
                                .size(27.0)
                                .strong()
                                .color(TEXT),
                        );

                        ui.label(
                            egui::RichText::new("Manage and organize your resources.")
                                .size(13.0)
                                .color(MUTED),
                        );
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new("●  LOCAL")
                                .strong()
                                .size(12.0)
                                .color(GREEN),
                        );
                    });
                });

                ui.add_space(22.0);

                // Summary cards
                let total = self.items.len();
                let editing = self.editing_id.is_some();

                ui.columns(3, |columns| {
                    summary_card(&mut columns[0], "TOTAL RESOURCES", &total.to_string(), BLUE);
                    summary_card(&mut columns[1], "STORAGE", "Local JSON", GREEN);
                    summary_card(
                        &mut columns[2],
                        "FORM STATUS",
                        if editing { "Editing" } else { "Ready" },
                        if editing { BLUE } else { GREEN },
                    );
                });

                ui.add_space(22.0);

                // Feedback
                if let Some(error) = self.error.clone() {
                    ui.label(
                        egui::RichText::new(format!("⚠  {error}"))
                            .color(RED)
                            .size(13.0),
                    );
                    ui.add_space(8.0);
                }

                if let Some(message) = self.message.clone() {
                    ui.label(
                        egui::RichText::new(format!("✓  {message}"))
                            .color(GREEN)
                            .size(13.0),
                    );
                    ui.add_space(8.0);
                }

                // Responsive content: two panels on wide windows,
                // vertically stacked on narrow windows.
                let wide_layout = ui.available_width() >= 850.0;

                if wide_layout {
                    ui.columns(2, |columns| {
                        form_panel(&mut columns[0], self);
                        records_panel(&mut columns[1], self);
                    });
                } else {
                    form_panel(ui, self);
                    ui.add_space(16.0);
                    records_panel(ui, self);
                }
            });

        // Delete confirmation dialog
        if let Some(id) = self.pending_delete {
            egui::Window::new("Confirm deletion")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.set_min_width(300.0);

                    ui.label("Are you sure you want to delete this resource?");
                    ui.label(
                        egui::RichText::new("This action cannot be undone.")
                            .color(MUTED)
                            .size(12.0),
                    );

                    ui.add_space(14.0);

                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            self.pending_delete = None;
                        }

                        if ui
                            .button(egui::RichText::new("Delete").color(RED))
                            .clicked()
                        {
                            self.delete_item(id);
                            persist(self);
                        }
                    });
                });
        }
    }
}

fn summary_card(ui: &mut egui::Ui, label: &str, value: &str, accent: egui::Color32) {
    egui::Frame::none()
        .fill(PANEL)
        .stroke(egui::Stroke::new(1.0_f32, BORDER))
        .rounding(egui::Rounding::same(10.0))
        .inner_margin(egui::Margin::same(16.0))
        .show(ui, |ui| {
            ui.set_min_height(80.0);

            ui.label(egui::RichText::new(label).size(10.0).strong().color(MUTED));

            ui.add_space(8.0);

            ui.label(egui::RichText::new(value).size(19.0).strong().color(accent));
        });
}

fn form_panel(ui: &mut egui::Ui, app: &mut CrudApp) {
    egui::Frame::none()
        .fill(PANEL)
        .stroke(egui::Stroke::new(1.0_f32, BORDER))
        .rounding(egui::Rounding::same(12.0))
        .inner_margin(egui::Margin::same(18.0))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());

            ui.heading(if app.editing_id.is_some() {
                "Edit resource"
            } else {
                "Create resource"
            });

            ui.label(
                egui::RichText::new("Enter the resource details below.")
                    .size(12.0)
                    .color(MUTED),
            );

            ui.add_space(16.0);

            ui.label(
                egui::RichText::new("RESOURCE NAME")
                    .size(10.0)
                    .strong()
                    .color(MUTED),
            );

            ui.add(
                egui::TextEdit::singleline(&mut app.input_name)
                    .hint_text("e.g. PostgreSQL Database")
                    .desired_width(f32::INFINITY)
                    .font(egui::FontId::proportional(14.0))
                    .margin(egui::vec2(12.0, 10.0)),
            );

            ui.add_space(12.0);

            ui.label(
                egui::RichText::new("DESCRIPTION")
                    .size(10.0)
                    .strong()
                    .color(MUTED),
            );

            ui.add(
                egui::TextEdit::multiline(&mut app.input_description)
                    .hint_text("Describe this resource...")
                    .desired_rows(4)
                    .desired_width(f32::INFINITY)
                    .font(egui::FontId::proportional(14.0))
                    .margin(egui::vec2(12.0, 10.0)),
            );

            ui.add_space(16.0);

            let button_text = if app.editing_id.is_some() {
                "Save changes"
            } else {
                "Create resource"
            };

            if ui
                .add_sized(
                    [ui.available_width(), 38.0],
                    egui::Button::new(
                        egui::RichText::new(button_text)
                            .strong()
                            .color(egui::Color32::WHITE),
                    )
                    .fill(BLUE),
                )
                .clicked()
            {
                app.save_item();

                if app.error.is_none() {
                    persist(app);
                }
            }

            if app.editing_id.is_some() {
                ui.add_space(8.0);

                if ui
                    .add_sized(
                        [ui.available_width(), 34.0],
                        egui::Button::new("Cancel editing"),
                    )
                    .clicked()
                {
                    app.cancel_edit();
                }
            }
        });
}

fn records_panel(ui: &mut egui::Ui, app: &mut CrudApp) {
    egui::Frame::none()
        .fill(PANEL)
        .stroke(egui::Stroke::new(1.0_f32, BORDER))
        .rounding(egui::Rounding::same(12.0))
        .inner_margin(egui::Margin::same(18.0))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());

            ui.horizontal(|ui| {
                ui.heading("Resources");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(format!("{} total", app.items.len()))
                            .size(12.0)
                            .color(MUTED),
                    );
                });
            });

            ui.add_space(12.0);

            ui.add(
                egui::TextEdit::singleline(&mut app.search_query)
                    .hint_text("Search by name or description...")
                    .desired_width(f32::INFINITY),
            );

            ui.add_space(12.0);

            let filtered = app.filtered_items();

            egui::ScrollArea::vertical()
                .max_height(430.0)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    if filtered.is_empty() {
                        ui.add_space(24.0);

                        ui.vertical_centered(|ui| {
                            ui.label(
                                egui::RichText::new("No resources found")
                                    .size(16.0)
                                    .strong()
                                    .color(TEXT),
                            );

                            ui.label(
                                egui::RichText::new("Create a resource or try another search.")
                                    .size(12.0)
                                    .color(MUTED),
                            );
                        });

                        return;
                    }

                    for item in filtered {
                        egui::Frame::none()
                            .fill(PANEL_ALT)
                            .rounding(egui::Rounding::same(8.0))
                            .inner_margin(egui::Margin::same(12.0))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.vertical(|ui| {
                                        ui.label(
                                            egui::RichText::new(&item.name)
                                                .strong()
                                                .size(14.0)
                                                .color(TEXT),
                                        );

                                        ui.label(
                                            egui::RichText::new(format!("ID #{:03}", item.id))
                                                .size(10.0)
                                                .color(BLUE),
                                        );
                                    });

                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            if ui
                                                .button(egui::RichText::new("Delete").color(RED))
                                                .clicked()
                                            {
                                                app.pending_delete = Some(item.id);
                                            }

                                            if ui.button("Edit").clicked() {
                                                app.start_edit(item.id);
                                            }
                                        },
                                    );
                                });

                                if !item.description.is_empty() {
                                    ui.add_space(6.0);

                                    ui.label(
                                        egui::RichText::new(&item.description)
                                            .size(12.0)
                                            .color(MUTED),
                                    );
                                }
                            });

                        ui.add_space(8.0);
                    }
                });
        });
}

fn persist(app: &mut CrudApp) {
    match storage::save_items(&app.items) {
        Ok(()) => {}
        Err(error) => {
            app.error = Some(format!("Data could not be saved: {error}"));
        }
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Rust CRUD App",
        options,
        Box::new(|cc| {
            theme::configure_fonts(&cc.egui_ctx);

            let mut app = CrudApp::default();

            if let Ok(items) = storage::load_items() {
                app.load_items(items);
            }

            Ok(Box::new(app))
        }),
    )
}
