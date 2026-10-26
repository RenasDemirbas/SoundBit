# Architecture Decision Records

Every architectural decision (new crate, dependency direction, file format change, stack
change) is recorded here as `NNNN-short-title.md`. ADRs are immutable once accepted: to change a
decision, write a new ADR that supersedes the old one and update the old one's status.

| ADR | Title | Status |
|-----|-------|--------|
| [0001](0001-architecture-and-tech-stack.md) | Architecture and tech stack | Accepted (license superseded by 0007) |
| [0002](0002-song-domain-model.md) | Song domain model | Accepted |
| [0003](0003-deterministic-audio.md) | Deterministic, real-time safe audio | Accepted |
| [0004](0004-commands-undo-and-mcp.md) | Commands, undo and the MCP adapter | Accepted |
| [0005](0005-live-playback-and-desktop-app.md) | Live playback, desktop app and embedded MCP endpoint | Accepted |
| [0006](0006-sound-shaping-reverb-and-presets.md) | Sound shaping, reverb and data-driven presets | Accepted |
| [0007](0007-gpl-license.md) | License: GPL-3.0-or-later | Accepted |

## Template

```markdown
# NNNN. Title

- Status: Proposed | Accepted | Superseded by NNNN
- Date: YYYY-MM-DD

## Context
What forces are at play and why a decision is needed.

## Decision
What we decided.

## Consequences
What becomes easier or harder, and what follow-up work it creates.
```
