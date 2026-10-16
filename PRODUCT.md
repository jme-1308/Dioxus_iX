# Dioxus_iX

<!-- impeccable:product-schema 1 -->

## Platform

web

The library must support both Dioxus web and Dioxus desktop applications while retaining the Siemens iX design language. The platform value describes the interface system; it does not limit delivery to the browser. Desktop renderer and supported desktop operating systems remain open decisions. No platform-specific replacement of the iX identity has been requested.

## Stack

- Rust with Dioxus **v0.8.0-alpha.2**, explicitly selected by the user.
- Release reference: https://github.com/DioxusLabs/dioxus/releases/tag/v0.8.0-alpha.2
- Do not silently substitute another Dioxus release.
- Implementation architecture remains undecided: Rust implementations of the components, bindings to upstream Web Components, or a hybrid approach.
- No application scaffold, build command, hosting target, or distribution configuration exists in this project at initialization.

## Users

Dioxus application developers who need reusable Siemens iX components and UI patterns for web and desktop applications. Siemens iX's upstream context is industrial software. Specific downstream industries, end-user roles, and workflows have not yet been defined.

## Product Purpose

Create a Dioxus UI library based on Siemens Industrial Experience (iX), covering a broad range of UI components, UI patterns, and design tools according to Siemens iX guidelines.

The user prefers a **1:1 clone**. Treat close visual and behavioral fidelity to upstream as the target, including component states and interactions, rather than inventing a replacement design system. Actual coverage and parity must be established through implementation and verification; they are not achieved at initialization.

## Positioning

Bring the Siemens iX interface system to the explicitly selected Dioxus release, with a shared library usable in web and desktop applications. Fidelity to iX is the defining requirement.

## Operating Context

- Project: `C:\Users\johan\Documents\code\Dioxus_iX`.
- User-provided upstream checkout: `C:\Users\johan\Documents\code\ix`.
- Upstream repository: https://github.com/siemens/ix
- Upstream documentation entry point: https://ix.siemens.io
- Consult the upstream implementation, examples, patterns, and applicable guidelines when defining component behavior and scope.

## Capabilities and Constraints

- Support Dioxus web and desktop consumers.
- Broad intended scope includes UI components, UI patterns, and design tools; the initial component inventory, implementation order, and exact tooling deliverables remain undecided.
- Preserve upstream terminology and expected component behavior when adapting the interface to Dioxus.
- Web and desktop parity must be verified separately; neither is currently implemented.
- Exact upstream release or commit to target for ongoing parity remains undecided. The checkout snapshot below records inspected evidence, not a user-approved permanent version pin.
- Packaging, public Rust API, third-party integrations, and deployment requirements remain open.

## Brand Commitments

- Use the Siemens iX **Brand** option, as explicitly selected by the user.
- Classic is not the requested design target.
- Follow the selected upstream identity and guidelines. Future design work must preserve that commitment.
- Default light/dark behavior and the exact Brand assets available for reuse have not yet been established.

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
- `patterns/react-patterns/scripts/setup-corporate-theme.mjs` references a separate corporate-theme package and an optional download of `brand-theme.css`. Availability of the complete requested Brand theme for this port remains to be verified.

This project currently contains Impeccable setup files and this product record, with no implemented UI library or runnable preview. The upstream checkout is reference material; initialization does not modify it.

## Product Principles

1. Preserve Siemens iX Brand identity and guidelines across components and patterns.
2. Make fidelity measurable against upstream behavior, states, and visual references.
3. Support both requested delivery targets without assuming success on one proves success on the other.
4. Keep implementation compatible with the user's explicit Dioxus version.
5. Record coverage and open decisions honestly; do not claim unimplemented parity.
