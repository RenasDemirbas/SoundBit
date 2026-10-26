//! `soundbit play`: listen to a project.

use std::path::Path;
use std::thread;
use std::time::Duration;

use soundbit_app::playback::player;
use soundbit_app::{Playback, Session};
use soundbit_audio_io::AudioOutput;

/// How often the progress line is refreshed.
const POLL: Duration = Duration::from_millis(100);

pub(crate) fn run(input: &Path, from_bar: u16, looping: bool) -> anyhow::Result<()> {
    let session = Session::open(input)?;
    let (control, mut rt) = player();
    let output = AudioOutput::open_default(move |buffer| rt.process(buffer))?;
    let info = output.info();
    let mut playback = Playback::new(control, info.sample_rate);
    playback.play(session.editor(), from_bar, looping)?;

    let song = session.song();
    println!(
        "Playing \"{}\" on {} ({} Hz). Press Ctrl+C to stop.",
        song.meta().title,
        info.device,
        info.sample_rate
    );
    let bars = song.length_bars();
    while playback.is_playing() {
        if let Some(position) = playback.position_bars() {
            eprint!("\rbar {:>3} / {bars}", position.floor());
        }
        playback.sync(session.editor())?;
        thread::sleep(POLL);
    }
    eprintln!();
    Ok(())
}
