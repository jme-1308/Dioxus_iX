# Dioxus iX project rules

Build the Siemens iX Brand component library for Dioxus **=0.8.0-alpha.2**, supporting web and the Dioxus desktop WebView renderer. The read-only reference is `../ix`, pinned to `8bf599bc68365d7752e12832e71b625883b93d7a` (core 5.2.1).

Keep implementation, styles, examples, and assets in Rust source. Small JavaScript strings embedded in Rust may provide native DOM activation, focus, measurement, observation, pointer capture, and export plumbing; component behavior and application state stay in Rust. Cargo files, README, license notices, .gitignore, this file, and project-local skill instructions are approved exceptions. Development tooling stays outside the published crate's explicit inclusion list.

Use Brand, not Classic. Exclude deprecated/legacy components and examples, Siemens logos/wordmarks, and separately licensed corporate fonts/assets. Keep required MIT notices intact. Supporting data types used by current components are allowed.

## Working records

- [README parity ledger](README.md#parity-ledger) is the authoritative status record: upstream requirement, implementation, web evidence, desktop evidence, and remaining gap. Mapping counts and fixture availability are separate from parity.
- [README handoff](README.md#handoff) records the latest batch boundary, processes, unresolved work, and next action.
- Use the project-local [rust-dioxus-ix skill](.agents/skills/rust-dioxus-ix/SKILL.md) for component porting. Use Impeccable when UI design or visual review is relevant; do not restart its setup within an existing session.

## Batch discipline

Establish the upstream contract and acceptance criteria before editing. Give each agent explicit, distinct file/component ownership. Integrate handoffs, run focused tests and code review, then declare an integration freeze. Agents stop editing after handing off frozen files.

After the freeze, perform one coordinated web/native verification pass, update the ledger with exact commands/results and limitations, then verify packaging. Reopen only affected files for a confirmed defect and repeat only checks whose evidence changed. Preserve incremental caches; do not routinely clean builds.

Keep synthetic DOM events distinct from physical operating-system input. An implemented API, successful compilation, a fixture, or a green test suite does not prove complete parity.

Only one chat may own an active batch in this checkout. Pause the old goal and finish its handoff before another chat starts changes here.
