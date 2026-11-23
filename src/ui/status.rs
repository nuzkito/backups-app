//! Bottom panel: result of the last backup, progress, and the primary action.

use std::path::Path;

use eframe::egui::{self, RichText};

use super::theme::{self, Tone};
use super::{BackupsApp, format, widgets};
use crate::config::LastBackup;
use crate::engine::{Outcome, Progress, Report};

/// Width of the primary and secondary buttons of the action bar.
const BUTTON_WIDTH: f32 = 160.0;
const CANCEL_WIDTH: f32 = 110.0;

impl BackupsApp {
    /// Configuration problem, validation error and result of the last backup in this session.
    pub(super) fn messages(&mut self, ui: &mut egui::Ui) {
        if let Some(e) = &self.config_error {
            widgets::banner(ui, Tone::Warn, "Your settings are not saved", |ui| {
                ui.add(egui::Label::new(RichText::new(e).color(theme::TEXT)).wrap());
                ui.add(
                    egui::Label::new(
                        RichText::new(
                            "Backups App does not change this file. Fix or delete it, \
                             then open Backups App again.",
                        )
                        .size(12.5)
                        .color(theme::TEXT_WEAK),
                    )
                    .wrap(),
                );
            });
        }
        if let Some(e) = &self.error {
            widgets::banner(ui, Tone::Error, e, |_| {});
        }
        match &self.result {
            Some(Outcome::Done(report)) => {
                if done_banner(ui, report)
                    && let Some(dir) = report.path.parent()
                {
                    open_folder(dir);
                }
            }
            Some(Outcome::Cancelled) => {
                widgets::banner(ui, Tone::Info, "Backup cancelled", |ui| {
                    ui.label(RichText::new("No file was created.").color(theme::TEXT_WEAK));
                });
            }
            Some(Outcome::Failed(msg)) => {
                widgets::banner(ui, Tone::Error, "The backup failed", |ui| {
                    ui.add(egui::Label::new(RichText::new(msg).color(theme::TEXT)).wrap());
                });
            }
            None => {}
        }
    }

    /// Bottom bar: progress or last backup on the left, and the primary button on the right.
    pub(super) fn action_bar(&mut self, ui: &mut egui::Ui) {
        if let Some(task) = &self.task {
            if progress_bar(ui, task.progress.as_ref()) {
                task.cancel();
            }
            return;
        }
        if last_backup_bar(ui, self.config.last_backup.as_ref()) {
            self.start(ui.ctx());
        }
    }
}

/// "Backup complete" banner. Returns `true` if the user asks to open the destination folder.
fn done_banner(ui: &mut egui::Ui, r: &Report) -> bool {
    widgets::banner(ui, Tone::Success, "Backup complete", |ui| {
        theme::lines(ui, |ui| {
            ui.add(
                egui::Label::new(RichText::new(r.path.display().to_string()).color(theme::TEXT))
                    .truncate(),
            );
            ui.label(
                RichText::new(format!(
                    "{} files · {}",
                    format::count(r.files),
                    format::size(r.size)
                ))
                .size(12.5)
                .color(theme::TEXT_WEAK),
            );
        });
        if !r.skipped.is_empty() {
            theme::lines(ui, |ui| {
                warning_title(
                    ui,
                    format::plural(r.skipped.len(), "file skipped", "files skipped"),
                );
                path_list(
                    ui,
                    "skipped",
                    r.skipped
                        .iter()
                        .map(|s| format!("{} — {}", s.path.display(), s.reason)),
                );
            });
        }
        if !r.long_names.is_empty() {
            theme::lines(ui, |ui| {
                warning_title(
                    ui,
                    format::plural(
                        r.long_names.len(),
                        "path too long for Windows Explorer",
                        "paths too long for Windows Explorer",
                    ),
                );
                ui.add(
                    egui::Label::new(
                        RichText::new(
                            "They were copied, but Explorer cannot open the ZIP. \
                         Open it with 7-Zip or shorten these names.",
                        )
                        .size(12.5)
                        .color(theme::TEXT_WEAK),
                    )
                    .wrap(),
                );
                path_list(
                    ui,
                    "long_names",
                    r.long_names.iter().map(|p| p.display().to_string()),
                );
            });
        }
        ui.button("Open destination folder").clicked()
    })
}

fn warning_title(ui: &mut egui::Ui, text: String) {
    ui.label(
        RichText::new(text)
            .family(theme::semibold())
            .color(Tone::Warn.fg()),
    );
}

