//! Developer-authored demo pictograms, bundled as readable SVG files.
//! They contain no company marks, external references, fonts or scripts.
use base64::{Engine as _, engine::general_purpose::STANDARD};

fn svg_data_uri(svg: &str) -> String {
    format!("data:image/svg+xml;base64,{}", STANDARD.encode(svg))
}

/// Profile silhouette for the Avatar image-prop example.
pub(crate) fn avatar_image() -> String {
    svg_data_uri(include_str!("assets/avatar.svg"))
}

/// Four-room floor grid for the application switcher.
pub(crate) fn floor_app_icon() -> String {
    svg_data_uri(include_str!("assets/floor-app.svg"))
}

/// Calculator display and keys for the application switcher.
pub(crate) fn calculator_app_icon() -> String {
    svg_data_uri(include_str!("assets/calculator-app.svg"))
}
