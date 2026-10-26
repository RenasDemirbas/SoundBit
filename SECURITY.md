# Security

## Reporting a vulnerability

Please **do not open a public issue** for security problems. Use GitHub's
[private vulnerability reporting](https://github.com/RenasDemirbas/SoundBit/security/advisories/new)
instead. You can expect a first answer within a week. Fixes go into the latest release.

| Version | Supported |
| --- | --- |
| 0.1.x (latest) | Yes |

## Threat model

SoundBit runs on your computer: the desktop app, or the `soundbit` command-line tool. Both can
expose an MCP server that AI agents use to edit songs and write files. That server is what this
design protects.

| Threat | Defence |
| --- | --- |
| Another device on the network reaches the MCP endpoint | The desktop app and `soundbit mcp --http` bind to `127.0.0.1` only. |
| A website open in your browser sends requests to `http://127.0.0.1:7413` (DNS rebinding, cross-origin requests) | Requests whose `Host` header is not a loopback name are refused, and every request with an `Origin` header is refused, so browsers cannot talk to the endpoint. `crates/soundbit-mcp/tests/http.rs` tests both. |
| An agent reads or writes files outside its workspace | File tools only accept relative paths inside the workspace (`--root`, or `Music/SoundBit` in the desktop app). Absolute paths, `..` and symlinks leading out are rejected before anything is touched. |
| An agent produces a runaway song or export | Song sizes are limited in the domain model, and exports are capped at 600 seconds. |
| A malicious `.sbproj` file | Project files are parsed as JSON and validated with the same rules as editing; there is no code in a project file. |
| An agent ruins your song | Every change is undoable and labelled `AI:` in the history. |

**Out of scope:** programs already running on your computer under your account (they can reach a
local port like any MCP client), your MCP client, and the AI model's choices.

## Privacy

- **No telemetry, analytics, accounts or update checks.** SoundBit opens no network connection
  except the local MCP endpoint.
- Song data travels only to the MCP client you connect, which sends it to the model provider you
  configured. That is the client's data handling, not SoundBit's.
