# 0001. Architecture and tech stack

- Status: Accepted
- Date: 2026-10-25

## Context

SoundBit is a chiptune workstation for Linux, Windows and macOS with two equal front ends: a
tracker-style GUI for humans and an MCP server for AI agents. Both must edit the same song, and
every change must be visible and undoable from either side. The project will be open source, so
contributors must be able to find where code belongs without learning the whole codebase.

Constraints:

- Real-time audio needs predictable latency: no garbage collector pauses, no allocation or
  locking on the audio thread.
- Offline rendering must be deterministic so audio output can be regression-tested.
- One codebase must ship on three desktop platforms.
- The GUI needs dense, fast grid editing (pattern editor, arrangement).

## Decision

**Language and runtime.** Rust for everything except the GUI front end. It gives native
performance without a GC, strong cross-platform tooling, and the official MCP SDK (`rmcp`).

**Desktop shell and GUI.** Tauri 2 hosting a Svelte 5 + TypeScript front end. A web UI lowers
the barrier for contributors and makes custom canvas-based editors straightforward, while the
audio engine stays in native Rust.

**Architecture.** Hexagonal (ports and adapters) as a Cargo workspace:

| Crate               | Role                                          | May depend on            |
|---------------------|-----------------------------------------------|--------------------------|
| `soundbit-core`     | Pure domain model and validation              | —                        |
| `soundbit-dsp`      | RT-safe DSP primitives                        | —                        |
| `soundbit-engine`   | Sequencer, mixer, offline renderer            | core, dsp                |
| `soundbit-audio-io` | `AudioBackend` port + platform backends       | —                        |
| `soundbit-formats`  | Project file, export, `.ceol` import          | core, engine             |
| `soundbit-app`      | Commands, queries, undo/redo, sessions        | core, engine, formats    |
| `soundbit-mcp`      | MCP adapter                                   | app                      |
| `soundbit-cli`      | `soundbit` binary (composition root)          | app, mcp, audio-io       |
| `apps/desktop`      | Tauri app (composition root, added later)     | app, mcp, audio-io       |

All song mutations go through undoable commands in `soundbit-app`. Adapters (GUI, MCP) never
touch the domain model directly and never depend on each other; only composition roots wire
adapters together.

**Enforcement.** The dependency table is enforced by `cargo xtask check-deps`, file-size limits
by `cargo xtask check-size`, function limits by Clippy, licenses and advisories by
`cargo-deny`. CI runs everything on Linux, Windows and macOS.

**License.** MIT OR Apache-2.0, the Rust ecosystem convention. *Superseded by
[ADR 0007](0007-gpl-license.md): SoundBit is licensed under GPL-3.0-or-later.*

## Consequences

- Clear placement rules: domain logic in `core`, sound in `dsp`/`engine`, behaviour in `app`,
  protocols at the edges.
- `core`, `dsp` and `engine` can be tested headless and fast, including golden-audio tests.
- GUI changes need a typed IPC boundary between Svelte and Rust; this is extra code but keeps the
  front end replaceable.
- Contributors need both Rust and TypeScript for full-stack work; most features touch only one.
- Changing a dependency edge requires a new ADR and an update to `xtask/src/deps.rs`.
