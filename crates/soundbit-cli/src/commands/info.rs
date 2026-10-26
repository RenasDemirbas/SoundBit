//! `soundbit info`: summarize a project.

use std::path::Path;

use soundbit_app::Session;

pub(crate) fn run(input: &Path) -> anyhow::Result<()> {
    let session = Session::open(input)?;
    let song = session.song();
    let timing = song.timing();
    let scale = song.scale();

    println!("Title:       {}", song.meta().title);
    if !song.meta().author.is_empty() {
        println!("Author:      {}", song.meta().author);
    }
    println!(
        "Tempo:       {} bpm, {} beats per bar, {} steps per beat, swing {} %",
        timing.bpm.get(),
        timing.beats_per_bar.get(),
        timing.steps_per_beat.get(),
        timing.swing.get()
    );
    println!("Scale:       {} {:?}", scale.root, scale.kind);
    println!("Chip:        {}", song.chip_profile().name());
    println!(
        "Contents:    {} instruments, {} patterns, {} tracks",
        song.instruments().count(),
        song.patterns().count(),
        song.track_count()
    );
    println!(
        "Length:      {} bars ({:.1} s)",
        song.length_bars(),
        song.length_seconds()
    );

    let violations = song.profile_violations();
    if violations.is_empty() {
        println!("Profile:     OK");
    } else {
        println!("Profile:     {} problem(s)", violations.len());
        for violation in violations {
            println!("  - {violation}");
        }
    }
    Ok(())
}
