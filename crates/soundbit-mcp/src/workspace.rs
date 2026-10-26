//! The directory an agent may read and write.
//!
//! Agents pass relative paths. Absolute paths, `..` components and symlinks that lead outside
//! the workspace are rejected, so a confused or malicious prompt cannot touch other files.

use std::fs;
use std::path::{Component, Path, PathBuf};

/// A sandbox directory for project files and exports.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Workspace {
    root: PathBuf,
}

impl Workspace {
    /// Uses `root`, which must be an existing directory.
    pub(crate) fn new(root: &Path) -> std::io::Result<Self> {
        let root = root.canonicalize()?;
        if root.is_dir() {
            Ok(Self { root })
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotADirectory,
                "not a directory",
            ))
        }
    }

    /// The workspace directory.
    pub(crate) fn root(&self) -> &Path {
        &self.root
    }

    /// Resolves a relative path for writing, adding `extension` if it is missing and creating
    /// parent directories inside the workspace.
    pub(crate) fn resolve(&self, relative: &str, extension: &str) -> Result<PathBuf, String> {
        let mut path = PathBuf::from(relative.trim());
        if path.as_os_str().is_empty() {
            return Err("path must not be empty".to_owned());
        }
        if !path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
        {
            return Err(format!(
                "path `{relative}` must be relative to the workspace, without `..`, e.g. \
                 `songs/theme.{extension}`"
            ));
        }
        if path.extension().is_none_or(|ext| ext != extension) {
            path.as_mut_os_string().push(format!(".{extension}"));
        }

        let full = self.root.join(&path);
        let parent = full.parent().unwrap_or(&self.root);
        fs::create_dir_all(parent)
            .map_err(|error| format!("cannot create {}: {error}", parent.display()))?;
        let real_parent = parent.canonicalize().map_err(|error| error.to_string())?;
        if !real_parent.starts_with(&self.root) {
            return Err(format!("path `{relative}` leads outside the workspace"));
        }
        Ok(full)
    }

    /// The path relative to the workspace, for showing to agents.
    pub(crate) fn display(&self, path: &Path) -> String {
        path.strip_prefix(&self.root)
            .unwrap_or(path)
            .display()
            .to_string()
    }

    /// Project files in the workspace and its subdirectories, sorted.
    pub(crate) fn projects(&self, extension: &str) -> Vec<String> {
        let mut found = Vec::new();
        collect(&self.root, extension, 0, &mut found);
        let mut names: Vec<String> = found.iter().map(|path| self.display(path)).collect();
        names.sort();
        names
    }
}

/// Maximum directory depth searched by [`Workspace::projects`].
const MAX_DEPTH: usize = 4;

fn collect(dir: &Path, extension: &str, depth: usize, found: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for path in entries.flatten().map(|entry| entry.path()) {
        if path.is_dir() {
            if depth < MAX_DEPTH {
                collect(&path, extension, depth + 1, found);
            }
        } else if path.extension().is_some_and(|ext| ext == extension) {
            found.push(path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn workspace() -> (tempfile::TempDir, Workspace) {
        let dir = tempfile::tempdir().expect("temp dir");
        let workspace = Workspace::new(dir.path()).expect("valid dir");
        (dir, workspace)
    }

    #[test]
    fn resolves_relative_paths_and_adds_extension() {
        let (_dir, workspace) = workspace();
        let path = workspace.resolve("songs/theme", "sbproj").expect("valid");
        assert!(path.starts_with(workspace.root()));
        assert_eq!(
            workspace.display(&path),
            Path::new("songs/theme.sbproj").display().to_string()
        );
        assert!(path.parent().is_some_and(Path::is_dir));
    }

    #[test]
    fn rejects_escaping_paths() {
        let (_dir, workspace) = workspace();
        for bad in ["../x", "/etc/passwd", "a/../../x", ""] {
            assert!(
                workspace.resolve(bad, "wav").is_err(),
                "{bad} should be rejected"
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlinks_out_of_the_workspace() {
        let (dir, workspace) = workspace();
        let outside = tempfile::tempdir().expect("temp dir");
        std::os::unix::fs::symlink(outside.path(), dir.path().join("link")).expect("symlink");
        assert!(workspace.resolve("link/x", "wav").is_err());
    }
}
