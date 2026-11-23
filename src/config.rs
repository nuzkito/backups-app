//! Application configuration, stored as TOML next to the executable (portable application).

use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Example of the file:
///
/// ```toml
/// sources = ['C:\Users\Ana\Documents']
/// destination = 'E:\Backups'
/// ignored = [
///     "node_modules",
///     "*.tmp",
/// ]
///
/// [last_backup]
/// name = "backup-2026-11-21-18-30-00.zip"
/// date = "2026-11-21T17:30:00Z"
/// size = 3456789012
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub sources: Vec<PathBuf>,
    pub destination: Option<PathBuf>,
    /// Exclusion patterns (see `engine::Ignore`).
    pub ignored: Vec<String>,
    pub last_backup: Option<LastBackup>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LastBackup {
    /// File name of the ZIP, without the folder.
    pub name: String,
    /// Start of the backup, the same time as in the file name.
    pub date: jiff::Timestamp,
    /// Size of the ZIP, in bytes.
    pub size: u64,
}

fn config_path() -> Result<PathBuf> {
    let exe = std::env::current_exe().context("Cannot find the executable")?;
    let dir = exe
        .parent()
        .context("Cannot find the folder of the executable")?;
    Ok(dir.join("config.toml"))
}

/// One-line description of a parse error: "line 3: invalid string". The `Display` of the
/// error has several lines, with a copy of the text, which does not fit in a notice.
fn parse_error(text: &str, err: &toml::de::Error) -> String {
    let message = err.message().trim_end_matches('\n');
    match err.span() {
        Some(span) => {
            let line = text[..span.start.min(text.len())].matches('\n').count() + 1;
            format!("line {line}: {message}")
        }
        None => message.to_owned(),
    }
}

impl Config {
    /// Loads the configuration. A missing file gives an empty configuration. A file that cannot
    /// be read or parsed is an error, so that the caller does not replace it with an empty one.
    pub fn load() -> Result<Self> {
        Self::load_from(&config_path()?)
    }

    fn load_from(path: &Path) -> Result<Self> {
        let text = match std::fs::read_to_string(path) {
            Ok(text) => text,
            Err(e) if e.kind() == ErrorKind::NotFound => return Ok(Self::default()),
            Err(e) => return Err(e).with_context(|| format!("Cannot read {}", path.display())),
        };
        toml::from_str(&text).map_err(|e| {
            anyhow::anyhow!(
                "{} is not valid: {}",
                path.display(),
                parse_error(&text, &e)
            )
        })
    }

    pub fn save(&self) -> Result<()> {
        self.save_to(&config_path()?)
    }

    /// Writes a temporary file and renames it over `path`: if the write stops halfway (for
    /// example, a full disk), the previous configuration stays complete.
    fn save_to(&self, path: &Path) -> Result<()> {
        let tmp = path.with_extension("toml.tmp");
        let write_error = || format!("Cannot write {}", path.display());
        std::fs::write(&tmp, toml::to_string_pretty(self)?).with_context(write_error)?;
        std::fs::rename(&tmp, path).with_context(write_error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let c = Config {
            sources: vec![PathBuf::from("/a"), PathBuf::from("/b")],
            destination: Some(PathBuf::from("/d")),
            ignored: vec!["node_modules".into(), "*.tmp".into()],
            last_backup: Some(LastBackup {
                name: "x.zip".into(),
                date: jiff::Timestamp::from_second(1_800_000_000).unwrap(),
                size: 5,
            }),
        };
        let s = toml::to_string_pretty(&c).unwrap();
        let back: Config = toml::from_str(&s).unwrap();
        assert_eq!(back.sources, c.sources);
        assert_eq!(back.destination, c.destination);
        assert_eq!(back.ignored, c.ignored);
        let last = back.last_backup.unwrap();
        assert_eq!(last.date, c.last_backup.unwrap().date);
        assert_eq!(last.size, 5);
    }

    #[test]
    fn empty_file_is_default() {
        let c: Config = toml::from_str("").unwrap();
        assert!(c.sources.is_empty());
    }

    #[test]
    fn missing_file_is_default() {
        let d = crate::test_dir("config-missing");
        assert!(Config::load_from(&d.join("config.toml")).is_ok());
    }

    #[test]
    fn invalid_file_is_an_error() {
        let d = crate::test_dir("config-invalid");
        let path = d.join("config.toml");
        std::fs::write(&path, "ignored = []\nsources = 3\n").unwrap();
        let err = Config::load_from(&path).unwrap_err().to_string();
        assert!(err.contains("is not valid: line 2: "), "{err}");
        assert!(!err.contains('\n'), "{err}");
    }

    #[test]
    fn save_replaces_the_file() {
        let d = crate::test_dir("config-save");
        let path = d.join("config.toml");
        std::fs::write(&path, "old").unwrap();
        let c = Config {
            ignored: vec!["*.tmp".into()],
            ..Config::default()
        };
        c.save_to(&path).unwrap();
        assert_eq!(Config::load_from(&path).unwrap().ignored, ["*.tmp"]);
        assert_eq!(std::fs::read_dir(&d).unwrap().count(), 1);
    }
}
