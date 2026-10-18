//! Theme scope and embedded styles, shared by web and desktop renderers.

use dioxus::prelude::*;

/// Color scheme. System follows the browser or desktop WebView preference.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ColorScheme {
    Light,
    Dark,
    #[default]
    System,
}

impl ColorScheme {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::Dark => "dark",
            Self::System => "system",
        }
    }
}

/// Component density, matching the upstream normal/compact distinction.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Density {
    #[default]
    Normal,
    Compact,
}

impl Density {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Compact => "compact",
        }
    }
}

/// Install the Rust-embedded theme around an application or preview.
///
/// `style` permits consumers with authorized Brand tokens to override the
/// provisional public-source palette without changing any component code.
/// The theme consumes the upstream `--si-sys-*` semantic color and sizing
/// properties, `--si-ref-typography-font-family-sans`, and the
/// `--si-sys-typography-body` font shorthand when supplied on this scope or
/// inherited from an ancestor. The font-family reference is separate from the
/// body shorthand so an explicit `--ix-font` override still takes precedence.
/// Existing `--ix-*` overrides supplied through `style` take precedence over
/// these aliases. Missing upstream tokens retain the provisional defaults.
/// Proprietary fonts and corporate-theme files are not bundled; callers are
/// responsible for their authorization to supply those assets or token values.
///
/// [`crate::Typography`] also consumes the per-format
/// `--si-sys-typography-*` shorthands, semantic text colors and reference
/// letter-spacing/weight tokens at the text element. This permits inherited
/// density-specific tokens and nested scopes without replacing the component.
/// Bundled numeric typography fallbacks remain provisional; selecting compact
/// density does not manufacture a complete Brand typography token set.
#[component]
pub fn IxTheme(
    #[props(default)] scheme: ColorScheme,
    #[props(default)] density: Density,
    #[props(default)] class: String,
    #[props(default)] style: String,
    children: Element,
) -> Element {
    rsx! {
        style { {BASE_STYLES} }
        style { {crate::actions::STYLES} }
        style { {crate::spinner_styles::STYLES} }
        style { {crate::button_styles::STYLES} }
        style { {crate::icon_button_styles::STYLES} }
        style { {crate::toggle_button_styles::STYLES} }
        style { {crate::application::STYLES} }
        style { {crate::header_layout::STYLES} }
        style { {crate::chat::STYLES} }
        style { {crate::charts::STYLES} }
        style { {crate::chart_advanced::STYLES} }
        style { {crate::chart_toolbox::STYLES} }
        style { {crate::data_grid::STYLES} }
        style { {crate::composition::STYLES} }
        style { {crate::datetime::STYLES} }
        style { {crate::dialogs::STYLES} }
        style { {crate::forms::STYLES} }
        style { {crate::checkbox_styles::STYLES} }
        style { {crate::radio_styles::STYLES} }
        style { {crate::toggle_styles::STYLES} }
        style { {crate::group_styles::STYLES} }
        style { {crate::input_styles::STYLES} }
        style { {crate::number_input_styles::STYLES} }
        style { {crate::expanding_search_styles::STYLES} }
        style { {crate::slider_styles::STYLES} }
        style { {crate::select_visual_styles::STYLES} }
        style { {crate::upload_styles::STYLES} }
        style { {crate::layout::STYLES} }
        style { {crate::notifications::STYLES} }
        style { {crate::overlays::STYLES} }
        style { {crate::dropdown::STYLES} }
        style { {crate::patterns::STYLES} }
        style { {crate::tree::STYLES} }
        div {
            class: "ix-theme {class}",
            "data-ix-theme": "brand",
            "data-ix-color-schema": scheme.as_str(),
            "data-ix-density": density.as_str(),
            style,
            {children}
        }
    }
}

