//! Source-aligned sizing, focus, layout, and 9-variant state token styles for the migrated Button family.
//!
//! Scoped strictly to `.ix-theme .ix-button.ix-button-tokenized` so legacy `.ix-button` rules
//! in `src/actions.rs` continue to govern sibling controls (`SplitButton` and dropdown triggers) until their own slices are migrated.

pub(crate) const STYLES: &str = include_str!("styles/button.css");
