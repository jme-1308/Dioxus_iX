---
name: rust-dioxus-ix
description: Implement, review, and debug Rust/Dioxus components ported from Siemens iX. Use for component behavior, controlled state, events, lifecycle, accessibility, and web/desktop compatibility.
---

Use [AGENTS.md](../../../AGENTS.md) for permanent project rules and version pins. Consult the relevant rows in the [README parity ledger](../../../README.md#parity-ledger), not the entire project history.

## Establish the contract

Inspect the relevant pinned upstream component and its tests. State the missing behavior, controlled/uncontrolled semantics, event order and cancellation, and acceptance criteria. Treat the contract audit as an inventory aid: missing metadata and unreviewed members remain explicit gaps, never inferred parity.

## Implement and integrate

Keep component state and behavior in Rust; reuse existing helpers and dependencies. Guard asynchronous results against supersession and unmount, release owned observers/listeners/tasks, and avoid borrow or lock guards across `.await`. Preserve native form, keyboard, focus, and controlled-value behavior.

Assign agents distinct files or components and name shared read-only files. Require a handoff listing changed behavior, tests, limitations, and frozen files. Integrate focused tests and code review before freezing source; no agent edits after freeze unless a confirmed defect is reassigned.

## Verify and record

Choose checks that cover the changed contract. After integration freezes, run the relevant web/desktop checks together once. Reuse build caches and repeat only invalidated checks after a defect fix. Use Impeccable for relevant visual design/review rather than as a compulsory second implementation workflow.

Update the README ledger with implementation status, separate web and desktop evidence, exact commands/results, and remaining gaps. Label synthetic DOM checks and physical input separately. Keep mapping counts outside completion claims.

At a batch boundary, update the README handoff with active processes, unresolved work, and the exact next action. Honor the single-chat ownership rule before continuing in another chat.
