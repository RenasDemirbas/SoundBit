//! Every example project in `examples/songs` must load, respect its chip profile and render.

#![allow(
    clippy::expect_used,
    reason = "tests panic on unexpected errors by design"
)]

use std::fs;
use std::path::PathBuf;

use soundbit_app::{RenderOptions, Session};

fn example_projects() -> Vec<PathBuf> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/songs");
    let mut projects: Vec<PathBuf> = fs::read_dir(&dir)
        .expect("examples/songs exists")
        .map(|entry| entry.expect("readable entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "sbproj"))
        .collect();
    projects.sort();
    projects
}

#[test]
fn examples_are_valid_and_audible() {
    let projects = example_projects();
    assert!(!projects.is_empty(), "no example projects found");

    let dir = tempfile::tempdir().expect("temp dir");
    for path in projects {
        let session = Session::open(&path).expect("example loads");
        let violations = session.song().profile_violations();
        assert_eq!(violations, [], "{} breaks its chip profile", path.display());

        let options = RenderOptions {
            sample_rate: 22_050,
            ..RenderOptions::default()
        };
        let report = session
            .export_wav(dir.path().join("out.wav"), &options)
            .expect("renders");
        assert!(report.peak > 0.1, "{} is nearly silent", path.display());
        assert!(report.peak < 1.0, "{} clips", path.display());
    }
}
