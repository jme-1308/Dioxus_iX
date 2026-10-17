//! Export a mounted chart's SVG plot through the host's native image/canvas APIs.
//!
//! The small DOM bridge is embedded in Rust and works through Dioxus document
//! providers on both web and desktop. Chart state and geometry remain Rust.
use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde::{Deserialize, Serialize};
use std::fmt;

const PNG_PREFIX: &str = "data:image/png;base64,";
const MAX_PIXELS: u64 = 16_777_216;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChartExportError {
    InvalidScale,
    InvalidPng,
    Renderer(String),
}
impl fmt::Display for ChartExportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidScale => {
                f.write_str("Choose a finite image scale greater than zero and at most four.")
            }
            Self::InvalidPng => f.write_str("The renderer did not return a valid PNG image."),
            Self::Renderer(message) => f.write_str(message),
        }
    }
}
impl std::error::Error for ChartExportError {}

/// A PNG of the current plot, with resolved theme colors and installed fonts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChartPng {
    pub data_url: String,
    pub width: u32,
    pub height: u32,
}
impl ChartPng {
    /// Decode the browser's PNG payload and verify its signature and dimensions.
    pub fn bytes(&self) -> Result<Vec<u8>, ChartExportError> {
        if self.width == 0
            || self.height == 0
            || u64::from(self.width) * u64::from(self.height) > MAX_PIXELS
        {
            return Err(ChartExportError::InvalidPng);
        }
        let encoded = self
            .data_url
            .strip_prefix(PNG_PREFIX)
            .ok_or(ChartExportError::InvalidPng)?;
        if encoded.len() > (MAX_PIXELS as usize * 4 + 1_048_576) * 4 / 3 {
            return Err(ChartExportError::InvalidPng);
        }
        let bytes = STANDARD
            .decode(encoded)
            .map_err(|_| ChartExportError::InvalidPng)?;
        if bytes.len() < 33
            || bytes[..8] != *b"\x89PNG\r\n\x1a\n"
            || bytes[12..16] != *b"IHDR"
            || u32::from_be_bytes(bytes[16..20].try_into().unwrap()) != self.width
            || u32::from_be_bytes(bytes[20..24].try_into().unwrap()) != self.height
        {
            return Err(ChartExportError::InvalidPng);
        }
        Ok(bytes)
    }

    /// Request the host browser/WebView's normal PNG download flow.
    /// Call from an explicit user action; a host may apply its download policy.
    /// Success means the request was dispatched, not that a file was saved.
    pub async fn download(&self, file_name: &str) -> Result<(), ChartExportError> {
        self.bytes()?;
        #[derive(Serialize)]
        struct Download<'a> {
            data_url: &'a str,
            file_name: String,
        }
        let task = crate::dom_query::eval(DOWNLOAD);
        task.send(Download {
            data_url: &self.data_url,
            file_name: png_file_name(file_name),
        })
        .map_err(renderer_error)?;
        let result: BridgeResult = task.join().await.map_err(renderer_error)?;
        result.into_result()
    }
}

fn png_file_name(file_name: &str) -> String {
    let clean: String = file_name
        .chars()
        .filter(|c| {
            !c.is_control() && !matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|')
        })
        .take(180)
        .collect();
    let clean = clean.trim().trim_matches('.');
    let clean = if clean.is_empty() { "chart" } else { clean };
    if clean.to_ascii_lowercase().ends_with(".png") {
        clean.into()
    } else {
        format!("{clean}.png")
    }
}

fn renderer_error(error: impl fmt::Display) -> ChartExportError {
    ChartExportError::Renderer(error.to_string())
}

#[derive(Deserialize)]
struct BridgeResult {
    error: Option<String>,
}
impl BridgeResult {
    fn into_result(self) -> Result<(), ChartExportError> {
        self.error
            .map_or(Ok(()), |error| Err(ChartExportError::Renderer(error)))
    }
}

/// Capture the first SVG inside a mounted element (or that element itself).
/// Pixel scale applies to the PNG, while the plot's layout remains unchanged.
/// Temporary selection overlays and HTML controls/tooltips are excluded.
pub async fn export_chart_png(
    element_id: &str,
    pixel_ratio: f64,
) -> Result<ChartPng, ChartExportError> {
    if !pixel_ratio.is_finite() || pixel_ratio <= 0.0 || pixel_ratio > 4.0 {
        return Err(ChartExportError::InvalidScale);
    }
    #[derive(Serialize)]
    struct Capture<'a> {
        element_id: &'a str,
        pixel_ratio: f64,
    }
    #[derive(Deserialize)]
    struct CaptureResult {
        data_url: Option<String>,
        width: Option<u32>,
        height: Option<u32>,
        error: Option<String>,
    }
    let task = crate::dom_query::eval(CAPTURE);
    task.send(Capture {
        element_id,
        pixel_ratio,
    })
    .map_err(renderer_error)?;
    let result: CaptureResult = task.join().await.map_err(renderer_error)?;
    if let Some(error) = result.error {
        return Err(ChartExportError::Renderer(error));
    }
    let png = ChartPng {
        data_url: result.data_url.ok_or(ChartExportError::InvalidPng)?,
        width: result.width.ok_or(ChartExportError::InvalidPng)?,
        height: result.height.ok_or(ChartExportError::InvalidPng)?,
    };
    png.bytes()?;
    Ok(png)
}

