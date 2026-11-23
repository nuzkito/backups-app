use std::path::Path;

use jiff::Timestamp;
use jiff::tz::TimeZone;

/// Formats a size in bytes with binary units: 1536 → "1.5 KB".
pub fn size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut v = bytes as f64;
    let mut i = 0;
    while v >= 1024.0 && i < UNITS.len() - 1 {
        v /= 1024.0;
        i += 1;
    }
    if i == 0 {
        format!("{bytes} B")
    } else {
        format!("{v:.1} {}", UNITS[i])
    }
}

/// Formats an integer with commas as the thousands separator: 12843 → "12,843".
pub fn count(n: u64) -> String {
    let digits = n.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// Count with the singular or plural noun phrase: "1 file skipped", "12 files skipped".
pub fn plural(n: usize, one: &str, many: &str) -> String {
    match n {
        1 => format!("1 {one}"),
        n => format!("{} {many}", count(n as u64)),
    }
}

/// Formats a moment in the time zone of the system: "2026-11-21 · 18:30".
pub fn date(time: Timestamp) -> String {
    date_in(time, TimeZone::system())
}

fn date_in(time: Timestamp, tz: TimeZone) -> String {
    time.to_zoned_datetime(tz)
        .strftime("%Y-%m-%d · %H:%M")
        .to_string()
}

/// Splits a path into the folder name and its parent path. Accepts both `\` and `/` as
/// separators on any system: `D:\Proyectos` → ("Proyectos", "D:\").
pub fn split_path(path: &Path) -> (String, String) {
    let full = path.display().to_string();
    let trimmed = full.trim_end_matches(['\\', '/']);
    match trimmed.rfind(['\\', '/']) {
        Some(i) if i + 1 < trimmed.len() => {
            let parent = &trimmed[..=i];
            // Strip the trailing separator, except at a drive root ("D:\").
            let parent = match parent.trim_end_matches(['\\', '/']) {
                p if p.is_empty() || p.ends_with(':') => parent,
                p => p,
            };
            (trimmed[i + 1..].to_owned(), parent.to_owned())
        }
        _ => (full, String::new()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes() {
        assert_eq!(size(0), "0 B");
        assert_eq!(size(1023), "1023 B");
        assert_eq!(size(1536), "1.5 KB");
        assert_eq!(size(3_512_000_000), "3.3 GB");
    }

    #[test]
    fn counts() {
        assert_eq!(count(0), "0");
        assert_eq!(count(999), "999");
        assert_eq!(count(12_843), "12,843");
        assert_eq!(count(1_000_000), "1,000,000");
    }

    #[test]
    fn plurals() {
        assert_eq!(plural(1, "file", "files"), "1 file");
        assert_eq!(plural(1200, "file", "files"), "1,200 files");
    }

    #[test]
    fn dates() {
        let t: Timestamp = "2026-11-21T18:30:59Z".parse().unwrap();
        assert_eq!(date_in(t, TimeZone::UTC), "2026-11-21 · 18:30");
    }

    #[test]
    fn paths() {
        let s = |p: &str| split_path(Path::new(p));
        assert_eq!(s(r"D:\Proyectos"), ("Proyectos".into(), r"D:\".into()));
        assert_eq!(
            s(r"C:\Users\Ana\Documentos\"),
            ("Documentos".into(), r"C:\Users\Ana".into())
        );
        assert_eq!(s("/home/ana"), ("ana".into(), "/home".into()));
        assert_eq!(s("/home"), ("home".into(), "/".into()));
        assert_eq!(s("D:"), ("D:".into(), String::new()));
    }
}
