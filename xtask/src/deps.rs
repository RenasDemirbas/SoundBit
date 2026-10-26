//! Enforces the crate dependency rules described in `CONTRIBUTING.md` ("Architecture").

use std::collections::BTreeSet;

use anyhow::{Context, Result};
use cargo_metadata::{DependencyKind, MetadataCommand, Package};

use crate::report;

/// Workspace crates each crate may depend on; anything not listed is forbidden.
/// Dev-dependencies are not checked. Changing this table requires an ADR.
const ALLOWED: &[(&str, &[&str])] = &[
    ("soundbit-core", &[]),
    ("soundbit-dsp", &[]),
    ("soundbit-engine", &["soundbit-core", "soundbit-dsp"]),
    ("soundbit-audio-io", &[]),
    ("soundbit-formats", &["soundbit-core", "soundbit-engine"]),
    (
        "soundbit-app",
        &["soundbit-core", "soundbit-engine", "soundbit-formats"],
    ),
    ("soundbit-mcp", &["soundbit-app"]),
    (
        "soundbit-cli",
        &["soundbit-app", "soundbit-mcp", "soundbit-audio-io"],
    ),
    (
        "soundbit-desktop",
        &["soundbit-app", "soundbit-mcp", "soundbit-audio-io"],
    ),
    ("xtask", &[]),
];

pub(crate) fn check() -> Result<()> {
    let metadata = MetadataCommand::new()
        .no_deps()
        .exec()
        .context("failed to run `cargo metadata`")?;
    let packages = metadata.workspace_packages();
    let members: BTreeSet<&str> = packages.iter().map(|p| p.name.as_str()).collect();

    let violations: Vec<String> = packages
        .iter()
        .flat_map(|package| violations_for(package, &members))
        .collect();
    report::finish("dependency rule", &violations)
}

fn violations_for(package: &Package, members: &BTreeSet<&str>) -> Vec<String> {
    let name = package.name.as_str();
    let Some(allowed) = allowed_for(name) else {
        return vec![format!(
            "`{name}` is missing from the table in xtask/src/deps.rs (adding it needs an ADR)"
        )];
    };
    package
        .dependencies
        .iter()
        .filter(|dep| dep.kind != DependencyKind::Development)
        .filter(|dep| members.contains(dep.name.as_str()))
        .filter(|dep| !allowed.contains(&dep.name.as_str()))
        .map(|dep| format!("`{name}` must not depend on `{}`", dep.name))
        .collect()
}

fn allowed_for(name: &str) -> Option<&'static [&'static str]> {
    ALLOWED
        .iter()
        .find(|(crate_name, _)| *crate_name == name)
        .map(|(_, allowed)| *allowed)
}
