# omarchy-onboard — Omarchy migration assistant

Rust workspace. Toolchain and tasks via `mise` (`mise run build|test|lint|scan|plan`).

## Phases → crates

| Phase | Runs on | Crate | Core trait |
|---|---|---|---|
| Discover | source (Mac) | `omarchy-onboard-checks` | `Check` → `Vec<Finding>` |
| Propose | target (Omarchy) | `omarchy-onboard-rules` | `Rule` → `Vec<Proposal>` |
| Migrate | target | `omarchy-onboard` (cli) | executor over `Operation` |

`omarchy-onboard-core` holds the model and is platform-free. Adding a check = new module in
`crates/checks/src/<platform>/` + register in `all()`. Adding a rule = new module in
`crates/rules/src/` + register in `all()`. Package/app mappings live in
`crates/rules/src/maps/*.toml`, not in Rust.

## Principles

- Findings are facts, never actions. Proposals carry the *semantically equivalent*
  operation (install the package, not copy its files). `PullFiles` is only for user-owned data.
- Every proposal is traceable to its findings (`Proposal::findings`) and has a default
  decision; users accept/skip per item or per `Group`.
- Checks are gated by `CheckMeta::platforms`, not `cfg`, so the catalogue is visible everywhere.
