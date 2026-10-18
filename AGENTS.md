# Dioxus iX project rules

Build a good approximation of Siemens iX Brand in Dioxus **=0.8.0-alpha.2** for web and desktop WebView. The read-only upstream is `../ix` at `8bf599bc68365d7752e12832e71b625883b93d7a` (core 5.2.1).

## Acceptance target

- The user replaced the former 1:1 clone requirement with **a good visual approximation and reliable core interactions** on 2026-10-18.
- Prioritize recognizable iX appearance, coherent sizing/spacing/states, useful examples, and common interactions: clicking, typing, selecting, opening/closing, disabled/loading behavior, and basic keyboard/focus accessibility.
- Idiomatic Rust/Dioxus adaptations are acceptable. Full upstream API coverage, identical DOM wrappers, invisible CSS values, upstream quirks, millisecond timing, frame-by-frame event chronology, and exhaustive source-member audits are not acceptance requirements.
- Use upstream as a reference, not an exact-equivalence oracle. Verify representative visible states and normal workflows. Investigate deeper only for a reproduced user-facing defect, a regression, or a specific user request.
- Keep existing useful functionality and tests. Avoid adding large comparison matrices, timing trackers, or audit fan-outs. Once a capability works and looks right, proceed to broader component coverage.

## Scope

- Use Brand, not Classic. Exclude deprecated APIs, Siemens marks, and separately licensed corporate fonts/assets; preserve MIT notices.
- Keep behavior, state, styles, examples, and assets in Rust. Embedded JavaScript is limited to native DOM plumbing. Development tooling stays outside the published crate.
- Nightly settings remain local and gitignored. Ask before changing profiles, toolchains, linkers, or pinned Dioxus versions; never run `cargo clean`.

## Records and tools

- Track remaining work in [IMPLEMENTATION.md](IMPLEMENTATION.md), not README. The [parity ledger](IMPLEMENTATION.md#parity-ledger) is authoritative; the [handoff](IMPLEMENTATION.md#handoff) records the latest boundary and next action. Historical parity gaps are reference material, not automatic blockers for the current approximation target. Compilation alone does not establish usable behavior or appearance.
- Use the [Rust/Dioxus skill](.agents/skills/rust-dioxus-ix/SKILL.md) for component work and Impeccable for UI design or visual review.
- Use rust-analyzer MCP tools when present; call `symbols` before positional tools when coordinates are unknown. Fall back immediately when the tools are absent.
- `just --list` is the command index. Use `check-lib` while editing and run `lint`, `test`, and the full `check` before handoff; the Stop hook enforces the final check.

## Batch workflow

- Only one chat owns an active batch. Define the visible capability and core interaction acceptance criteria before editing.
- Delegations include exclusive file ownership, upstream paths, known findings, and acceptance criteria. Review focused tests before freezing handed-off files.
- After the freeze, run a bounded representative web/native pass, update changed ledger evidence and limitations, then verify packaging. Reopen only confirmed defects and rerun only invalidated checks.