/// Short scrollable list of paths, one per line.
fn path_list(ui: &mut egui::Ui, id: &str, lines: impl Iterator<Item = String>) {
    // Height for a whole number of lines, so that no line is cut at the bottom edge.
    const VISIBLE_LINES: f32 = 5.0;
    let line = ui.fonts_mut(|f| f.row_height(&egui::FontId::proportional(12.0)));
    let max_height = VISIBLE_LINES * line + (VISIBLE_LINES - 1.0) * theme::TIGHT;
    // Solid bar: the default floating bar is almost invisible until hovered, and the user
    // would not know that the list has more lines.
    ui.spacing_mut().scroll = egui::style::ScrollStyle::solid();
    egui::ScrollArea::vertical()
        .max_height(max_height)
        .id_salt(id)
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = theme::TIGHT;
            for line in lines {
                ui.add(
                    egui::Label::new(RichText::new(line).size(12.0).color(theme::TEXT)).truncate(),
                );
            }
        });
}

/// Progress of the running backup and the "Cancel" button. Returns `true` if the user
/// cancels.
fn progress_bar(ui: &mut egui::Ui, progress: Option<&Progress>) -> bool {
    if let Some(p) = progress {
        let fraction = p.fraction();
        // Row with the height of its text, not of a control.
        ui.spacing_mut().interact_size.y = 0.0;
        egui::Sides::new().show(
            ui,
            |ui| {
                ui.label(
                    RichText::new("Copying…")
                        .family(theme::semibold())
                        .color(theme::TEXT),
                );
            },
            |ui| {
                ui.label(
                    RichText::new(format!("{:.0}%", fraction * 100.0))
                        .family(theme::semibold())
                        .color(theme::ACCENT),
                );
            },
        );
        widgets::progress_bar(ui, fraction);
        ui.spacing_mut().interact_size.y = theme::CONTROL_HEIGHT;
    }
    egui::Sides::new()
        .shrink_left()
        .truncate()
        .show(
            ui,
            |ui| match progress {
                Some(p) => {
                    theme::lines(ui, |ui| {
                        ui.label(
                            RichText::new(format!(
                                "{} of {} files · {} of {}",
                                format::count(p.files_done),
                                format::count(p.total_files),
                                format::size(p.bytes_done),
                                format::size(p.total_bytes)
                            ))
                            .size(12.5)
                            .color(theme::TEXT),
                        );
                        ui.add(
                            egui::Label::new(
                                RichText::new(p.current.display().to_string())
                                    .size(12.0)
                                    .color(theme::TEXT_WEAK),
                            )
                            .truncate(),
                        );
                    });
                }
                None => {
                    ui.add(egui::Spinner::new().color(theme::ACCENT));
                    ui.label(RichText::new("Preparing the backup…").color(theme::TEXT));
                }
            },
            |ui| widgets::secondary_button(ui, "Cancel", CANCEL_WIDTH).clicked(),
        )
        .1
}

/// Date and size of the last backup and the "Back up now" button. Returns `true` if the user
/// starts a backup.
fn last_backup_bar(ui: &mut egui::Ui, last: Option<&LastBackup>) -> bool {
    egui::Sides::new()
        .shrink_left()
        .truncate()
        .show(
            ui,
            |ui| {
                theme::lines(ui, |ui| {
                    ui.label(
                        RichText::new("LAST BACKUP")
                            .size(11.0)
                            .extra_letter_spacing(0.8)
                            .color(theme::TEXT_WEAK),
                    );
                    match last {
                        Some(l) => {
                            ui.label(
                                RichText::new(format!(
                                    "{} · {}",
                                    format::date(l.date),
                                    format::size(l.size)
                                ))
                                .color(theme::TEXT),
                            )
                            .on_hover_text(&l.name);
                        }
                        None => {
                            ui.label(RichText::new("No backups yet").color(theme::TEXT));
                        }
                    }
                });
            },
            |ui| widgets::primary_button(ui, "Back up now", BUTTON_WIDTH).clicked(),
        )
        .1
}

fn open_folder(path: &Path) {
    // Absolute path: with only "explorer", Windows looks first in the folder of the
    // executable, and an `explorer.exe` next to `backups.exe` (for example, in Downloads)
    // would run instead of the real one.
    #[cfg(windows)]
    let program = {
        let root = std::env::var_os("SystemRoot").unwrap_or_else(|| r"C:\Windows".into());
        std::path::PathBuf::from(root).join("explorer.exe")
    };
    #[cfg(not(windows))]
    let program = "xdg-open";
    let _ = std::process::Command::new(program).arg(path).spawn();
}
