# omarchy-onboard — Omarchy migration assistant

Rust workspace. Toolchain and tasks via `mise` (`mise run build|test|lint|scan|plan`).

## Model

The unit of authorship is a **Topic** — one row of the spreadsheet: how to look for a
concern on each source platform, and what to propose on the target.

| Phase | Runs on | Topic method | Output |
|---|---|---|---|
| Discover | source (Mac) | `Topic::discover(SourceContext)` | `Vec<Finding>` — facts, never actions |
| Propose | target (Omarchy) | `Topic::propose(mine, all, TargetContext)` | `Vec<Proposal>` — each composed of `Operation` primitives |
| Migrate | target | `Executor::apply_all` (`omarchy-onboard-target`) | runs primitives: pacman/yay/`omarchy-*`/files |

Crates: `core` (model, platform-free) · `topics` (all topics + mapping TOML) · `target`
(executor, `PackageIndex`) · `net` (pairing, transport) · `cli`.

## Adding a topic

```
crates/topics/src/<id>/mod.rs      TopicMeta + propose  (+ any value structs)
crates/topics/src/<id>/macos.rs    discover on macOS      (unix.rs if shared with Linux)
crates/topics/src/<id>/windows.rs  later
```

Register in `topics::all()`. Mapping tables live beside the topic (`homebrew/map.toml`).
Add a propose test in `crates/topics/tests/propose.rs` (canned findings → expected proposals)
and, if discover reads files, a discover test against a temp home in `tests/discover.rs`.

## Principles

- Findings are facts, never actions, and never contain secrets — reference files with `FileRef`.
- Propose the *semantically equivalent* thing: `InstallPackages`, `InstallEditorExtension`,
  `WriteConfig`, `SetTheme`. `PullFiles` is only for user-owned data. `RunCommand` is the
  escape hatch — treat it like `unsafe`. If a topic can't express itself, add a primitive.
- A proposal is an **action** (checklist; runs operations, in order, stop on first failure) or a
  **note** (`covered` / `not_needed` / `suggestion` / `unknown`) — shown after the plan, never a
  decision. Notes are where we make suggestions when nothing programmatic can be done.
- Every proposal names its findings and has a default; users accept/skip per item or per `Group`.
- Topics are gated by `TopicMeta::sources`, not `cfg`, so the catalogue is visible everywhere.
- `propose` sees the whole `Discovery` for cross-topic cases (font referenced by terminal config).

## Pairing

`serve` advertises over mDNS with user-data `SHA256("tag"‖code)[..8]`; `migrate <code>`
scans for that tag, connects over iroh (QUIC, keypair-authenticated, relay disabled), and
proves the code with `SHA256(code ‖ TLS-EKM)` on the first stream. One bi-stream per
request, length-prefixed JSON (`net/src/protocol.rs`); `GetFile` is followed by a tar stream.

## Tests

`cargo test --workspace` — unit + propose/discover/executor tests, no network.
`cargo test -p omarchy-onboard-net -- --ignored` — real pairing over local sockets;
occasionally times out on the first run after a cold start (mDNS), passes on rerun.

Manual, one machine: `omarchy-onboard serve --code TEST-2345 &` then
`omarchy-onboard migrate TEST-2345 --yes --dry-run`. Needs macOS Local Network permission
for the process that spawned it (see `~/.claude/CLAUDE.md`).
