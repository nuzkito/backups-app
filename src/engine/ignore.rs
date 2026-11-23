//! Ignored items: patterns matched against the name of each file or folder.
//!
//! Supports the wildcards `*` (any text) and `?` (one character). Matching is case-insensitive. If a folder matches, it is skipped entirely.

use std::ffi::OsStr;

#[derive(Debug, Default, Clone)]
pub struct Ignore {
    patterns: Vec<Vec<char>>,
}

impl Ignore {
    /// Empty patterns are skipped.
    pub fn new<S: AsRef<str>>(patterns: &[S]) -> Self {
        let patterns = patterns
            .iter()
            .map(|p| p.as_ref().trim())
            .filter(|p| !p.is_empty())
            .map(|p| p.to_lowercase().chars().collect())
            .collect();
        Self { patterns }
    }

    pub fn matches(&self, name: &OsStr) -> bool {
        if self.patterns.is_empty() {
            return false;
        }
        let name: Vec<char> = name.to_string_lossy().to_lowercase().chars().collect();
        self.patterns.iter().any(|p| glob(p, &name))
    }
}

fn glob(pattern: &[char], name: &[char]) -> bool {
    let (mut p, mut n) = (0, 0);
    // Position of the last `*` and of the name character where it was tried.
    let mut star: Option<(usize, usize)> = None;
    while n < name.len() {
        if p < pattern.len() && (pattern[p] == '?' || pattern[p] == name[n]) {
            p += 1;
            n += 1;
        } else if p < pattern.len() && pattern[p] == '*' {
            star = Some((p, n));
            p += 1;
        } else if let Some((sp, sn)) = star {
            p = sp + 1;
            n = sn + 1;
            star = Some((sp, sn + 1));
        } else {
            return false;
        }
    }
    pattern[p..].iter().all(|&c| c == '*')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn m(patterns: &[&str], name: &str) -> bool {
        Ignore::new(patterns).matches(OsStr::new(name))
    }

    #[test]
    fn exact_names() {
        assert!(m(&["node_modules", "vendor"], "node_modules"));
        assert!(m(&["node_modules", " vendor "], "vendor"));
        assert!(m(&["Vendor"], "VENDOR"));
        assert!(!m(&["vendor"], "vendors"));
        assert!(!m(&[], "vendor"));
        assert!(!m(&["", " "], "vendor"));
    }

    #[test]
    fn wildcards() {
        assert!(m(&["*.tmp"], "a.tmp"));
        assert!(m(&["*.tmp"], ".tmp"));
        assert!(!m(&["*.tmp"], "a.tmp.txt"));
        assert!(m(&["~$*"], "~$informe.docx"));
        assert!(m(&["cache?"], "cache1"));
        assert!(!m(&["cache?"], "cache"));
        assert!(m(&["*a*b*"], "xxaYYbzz"));
        assert!(m(&["*"], "lo-que-sea"));
    }
}
