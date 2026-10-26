//! Upgrades of older project files, one format version at a time.
//!
//! Each step rewrites the JSON of version `n` into version `n + 1`. Loading runs every step
//! from the file's version up to [`FORMAT_VERSION`](crate::FORMAT_VERSION), then reads the
//! result like a current file, with the same validation.

mod v1_to_v2;

use serde_json::Value;

/// Migration from version `index + 1` to `index + 2`.
const STEPS: [fn(&mut Value); 1] = [v1_to_v2::migrate];

/// Oldest format version that can still be loaded.
pub(crate) const OLDEST_VERSION: u32 = 1;

/// Upgrades `song`, the `"song"` object of a project file, from version `from` to the
/// current version.
pub(crate) fn upgrade(song: &mut Value, from: u32) {
    let first = usize::try_from(from.saturating_sub(OLDEST_VERSION)).unwrap_or(usize::MAX);
    for step in STEPS.iter().skip(first) {
        step(song);
    }
}
