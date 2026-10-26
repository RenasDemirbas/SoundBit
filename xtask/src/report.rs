//! Shared reporting for checks.

use anyhow::{Result, bail};

/// Prints every violation and fails if there is at least one.
pub(crate) fn finish(check: &str, violations: &[String]) -> Result<()> {
    for violation in violations {
        eprintln!("{check} violation: {violation}");
    }
    if violations.is_empty() {
        println!("{check} check passed");
        Ok(())
    } else {
        bail!("{} {check} violation(s)", violations.len())
    }
}
