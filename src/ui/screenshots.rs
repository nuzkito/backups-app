//! UI screenshots, rendered without opening any window.
//!
//! Each scene sets up a `BackupsApp` state, renders it with egui_kittest and saves a PNG to
//! `screenshots/<scene>.png`. Run with `make screenshots`.
//!
//! To add a new screen, add an entry to `scenes()`.

use std::path::{Path, PathBuf};
use std::time::Instant;

use eframe::egui;
use egui_kittest::Harness;

use super::*;
use crate::engine::{Progress, Report, Skipped};

/// Screenshot scale. 1.0 is the fastest; 2.0 gives sharper images.
const PIXELS_PER_POINT: f32 = 1.0;

struct Scene {
    name: &'static str,
    app: BackupsApp,
}

fn app(config: Config) -> BackupsApp {
    BackupsApp::with_config(config, false)
}

fn configured() -> Config {
    Config {
        sources: vec![
            PathBuf::from(r"C:\Users\Ana\Documents"),
            PathBuf::from(r"C:\Users\Ana\Pictures"),
            PathBuf::from(r"D:\Projects"),
        ],
        destination: Some(PathBuf::from(r"E:\Backups")),
        ignored: vec!["node_modules".into(), "vendor".into(), "*.tmp".into()],
        last_backup: Some(LastBackup {
            name: "backup-2026-11-21-18-30-00.zip".into(),
            date: local_time(2026, 11, 21, 18, 30),
            size: 3_456_789_012,
        }),
    }
}

/// Moment with this date and time in the time zone of the system, so that the screenshot shows
/// the same text on every computer.
fn local_time(year: i16, month: i8, day: i8, hour: i8, minute: i8) -> jiff::Timestamp {
    jiff::civil::date(year, month, day)
        .at(hour, minute, 0, 0)
        .to_zoned_datetime(jiff::tz::TimeZone::system())
        .unwrap()
        .timestamp()
}

fn report() -> Report {
    Report {
        path: PathBuf::from(r"E:\Backups\backup-2026-11-22-10-15-00.zip"),
        started: local_time(2026, 11, 22, 10, 15),
        size: 3_512_000_000,
        files: 12_843,
        skipped: vec![],
        long_names: vec![],
    }
}

fn scenes() -> Vec<Scene> {
    vec![
        Scene {
            name: "01-empty",
            app: app(Config::default()),
        },
        Scene {
            name: "02-configured",
            app: app(configured()),
        },
        Scene {
            name: "03-preparing",
            app: BackupsApp {
                task: Some(Task::idle(None)),
                ..app(configured())
            },
        },
        Scene {
            name: "04-copying",
            app: BackupsApp {
                task: Some(Task::idle(Some(Progress {
                    current: r"C:\Users\Ana\Pictures\Vacation\IMG_2041.jpg".into(),
                    files_done: 5_120,
                    total_files: 12_843,
                    bytes_done: 1_450_000_000,
                    total_bytes: 3_512_000_000,
                }))),
                ..app(configured())
            },
        },
        Scene {
            name: "05-done",
            app: BackupsApp {
                result: Some(Outcome::Done(report())),
                ..app(configured())
            },
        },
        Scene {
            name: "06-done-with-skipped",
            app: BackupsApp {
                result: Some(Outcome::Done(Report {
                    skipped: (1..=6)
                        .map(|i| Skipped {
                            path: PathBuf::from(format!(r"D:\Projects\locked-{i}.db")),
                            reason: "The file is in use by another process".into(),
                        })
                        .collect(),
                    ..report()
                })),
                ..app(configured())
            },
        },
        Scene {
            name: "06-done-with-long-names",
            app: BackupsApp {
                result: Some(Outcome::Done(Report {
                    long_names: vec![PathBuf::from(
                        r"C:\Users\Ana\Documents\Obsidian\Projects\Blog\Drafts\Screenshot 2025-03-19 at 14-37-54 a screenshot with a very long name that exceeds the limit of Windows Explorer.png",
                    )],
                    ..report()
                })),
                ..app(configured())
            },
        },
        Scene {
            name: "07-cancelled",
            app: BackupsApp {
                result: Some(Outcome::Cancelled),
                ..app(configured())
            },
        },
        Scene {
            name: "08-failed",
            app: BackupsApp {
                result: Some(Outcome::Failed(
                    "There is no space left on the destination disk.".into(),
                )),
                ..app(configured())
            },
        },
        Scene {
            name: "09-validation-error",
            app: BackupsApp {
                error: Some("Choose a destination folder.".into()),
                ..app(Config {
                    destination: None,
                    ..configured()
                })
            },
        },
        Scene {
            name: "10-invalid-config",
            app: BackupsApp {
                config_error: Some(
                    r"C:\Apps\Backups\config.toml is not valid: line 3: invalid type: integer `3`, expected a sequence"
                        .into(),
                ),
                ..app(Config::default())
            },
        },
    ]
}

/// Generates the screenshots. Filter by name with `SCREENSHOTS=copying,failed`.
#[test]
#[ignore = "generates screenshots: make screenshots"]
fn screenshots() {
    let start = Instant::now();
    let out_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("screenshots");
    std::fs::create_dir_all(&out_dir).unwrap();

    let filter: Vec<String> = std::env::var("SCREENSHOTS")
        .map(|s| s.split(',').map(|f| f.trim().to_owned()).collect())
        .unwrap_or_default();

    for scene in scenes() {
        if !filter.is_empty() && !filter.iter().any(|f| scene.name.contains(f.as_str())) {
            continue;
        }
        let mut harness = Harness::builder()
            .with_size(egui::Vec2::from(WINDOW_SIZE))
            .with_pixels_per_point(PIXELS_PER_POINT)
            .wgpu()
            .build_eframe(|cc| {
                apply_theme(&cc.egui_ctx);
                scene.app
            });
        // The spinner requests repaints continuously, so run a fixed number of steps
        // (enough for egui to compute the sizes) instead of waiting for it to settle.
        harness.run_steps(3);
        let image = harness.render().expect("cannot render the scene");
        let path = out_dir.join(format!("{}.png", scene.name));
        image.save(&path).unwrap();
        println!("{}", path.display());
    }
    println!("Screenshots generated in {:.2?}", start.elapsed());
}
