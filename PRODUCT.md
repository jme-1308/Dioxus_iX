# Dioxus_iX

<!-- impeccable:product-schema 1 -->

## Platform

web

The library must support both Dioxus web and Dioxus desktop applications while retaining the Siemens iX design language. The platform value describes the interface system; it does not limit delivery to the browser. Desktop uses a WebView; broader OS coverage remains to be established. No platform-specific replacement of the iX identity has been requested.

## Stack

- Rust with Dioxus **v0.8.0-alpha.2**, explicitly selected by the user.
- Release reference: https://github.com/DioxusLabs/dioxus/releases/tag/v0.8.0-alpha.2
- Do not silently substitute another Dioxus release.
- Native Rust/Dioxus components and Rust-embedded styles/assets; no upstream Web Component runtime. Dioxus owns normal DOM/lifecycle/event work. Small custom JavaScript bindings are permitted only for demonstrated missing or broken framework capabilities. Existing substantial bridges are the first simplification priority.
- A library and interactive gallery exist. `just --list` is the command index; preserve build caches and the existing dependency/toolchain configuration.

## Users

Dioxus application developers who need reusable Siemens iX components and UI patterns for web and desktop applications. Siemens iX's upstream context is industrial software. Specific downstream industries, end-user roles, and workflows have not yet been defined.

## Product Purpose

Create a Dioxus UI library based on Siemens Industrial Experience (iX), covering a broad range of UI components, UI patterns, and design tools according to Siemens iX guidelines.

The user clarified the goal on **2026-10-18**: build **what Siemens iX should be if implemented for Dioxus**, aiming to retain all nondeprecated functionality. Preserve the recognizable Brand design language and coherent states, while using idiomatic Rust/Dioxus implementation and good visual approximation.

Functional coverage should remain broad; common workflows are the first implementation and verification priority, not the final feature limit. The baseline includes: clicking, typing, selection, opening and closing surfaces, disabled/loading states, and basic keyboard/focus accessibility. Components should look convincing in representative desktop/narrow layouts and light/dark themes. Useful examples should make those capabilities easy to try.

Exact DOM structure, identical JavaScript/custom-element API syntax, invisible computed-style equality, upstream bugs, millisecond timing and frame-by-frame event chronology are **outside the acceptance target**. Missing user-facing capabilities remain recorded work; simplifying implementation does not authorize dropping features. Existing detailed comparisons remain historical reference. Further deep investigation needs a concrete user-facing defect, a regression, or an explicit request; it must not displace broader component coverage.

## Positioning

Bring the Siemens iX interface system to the explicitly selected Dioxus release, with a shared library usable in web and desktop applications. Recognizable iX appearance, practical functionality, and broad component coverage define success.

## Operating Context

- Project: `/run/media/johan/storage/Dioxus_iX`.
- User-provided upstream checkout: `/run/media/johan/storage/ix`.
- Upstream repository: https://github.com/siemens/ix
- Upstream documentation entry point: https://ix.siemens.io
- Consult the upstream implementation, examples, patterns, and applicable guidelines when defining component behavior and scope.

## Capabilities and Constraints

- Support Dioxus web and desktop consumers.
- No JavaScript fallback paths, across every component, task and agent. Use Dioxus composition and explicit mounted-element registration; report unsupported capabilities instead of silently routing them to JavaScript.
- Keep authored implementation in Rust source files. Use Dioxus APIs for rendering, lifecycle, events, focus, measurements and native inputs/files first. A custom JavaScript binding requires a specific demonstrated framework gap, must remain small, and must be documented. Simplifying existing oversized bridges takes precedence over other tasks; preserve functionality while replacing their implementation. Do not introduce standalone JavaScript or TypeScript runtime files.
- Broad intended scope includes components, patterns, and useful examples. IMPLEMENTATION.md records current capability coverage and priorities.
- Preserve familiar terminology and interaction capabilities. Adapt APIs to Rust/Dioxus rather than copying JavaScript syntax; track unsupported functionality explicitly. A focused rewrite of overbuilt subsystems is acceptable while retaining working components, styles/assets, public behavior and useful tests.
- Check representative web and desktop workflows separately. Do not infer usability from compilation alone.
- Read-only reference: core 5.2.1 at 8bf599bc68365d7752e12832e71b625883b93d7a. This provides stable design/behavior guidance; exact runtime equivalence is not required.
- Preserve MIT notices and existing packaging boundaries. Network/services and separately licensed assets remain outside the bundled library.

## Brand Commitments

- Use the Siemens iX **Brand** option, as explicitly selected by the user.
- Classic is not the requested design target.
- Follow the selected upstream identity and guidelines. Future design work must preserve that commitment.
- Light/dark and normal/compact scopes exist. The user has no complete authorized Brand token stylesheet; use the established provisional appearance without treating exact Brand metrics as a blocker.

## Evidence on Hand

The upstream checkout was inspected during initialization:

- Commit: `8bf599bc68365d7752e12832e71b625883b93d7a`.
- `packages/core/package.json` reports `@siemens/ix` version `5.2.1`.
- `README.md` describes Siemens iX's purpose and links to its documentation.
- `ARCHITECTURE.md` identifies `packages/core` as the Stencil Web Component source and explains framework wrappers, patterns, examples, and testing.
- `packages/core/src/components`: component implementations and behavior references.
- `packages/core/scss`: styling references; the inspected theme directory contains `classic` and `core`.
- `examples` and `patterns`: usage examples and higher-level compositions.
- `testing/visual-testing`: visual regression tests and screenshot fixtures. Verify theme and freshness before using an individual fixture as visual authority.
- `patterns/react-patterns/scripts/setup-corporate-theme.mjs` references a separate corporate-theme package and an optional download of `brand-theme.css`. The user confirmed that no complete authorized Brand stylesheet is available.

The project now contains a Rust component library and runnable gallery. The upstream checkout remains read-only reference material.

## Product Principles

For future landing pages and showcases, the user selected a hybrid workflow: use visual exploration when it helps establish the composition, then implement and refine in code. Library components, application screens, and documentation use code-first work within the established iX identity.

1. Preserve Siemens iX Brand identity and guidelines across components and patterns.
2. Check representative appearance and core workflows; prioritize user-visible quality over microscopic equivalence.
3. Support both requested delivery targets without assuming success on one proves success on the other.
4. Keep implementation compatible with the user's explicit Dioxus version.
5. Record supported capabilities and meaningful limitations honestly. Ship completed capabilities and move on; do not pursue millisecond timing or internal-equivalence audits.
