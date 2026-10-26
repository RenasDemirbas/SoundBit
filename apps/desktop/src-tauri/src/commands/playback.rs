//! Transport controls.

use soundbit_app::Playback;
use tauri::State;

use super::CommandResult;
use crate::dto::PlaybackDto;
use crate::state::AppState;

const NO_AUDIO: &str = "no audio device is available";

fn report(playback: Option<&Playback>) -> PlaybackDto {
    PlaybackDto {
        playing: playback.is_some_and(Playback::is_playing),
        position_bars: playback.and_then(Playback::position_bars),
        position_seconds: playback.and_then(Playback::position_seconds),
    }
}

#[tauri::command]
pub(crate) fn play(
    state: State<'_, AppState>,
    from_bar: u16,
    looping: bool,
) -> CommandResult<PlaybackDto> {
    let session = state.session();
    let mut playback = state.playback();
    let player = playback.as_mut().ok_or(NO_AUDIO)?;
    player
        .play(session.editor(), from_bar, looping)
        .map_err(|error| error.to_string())?;
    Ok(report(playback.as_ref()))
}

#[tauri::command]
pub(crate) fn stop(state: State<'_, AppState>) -> CommandResult<PlaybackDto> {
    let mut playback = state.playback();
    let player = playback.as_mut().ok_or(NO_AUDIO)?;
    player.stop().map_err(|error| error.to_string())?;
    Ok(report(None))
}

#[tauri::command]
pub(crate) fn playback_state(state: State<'_, AppState>) -> PlaybackDto {
    report(state.playback().as_ref())
}
