//! Theme scope and embedded styles, shared by web and desktop renderers.

use dioxus::prelude::*;

#[derive(Clone, Copy)]
struct ThemeStyleBundle;

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
/// Nested themes reuse their ancestor's bundled styles while retaining their
/// own scheme, density and token overrides. Independently rooted themes each
/// own a bundle for their full Dioxus lifetime. Independent roots fill their
/// containing block; nested themes size naturally unless `style` overrides it.
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
    // Capture the inherited bundle before providing our own marker. The cached
    // context hook keeps an outer theme from observing itself on later renders.
    let inherited_styles = try_use_context::<ThemeStyleBundle>().is_some();
    use_context_provider(|| ThemeStyleBundle);
    rsx! {
        if !inherited_styles {
            style { {BASE_STYLES} }
            style { {crate::actions::STYLES} }
            style { {crate::spinner_styles::STYLES} }
            style { {crate::progress_styles::STYLES} }
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
        }
        div {
            class: "ix-theme {class}",
            "data-ix-theme": "brand",
            "data-ix-theme-root": (!inherited_styles).to_string(),
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
pub(crate) const BASE_STYLES: &str = include_str!("styles/theme.css");

#[cfg(test)]
mod tests;
