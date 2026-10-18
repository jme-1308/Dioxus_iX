---
name: rust-dioxus-ix
description: Implement, review, and debug Rust/Dioxus components ported from Siemens iX. Use for component behavior, controlled state, events, lifecycle, accessibility, and web/desktop compatibility.
---

Follow [AGENTS.md](../../../AGENTS.md) and consult only the relevant [parity ledger](../../../IMPLEMENTATION.md#parity-ledger) rows.

## Priority and implementation choice

The repository's Dioxus-first JavaScript simplification policy takes precedence over other backlog work and historical parity requirements, including for delegated agents. Check the pinned Dioxus source and use its built-in mounted-element, focus, measurement, event and input/file APIs before writing or retaining custom JavaScript. Require a specific demonstrated API gap for each unavoidable bridge; generic DOM or cross-platform arguments are insufficient. Do not retain JavaScript fallback paths: custom content uses Dioxus composition or explicit mounted-element registration, and unsupported capabilities are reported rather than silently routed to JavaScript. Preserve the intended upstream functional breadth through idiomatic Dioxus replacements; ordinary workflows are the verification baseline, not a feature ceiling. Record unsupported capabilities explicitly. Prefer removing redundant infrastructure without dropping useful functionality. See AGENTS.md and R02 in IMPLEMENTATION.md.

## Contract

Read the pinned upstream component and tests before editing. State observable behavior, controlled state, event order/cancellation, and acceptance criteria; keep unknown or unreviewed members explicit.

Use rust-analyzer MCP for semantic lookups when available. Positions are zero-based: when unknown, call `symbols`, then place hover or definition on the declaration identifier (ranges may start at docs or attributes). Use references only when text search is ambiguous or cross-module. The project already configures the workspace and `web` feature.

## Implementation

Keep behavior and application state in Rust. Preserve form, keyboard, focus, and controlled-value semantics. Cancel stale async work, release owned resources, and do not hold borrows or locks across `.await`.

Give delegated tasks their established contract and exclusive files. Handoffs state behavior, tests, limitations, and frozen files; reopen only confirmed defects.

## Evidence

Run focused tests, review, then freeze. Run relevant web/desktop checks once after integration, preserve caches, and repeat only invalidated checks. Update changed ledger evidence and the batch handoff; distinguish synthetic DOM evidence from physical input.
