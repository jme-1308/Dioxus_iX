//! One-shot focus ownership facts for the current Rust-owned Category host.
//!
//! Pinned dioxus-html =0.8.0-alpha.2 MountedData (events/mounted.rs) supplies
//! focus/measurement but no active-element/containment query. FocusData
//! (events/focus.rs) has no target/relatedTarget, and PointerData has no related
//! target across renderers. Rust uses this bounded fact after focusout; its mounted owner/generation guards authorize all resulting actions.
//! No scope discovery, retained nodes, listeners or focus execution occurs here.
#[cfg(any(feature = "web", feature = "desktop"))]
pub(crate) const CATEGORY_FOCUS_CONTAINMENT: &str = r#"
const host=document.getElementById(await dioxus.recv());
const connected=host instanceof HTMLElement&&host.isConnected&&host.hasAttribute('data-menu-category-root');
return {connected,inside:connected&&host.contains(document.activeElement)};
"#;
