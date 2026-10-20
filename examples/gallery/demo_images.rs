//! Developer-authored demo pictograms, bundled as readable SVG Rust data.
//! They contain no company marks, external references, fonts or scripts.
use base64::{Engine as _, engine::general_purpose::STANDARD};

fn svg_data_uri(svg: &str) -> String {
    format!("data:image/svg+xml;base64,{}", STANDARD.encode(svg))
}

/// Profile silhouette for the Avatar image-prop example.
pub(crate) fn avatar_image() -> String {
    svg_data_uri(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64" viewBox="0 0 64 64">
  <rect width="64" height="64" fill="#000028"/>
  <circle cx="32" cy="23" r="10" fill="#00cccc"/>
  <path d="M12 60v-3a20 20 0 0 1 40 0v3z" fill="#00cccc"/>
</svg>"##,
    )
}

/// Four-room floor grid for the application switcher.
pub(crate) fn floor_app_icon() -> String {
    svg_data_uri(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64" viewBox="0 0 64 64">
  <rect x="2" y="2" width="60" height="60" rx="10" fill="#000028"/>
  <path d="M14 14h36v36H14zM14 32h36M32 14v36" fill="none" stroke="#00cccc" stroke-width="4" stroke-linejoin="round"/>
  <circle cx="23" cy="23" r="3" fill="#f2ffff"/>
</svg>"##,
    )
}

/// Calculator display and keys for the application switcher.
pub(crate) fn calculator_app_icon() -> String {
    svg_data_uri(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64" viewBox="0 0 64 64">
  <rect x="2" y="2" width="60" height="60" rx="10" fill="#000028"/>
  <rect x="15" y="9" width="34" height="46" rx="4" fill="#00cccc"/>
  <rect x="20" y="15" width="24" height="10" rx="1" fill="#000028"/>
  <path d="M25 20h14" stroke="#f2ffff" stroke-width="2"/>
  <path d="M20 31h6v6h-6zM29 31h6v6h-6zM38 31h6v6h-6zM20 41h6v6h-6zM29 41h6v6h-6zM38 41h6v6h-6z" fill="#000028"/>
</svg>"##,
    )
}
