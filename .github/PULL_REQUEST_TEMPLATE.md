## What changes

<!-- What is different for someone making music with SoundBit? Link the issue if there is one. -->

## How it was tested

<!-- Commands you ran and their result. A short render or screenshot for audible or visual changes. -->

## Checklist

- [ ] `cargo fmt`, `cargo clippy -D warnings`, `cargo test` and `cargo xtask check` pass
- [ ] Desktop: `npm run check`, `npm run lint`, `npm test` pass (if the front end changed)
- [ ] Audio thread code stays allocation-free; golden hashes changed only on purpose (explained above)
- [ ] MCP schema snapshot reviewed (if tools changed)
- [ ] `CHANGELOG.md` updated
