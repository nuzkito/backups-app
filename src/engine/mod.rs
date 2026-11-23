//! Backup engine. Does not depend on egui or Windows.

mod archive;
mod ignore;
mod scan;
mod validate;

use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

use anyhow::{Context, Result, bail};

pub use ignore::Ignore;
pub use validate::validate;

#[derive(Debug, Clone)]
pub struct Skipped {
    pub path: PathBuf,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct Report {
    pub path: PathBuf,
    /// Start of the backup. The file name has this time.
    pub started: jiff::Timestamp,
    pub size: u64,
    pub files: u64,
    pub skipped: Vec<Skipped>,
    /// Entries copied with a name that is too long for Windows Explorer. The ZIP is valid, but
    /// Explorer cannot open it.
    pub long_names: Vec<PathBuf>,
}

/// Backup progress, sent before each file is copied.
#[derive(Debug, Clone)]
pub struct Progress {
    pub current: PathBuf,
    pub files_done: u64,
    pub bytes_done: u64,
    pub total_files: u64,
    pub total_bytes: u64,
}

impl Progress {
    /// Copied fraction of the total bytes, from 0.0 to 1.0.
    pub fn fraction(&self) -> f32 {
        if self.total_bytes == 0 {
            return 0.0;
        }
        (self.bytes_done as f32 / self.total_bytes as f32).clamp(0.0, 1.0)
    }
}

#[derive(Debug)]
pub struct Cancelled;

impl fmt::Display for Cancelled {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Backup cancelled")
    }
}

impl std::error::Error for Cancelled {}

#[derive(Debug)]
pub enum Outcome {
    Done(Report),
    Cancelled,
    Failed(String),
}

fn backup_file_name(time: &jiff::ZonedDateTime) -> String {
    format!("backup-{}.zip", time.strftime("%Y-%m-%d-%H-%M-%S"))
}

/// `true` if `path` exists, also if it is a symbolic link (valid or broken).
fn exists(path: &Path) -> bool {
    path.symlink_metadata().is_ok()
}

/// Name with a suffix for the attempt `n`: `backup-X.zip`, `backup-X-2.zip`, `backup-X-3.zip`.
fn numbered(name: &str, n: u32) -> String {
    if n == 1 {
        return name.to_string();
    }
    match name.strip_suffix(".zip") {
        Some(stem) => format!("{stem}-{n}.zip"),
        None => format!("{name}-{n}"),
    }
}

/// Chooses a final name that does not exist yet and creates its temporary file with
/// `create_new`. An existing file (or a symbolic link with the same name) is never opened,
/// truncated or replaced. Returns the final path, the temporary path and the open file.
fn create_tmp(dest: &Path, name: &str) -> Result<(PathBuf, PathBuf, File)> {
    for n in 1..=100 {
        let name = numbered(name, n);
        let final_path = dest.join(&name);
        let tmp_path = dest.join(format!("{name}.tmp"));
        if exists(&final_path) {
            continue;
        }
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&tmp_path)
        {
            Ok(file) => return Ok((final_path, tmp_path, file)),
            Err(e) if e.kind() == ErrorKind::AlreadyExists => continue,
            Err(e) => {
                return Err(e).with_context(|| format!("Cannot create {}", tmp_path.display()));
            }
        }
    }
    bail!(
        "Cannot find a free name for the backup in {}",
        dest.display()
    )
}

fn describe(err: &anyhow::Error) -> String {
    let full = err.chain().any(|c| {
        c.downcast_ref::<std::io::Error>()
            .is_some_and(|e| e.kind() == ErrorKind::StorageFull)
    });
    if full {
        "The destination disk is full.".to_string()
    } else {
        format!("{err:#}")
    }
}

/// Runs a full backup. Blocks until it finishes: call it from a thread.
pub fn run(
    sources: &[PathBuf],
    dest: &Path,
    ignore: &Ignore,
    cancel: &AtomicBool,
    notify: impl FnMut(Progress),
) -> Outcome {
    match run_inner(sources, dest, ignore, cancel, notify) {
        Ok(r) => Outcome::Done(r),
        Err(e) if e.downcast_ref::<Cancelled>().is_some() => Outcome::Cancelled,
        Err(e) => Outcome::Failed(describe(&e)),
    }
}

