# 0004. Commands, undo and the MCP adapter

- Status: Accepted
- Date: 2026-10-25

## Context

Humans (GUI, CLI) and AI agents (MCP) edit the same song. Every change must be undoable, agent
changes must be recognizable, and a failed change — especially a multi-step one from an agent —
must not leave the song half-edited. The MCP surface is a public API: agents depend on its
names and schemas, and they need errors they can act on.

## Decision

**Commands and edits.** `soundbit-app` exposes `Command`, an enum grouped by domain with an
atomic `Batch`. Commands are planned against the current song into private `Edit`s: primitive
changes that map one-to-one onto `Song` methods and return their exact inverse when applied.
The undo history stores inverses; undoing returns the redo edit, and vice versa. Partial
updates use `*Changes` structs (`None` keeps the value), so GUI and MCP share the merging.

**Editor.** `Editor` owns the song, the history (500 steps), a chip-profile policy and a
revision counter. It applies each command to a draft copy and commits only on success. This
makes batches atomic without rollback code; the cost (cloning a song of a few KB to a few
hundred KB) is negligible next to human or agent editing speed. Under `ProfilePolicy::Enforce`
a command that adds new profile violations is rejected. `Origin::Agent` prefixes labels with
`AI:`. Ids are never reused within a session, even after undo, so an agent never confuses an
old id with a new entity; the id counter is therefore excluded from song equality.

**MCP adapter.** `soundbit-mcp` uses the official Rust SDK (`rmcp`). Tools are grouped by
domain, one router per file. Input types are separate from domain types (the public schema
must not change when the model does); enum choices use a `Choice<T>` wrapper whose schema is
generated from the domain enum. Tool failures are returned as tool errors with actionable
messages, never as protocol errors. Read tools add text renderings (tracker grid, arrangement
grid) because LLMs reason about those better than raw JSON. File access is sandboxed to a
workspace directory. `soundbit mcp` serves stdio with a headless session.

## Consequences

- Adding an editing feature means adding a command variant and its `into_edit` mapping; GUI,
  MCP and CLI then get undo and atomicity for free.
- 27 tools cover the song model; their schemas are pinned by a snapshot test.
- Live collaboration with the GUI (HTTP endpoint, change notifications, MCP resources) is
  deferred until the GUI exists; `Editor::revision()` is the hook for it.
- `song_new` and `song_open` replace the session and clear the history; they refuse to discard
  unsaved changes unless the agent confirms.
