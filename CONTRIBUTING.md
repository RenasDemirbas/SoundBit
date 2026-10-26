# Contributing to SoundBit

Thanks for wanting to help. Bug reports, presets, example songs, new MCP tools, DSP
improvements and documentation fixes are all welcome.

## Ground rules

1. **Two equal front doors.** Everything a human can do in the editor, an agent can do through
   MCP, and the other way round. Both drive the same application core (`soundbit-app`); neither
   contains music logic of its own.
2. **Every change is undoable.** Song edits are `Command`s executed by the editor. Never mutate a
   `Song` anywhere else.
3. **The audio thread is sacred.** No allocation, locks, I/O, logging or panics in anything the
   audio callback calls. See [Real-time audio](#real-time-audio).
4. **Bit-exact rendering.** The same song renders to the same samples on every platform, in debug
   and release.
5. **No network access** besides the local MCP endpoint. No telemetry, no update checks.
6. **No new dependencies** (crates or npm packages) without a reason in the pull request. They
   must be GPLv3-compatible; `cargo deny` checks it.
7. **Same license.** SoundBit is licensed under the GNU GPL, version 3 or later. By sending a pull
   request you agree that your contribution is released under the same terms.

## Development setup

You need a stable Rust toolchain (see `rust-toolchain.toml`) and, for the desktop app, Node.js
22 or newer. On Linux, install the ALSA and WebKitGTK development packages:

```sh
sudo apt-get install -y libasound2-dev libwebkit2gtk-4.1-dev libsoup-3.0-dev \
  libjavascriptcoregtk-4.1-dev librsvg2-dev libayatana-appindicator3-dev
```

```sh
git clone https://github.com/RenasDemirbas/SoundBit.git
cd SoundBit
cargo build --workspace
cd apps/desktop && npm install && npm run tauri dev
```

## Before you open a pull request

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo xtask check                 # dependency graph + file-size limits
cargo deny check                  # when dependencies change
cd apps/desktop && npm run check && npm run lint && npm test && npm run build
```

Report real results in the pull request. If something fails or you skipped it, say so.

## Architecture

SoundBit uses ports and adapters. The domain is pure; everything that touches the outside
world is an adapter at the edge.

| Crate | Holds | May depend on |
|---|---|---|
| `soundbit-core` | Song, tracks, patterns, notes, instruments, chip profiles. Pure data and validation | nothing |
| `soundbit-dsp` | Oscillators, envelopes, FM, filter, reverb. `no_std`, pure math | nothing |
| `soundbit-engine` | Sequencer, voices, mixer, offline renderer, real-time player | core, dsp |
| `soundbit-formats` | `.sbproj` files and migrations, preset files, WAV export | core (engine for export) |
| `soundbit-app` | Commands, undo/redo, sessions, preset library. The only place songs change | core, engine, formats |
| `soundbit-mcp` | MCP tools, inputs and views | app |
| `soundbit-audio-io` | Audio output through `cpal` | nothing in the workspace |
| `soundbit-cli`, `apps/desktop/src-tauri` | Composition roots: wiring only | everything above |

Adapters never depend on each other, and there is no "common" or "utils" crate. The allowed graph
lives in `xtask/src/deps.rs`; `cargo xtask check` enforces it. If you need a new dependency
direction, the design probably needs rethinking: open an issue first.

When a feature touches several layers, build it bottom-up (core, then engine and formats, then
app, then the adapters) with tests at each step. Architectural decisions are recorded in
[`docs/adr/`](docs/adr).

## Real-time audio

Inside the audio callback and everything it calls:

- No heap allocation or deallocation, no `Box::new`, no `Vec::push` beyond capacity.
- No locks, blocking channels, syscalls, file or network I/O.
- No logging and no panics (`unwrap`, `expect`, out-of-bounds indexing). Clamp and continue.
- Communicate only through the preallocated lock-free queues in `soundbit_engine::realtime`.
- Mark RT-safe functions in `soundbit-engine` with `// RT-SAFE`.
- Voice state lives inline in preallocated pools. Delay lines are allocated once in
  `Engine::new`.

`crates/soundbit-engine/tests/realtime.rs` runs the engine under a counting allocator and fails
on any allocation.

**Determinism.** In `soundbit-dsp` and `soundbit-engine`, use only `+ - * /`, comparisons and
conversions on floats. Never call `sin`, `cos`, `exp`, `powf`, `tanh` and friends: their last bits
differ between operating systems. Use `soundbit_dsp::math` (`sine`, `exp2`, `semitone_ratio`,
`note_frequency`, `soft_clip`). Noise comes from fixed-seed LFSRs. `SHOWCASE_HASH` and
`SHAPED_HASH` in `crates/soundbit-engine/tests/render.rs` pin the audio output; change them only for
intentional sound changes and explain why in the pull request.

## Code quality

| Metric | Soft limit | Hard limit | Enforced by |
|---|---:|---:|---|
| Lines per `.rs` / `.ts` file | 300 | 500 | `cargo xtask check` |
| Lines per Svelte component | 200 | 350 | `cargo xtask check` |
| Lines per function | 40 | 80 | Clippy |
| Parameters per function | 5 | 7 | Clippy |

- One responsibility per module, named after what it does. No `utils.rs`, `misc.rs`, `manager.rs`.
- Make invalid states unrepresentable: bounded values use the `ranged!` newtypes, choices are
  enums, IDs are opaque newtypes.
- Musical time is in steps in the domain; seconds and samples exist only in the engine.
- No `unsafe` outside `soundbit-audio-io` and `soundbit-dsp`, and there only with a `// SAFETY:`
  comment. No `unwrap()` / `expect()` outside tests and `main`.
- Errors expose their cause through `source()` or in the message, never both.
- Comments explain *why*. Every public Rust item has a doc comment. No commented-out code and no
  `TODO` without an issue number.
- TypeScript is `strict` with no `any`. Components are presentational; state lives in stores,
  and only `apps/desktop/src/lib/ipc/` talks to Tauri.

## MCP tools

The MCP surface is a public API for agents. A tool's description and parameter descriptions are
its whole manual.

- Names are `noun_verb` (`pattern_set_notes`, `arrangement_place_many`).
- A handler parses input, builds a `Command` (or reads the song), runs it and returns a view. No
  music logic in `soundbit-mcp`.
- Every input field has a doc comment with units and ranges. Accept musician-friendly input
  (`"C4"`, `"F#3"`) and convert at the edge.
- Problems are tool errors that say what was wrong, the allowed range and how to fix it.
- Tool schemas are snapshot-tested. A schema change is a breaking change for agents: review it
  with `cargo insta review` and note it in `CHANGELOG.md`.

## Presets, samples and references

- Presets are data in `assets/presets/*.json`, never hard-coded. Each file has a `source`.
- Check the license of every source before taking anything, record it in
  `docs/references/<source>.md` and, for ported code or data, in `THIRD_PARTY_NOTICES.md`.
- Prefer reimplementing from understanding over copying code.
- Never add copyrighted songs as examples or test fixtures; write your own.

## Project files

`.sbproj` files are versioned JSON. Every change to the song layout bumps `FORMAT_VERSION` and
ships a migration in `crates/soundbit-formats/src/migrations/` with a test against an old
fixture.

## Commits and pull requests

- [Conventional Commits](https://www.conventionalcommits.org): `feat(engine): add pulse duty sweep`.
  Small and focused.
- One topic per pull request. Describe what changes for the person making music.
- A bug fix comes with a test that fails without the fix.
- Update `CHANGELOG.md` (Keep a Changelog) for anything users or agents notice.
