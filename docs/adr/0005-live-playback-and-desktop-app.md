# 0005. Live playback, desktop app and the embedded MCP endpoint

- Status: Accepted
- Date: 2026-10-25

## Context

Users need to hear songs while editing, and changes made by an AI agent should appear in the
open editor as they happen. Audio callbacks must never block (ADR 0003). The GUI, the audio
thread and the MCP server all touch the same song.

## Decision

**Real-time player.** `soundbit_engine::realtime::player()` returns a `PlayerControl` and an
`RtPlayer` connected by two `rtrb` queues. The control side builds an `Engine` (or a new
`Schedule` after an edit) and moves it to the audio thread; replaced engines and schedules are
moved back and dropped on the control side. `Engine` preallocates player slots for the maximum
track count, so swapping a schedule mid-song only re-seeks event cursors. A counting-allocator
test proves `RtPlayer::process` never allocates. Position and play state are shared through
atomics.

**Audio output.** `soundbit_audio_io::AudioOutput` opens the default `cpal` device on its own
thread (streams are not `Send` everywhere), adapts stereo `f32` to the device's channel count
and sample format, and is `Send` so it can live in application state. It knows nothing about
songs; composition roots pass a closure that calls `RtPlayer::process`. The planned
`AudioBackend` trait was not needed yet: there is one backend, and tests use `RtPlayer`
directly.

**Playback in the app layer.** `soundbit_app::Playback` compiles the editor's song, starts
playback, and on `sync` re-sends the schedule when `(editor id, revision)` changed.

**Desktop app.** `apps/desktop` is a Tauri 2 app with a Svelte 5 front end. The Rust side is a
composition root: one `SharedSession` (`Arc<Mutex<Session>>`), the audio output, and an
`HttpEndpoint` serving MCP on `127.0.0.1:7413`, all sharing the session. A sync thread polls
the editor version every 60 ms, syncs playback, and emits `song-changed`; the front end then
reloads the state. Edits from the GUI go through one tagged `Action` IPC type that maps to app
commands. Canvas drawing and geometry are pure functions with unit tests.

**Endpoint security.** The endpoint binds to loopback only. `rmcp` rejects non-loopback `Host`
headers (DNS rebinding) and Origin validation is enforced with an empty allow-list, so web
pages cannot call it. We did not add a bearer token: any local process could already launch
`soundbit mcp` over stdio, so a token would not raise the bar for local attackers while making
client setup harder. A fixed port keeps MCP client configuration stable.

## Consequences

- Agents and users edit the same song with one undo history; AI steps show up labelled `AI:`.
- Polling adds up to 60 ms latency to live updates; an event-driven notification can replace
  it later without changing the IPC contract.
- Only one desktop instance can serve MCP on the default port; others report the error and
  keep working without MCP.
- CI must install ALSA and WebKitGTK development packages on Linux and runs a front-end job
  (svelte-check, ESLint, Vitest, Vite build).
- `soundbit mcp` (stdio) does not yet attach to a running desktop app; clients that want live
  editing connect to the HTTP endpoint instead.