fn run_inner(
    sources: &[PathBuf],
    dest: &Path,
    ignore: &Ignore,
    cancel: &AtomicBool,
    notify: impl FnMut(Progress),
) -> Result<Report> {
    let (sources, dest) = validate(sources, dest)?;
    let scan = scan::scan(&sources, ignore);

    let started = jiff::ZonedDateTime::now();
    let (final_path, tmp_path, file) = create_tmp(&dest, &backup_file_name(&started))?;

    // From here on, `tmp_path` is a file created by this run: remove it if anything fails.
    let result = write_backup(
        file,
        scan,
        &tmp_path,
        final_path,
        started.timestamp(),
        cancel,
        notify,
    );
    if result.is_err() {
        let _ = std::fs::remove_file(&tmp_path);
    }
    result
}

fn write_backup(
    file: File,
    scan: scan::Scan,
    tmp_path: &Path,
    final_path: PathBuf,
    started: jiff::Timestamp,
    cancel: &AtomicBool,
    notify: impl FnMut(Progress),
) -> Result<Report> {
    let written = archive::write_zip(file, &scan, cancel, notify)?;
    // `rename` replaces the destination, so check again that nobody created it meanwhile.
    if exists(&final_path) {
        bail!("{} already exists", final_path.display());
    }
    std::fs::rename(tmp_path, &final_path)
        .with_context(|| format!("Cannot create {}", final_path.display()))?;
    let size = std::fs::metadata(&final_path).map(|m| m.len()).unwrap_or(0);
    let mut skipped = scan.skipped;
    skipped.extend(written.skipped);
    Ok(Report {
        path: final_path,
        started,
        size,
        files: written.files,
        skipped,
        long_names: written.long_names,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Read;
    use std::sync::atomic::Ordering;

    use crate::test_dir as tmp;

    #[test]
    fn progress_fraction() {
        let p = |done, total| Progress {
            current: PathBuf::new(),
            files_done: 0,
            bytes_done: done,
            total_files: 0,
            total_bytes: total,
        };
        assert_eq!(p(0, 0).fraction(), 0.0);
        assert_eq!(p(50, 200).fraction(), 0.25);
        assert_eq!(p(300, 200).fraction(), 1.0);
    }

    #[test]
    fn file_name_format() {
        let time = jiff::civil::date(2026, 1, 2)
            .at(3, 4, 5, 0)
            .to_zoned_datetime(jiff::tz::TimeZone::UTC)
            .unwrap();
        assert_eq!(backup_file_name(&time), "backup-2026-01-02-03-04-05.zip");
    }

    #[test]
    fn full_backup_roundtrip() {
        let s1 = tmp("e-s1/Docs");
        let s2 = tmp("e-s2/Docs");
        let d = tmp("e-dest");
        fs::create_dir_all(s1.join("sub")).unwrap();
        fs::write(s1.join("sub/a.txt"), "hola mundo").unwrap();
        fs::write(s1.join("foto.jpg"), [1u8, 2, 3]).unwrap();
        fs::write(s2.join("b.txt"), "adios").unwrap();

        let cancel = AtomicBool::new(false);
        let mut events = 0;
        let out = run(&[s1, s2], &d, &Ignore::default(), &cancel, |_| events += 1);
        let Outcome::Done(report) = out else {
            panic!("{out:?}")
        };
        let name = report.path.file_name().unwrap().to_string_lossy();
        let started = report
            .started
            .to_zoned_datetime(jiff::tz::TimeZone::system());
        assert_eq!(name, backup_file_name(&started));
        assert_eq!(report.files, 3);
        assert!(report.skipped.is_empty());
        assert!(report.long_names.is_empty());
        assert!(events >= 3);

        let leftovers: Vec<_> = fs::read_dir(&d)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(leftovers.len(), 1, "{leftovers:?}");
        assert!(leftovers[0].ends_with(".zip"));

        let mut z = zip::ZipArchive::new(File::open(&report.path).unwrap()).unwrap();
        let mut text = String::new();
        z.by_name("Docs/sub/a.txt")
            .unwrap()
            .read_to_string(&mut text)
            .unwrap();
        assert_eq!(text, "hola mundo");
        assert_eq!(
            z.by_name("Docs/foto.jpg").unwrap().compression(),
            zip::CompressionMethod::Stored
        );
        assert_eq!(
            z.by_name("Docs/sub/a.txt").unwrap().compression(),
            zip::CompressionMethod::Deflated
        );
        let year = z
            .by_name("Docs/sub/a.txt")
            .unwrap()
            .last_modified()
            .unwrap()
            .year();
        assert!(year >= 2026, "{year}");
        text.clear();
        z.by_name("Docs (2)/b.txt")
            .unwrap()
            .read_to_string(&mut text)
            .unwrap();
        assert_eq!(text, "adios");
    }

    #[test]
    fn long_names_are_copied_and_reported() {
        let s = tmp("l-src");
        let d = tmp("l-dest");
        let long = format!("{}.txt", "x".repeat(250));
        fs::write(s.join(&long), "largo").unwrap();
        fs::write(s.join("corto.txt"), "corto").unwrap();
        let cancel = AtomicBool::new(false);
        let Outcome::Done(report) = run(
            std::slice::from_ref(&s),
            &d,
            &Ignore::default(),
            &cancel,
            |_| {},
        ) else {
            panic!()
        };
        assert_eq!(report.files, 2);
        assert_eq!(report.long_names, [s.join(&long)]);
        let mut z = zip::ZipArchive::new(File::open(&report.path).unwrap()).unwrap();
        let root = s.file_name().unwrap().to_string_lossy();
        assert!(z.by_name(&format!("{root}/{long}")).is_ok());
    }

    #[test]
    fn numbered_names() {
        assert_eq!(numbered("backup-X.zip", 1), "backup-X.zip");
        assert_eq!(numbered("backup-X.zip", 2), "backup-X-2.zip");
        assert_eq!(numbered("x", 3), "x-3");
    }

    #[test]
    fn create_tmp_keeps_existing_files() {
        let d = tmp("t-existing");
        // The final name of attempt 1 and the temporary name of attempt 2 already exist.
        fs::write(d.join("b.zip"), "zip anterior").unwrap();
        fs::write(d.join("b-2.zip.tmp"), "otro").unwrap();
        let (final_path, tmp_path, _file) = create_tmp(&d, "b.zip").unwrap();
        assert_eq!(final_path, d.join("b-3.zip"));
        assert_eq!(tmp_path, d.join("b-3.zip.tmp"));
        assert_eq!(fs::read_to_string(d.join("b.zip")).unwrap(), "zip anterior");
        assert_eq!(fs::read_to_string(d.join("b-2.zip.tmp")).unwrap(), "otro");
    }

    #[cfg(unix)]
    #[test]
    fn create_tmp_does_not_follow_symlinks() {
        let d = tmp("t-link");
        let victim = tmp("t-link-victim").join("importante.txt");
        fs::write(&victim, "datos").unwrap();
        std::os::unix::fs::symlink(&victim, d.join("b.zip.tmp")).unwrap();
        let (final_path, _, _file) = create_tmp(&d, "b.zip").unwrap();
        assert_eq!(final_path, d.join("b-2.zip"));
        assert_eq!(fs::read_to_string(&victim).unwrap(), "datos");
    }

    #[test]
    fn failure_keeps_foreign_tmp() {
        // A backup that fails must only remove its own temporary file.
        let s = tmp("f-src");
        let d = tmp("f-dest");
        fs::write(s.join("a.txt"), "x").unwrap();
        let foreign = d.join("ajeno.zip.tmp");
        fs::write(&foreign, "otro").unwrap();
        let cancel = AtomicBool::new(true);
        assert!(matches!(
            run(&[s], &d, &Ignore::default(), &cancel, |_| {}),
            Outcome::Cancelled
        ));
        assert_eq!(fs::read_to_string(&foreign).unwrap(), "otro");
        assert_eq!(fs::read_dir(&d).unwrap().count(), 1);
    }

    #[test]
    fn cancel_removes_tmp() {
        let s = tmp("c-src");
        let d = tmp("c-dest");
        fs::write(s.join("a.txt"), "x").unwrap();
        let cancel = AtomicBool::new(true);
        cancel.store(true, Ordering::Relaxed);
        assert!(matches!(
            run(&[s], &d, &Ignore::default(), &cancel, |_| {}),
            Outcome::Cancelled
        ));
        assert_eq!(fs::read_dir(&d).unwrap().count(), 0);
    }

    #[test]
    fn invalid_config_fails() {
        let d = tmp("i-dest");
        let cancel = AtomicBool::new(false);
        assert!(matches!(
            run(&[], &d, &Ignore::default(), &cancel, |_| {}),
            Outcome::Failed(_)
        ));
    }
}
