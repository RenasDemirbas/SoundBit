# 0002. Song domain model

- Status: Accepted
- Date: 2026-10-25

## Context

`soundbit-core` needs a song model that both humans (tracker GUI) and AI agents (MCP) edit.
Agents make mistakes, so invalid input must be rejected with clear errors instead of producing
a broken song. Every edit must be undoable. Songs are saved to disk and must load with the same
guarantees.

## Decision

**One-bar patterns.** A pattern is exactly one bar long; the bar length comes from the song's
time settings (steps per beat × beats per bar). This follows Bosca Ceoil and classic trackers,
keeps the arrangement a simple bar grid, and is easy for agents to reason about. Changing the
time settings is rejected if an existing pattern would no longer fit.

**`Song` is the aggregate root.** Instruments, patterns and tracks are stored in id-keyed,
ordered collections inside `Song` and are only changed through `Song` methods. Every method is
all-or-nothing and returns what it replaced or removed (`Removed { id, index, value }`), so
the app layer can build exact undo steps without cloning whole songs. Ids are never reused.

**Validated values.** Bounded parameters are `ranged!` newtypes that validate on construction
and on deserialization. Deserializing a `Song` runs the same reference and bar checks as the
editing methods. The JSON layout is readable (`{"id": 1, "name": …}` entries, clips as
`{bar, pattern}` lists, tagged voices) and is the basis for the project file format.

**Soft chip profiles.** A chip profile never blocks a single edit inside `core`; switching
profiles or reordering a song mid-edit would otherwise be impossible. `Song::profile_violations`
lists every violation with an actionable message. The app layer decides policy: it rejects
commands that introduce new violations.

**Voices.** Pulse (4 duty cycles), triangle, sawtooth, sine, noise (long/short), 4-bit 32-sample
wavetables, and 4-operator FM with YM2612 parameter ranges. Sample-based voices (NES DPCM, SNES)
are deferred until sample storage is designed.

## Consequences

- Multi-bar patterns are not possible without a new ADR and a format migration.
- Edits from any adapter go through one validation path; agents get errors such as
  `pattern 3 is placed on track 1 at bar 0; clear it from the arrangement first`.
- `serde` is a dependency of `core`. The file format in `soundbit-formats` wraps the song with a
  `format_version` and migrations, rather than duplicating every type as a DTO.
- Per-note effects (slide, vibrato, arpeggio, duty sweeps) need a follow-up change to `Note`.