/// Retain a chart pointer drag when the pointer leaves its SVG.
pub async fn capture_chart_pointer(
    element_id: &str,
    pointer_id: i32,
) -> Result<(), ChartExportError> {
    #[derive(Serialize)]
    struct CapturePointer<'a> {
        element_id: &'a str,
        pointer_id: i32,
    }
    let task = crate::dom_query::eval(CAPTURE_POINTER);
    task.send(CapturePointer {
        element_id,
        pointer_id,
    })
    .map_err(renderer_error)?;
    let result: BridgeResult = task.join().await.map_err(renderer_error)?;
    result.into_result()
}

// User-authorized DOM plumbing only: read resolved SVG presentation, rasterize
// with the platform's installed fonts, and return data over Dioxus' typed channel.
// IDs and filenames travel as serialized data, never interpolated executable code.
const CAPTURE: &str = r#"
const {element_id, pixel_ratio} = await dioxus.recv();
let url;
try {
  const host = document.getElementById(element_id);
  const svg = host?.localName === 'svg' ? host : host?.querySelector('svg');
  if (!svg) throw new Error('Mount the chart before exporting an image.');
  const bounds = svg.getBoundingClientRect();
  const width = Math.round(bounds.width * pixel_ratio), height = Math.round(bounds.height * pixel_ratio);
  if (width < 1 || height < 1 || width * height > 16777216) throw new Error('The chart is hidden or its image dimensions are too large.');
  const copy = svg.cloneNode(true), originals = [svg, ...svg.querySelectorAll('*')], copies = [copy, ...copy.querySelectorAll('*')];
  const properties = ['fill','fill-opacity','stroke','stroke-width','stroke-opacity','stroke-dasharray','stroke-linecap','stroke-linejoin','font-family','font-size','font-weight','font-style','text-anchor','dominant-baseline','opacity','visibility','display','color','paint-order','stop-color','stop-opacity','letter-spacing','word-spacing','shape-rendering'];
  originals.forEach((node, index) => {
    const style = getComputedStyle(node);
    properties.forEach(property => {
      let value = style.getPropertyValue(property);
      if (value.startsWith('url(')) { const fragment = value.match(/#([^\)"']+)/); if (fragment) value = `url(#${fragment[1]})`; }
      copies[index].style.setProperty(property, value);
    });
    copies[index].removeAttribute('class');
  });
  copy.querySelectorAll('[data-chart-export-exclude]').forEach(node => node.remove());
  copy.setAttribute('xmlns', 'http://www.w3.org/2000/svg');
  copy.setAttribute('width', String(bounds.width)); copy.setAttribute('height', String(bounds.height));
  const source = new XMLSerializer().serializeToString(copy);
  url = URL.createObjectURL(new Blob([source], {type:'image/svg+xml;charset=utf-8'}));
  const image = new Image(); image.src = url; await image.decode();
  const canvas = document.createElement('canvas'); canvas.width = width; canvas.height = height;
  const context = canvas.getContext('2d'); if (!context) throw new Error('This renderer cannot create PNG images.');
  context.drawImage(image, 0, 0, width, height);
  return {data_url:canvas.toDataURL('image/png'), width, height};
} catch (error) { return {error:String(error?.message || error)}; }
finally { if (url) URL.revokeObjectURL(url); }
"#;

const DOWNLOAD: &str = r#"
const {data_url, file_name} = await dioxus.recv();
try {
  const link = document.createElement('a'); link.href = data_url; link.download = file_name;
  link.hidden = true; document.body.append(link); link.click(); link.remove(); return {};
} catch (error) { return {error:String(error?.message || error)}; }
"#;

const CAPTURE_POINTER: &str = r#"
const {element_id, pointer_id} = await dioxus.recv();
try {
  const node = document.getElementById(element_id);
  if (!node) throw new Error('The chart is no longer mounted.');
  node.setPointerCapture(pointer_id); return {};
} catch (error) { return {error:String(error?.message || error)}; }
"#;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn decoded_image_checks_real_dimensions_and_rejects_mismatched_or_non_png_data() {
        let mut png = ChartPng { data_url: "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+jRZkAAAAASUVORK5CYII=".into(), width:1, height:1 };
        assert!(png.bytes().unwrap().starts_with(b"\x89PNG"));
        png.width = 2;
        assert_eq!(png.bytes(), Err(ChartExportError::InvalidPng));
        png.width = 1;
        png.data_url = "data:image/svg+xml,<svg/>".into();
        assert_eq!(png.bytes(), Err(ChartExportError::InvalidPng));
    }
    #[test]
    fn export_file_names_are_plain_png_names_without_paths_or_control_characters() {
        assert_eq!(png_file_name("pump/flow\n"), "pumpflow.png");
        assert_eq!(png_file_name(".."), "chart.png");
        assert_eq!(png_file_name("Chart.PNG"), "Chart.PNG");
    }
}
