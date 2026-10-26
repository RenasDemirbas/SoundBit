# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this
project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Cargo workspace skeleton with the layered crate structure from ADR 0001.
- `cargo xtask check` enforcing crate dependency rules and file-size limits.
- CI on Linux, Windows and macOS: formatting, Clippy, tests, cargo-deny.
- `soundbit-core` song model (ADR 0002): pitches with note-name parsing, scales, time settings,
  one-bar patterns, instruments (pulse, triangle, sawtooth, sine, noise, wavetable, 4-op FM),
  tracks with clips, loop range, and the `Song` aggregate with undo-friendly edits.
- Chip profiles (NES, Game Boy, Commodore 64, Genesis) with actionable violation reports.
- Validated JSON (de)serialization of songs.
- `soundbit-dsp` (`no_std`): band-limited pulse and sawtooth, stepped triangle, sine, NES-style
  LFSR noise, wavetable, ADSR, and 4-operator YM2612-style FM with 8 algorithms.
- `soundbit-engine`: sample-accurate scheduling, 8-voice pools per track, stereo mixing with a
  soft clipper, and bit-exact offline rendering (ADR 0003). `Engine::process` never allocates.
- `.sbproj` project files (versioned JSON) and 16-bit WAV export.
- `soundbit render` and `soundbit info` commands.
- Example song `examples/songs/first-light.sbproj` (NES profile).
- Command layer in `soundbit-app` (ADR 0004): domain commands, atomic batches, undo/redo
  history, chip-profile enforcement, `AI:`-labelled agent changes, unsaved-change tracking.
- MCP server (`soundbit mcp`) with 27 tools for songs, instruments, patterns, tracks,
  arrangement, history, files and WAV export; tracker-grid views for agents; workspace
  sandbox for file access.
- Live playback (ADR 0005): a real-time player with lock-free control queues, seek, loop and
  glitch-free song updates while playing; `soundbit-audio-io` output via cpal;
  `soundbit play`.
- MCP over streamable HTTP on localhost (`soundbit mcp --http <port>`), with DNS-rebinding
  and browser-origin protection.
- Desktop app (Tauri 2 + Svelte 5): arrangement and piano-roll editors on canvas, instrument
  and pattern panels, transport, undo/redo, open/save/export, and an embedded MCP endpoint so
  AI edits appear live.
- Arrangement view: track headers and the bar ruler stay in view and scroll with the clips
  (tracks below the visible area were unreachable before); clips show a preview of their
  notes and fitted labels; hovering an empty bar previews where the selected pattern goes.
- Pattern list grouped by instrument with search and usage counts; compact instrument editor;
  the piano roll widens its steps to fill the panel.
- Desktop UI redesign: minimal top bar (File menu, title, transport pill, song-settings
  popover, icon undo/redo); shared panel headers, icon buttons, segmented controls, styled
  sliders, selects and scrollbars; hover actions on patterns and tracks; status-bar pills for
  MCP and audio; information messages fade after five seconds.
- Color themes: Graphite (default), Midnight, Game Boy, Synthwave and Paper (light), chosen
  from the palette menu in the top bar and remembered. Canvas editors and instrument colors
  follow the theme.
- Transport shows elapsed time and song length (`m:ss.t / m:ss`) next to the bar position,
  measured from the samples actually played.
- Instrument sound shaping (ADR 0006): pitch envelope (kicks, toms, timpani, lasers), delayed
  vibrato, unison with up to four detuned layers, and a resonant low/high/band-pass filter
  whose cutoff can follow the amplitude envelope. All default to off.
- A shared reverb room per song (size, damping) with a reverb send per instrument.
- Preset library in `assets/presets/` (orchestral, drums, chip, FM; 32 instruments), used by
  the desktop app's "Add instrument" menu and by MCP.
- MCP tools `preset_list`, `instrument_add_preset` and `arrangement_place_many` (a whole
  arrangement in one call and one undo step), bringing the total to 30.
- `export_wav` (MCP) reports the RMS level of the mix and the peak and RMS level of every
  track.
- Desktop instrument editor: envelope, filter, pitch sweep, vibrato, unison and reverb send
  controls; reverb room settings in the song settings.
- Project format migrations (`soundbit-formats/src/migrations/`).
- Logo and brand assets (`assets/brand/`): an eighth note dissolving into pixels, in Game Boy
  greens; new desktop app icons and favicon.
- Example songs `clash-of-banners` (orchestral battle theme) and `da-dada` (chiptune song).
- Community files: `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `SECURITY.md`,
  `THIRD_PARTY_NOTICES.md`, issue and pull request templates.

### Fixed

- MCP tools show workspace paths with `/` on every platform; on Windows they used `\`, unlike
  the paths agents write.

### Changed

- SoundBit is now licensed under the GNU GPL, version 3 or later (ADR 0007), instead of
  MIT OR Apache-2.0.
- Project files are written as `format_version` 2. Version 1 files still load (their
  instruments get sound shaping off); older builds cannot open version 2 files.
- MCP schema (additive, no fields removed): `instrument_add` and `instrument_update` accept
  `pitch_envelope`, `vibrato`, `unison`, `filter` and `reverb_send_percent`; `song_update`
  accepts `reverb_size` and `reverb_damping`; `song_get` and instrument views include the new
  settings. `instrument_update` with nothing to change is now a tool error.
