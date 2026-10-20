use base64::{Engine as _, engine::general_purpose::STANDARD};

pub(crate) fn svg_data_uri(svg: &str) -> String {
    format!("data:image/svg+xml;base64,{}", STANDARD.encode(svg))
}
