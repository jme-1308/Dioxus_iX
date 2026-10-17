# Dioxus iX project rules

Port Siemens iX Brand to Dioxus **=0.8.0-alpha.2** for web and desktop WebView. The read-only upstream is `../ix` at `8bf599bc68365d7752e12832e71b625883b93d7a` (core 5.2.1).

## Scope

- Use Brand, not Classic. Exclude deprecated APIs, Siemens marks, and separately licensed corporate fonts/assets; preserve MIT notices.
- Keep behavior, state, styles, examples, and assets in Rust. Embedded JavaScript is limited to native DOM plumbing. Development tooling stays outside the published crate.
- Nightly settings remain local and gitignored. Ask before changing profiles, toolchains, linkers, or pinned Dioxus versions; never run `cargo clean`.

## Records and tools

- Track remaining work in [IMPLEMENTATION.md](IMPLEMENTATION.md), not README. The [parity ledger](IMPLEMENTATION.md#parity-ledger) is authoritative; the [handoff](IMPLEMENTATION.md#handoff) records the latest boundary and next action. Mappings, fixtures, compilation, and synthetic events are not parity proof.
- Use the [Rust/Dioxus skill](.agents/skills/rust-dioxus-ix/SKILL.md) for component work and Impeccable for UI design or visual review.
- Use rust-analyzer MCP tools when present; call `symbols` before positional tools when coordinates are unknown. Fall back immediately when the tools are absent.
- `just --list` is the command index. Use `check-lib` while editing and run `lint`, `test`, and the full `check` before handoff; the Stop hook enforces the final check.

## Batch workflow

- Only one chat owns an active batch. Establish the upstream contract and acceptance criteria before editing.
- Delegations include exclusive file ownership, upstream paths, known findings, and acceptance criteria. Review focused tests before freezing handed-off files.
- After the freeze, run one coordinated web/native pass, update changed ledger evidence and limitations, then verify packaging. Reopen only confirmed defects and rerun only invalidated checks.
