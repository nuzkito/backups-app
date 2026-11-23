//! Walks the source folders and computes the name of each entry inside the ZIP.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use super::Skipped;
use super::ignore::Ignore;

#[derive(Debug)]
pub enum Kind {
    Dir,
    File,
}

#[derive(Debug)]
pub struct Entry {
    pub path: PathBuf,
    /// Name inside the ZIP, with `/` as the separator.
    pub name: String,
    pub kind: Kind,
    /// `None` if the system does not give the modification time.
    pub modified: Option<SystemTime>,
}

#[derive(Debug, Default)]
pub struct Scan {
    pub entries: Vec<Entry>,
    pub skipped: Vec<Skipped>,
    pub total_files: u64,
    pub total_bytes: u64,
}

/// Unique root names: `Fotos`, `Fotos (2)`.
fn root_names(sources: &[PathBuf]) -> Vec<String> {
    let mut used: Vec<String> = Vec::new();
    for s in sources {
        let base = s
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "root".to_string());
        let mut name = base.clone();
        let mut n = 2;
        while used.contains(&name) {
            name = format!("{base} ({n})");
            n += 1;
        }
        used.push(name);
    }
    used
}

fn skip(out: &mut Scan, path: &Path, err: std::io::Error) {
    out.skipped.push(Skipped {
        path: path.to_path_buf(),
        reason: err.to_string(),
    });
}

/// Adds `path` (a folder or a file, with the metadata `meta`) and, for a folder, all its
/// contents: each folder before its contents, in name order. Symbolic links and other special
/// file types are ignored (links are not followed). Uses a stack, not recursion: a very deep
/// tree cannot overflow the stack of the thread.
///
/// The metadata of the contents comes from the folder listing (`DirEntry::metadata`), which on
/// Windows does not need a separate system call for each file.
fn walk(path: &Path, name: String, meta: fs::Metadata, ignore: &Ignore, out: &mut Scan) {
    let mut pending = vec![(path.to_path_buf(), name, meta)];
    while let Some((path, name, meta)) = pending.pop() {
        let modified = meta.modified().ok();
        if meta.is_file() {
            out.total_files += 1;
            out.total_bytes += meta.len();
            out.entries.push(Entry {
                path,
                name,
                kind: Kind::File,
                modified,
            });
            continue;
        }
        if !meta.is_dir() {
            continue;
        }
        let items = fs::read_dir(&path);
        out.entries.push(Entry {
            path: path.clone(),
            name: name.clone(),
            kind: Kind::Dir,
            modified,
        });
        let items = match items {
            Ok(items) => items,
            Err(e) => {
                skip(out, &path, e);
                continue;
            }
        };
        let mut children = Vec::new();
        for item in items {
            match item.and_then(|i| Ok((i.metadata()?, i))) {
                Ok((meta, i)) if !ignore.matches(&i.file_name()) => children.push((i, meta)),
                Ok(_) => {}
                Err(e) => skip(out, &path, e),
            }
        }
        // Reverse order: the stack gives the first name first.
        children.sort_by_key(|(i, _)| std::cmp::Reverse(i.file_name()));
        for (item, meta) in children {
            let child = format!("{name}/{}", item.file_name().to_string_lossy());
            pending.push((item.path(), child, meta));
        }
    }
}

/// Walks `sources`. Entries that match `ignore` are skipped (folders entirely). A source folder
/// is never ignored, even if its name matches, and a source that is a link is followed.
pub fn scan(sources: &[PathBuf], ignore: &Ignore) -> Scan {
    let mut out = Scan::default();
    for (src, root) in sources.iter().zip(root_names(sources)) {
        match fs::metadata(src) {
            Ok(m) => walk(src, root, m, ignore, &mut out),
            Err(e) => skip(&mut out, src, e),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_dir;
    use std::fs;

    #[test]
    fn names_and_duplicates() {
        let base = test_dir("scan");
        let a = base.join("x/Fotos");
        let b = base.join("y/Fotos");
        fs::create_dir_all(a.join("sub")).unwrap();
        fs::create_dir_all(&b).unwrap();
        fs::write(a.join("sub/f.txt"), b"hola").unwrap();
        fs::write(b.join("g.txt"), b"ab").unwrap();

        let s = scan(&[a, b], &Ignore::default());
        let names: Vec<_> = s.entries.iter().map(|e| e.name.as_str()).collect();
        assert!(names.contains(&"Fotos"));
        assert!(names.contains(&"Fotos/sub/f.txt"));
        assert!(names.contains(&"Fotos (2)"));
        assert!(names.contains(&"Fotos (2)/g.txt"));
        assert_eq!(s.total_files, 2);
        assert_eq!(s.total_bytes, 6);
    }

    #[test]
    fn ignored_entries() {
        let base = test_dir("scan-ignore");
        let src = base.join("vendor");
        fs::create_dir_all(src.join("node_modules/pkg")).unwrap();
        fs::create_dir_all(src.join("app")).unwrap();
        fs::write(src.join("node_modules/pkg/index.js"), b"x").unwrap();
        fs::write(src.join("app/main.js"), b"xy").unwrap();
        fs::write(src.join("app/borrar.tmp"), b"xyz").unwrap();

        // The source is named `vendor`, but a source is never ignored.
        let s = scan(&[src], &Ignore::new(&["node_modules", "vendor", "*.tmp"]));
        let names: Vec<_> = s.entries.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, ["vendor", "vendor/app", "vendor/app/main.js"]);
        assert_eq!(s.total_files, 1);
        assert_eq!(s.total_bytes, 2);
        assert!(s.skipped.is_empty());
    }

    #[test]
    fn folder_before_contents_in_name_order() {
        let base = test_dir("scan-order");
        let src = base.join("src");
        fs::create_dir_all(src.join("b/d")).unwrap();
        fs::write(src.join("c.txt"), b"").unwrap();
        fs::write(src.join("a.txt"), b"").unwrap();
        fs::write(src.join("b/d/e.txt"), b"").unwrap();
        fs::write(src.join("b/z.txt"), b"").unwrap();

        let s = scan(&[src], &Ignore::default());
        let names: Vec<_> = s.entries.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(
            names,
            [
                "src",
                "src/a.txt",
                "src/b",
                "src/b/d",
                "src/b/d/e.txt",
                "src/b/z.txt",
                "src/c.txt"
            ]
        );
    }

    #[cfg(unix)]
    #[test]
    fn links_are_not_followed() {
        let base = test_dir("scan-links");
        let src = base.join("src");
        let outside = base.join("outside");
        fs::create_dir_all(&src).unwrap();
        fs::create_dir_all(&outside).unwrap();
        fs::write(outside.join("f.txt"), b"x").unwrap();
        std::os::unix::fs::symlink(&outside, src.join("dir-link")).unwrap();
        std::os::unix::fs::symlink(outside.join("f.txt"), src.join("file-link")).unwrap();

        let s = scan(&[src], &Ignore::default());
        let names: Vec<_> = s.entries.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, ["src"]);
        assert_eq!(s.total_files, 0);
    }
}
