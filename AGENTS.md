# Dioxus iX project rules

Build the Dioxus-native counterpart of Siemens iX Brand, aiming to retain its functionality with a good visual approximation, in Dioxus **=0.8.0-alpha.2** for web and desktop WebView. The read-only upstream is `../ix` at `8bf599bc68365d7752e12832e71b625883b93d7a` (core 5.2.1).

## First priority: simplify custom JavaScript; use Dioxus first

This applies to **every task, agent and delegation**. Simplifying the existing handwritten JavaScript layer is the highest priority, before new components, gallery improvements, visual polish, other backlog tasks or further parity audits. See **R02** in IMPLEMENTATION.md. Keep unfinished unrelated work preserved; do not continue it ahead of simplification.

- Dioxus must own rendering, element lifecycle and normal event/DOM handling. Do not build a parallel handwritten DOM-management runtime. Custom JavaScript is a **small exception only when Dioxus has no usable alternative**.
- Do not introduce or retain JavaScript fallback paths. Custom consumer content must use Dioxus composition or explicit Rust/mounted-element registration; missing registration and legacy arbitrary-DOM compatibility do not justify a JavaScript fallback. Report an unsupported capability instead of silently switching to JavaScript.
- Use the pinned Dioxus APIs, mounted element handles, events, focus/measurement methods, native input/file facilities and Rust component ownership first. Do not duplicate capabilities that Dioxus already provides with handwritten JavaScript.
- Before introducing or retaining a custom bridge, identify the **specific missing or demonstrably broken Dioxus capability** on the relevant target. Check the installed/pinned source or reproduce the gap. Generic statements that browser operations need the DOM, or that web and desktop differ, are not sufficient justification.
- Remove redundant bridges and simplify the existing menu/overlay/focus/measurement infrastructure. Prefer Rust-owned state, explicit component relationships and Dioxus event handling over custom DOM registries, event transports, discovery observers and queued focus runtimes where the agreed core behavior permits it.
- Keep unavoidable JavaScript minimal and limited to the demonstrated native capability gap. Document that gap beside the bridge and in the current handoff. Moving JavaScript into a `.rs` string does not make it Rust or reduce its maintenance cost.
- Preserve normal clicks, selection, keyboard navigation, dismissal, focus, file handling and cleanup. Historical exact timing, DOM identity, source quirks and unusual-composition equivalence do not justify retaining complexity by themselves. Verify representative workflows; do not replace simplification with another exhaustive audit.

## Acceptance target

- The target is **how Siemens iX should work if built for Dioxus**: recognizable Brand appearance, idiomatic Dioxus implementation and an attempt to retain all nondeprecated functionality. Approximation concerns visual/internal equivalence; it is not a core-only feature ceiling.
- Prioritize recognizable iX appearance, coherent sizing/spacing/states, useful examples, and common interactions: clicking, typing, selecting, opening/closing, disabled/loading behavior, and basic keyboard/focus accessibility.
- Preserve user-facing capabilities and useful options through idiomatic Rust props, events and composition. Identical JavaScript/custom-element APIs, DOM wrappers, invisible CSS values, source quirks, millisecond timing, frame-by-frame event chronology and exhaustive source-member audits are not required. Record missing functionality explicitly instead of silently excluding it.
- Use upstream as a reference, not an exact-equivalence oracle. Verify representative visible states and normal workflows. Investigate deeper only for a reproduced user-facing defect, a regression, or a specific user request.
- Simplify or replace implementation machinery without dropping useful functionality or tests. A focused subsystem rewrite is acceptable; a whole-library rewrite is not assumed. Avoid adding large comparison matrices, timing trackers, or audit fan-outs. Once a capability works and looks right, proceed to broader component coverage.

## Scope

- Use Brand, not Classic. Exclude deprecated APIs, Siemens marks, and separately licensed corporate fonts/assets; preserve MIT notices.
- Keep behavior, state and examples in Rust. Styles and SVG assets may live in separate files bundled with the crate and embedded at compile time where appropriate; consuming applications must not need manual asset copying. Embedded JavaScript is limited to demonstrated native capability gaps. Development tooling stays outside the published crate.
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
