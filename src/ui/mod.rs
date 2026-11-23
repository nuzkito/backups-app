mod format;
#[cfg(test)]
mod icon;
mod logo;
#[cfg(test)]
mod screenshots;
mod sections;
mod status;
mod task;
mod theme;
mod widgets;

use std::time::Duration;

use eframe::egui;

use crate::config::{Config, LastBackup};
use crate::engine::{self, Outcome, Report};
use task::Task;

pub use theme::apply as apply_theme;

/// Initial window size, in points.
pub const WINDOW_SIZE: [f32; 2] = [600.0, 820.0];

pub struct BackupsApp {
    config: Config,
    task: Option<Task>,
    result: Option<Outcome>,
    error: Option<String>,
    /// Why `config.toml` could not be loaded. While it is set, the configuration is not saved,
    /// so that the user's file is not replaced.
    config_error: Option<String>,
    /// If `false`, configuration changes are not written to disk (screenshots).
    persist: bool,
    ignore_input: String,
}

impl BackupsApp {
    pub fn new() -> Self {
        match Config::load() {
            Ok(config) => Self::with_config(config, true),
            Err(e) => Self {
                config_error: Some(format!("{e:#}")),
                ..Self::with_config(Config::default(), false)
            },
        }
    }

    fn with_config(config: Config, persist: bool) -> Self {
        Self {
            config,
            task: None,
            result: None,
            error: None,
            config_error: None,
            persist,
            ignore_input: String::new(),
        }
    }

    fn save(&mut self) {
        if !self.persist || self.config_error.is_some() {
            return;
        }
        if let Err(e) = self.config.save() {
            self.error = Some(format!("{e:#}"));
        }
    }

    fn start(&mut self, ctx: &egui::Context) {
        let Some(dest) = self.config.destination.clone() else {
            self.error = Some("Choose a destination folder.".into());
            return;
        };
        // Validate before spawning the thread to report errors immediately.
        if let Err(e) = engine::validate(&self.config.sources, &dest) {
            self.error = Some(format!("{e:#}"));
            return;
        }
        self.error = None;
        self.result = None;
        self.task = Some(Task::spawn(
            ctx,
            self.config.sources.clone(),
            dest,
            engine::Ignore::new(&self.config.ignored),
        ));
    }

    fn poll(&mut self) {
        let Some(outcome) = self.task.as_mut().and_then(Task::poll) else {
            return;
        };
        self.task = None;
        if let Outcome::Done(report) = &outcome {
            self.config.last_backup = Some(last_backup(report));
            self.save();
        }
        self.result = Some(outcome);
    }
}

fn last_backup(report: &Report) -> LastBackup {
    LastBackup {
        name: report
            .path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default(),
        date: report.started,
        size: report.size,
    }
}

impl eframe::App for BackupsApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.poll();
        let busy = self.task.is_some();
        if busy {
            ui.ctx().request_repaint_after(Duration::from_millis(100));
        }

        egui::Panel::bottom("actions")
            .resizable(false)
            .frame(
                egui::Frame::new()
                    .fill(theme::CARD)
                    .stroke(egui::Stroke::new(1.0, theme::BORDER))
                    .inner_margin(egui::Margin::symmetric(
                        theme::GUTTER as i8,
                        theme::PAD as i8,
                    )),
            )
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = theme::PAD;
                self.messages(ui);
                self.action_bar(ui);
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(theme::BG))
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink(false)
                    .show(ui, |ui| {
                        egui::Frame::new()
                            .inner_margin(theme::margin(theme::GUTTER))
                            .show(ui, |ui| {
                                ui.spacing_mut().item_spacing.y = theme::PAD;
                                self.header(ui);
                                self.sources_section(ui, busy);
                                self.ignored_section(ui, busy);
                                self.destination_section(ui, busy);
                            });
                    });
            });
    }
}
