//! Repository automation, run with `cargo xtask <task>`.

#![forbid(unsafe_code)]

mod deps;
mod report;
mod size;

use std::path::Path;
use std::process::ExitCode;

const USAGE: &str = "usage: cargo xtask <check-deps | check-size | check>";

fn main() -> ExitCode {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap_or_else(|| Path::new("."));

    let results = match std::env::args().nth(1).as_deref() {
        Some("check-deps") => vec![deps::check()],
        Some("check-size") => vec![size::check(root)],
        Some("check") => vec![deps::check(), size::check(root)],
        _ => {
            eprintln!("{USAGE}");
            return ExitCode::from(2);
        }
    };

    let mut failed = false;
    for error in results.into_iter().filter_map(Result::err) {
        eprintln!("error: {error:#}");
        failed = true;
    }
    if failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