// Selected Brand color values from the public iX documentation stylesheet:
// https://ix.siemens.io/assets/css/styles.59cf43b1.css (inspected 2026-10-16).
// This is a semantic subset, not a copy of the corporate theme or a claim of
// complete token parity with the local 5.2.1 component source snapshot.
// Alias names follow the MIT-licensed public token vocabulary in
// packages/core/scss/theme/classic/{light,dark}/_system.scss, plus the public
// density sizing/typography sources. No Classic color values are copied.
// The existing orange Critical status maps to upstream Warning, while our
// yellow Warning status maps to upstream Caution; their names differ.
// These adapters enable authorized externally supplied tokens, not a bundled
// or certified complete Siemens Corporate Brand theme.
pub(crate) const BASE_STYLES: &str = r#"
.ix-theme {
 --ix-font:var(--si-ref-typography-font-family-sans,Arial,Helvetica,sans-serif);
 --ix-radius:var(--si-sys-sizing-border-radius-sm,4px);
 --ix-control-height:var(--si-sys-sizing-size-90,40px); --ix-space:8px;
 --ix-color-primary:var(--si-sys-color-background-accent,#006b80); --ix-color-on-primary:var(--si-sys-color-text-on-accent,#fff);
 --ix-color-secondary:var(--si-sys-color-text-accent,#005159); --ix-color-text:var(--si-sys-color-text-primary,#000028);
 --ix-color-text-secondary:var(--si-sys-color-text-secondary,#4c4c68); --ix-color-surface:var(--si-sys-color-background-0,#f5f5f5);
 --ix-color-surface-1:var(--si-sys-color-background-1,#fff); --ix-color-surface-2:var(--si-sys-color-background-2,#e0e0de);
 --ix-color-border:var(--si-sys-color-border-2,#858599); --ix-color-danger:var(--si-sys-color-background-danger,#d72339);
 --ix-color-danger-text:var(--si-sys-color-text-danger,#a60823); --ix-color-critical:var(--si-sys-color-background-warning,#c75300); --ix-color-on-critical:var(--si-sys-color-text-on-warning,#fff);
 --ix-color-success:var(--si-sys-color-background-success,#1c703f); --ix-color-warning:var(--si-sys-color-background-caution,#ffd00f); --ix-color-warning-text:var(--si-sys-color-text-caution,#876d00);
 --ix-color-info:var(--si-sys-color-background-information,#206ed9); --ix-color-info-text:var(--si-sys-color-text-information,#206ed9); --ix-color-focus:var(--si-sys-color-effects-focus,#006b80);
 --ix-color-on-danger:var(--si-sys-color-text-on-danger,#fff); --ix-color-on-warning:var(--si-sys-color-text-on-caution,#000028);
 --ix-color-on-success:var(--si-sys-color-text-on-success,#fff); --ix-color-on-info:var(--si-sys-color-text-on-information,#fff);
 font:var(--si-sys-typography-body,normal 16px/1.5 var(--ix-font));
 font-family:var(--ix-font);
 color:var(--ix-color-text); background:var(--ix-color-surface);
 accent-color:var(--ix-color-primary); color-scheme:light;
 min-height:100%; box-sizing:border-box;
}
.ix-theme[data-ix-color-schema=dark] {
 --ix-color-primary:var(--si-sys-color-background-accent,#00cccc); --ix-color-on-primary:var(--si-sys-color-text-on-accent,#000028);
 --ix-color-secondary:var(--si-sys-color-text-accent,#00ffb9); --ix-color-text:var(--si-sys-color-text-primary,#fff);
 --ix-color-text-secondary:var(--si-sys-color-text-secondary,#b3b3be); --ix-color-surface:var(--si-sys-color-background-0,#000028);
 --ix-color-surface-1:var(--si-sys-color-background-1,#23233c); --ix-color-surface-2:var(--si-sys-color-background-2,#4c4c68);
 --ix-color-border:var(--si-sys-color-border-2,#737389); --ix-color-danger:var(--si-sys-color-background-danger,#d72339);
 --ix-color-danger-text:var(--si-sys-color-text-danger,#ff8e9c); --ix-color-critical:var(--si-sys-color-background-warning,#ff9000); --ix-color-on-critical:var(--si-sys-color-text-on-warning,#000028);
 --ix-color-success:var(--si-sys-color-background-success,#28bf66); --ix-color-warning:var(--si-sys-color-background-caution,#ffd732); --ix-color-warning-text:var(--si-sys-color-text-caution,#ffd00f);
 --ix-color-info:var(--si-sys-color-background-information,#206ed9); --ix-color-info-text:var(--si-sys-color-text-information,#97c7ff); --ix-color-focus:var(--si-sys-color-effects-focus,#00ffb9);
 --ix-color-on-danger:var(--si-sys-color-text-on-danger,#fff); --ix-color-on-warning:var(--si-sys-color-text-on-caution,#000028);
 --ix-color-on-success:var(--si-sys-color-text-on-success,#000028); --ix-color-on-info:var(--si-sys-color-text-on-information,#fff);
 color-scheme:dark;
}
@media(prefers-color-scheme:dark) {
 .ix-theme[data-ix-color-schema=system] {
 --ix-color-primary:var(--si-sys-color-background-accent,#00cccc); --ix-color-on-primary:var(--si-sys-color-text-on-accent,#000028);
 --ix-color-secondary:var(--si-sys-color-text-accent,#00ffb9); --ix-color-text:var(--si-sys-color-text-primary,#fff);
 --ix-color-text-secondary:var(--si-sys-color-text-secondary,#b3b3be); --ix-color-surface:var(--si-sys-color-background-0,#000028);
 --ix-color-surface-1:var(--si-sys-color-background-1,#23233c); --ix-color-surface-2:var(--si-sys-color-background-2,#4c4c68);
 --ix-color-border:var(--si-sys-color-border-2,#737389); --ix-color-danger:var(--si-sys-color-background-danger,#d72339);
 --ix-color-danger-text:var(--si-sys-color-text-danger,#ff8e9c); --ix-color-critical:var(--si-sys-color-background-warning,#ff9000); --ix-color-on-critical:var(--si-sys-color-text-on-warning,#000028);
 --ix-color-success:var(--si-sys-color-background-success,#28bf66); --ix-color-warning:var(--si-sys-color-background-caution,#ffd732); --ix-color-warning-text:var(--si-sys-color-text-caution,#ffd00f);
 --ix-color-info:var(--si-sys-color-background-information,#206ed9); --ix-color-info-text:var(--si-sys-color-text-information,#97c7ff); --ix-color-focus:var(--si-sys-color-effects-focus,#00ffb9);
 --ix-color-on-danger:var(--si-sys-color-text-on-danger,#fff); --ix-color-on-warning:var(--si-sys-color-text-on-caution,#000028);
 --ix-color-on-success:var(--si-sys-color-text-on-success,#000028); --ix-color-on-info:var(--si-sys-color-text-on-information,#fff); color-scheme:dark;
 }
}
.ix-theme[data-ix-density=compact] { --ix-control-height:var(--si-sys-sizing-size-90,32px); --ix-space:6px; }
.ix-theme *, .ix-theme *::before, .ix-theme *::after { box-sizing:border-box; }
.ix-theme button,.ix-theme input,.ix-theme textarea,.ix-theme select { font:inherit; }
.ix-theme button { cursor:pointer; }
.ix-theme button:disabled,.ix-theme input:disabled,.ix-theme select:disabled { cursor:not-allowed; }
.ix-theme :focus-visible { outline:2px solid var(--ix-color-focus);outline-offset:3px; }
.ix-theme ::selection { color:var(--ix-color-on-primary);background:var(--ix-color-primary); }
.ix-theme input,.ix-theme textarea { caret-color:var(--ix-color-primary); }
.ix-theme a { color:var(--ix-color-primary);text-underline-offset:3px; }
.ix-theme { scrollbar-color:var(--ix-color-border) var(--ix-color-surface); }
.ix-icon { display:inline-block;flex-shrink:0;vertical-align:middle;fill:currentColor; }
.ix-sr-only { position:absolute;width:1px;height:1px;padding:0;margin:-1px;overflow:hidden;clip:rect(0,0,0,0);white-space:nowrap;border:0; }
@media(prefers-reduced-motion:reduce) {
 .ix-theme *,.ix-theme *::before,.ix-theme *::after { animation-duration:.01ms!important;transition-duration:.01ms!important;animation-iteration-count:1!important;scroll-behavior:auto!important; }
}
"#;
