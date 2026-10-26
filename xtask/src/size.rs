//! Enforces the file-size limits described in `CONTRIBUTING.md` ("Code quality").

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::report;

const SCANNED_DIRS: &[&str] = &["crates", "xtask", "apps"];
const SKIPPED_DIRS: &[&str] = &["target", "node_modules", "dist", "tests", ".svelte-kit"];

struct Limit {
    extension: &'static str,
    soft: usize,
    hard: usize,
}

const LIMITS: &[Limit] = &[
    Limit {
        extension: "rs",
        soft: 300,
        hard: 500,
    },
    Limit {
        extension: "ts",
        soft: 300,
        hard: 500,
    },
    Limit {
        extension: "svelte",
        soft: 200,
        hard: 350,
    },
];

pub(crate) fn check(root: &Path) -> Result<()> {
    let mut files = Vec::new();
    for dir in SCANNED_DIRS {
        collect_files(&root.join(dir), &mut files)?;
    }

    let mut violations = Vec::new();
    for path in &files {
        let Some(limit) = limit_for(path) else {
            continue;
        };
        let source =
            fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
        let lines = counted_lines(&source);
        let shown = path.strip_prefix(root).unwrap_or(path).display();
        if lines > limit.hard {
            violations.push(format!(
                "{shown}: {lines} lines (hard limit {})",
                limit.hard
            ));
        } else if lines > limit.soft {
            println!(
                "warning: {shown}: {lines} lines (soft limit {}), consider splitting",
                limit.soft
            );
        }
    }
    report::finish("file size", &violations)
}

fn collect_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(dir).with_context(|| format!("listing {}", dir.display()))? {
        let path = entry?.path();
        if !path.is_dir() {
            out.push(path);
        } else if !is_skipped(&path) {
            collect_files(&path, out)?;
        }
    }
    Ok(())
}

fn is_skipped(dir: &Path) -> bool {
    dir.file_name()
        .and_then(OsStr::to_str)
        .is_some_and(|name| SKIPPED_DIRS.contains(&name))
}

fn limit_for(path: &Path) -> Option<&'static Limit> {
    let extension = path.extension()?.to_str()?;
    LIMITS.iter().find(|limit| limit.extension == extension)
}

/// Counts lines up to the first `#[cfg(test)]`, so inline unit-test modules are not counted.
fn counted_lines(source: &str) -> usize {
    source
        .lines()
        .take_while(|line| line.trim() != "#[cfg(test)]")
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_all_lines_without_test_module() {
        assert_eq!(counted_lines("a\nb\nc\n"), 3);
    }

    #[test]
    fn stops_counting_at_test_module() {
        assert_eq!(counted_lines("a\nb\n#[cfg(test)]\nmod tests {}\n"), 2);
    }

    #[test]
    fn skips_configured_directories() {
        assert!(is_skipped(Path::new("crates/foo/target")));
        assert!(!is_skipped(Path::new("crates/foo/src")));
    }
}
