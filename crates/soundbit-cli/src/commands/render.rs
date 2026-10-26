//! `soundbit render`: export a project to WAV.

use std::path::Path;

use soundbit_app::{RenderOptions, Session};

pub(crate) fn run(input: &Path, output: Option<&Path>, sample_rate: u32) -> anyhow::Result<()> {
    let session = Session::open(input)?;
    let output = output.map_or_else(|| input.with_extension("wav"), Path::to_path_buf);
    let options = RenderOptions {
        sample_rate,
        ..RenderOptions::default()
    };
    let report = session.export_wav(&output, &options)?;
    println!(
        "Rendered {} ({:.1} s, peak {})",
        output.display(),
        report.duration_seconds,
        decibels(report.peak)
    );
    Ok(())
}

fn decibels(peak: f32) -> String {
    if peak > 0.0 {
        format!("{:.1} dBFS", 20.0 * peak.log10())
    } else {
        "silent".to_owned()
    }
}
