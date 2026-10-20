//! Source-aligned sizing, oval border radius, glyph sizing/color, and loading spinner rules for [`crate::IconButton`].
//!
//! Loaded after `crate::actions::STYLES` and `crate::button_styles::STYLES`, and scoped strictly to
//! `.ix-theme .ix-button.ix-icon-button-native` so shared `.ix-button.ix-button-tokenized` variant state
//! and focus rules apply while legacy `.ix-icon-button` rules remain isolated for unmigrated controls.

pub(crate) const STYLES: &str = include_str!("styles/icon_button.css");
