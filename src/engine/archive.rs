use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::SystemTime;

use anyhow::{Context, Result};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, DateTime, ZipWriter};

use super::scan::{Kind, Scan};
use super::{Cancelled, Progress, Skipped};

/// Extensions of formats that are already compressed.
const STORED_EXTENSIONS: &[&str] = &[
    "jpg", "jpeg", "png", "gif", "webp", "heic", "mp4", "mkv", "avi", "mov", "webm", "mp3", "aac",
    "ogg", "flac", "m4a", "zip", "7z", "rar", "gz", "bz2", "xz", "zst", "docx", "xlsx", "pptx",
    "odt", "ods", "odp", "pdf", "jar", "apk",
];

fn method_for(path: &Path) -> CompressionMethod {
    let stored = path
        .extension()
        .map(|e| e.to_string_lossy().to_ascii_lowercase())
        .is_some_and(|e| STORED_EXTENSIONS.contains(&e.as_str()));
    if stored {
        CompressionMethod::Stored
    } else {
        CompressionMethod::Deflated
    }
}

fn zip_time(modified: SystemTime) -> Option<DateTime> {
    let z = jiff::ZonedDateTime::try_from(modified).ok()?;
    DateTime::from_date_and_time(
        u16::try_from(z.year()).ok()?,
        z.month() as u8,
        z.day() as u8,
        z.hour() as u8,
        z.minute() as u8,
        z.second() as u8,
    )
    .ok()
}

fn options(modified: Option<SystemTime>, method: CompressionMethod) -> SimpleFileOptions {
    let mut o = SimpleFileOptions::default()
        .compression_method(method)
        .large_file(true);
    if let Some(t) = modified.and_then(zip_time) {
        o = o.last_modified_time(t);
    }
    o
}

/// Windows Explorer cannot open a ZIP with an entry name of this length (in UTF-8 bytes) or
/// more. Other programs (7-Zip, WinRAR) can open it.
const EXPLORER_MAX_NAME: usize = 260;

#[derive(Debug, Default)]
pub struct Written {
    pub skipped: Vec<Skipped>,
    pub files: u64,
    /// Entries in the ZIP with a name too long for Windows Explorer.
    pub long_names: Vec<PathBuf>,
}

fn too_long_for_explorer(name: &str) -> bool {
    name.len() >= EXPLORER_MAX_NAME
}

pub fn write_zip(
    file: File,
    scan: &Scan,
    cancel: &AtomicBool,
    mut notify: impl FnMut(Progress),
) -> Result<Written> {
    let mut zip = ZipWriter::new(file);
    let mut skipped = Vec::new();
    let mut long_names = Vec::new();
    let mut files_done = 0u64;
    let mut bytes_done = 0u64;
    let mut buf = vec![0u8; 256 * 1024];

    for entry in &scan.entries {
        if cancel.load(Ordering::Relaxed) {
            return Err(Cancelled.into());
        }
        let write_error = || format!("Cannot write {}", entry.name);
        match entry.kind {
            Kind::Dir => {
                zip.add_directory(
                    entry.name.as_str(),
                    options(entry.modified, CompressionMethod::Stored),
                )
                .with_context(write_error)?;
            }
            Kind::File => {
                notify(Progress {
                    current: entry.path.clone(),
                    files_done,
                    bytes_done,
                    total_files: scan.total_files,
                    total_bytes: scan.total_bytes,
                });
                let mut src = match File::open(&entry.path) {
                    Ok(f) => f,
                    Err(e) => {
                        skipped.push(Skipped {
                            path: entry.path.clone(),
                            reason: e.to_string(),
                        });
                        continue;
                    }
                };
                zip.start_file(
                    entry.name.as_str(),
                    options(entry.modified, method_for(&entry.path)),
                )
                .with_context(write_error)?;
                loop {
                    if cancel.load(Ordering::Relaxed) {
                        return Err(Cancelled.into());
                    }
                    let n = src
                        .read(&mut buf)
                        .with_context(|| format!("Cannot read {}", entry.path.display()))?;
                    if n == 0 {
                        break;
                    }
                    zip.write_all(&buf[..n]).with_context(write_error)?;
                    bytes_done += n as u64;
                }
                files_done += 1;
            }
        }
        if too_long_for_explorer(&entry.name) {
            long_names.push(entry.path.clone());
        }
    }
    let file = zip.finish().context("Cannot finish the ZIP file")?;
    // Write the data to disk before the caller renames the file: a power failure must not
    // leave a damaged ZIP with the final name.
    file.sync_all()
        .context("Cannot save the ZIP file to the disk")?;
    Ok(Written {
        skipped,
        files: files_done,
        long_names,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn methods() {
        assert_eq!(method_for(Path::new("a/b.JPG")), CompressionMethod::Stored);
        assert_eq!(method_for(Path::new("a/b.pdf")), CompressionMethod::Stored);
        assert_eq!(
            method_for(Path::new("a/b.txt")),
            CompressionMethod::Deflated
        );
        assert_eq!(
            method_for(Path::new("a/noext")),
            CompressionMethod::Deflated
        );
    }

    #[test]
    fn explorer_name_limit() {
        assert!(!too_long_for_explorer(&"x".repeat(259)));
        assert!(too_long_for_explorer(&"x".repeat(260)));
        // The limit is in bytes: "ó" uses 2 bytes in UTF-8.
        assert!(too_long_for_explorer(&"ó".repeat(130)));
    }
}
