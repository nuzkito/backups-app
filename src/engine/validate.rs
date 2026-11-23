use std::fs::OpenOptions;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use anyhow::{Result, bail};

/// Checks that a new file can be created in `dir`. The test file is created with `create_new`,
/// so an existing file (or a symbolic link planted with the same name) is never truncated or
/// deleted: only the file created here is removed.
fn can_write(dir: &Path) -> bool {
    for n in 0..100 {
        let probe = dir.join(format!(".backups-write-test-{}-{n}", std::process::id()));
        match OpenOptions::new().write(true).create_new(true).open(&probe) {
            Ok(file) => {
                drop(file);
                let _ = std::fs::remove_file(&probe);
                return true;
            }
            Err(e) if e.kind() == ErrorKind::AlreadyExists => continue,
            Err(_) => return false,
        }
    }
    false
}

/// Checks the configuration. Returns the canonicalized source and destination paths.
pub fn validate(sources: &[PathBuf], dest: &Path) -> Result<(Vec<PathBuf>, PathBuf)> {
    if sources.is_empty() {
        bail!("Add at least one source folder.");
    }
    let mut canon = Vec::new();
    for s in sources {
        let c = match s.canonicalize() {
            Ok(c) if c.is_dir() => c,
            _ => bail!("The source folder does not exist: {}", s.display()),
        };
        canon.push(c);
    }

    let dest_c = match dest.canonicalize() {
        Ok(c) if c.is_dir() => c,
        _ => bail!("The destination folder does not exist: {}", dest.display()),
    };
    if !can_write(&dest_c) {
        bail!("Cannot write to the destination folder: {}", dest.display());
    }

    for s in &canon {
        if dest_c.starts_with(s) {
            bail!(
                "The destination is inside the source folder {}. The ZIP would include itself.",
                s.display()
            );
        }
    }
    for (i, a) in canon.iter().enumerate() {
        for (j, b) in canon.iter().enumerate() {
            if i != j && a.starts_with(b) {
                bail!(
                    "The folder {} is inside {}. The files would appear twice.",
                    a.display(),
                    b.display()
                );
            }
        }
    }
    Ok((canon, dest_c))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    use crate::test_dir as tmp;

    #[test]
    fn no_sources() {
        let d = tmp("v-nosrc");
        assert!(validate(&[], &d).is_err());
    }

    #[test]
    fn missing_source_and_dest() {
        let d = tmp("v-missing");
        assert!(validate(&[d.join("nope")], &d).is_err());
        let s = tmp("v-missing-src");
        assert!(validate(&[s], &d.join("nope")).is_err());
    }

    #[test]
    fn dest_inside_source() {
        let s = tmp("v-destin");
        let d = s.join("out");
        fs::create_dir_all(&d).unwrap();
        assert!(validate(&[s], &d).is_err());
    }

    #[test]
    fn nested_sources() {
        let s = tmp("v-nested");
        let inner = s.join("inner");
        fs::create_dir_all(&inner).unwrap();
        let d = tmp("v-nested-dest");
        assert!(validate(&[s.clone(), inner], &d).is_err());
        assert!(validate(&[s.clone(), s], &d).is_err());
    }

    #[test]
    fn probe_keeps_existing_files() {
        let s = tmp("v-probe-src");
        let d = tmp("v-probe-dest");
        // A file with the old fixed probe name and one with the name the probe tries first.
        let old = d.join(".backups-write-test");
        let first = d.join(format!(".backups-write-test-{}-0", std::process::id()));
        fs::write(&old, "datos").unwrap();
        fs::write(&first, "datos").unwrap();
        assert!(validate(&[s], &d).is_ok());
        assert_eq!(fs::read_to_string(&old).unwrap(), "datos");
        assert_eq!(fs::read_to_string(&first).unwrap(), "datos");
        assert_eq!(fs::read_dir(&d).unwrap().count(), 2);
    }

    #[cfg(unix)]
    #[test]
    fn probe_does_not_follow_symlinks() {
        let s = tmp("v-link-src");
        let d = tmp("v-link-dest");
        let victim = tmp("v-link-victim").join("importante.txt");
        fs::write(&victim, "datos").unwrap();
        let link = d.join(format!(".backups-write-test-{}-0", std::process::id()));
        std::os::unix::fs::symlink(&victim, &link).unwrap();
        assert!(validate(&[s], &d).is_ok());
        assert_eq!(fs::read_to_string(&victim).unwrap(), "datos");
        assert!(link.symlink_metadata().is_ok());
    }

    #[test]
    fn ok() {
        let a = tmp("v-ok-a");
        let b = tmp("v-ok-b");
        let d = tmp("v-ok-d");
        assert!(validate(&[a, b], &d).is_ok());
    }
}
