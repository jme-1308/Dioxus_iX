//! Source-aligned sizing, icon/spinner constraints for [`crate::ToggleButton`], and shared
//! 9-variant selected/active state rules (plus 6-variant pressed token defaults) for
//! `.ix-button.ix-button-tokenized.ix-button-selectable` ([`crate::ToggleButton`] and [`crate::IconToggleButton`]).
//!
//! Loaded after [`crate::button_styles::STYLES`] and [`crate::icon_button_styles::STYLES`]:
//! text-toggle sizing, icon, and spinner constraints stay strictly on `.ix-toggle-button-native`
//! so `.ix-icon-button-native` dimensions never leak, while `.ix-button-selectable` carries
//! the shared selected, `.ix-toggle-active`/`:active` inner-active, and host `.active` rules
//! across all 9 button variants (danger variants intentionally omit default `--pressed*` tokens
//! to match `button.vars.scss` while honoring custom `--ix-button-danger-*--*--pressed*` overrides).

pub(crate) const STYLES: &str = include_str!("styles/toggle_button.css");
