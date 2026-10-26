# 0007. License: GPL-3.0-or-later

- Status: Accepted
- Date: 2026-10-26
- Supersedes: the license paragraph of [ADR 0001](0001-architecture-and-tech-stack.md)

## Context

SoundBit was started under MIT OR Apache-2.0, the usual choice for Rust libraries. Before the
first public release the project had to decide what it wants to protect: SoundBit is an
application, not a library ecosystem, and its maintainer wants improvements to it, including
in redistributed or modified builds, to stay open.

## Decision

SoundBit is licensed under the **GNU General Public License, version 3 or (at your option) any
later version** (`GPL-3.0-or-later`). The full text is in `LICENSE`. Contributions are accepted
under the same terms. The other projects of the maintainer use the same license.

All third-party dependencies remain under their own licenses. `deny.toml` only allows licenses
that are compatible with GPLv3 (MIT, Apache-2.0, BSD, ISC, Zlib, Unicode, and MPL-2.0 for
specific crates). Preset data files state the project license in their `source` field.

## Consequences

- Anyone distributing SoundBit or a modified version must provide the source under the GPL.
- Songs made with SoundBit are not covered by the GPL: project files and rendered audio belong
  to whoever made them.
- New dependencies must be GPLv3-compatible; GPL-2.0-only code cannot be used.
- Other projects can no longer embed SoundBit crates under a permissive license. If a crate
  (for example `soundbit-dsp`) should be reusable more widely later, it can be relicensed by
  its copyright holders in a new ADR.
