//! Header and configuration sections of the main panel: sources, exclusions and destination.

use std::path::Path;

use eframe::egui::{self, RichText};

use super::{BackupsApp, format, logo, theme, widgets};

impl BackupsApp {
    pub(super) fn header(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = theme::GAP;
            logo::paint(ui);
            theme::lines(ui, |ui| {
                ui.label(RichText::new("Backups App").heading().color(theme::TEXT));
                ui.label(
                    RichText::new("Save your folders to a ZIP file.")
                        .size(13.0)
                        .color(theme::TEXT_WEAK),
                );
            });
        });
    }

    pub(super) fn sources_section(&mut self, ui: &mut egui::Ui, busy: bool) {
        let count = self.config.sources.len();
        widgets::section(
            ui,
            1,
            "Sources",
            "Folders included in each backup.",
            (count > 0).then_some(count),
            |ui| {
                let mut remove = None;
                if !self.config.sources.is_empty() {
                    egui::ScrollArea::vertical()
                        .max_height(200.0)
                        .id_salt("sources")
                        .show(ui, |ui| {
                            for (i, s) in self.config.sources.iter().enumerate() {
                                if folder_row(ui, Some(s), |ui| {
                                    widgets::remove_button(ui, !busy, 26.0)
                                })
                                .clicked()
                                {
                                    remove = Some(i);
                                }
                            }
                        });
                }
                if let Some(i) = remove {
                    self.config.sources.remove(i);
                    self.save();
                }
                if widgets::add_button(ui, "Add folder", !busy).clicked()
                    && let Some(p) = rfd::FileDialog::new().pick_folder()
                    && !self.config.sources.contains(&p)
                {
                    self.config.sources.push(p);
                    self.save();
                }
            },
        );
    }

    pub(super) fn destination_section(&mut self, ui: &mut egui::Ui, busy: bool) {
        widgets::section(
            ui,
            3,
            "Destination",
            "Folder where the ZIP file of each backup is saved.",
            None,
            |ui| {
                let label = if self.config.destination.is_some() {
                    "Change"
                } else {
                    "Choose"
                };
                let clicked = folder_row(ui, self.config.destination.as_deref(), |ui| {
                    ui.add_enabled(!busy, egui::Button::new(label)).clicked()
                });
                if clicked && let Some(p) = rfd::FileDialog::new().pick_folder() {
                    self.config.destination = Some(p);
                    self.save();
                }
            },
        );
    }

    pub(super) fn ignored_section(&mut self, ui: &mut egui::Ui, busy: bool) {
        let count = self.config.ignored.len();
        let mut remove = None;
        let mut add = false;
        widgets::section(
            ui,
            2,
            "Exclusions",
            "Names of files and folders that are not copied. Supports * and ?.",
            (count > 0).then_some(count),
            |ui| {
                if count > 0 {
                    ui.horizontal_wrapped(|ui| {
                        for (i, p) in self.config.ignored.iter().enumerate() {
                            if widgets::chip(ui, p, !busy) {
                                remove = Some(i);
                            }
                        }
                    });
                }

                ui.horizontal(|ui| {
                    let button_width = 80.0;
                    let edit = ui.add_enabled(
                        !busy,
                        egui::TextEdit::singleline(&mut self.ignore_input)
                            .desired_width(
                                ui.available_width() - button_width - ui.spacing().item_spacing.x,
                            )
                            .min_size(egui::Vec2::new(0.0, theme::CONTROL_HEIGHT))
                            .align(egui::Align2::LEFT_CENTER)
                            .font(egui::TextStyle::Monospace)
                            .margin(egui::Margin::symmetric(10, 0))
                            .hint_text("Add a pattern, for example *.log"),
                    );
                    if edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        add = true;
                        edit.request_focus();
                    }
                    let can_add = !busy && !self.ignore_input.trim().is_empty();
                    if ui
                        .add_enabled(
                            can_add,
                            egui::Button::new("Add").min_size(egui::Vec2::new(button_width, 0.0)),
                        )
                        .clicked()
                    {
                        add = true;
                    }
                });
            },
        );

        if remove.is_none() && !add {
            return;
        }
        let patterns = &mut self.config.ignored;
        if let Some(i) = remove {
            patterns.remove(i);
        }
        if add {
            for p in split_patterns(&self.ignore_input) {
                if !patterns.iter().any(|q| q.eq_ignore_ascii_case(p)) {
                    patterns.push(p.to_owned());
                }
            }
            self.ignore_input.clear();
        }
        self.save();
    }
}

/// Splits the text of the input field into patterns: the user can type several, separated by
/// commas.
fn split_patterns(text: &str) -> impl Iterator<Item = &str> {
    text.split([',', '\n'])
        .map(str::trim)
        .filter(|p| !p.is_empty())
}

/// Folder row: icon, name and path on two lines, and whatever `right` adds on the right.
fn folder_row<R>(
    ui: &mut egui::Ui,
    path: Option<&Path>,
    right: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    egui::Frame::new()
        .fill(theme::INSET)
        .stroke(egui::Stroke::new(1.0, theme::BORDER))
        .corner_radius(egui::CornerRadius::same(3))
        .inner_margin(theme::margin(theme::SPACE))
        .show(ui, |ui| {
            egui::Sides::new()
                .shrink_left()
                .truncate()
                .show(
                    ui,
                    |ui| {
                        ui.spacing_mut().item_spacing.x = theme::GAP;
                        widgets::folder_tile(ui, path.is_some());
                        theme::lines(ui, |ui| {
                            let (name, detail, name_color) = match path {
                                Some(path) => {
                                    let (name, parent) = format::split_path(path);
                                    (name, parent, theme::TEXT)
                                }
                                None => (
                                    "No destination folder".to_owned(),
                                    "Choose where to save the backups.".to_owned(),
                                    theme::TEXT_WEAK,
                                ),
                            };
                            ui.add(
                                egui::Label::new(
                                    RichText::new(name)
                                        .family(theme::semibold())
                                        .color(name_color),
                                )
                                .truncate(),
                            );
                            ui.add(
                                egui::Label::new(
                                    RichText::new(detail).size(12.0).color(theme::TEXT_WEAK),
                                )
                                .truncate(),
                            );
                        });
                    },
                    right,
                )
                .1
        })
        .inner
}
