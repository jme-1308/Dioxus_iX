//! Interactive orthographic surfaces and sampled time-series plots in Rust.
//!
//! Surface geometry/projection/lighting are original Rust calculations. Factual
//! defaults were checked against the published echarts-gl 2.0.9 Grid3DModel,
//! componentViewControlMixin, SurfaceView, Grid3DView, and SurfaceSeries sources; palette stops come
//! from iX's public MIT Brand themes. This does not embed or translate a GL
//! engine or its shader implementation.
use crate::{ChartScale, chart_color, chart_line_path};
use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;
use std::{
    cell::Cell,
    rc::Rc,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_ADVANCED_CHART: AtomicU64 = AtomicU64::new(0);

/// Coordinates always refer to the SVG viewport, including when an event was
/// dispatched by a child polygon, path, or overview handle.
#[derive(Clone, Copy, Debug, PartialEq)]
struct SvgBounds {
    left: f64,
    top: f64,
    width: f64,
    height: f64,
    view_width: f64,
    view_height: f64,
}
impl SvgBounds {
    fn from_rect(
        rect: dioxus::html::geometry::PixelsRect,
        (view_width, view_height): (f64, f64),
    ) -> Option<Self> {
        ([
            rect.origin.x,
            rect.origin.y,
            rect.size.width,
            rect.size.height,
            view_width,
            view_height,
        ]
        .into_iter()
        .all(f64::is_finite)
            && rect.size.width > 0.0
            && rect.size.height > 0.0
            && view_width > 0.0
            && view_height > 0.0)
            .then_some(Self {
                left: rect.origin.x,
                top: rect.origin.y,
                width: rect.size.width,
                height: rect.size.height,
                view_width,
                view_height,
            })
    }
    fn point(self, client: (f64, f64)) -> (f64, f64) {
        (
            (client.0 - self.left) * self.view_width / self.width,
            (client.1 - self.top) * self.view_height / self.height,
        )
    }
}

// Pinned Dioxus 0.8.0-alpha.2 desktop MountedData::get_client_rect only accepts
// HTMLElement. Each chart mounts a dedicated borderless, paddingless HTML div
// immediately enclosing its block SVG so Dioxus measures the rendered SVG
// viewport directly without a JavaScript geometry fallback.
async fn mounted_svg_bounds(node: &MountedData, view_size: (f64, f64)) -> Option<SvgBounds> {
    let rect = node.get_client_rect().await.ok()?;
    SvgBounds::from_rect(rect, view_size)
}

#[derive(Clone, Copy)]
struct ChartDrag<K> {
    pointer: i32,
    start: (f64, f64),
    current: (f64, f64),
    bounds: Option<SvgBounds>,
    kind: K,
}
impl<K: Copy> ChartDrag<K> {
    fn move_pointer(&mut self, pointer: i32, client: (f64, f64)) -> bool {
        if self.pointer != pointer {
            return false;
        }
        self.current = client;
        true
    }
    fn points(self) -> Option<((f64, f64), (f64, f64))> {
        self.bounds
            .map(|bounds| (bounds.point(self.start), bounds.point(self.current)))
    }
}

// Bounds queries and pointer capture are asynchronous on desktop. An epoch and
// pointer owner prevent an old reply from resurrecting a canceled/replaced drag.
// Pinned Dioxus 0.8.0-alpha.2 MountedData/RenderedElementBacking and PointerData
// lack setPointerCapture/hasPointerCapture/releasePointerCapture, so the tiny
// capture/release bridges target the physical SVG by ID after Dioxus measurement.
fn begin_drag<K: Copy + 'static>(
    mut drag: Signal<Option<ChartDrag<K>>>,
    epoch: Rc<Cell<u64>>,
    node: Rc<MountedData>,
    id: String,
    event: &PointerEvent,
    kind: K,
    view_size: (f64, f64),
) {
    if drag.peek().is_some() {
        return;
    }
    let pointer = event.pointer_id();
    let point = event.client_coordinates();
    drag.set(Some(ChartDrag {
        pointer,
        start: (point.x, point.y),
        current: (point.x, point.y),
        bounds: None,
        kind,
    }));
    let sequence = epoch.get().wrapping_add(1);
    epoch.set(sequence);
    spawn(async move {
        let Some(bounds) = mounted_svg_bounds(&node, view_size).await else {
            if epoch.get() == sequence {
                if let Ok(mut value) = drag.try_write() {
                    *value = None;
                }
            }
            return;
        };
        if epoch.get() != sequence {
            return;
        }
        if !drag
            .try_peek()
            .ok()
            .is_some_and(|value| value.is_some_and(|value| value.pointer == pointer))
        {
            return;
        }
        if let Ok(mut value) = drag.try_write() {
            if let Some(value) = value.as_mut() {
                value.bounds = Some(bounds);
            }
        }
        if crate::capture_chart_pointer(&id, pointer).await.is_err() && epoch.get() == sequence {
            if let Ok(mut value) = drag.try_write() {
                *value = None;
            }
        }
        if epoch.get() != sequence
            && !drag
                .try_peek()
                .ok()
                .is_some_and(|value| value.is_some_and(|value| value.pointer == pointer))
        {
            release_drag_pointer(&id, pointer).await;
        }
    });
}

// Pinned Dioxus 0.8.0-alpha.2 MountedData/PointerData has no pointer-capture API.
// Escape and keyboard operations can end a drag before pointerup; release its
// native capture as well as its Rust state so sibling controls remain able to
// receive this pointer before the physical button is released.
async fn release_drag_pointer(id: &str, pointer: i32) {
    let task = crate::dom_query::eval(
        r#"
        const [id, pointer] = await dioxus.recv();
        const node = document.getElementById(id);
        if (node?.hasPointerCapture(pointer)) node.releasePointerCapture(pointer);
        return null;
    "#,
    );
    if task.send((id, pointer)).is_ok() {
        let _ = task.join::<serde_json::Value>().await;
    }
}

/// A normalized inclusive viewing window over a sequence of samples.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChartZoom {
    pub start: f64,
    pub end: f64,
}
impl Default for ChartZoom {
    fn default() -> Self {
        Self {
            start: 0.0,
            end: 0.1,
        }
    }
}
impl ChartZoom {
    pub fn new(start: f64, end: f64) -> Self {
        if !start.is_finite() || !end.is_finite() {
            return Self::default();
        }
        let lower = start.min(end).clamp(0.0, 0.9999);
        let upper = start.max(end).clamp(lower + 0.0001, 1.0);
        Self {
            start: lower,
            end: upper,
        }
    }
    pub fn pan(self, fraction: f64) -> Self {
        let current = Self::new(self.start, self.end);
        let Self {
            start: current_start,
            end: current_end,
        } = current;
        if !fraction.is_finite() {
            return current;
        }
        let span = current_end - current_start;
        let start = (current_start + fraction).clamp(0.0, 1.0 - span);
        Self {
            start,
            end: start + span,
        }
    }
    pub fn zoom(self, factor: f64, anchor: f64) -> Self {
        let current = Self::new(self.start, self.end);
        let Self {
            start: current_start,
            end: current_end,
        } = current;
        if !factor.is_finite() || factor <= 0.0 {
            return current;
        }
        let anchor = if anchor.is_finite() {
            anchor.clamp(0.0, 1.0)
        } else {
            0.5
        };
        let span = current_end - current_start;
        let next = (span * factor).clamp(0.0001, 1.0);
        let start = (current_start + span * anchor - next * anchor).clamp(0.0, 1.0 - next);
        Self {
            start,
            end: start + next,
        }
    }
    pub fn indices(self, count: usize) -> std::ops::Range<usize> {
        if count < 2 {
            return 0..count;
        }
        let current = Self::new(self.start, self.end);
        let start = (current.start * (count - 1) as f64).floor() as usize;
        let end = ((current.end * (count - 1) as f64).ceil() as usize + 1).min(count);
        start.min(count - 2)..end.max(start + 2).min(count)
    }
}

/// Largest-triangle-three-buckets sampling, retaining original finite indices.
/// Endpoints are preserved; a budget below three returns just the endpoints.
pub fn chart_lttb(values: &[f64], budget: usize) -> Vec<usize> {
    let finite: Vec<_> = values
        .iter()
        .enumerate()
        .filter_map(|(i, v)| v.is_finite().then_some(i))
        .collect();
    let count = finite.len();
    if count <= budget || count < 3 {
        return finite;
    }
    if budget < 3 {
        return vec![finite[0], finite[count - 1]];
    }
    let bucket = (count - 2) as f64 / (budget - 2) as f64;
    let mut result = Vec::with_capacity(budget);
    let mut selected = 0;
    result.push(finite[0]);
    for index in 0..budget - 2 {
        let average_start = (((index + 1) as f64 * bucket).floor() as usize + 1).min(count - 1);
        let average_end = (((index + 2) as f64 * bucket).floor() as usize + 1).min(count);
        let average_count = (average_end - average_start).max(1) as f64;
        let average_x = finite[average_start..average_end]
            .iter()
            .map(|i| *i as f64)
            .sum::<f64>()
            / average_count;
        let average_y = finite[average_start..average_end]
            .iter()
            .map(|i| values[*i])
            .sum::<f64>()
            / average_count;
        let range_start = ((index as f64 * bucket).floor() as usize + 1).min(count - 2);
        let range_end = (((index + 1) as f64 * bucket).floor() as usize + 1).min(count - 1);
        let a = finite[selected];
        let mut largest = -1.0;
        for (candidate, &b) in finite
            .iter()
            .enumerate()
            .take(range_end.max(range_start + 1))
            .skip(range_start)
        {
            let area = ((a as f64 - average_x) * (values[b] - values[a])
                - (a as f64 - b as f64) * (average_y - values[a]))
                .abs();
            if area > largest {
                largest = area;
                selected = candidate;
            }
        }
        result.push(finite[selected]);
    }
    result.push(finite[count - 1]);
    result
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SurfacePoint {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// Rectangular surface samples. `None` creates a genuine hole, not a zero plane.
#[derive(Clone, Debug, PartialEq)]
pub struct ChartSurface {
    pub columns: usize,
    pub points: Vec<Option<SurfacePoint>>,
}
impl ChartSurface {
    /// Sample the square [-1,1]² at a bounded resolution of 2–201 points per side.
    pub fn from_equation(resolution: usize, equation: impl Fn(f64, f64) -> Option<f64>) -> Self {
        let columns = resolution.clamp(2, 201);
        let points = (0..columns)
            .flat_map(|row| {
                let equation = &equation;
                (0..columns).map(move |column| {
                    let x =
                        ((-1.0 + 2.0 * column as f64 / (columns - 1) as f64) * 1e10).round() / 1e10;
                    let y =
                        ((-1.0 + 2.0 * row as f64 / (columns - 1) as f64) * 1e10).round() / 1e10;
                    equation(x, y)
                        .filter(|z| z.is_finite())
                        .map(|z| SurfacePoint { x, y, z })
                })
            })
            .collect();
        Self { columns, points }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SurfaceCamera {
    pub yaw: f64,
    pub pitch: f64,
    pub zoom: f64,
    pub pan_x: f64,
    pub pan_y: f64,
}
impl Default for SurfaceCamera {
    fn default() -> Self {
        Self {
            yaw: 40.0,
            pitch: 20.0,
            zoom: 1.0,
            pan_x: 0.0,
            pan_y: 0.0,
        }
    }
}
impl SurfaceCamera {
    pub fn normalized(self) -> Self {
        let default = Self::default();
        let finite = |value: f64, fallback| if value.is_finite() { value } else { fallback };
        Self {
            yaw: finite(self.yaw, default.yaw).rem_euclid(360.0),
            pitch: finite(self.pitch, default.pitch).clamp(-90.0, 90.0),
            // ECharts GL's default orthographic size is 150 (+ its unit base),
            // with a 20..400 size range. Keep zoom as a magnification factor.
            zoom: finite(self.zoom, 1.0).clamp(151.0 / 401.0, 151.0 / 21.0),
            pan_x: finite(self.pan_x, 0.0),
            pan_y: finite(self.pan_y, 0.0),
        }
    }
    fn project(self, p: SurfacePoint, width: f64, height: f64) -> (f64, f64, f64) {
        let camera = self.normalized();
        let (sy, cy) = camera.yaw.to_radians().sin_cos();
        let (sp, cp) = camera.pitch.to_radians().sin_cos();
        // Data x/y/z become world x/z/-y: positive source y runs away
        // from the viewer. Project that cube onto an orbit camera's basis.
        let horizontal = cy * p.x + sy * p.y;
        let depth = sy * p.x - cy * p.y;
        let vertical = cp * p.z - sp * depth;
        // A 100-unit cube in a 151-unit orthographic viewing volume. As in
        // the source, the vertical extent is fixed; narrow viewports clip.
        let scale = height * 50.0 / 151.0 * camera.zoom;
        (
            width * 0.5 + (horizontal + camera.pan_x) * scale,
            height * 0.5 - (vertical + camera.pan_y) * scale,
            depth * cp + p.z * sp,
        )
    }
}

#[derive(Clone)]
struct SurfaceFace {
    points: String,
    depth: f64,
    vertices: [SurfacePoint; 4],
    indices: [usize; 4],
    projected: [(f64, f64, f64); 4],
    light_color: String,
    dark_color: String,
}

fn surface_cross(a: SurfacePoint, b: SurfacePoint, c: SurfacePoint) -> [f64; 3] {
    let u = [b.x - a.x, b.y - a.y, b.z - a.z];
    let v = [c.x - a.x, c.y - a.y, c.z - a.z];
    [
        u[1] * v[2] - u[2] * v[1],
        u[2] * v[0] - u[0] * v[2],
        u[0] * v[1] - u[1] * v[0],
    ]
}

/// Public Brand visualMap colors, reordered from high-to-low to low-to-high.
/// These are data colors, independent of the 17-color discrete series palette.
fn surface_color(z: f64, light: f64, dark: bool) -> String {
    let colors = if dark {
        [
            [255.0, 215.0, 50.0],
            [255.0, 144.0, 0.0],
            [255.0, 38.0, 64.0],
        ]
    } else {
        [
            [233.0, 195.0, 42.0],
            [233.0, 100.0, 1.0],
            [215.0, 35.0, 57.0],
        ]
    };
    let t = (z + 1.0).clamp(0.0, 2.0);
    let (a, b, fraction) = if t <= 1.0 {
        (colors[0], colors[1], t)
    } else {
        (colors[1], colors[2], t - 1.0)
    };
    let channels: Vec<_> = (0..3)
        .map(|i| ((a[i] + (b[i] - a[i]) * fraction) * light).clamp(0.0, 255.0))
        .collect();
    format!(
        "rgb({:.0} {:.0} {:.0})",
        channels[0], channels[1], channels[2]
    )
}

fn surface_light(normal: [f64; 3], camera: SurfaceCamera) -> f64 {
    let length = normal.iter().map(|v| v * v).sum::<f64>().sqrt();
    if length <= f64::EPSILON {
        return 0.4;
    }
    let camera = camera.normalized();
    let (sb, cb) = camera.yaw.to_radians().sin_cos();
    let (sa, ca) = camera.pitch.to_radians().sin_cos();
    let toward_camera = [sb * ca, -cb * ca, sa];
    let facing = if normal
        .iter()
        .zip(toward_camera)
        .map(|(n, c)| n * c)
        .sum::<f64>()
        < 0.0
    {
        -1.0
    } else {
        1.0
    };
    // Original Lambert calculation using grid3D's main alpha=30/beta=40,
    // intensity=1, white ambient intensity=.4, and double-sided normals.
    let (slb, clb) = 40.0_f64.to_radians().sin_cos();
    let (sla, cla) = 30.0_f64.to_radians().sin_cos();
    let toward_light = [slb * cla, -clb * cla, sla];
    0.4 + (normal
        .iter()
        .zip(toward_light)
        .map(|(n, l)| n * l)
        .sum::<f64>()
        / length
        * facing)
        .max(0.0)
}

/// The three back walls change as the camera crosses the cube, as do its
/// outer axis edges. Coordinates are source coordinates, before projection.
fn surface_back_corner(camera: SurfaceCamera) -> SurfacePoint {
    let camera = camera.normalized();
    let (sy, cy) = camera.yaw.to_radians().sin_cos();
    SurfacePoint {
        x: if sy >= 0.0 { -1.0 } else { 1.0 },
        y: if cy >= 0.0 { 1.0 } else { -1.0 },
        z: if camera.pitch >= 0.0 { -1.0 } else { 1.0 },
    }
}

fn surface_grid(camera: SurfaceCamera, ticks: &[f64]) -> Vec<(SurfacePoint, SurfacePoint)> {
    let back = surface_back_corner(camera);
    let mut lines = Vec::with_capacity(ticks.len() * 6);
    for &tick in ticks {
        lines.extend([
            (
                SurfacePoint {
                    x: tick,
                    y: -1.0,
                    z: back.z,
                },
                SurfacePoint {
                    x: tick,
                    y: 1.0,
                    z: back.z,
                },
            ),
            (
                SurfacePoint {
                    x: -1.0,
                    y: tick,
                    z: back.z,
                },
                SurfacePoint {
                    x: 1.0,
                    y: tick,
                    z: back.z,
                },
            ),
            (
                SurfacePoint {
                    x: back.x,
                    y: tick,
                    z: -1.0,
                },
                SurfacePoint {
                    x: back.x,
                    y: tick,
                    z: 1.0,
                },
            ),
            (
                SurfacePoint {
                    x: back.x,
                    y: -1.0,
                    z: tick,
                },
                SurfacePoint {
                    x: back.x,
                    y: 1.0,
                    z: tick,
                },
            ),
            (
                SurfacePoint {
                    x: tick,
                    y: back.y,
                    z: -1.0,
                },
                SurfacePoint {
                    x: tick,
                    y: back.y,
                    z: 1.0,
                },
            ),
            (
                SurfacePoint {
                    x: -1.0,
                    y: back.y,
                    z: tick,
                },
                SurfacePoint {
                    x: 1.0,
                    y: back.y,
                    z: tick,
                },
            ),
        ]);
    }
    lines
}
fn surface_faces(
    surface: &ChartSurface,
    camera: SurfaceCamera,
    width: f64,
    height: f64,
) -> Vec<SurfaceFace> {
    let columns = surface.columns;
    if columns < 2 || columns > surface.points.len() / 2 {
        return vec![];
    }
    let mut quads = vec![];
    let mut normals = vec![[0.0; 3]; surface.points.len()];
    for row in 0..surface.points.len() / columns - 1 {
        for column in 0..columns - 1 {
            let indices = [
                row * columns + column,
                row * columns + column + 1,
                (row + 1) * columns + column + 1,
                (row + 1) * columns + column,
            ];
            let [Some(a), Some(b), Some(c), Some(d)] = indices.map(|i| surface.points[i]) else {
                continue;
            };
            let points = [a, b, c, d];
            if points
                .iter()
                .any(|p| !p.x.is_finite() || !p.y.is_finite() || !p.z.is_finite())
            {
                continue;
            }
            // Area-weighted vertex normals smooth neighboring cells, while
            // holes exclude the whole affected quad, including its normals.
            for triangle in [[0, 1, 2], [0, 2, 3]] {
                let normal = surface_cross(
                    points[triangle[0]],
                    points[triangle[1]],
                    points[triangle[2]],
                );
                for corner in triangle {
                    for (component, value) in normal.iter().enumerate() {
                        normals[indices[corner]][component] += value;
                    }
                }
            }
            quads.push((indices, points));
        }
    }
    let mut faces = Vec::with_capacity(quads.len());
    for (indices, points) in quads {
        let projected = points.map(|p| camera.project(p, width, height));
        let sample = SurfacePoint {
            x: points.iter().map(|p| p.x).sum::<f64>() / 4.0,
            y: points.iter().map(|p| p.y).sum::<f64>() / 4.0,
            z: points.iter().map(|p| p.z).sum::<f64>() / 4.0,
        };
        let mut normal = [0.0; 3];
        for index in indices {
            let vertex = normals[index];
            let length = vertex
                .iter()
                .map(|v| v * v)
                .sum::<f64>()
                .sqrt()
                .max(f64::EPSILON);
            for (component, value) in vertex.iter().enumerate() {
                normal[component] += value / length;
            }
        }
        let light = surface_light(normal, camera);
        faces.push(SurfaceFace {
            points: projected
                .iter()
                .map(|(x, y, _)| format!("{x:.2},{y:.2}"))
                .collect::<Vec<_>>()
                .join(" "),
            depth: projected.iter().map(|p| p.2).sum::<f64>() / 4.0,
            vertices: points,
            indices,
            projected,
            light_color: surface_color(sample.z, light, false),
            dark_color: surface_color(sample.z, light, true),
        });
    }
    faces.sort_by(|a, b| a.depth.total_cmp(&b.depth));
    faces
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct SurfaceInspection {
    /// Only a surface hit has a series datum. Grid-wall hits have a pointer only.
    index: Option<usize>,
    point: SurfacePoint,
}

/// Orthographic ray/triangle intersection in projected coordinates. Depth is
/// interpolated at the actual cursor, never averaged across a face. This also
/// rejects edge-on triangles and misses outside the triangle's boundaries.
fn surface_triangle_hit(
    projected: [(f64, f64, f64); 3],
    cursor: (f64, f64),
) -> Option<([f64; 3], f64)> {
    let [a, b, c] = projected;
    let denominator = (b.1 - c.1) * (a.0 - c.0) + (c.0 - b.0) * (a.1 - c.1);
    if !denominator.is_finite() || denominator.abs() <= 1e-10 {
        return None;
    }
    let u = ((b.1 - c.1) * (cursor.0 - c.0) + (c.0 - b.0) * (cursor.1 - c.1)) / denominator;
    let v = ((c.1 - a.1) * (cursor.0 - c.0) + (a.0 - c.0) * (cursor.1 - c.1)) / denominator;
    let weights = [u, v, 1.0 - u - v];
    if weights.iter().any(|v| !v.is_finite() || *v < -1e-8) {
        return None;
    }
    Some((weights, u * a.2 + v * b.2 + weights[2] * c.2))
}

fn surface_interpolate(vertices: [SurfacePoint; 3], weights: [f64; 3]) -> SurfacePoint {
    let component =
        |get: fn(SurfacePoint) -> f64| vertices.iter().zip(weights).map(|(p, w)| get(*p) * w).sum();
    SurfacePoint {
        x: component(|p| p.x),
        y: component(|p| p.y),
        z: component(|p| p.z),
    }
}

fn surface_distance(a: SurfacePoint, b: SurfacePoint) -> f64 {
    (a.x - b.x).powi(2) + (a.y - b.y).powi(2) + (a.z - b.z).powi(2)
}

/// Original Rust picking: find the nearest visible triangle along the ray,
/// then its nearest actual 3D vertex, matching the source's selection rule.
fn surface_pick(faces: &[SurfaceFace], cursor: (f64, f64)) -> Option<SurfaceInspection> {
    let mut nearest = None;
    let mut depth = f64::NEG_INFINITY;
    for face in faces {
        // Published surface geometry splits each quad along corners 0–2.
        for triangle in [[0, 1, 2], [0, 2, 3]] {
            let Some((weights, hit_depth)) =
                surface_triangle_hit(triangle.map(|i| face.projected[i]), cursor)
            else {
                continue;
            };
            if hit_depth <= depth {
                continue;
            }
            let vertices = triangle.map(|i| face.vertices[i]);
            let hit = surface_interpolate(vertices, weights);
            let corner = (0..3).min_by(|a, b| {
                surface_distance(vertices[*a], hit).total_cmp(&surface_distance(vertices[*b], hit))
            })?;
            nearest = Some(SurfaceInspection {
                index: Some(face.indices[triangle[corner]]),
                point: vertices[corner],
            });
            depth = hit_depth;
        }
    }
    nearest
}

fn surface_wall_pick(
    camera: SurfaceCamera,
    width: f64,
    height: f64,
    cursor: (f64, f64),
) -> Option<SurfaceInspection> {
    let back = surface_back_corner(camera);
    let walls = [
        [
            SurfacePoint {
                x: back.x,
                y: -1.0,
                z: -1.0,
            },
            SurfacePoint {
                x: back.x,
                y: 1.0,
                z: -1.0,
            },
            SurfacePoint {
                x: back.x,
                y: 1.0,
                z: 1.0,
            },
            SurfacePoint {
                x: back.x,
                y: -1.0,
                z: 1.0,
            },
        ],
        [
            SurfacePoint {
                x: -1.0,
                y: back.y,
                z: -1.0,
            },
            SurfacePoint {
                x: 1.0,
                y: back.y,
                z: -1.0,
            },
            SurfacePoint {
                x: 1.0,
                y: back.y,
                z: 1.0,
            },
            SurfacePoint {
                x: -1.0,
                y: back.y,
                z: 1.0,
            },
        ],
        [
            SurfacePoint {
                x: -1.0,
                y: -1.0,
                z: back.z,
            },
            SurfacePoint {
                x: 1.0,
                y: -1.0,
                z: back.z,
            },
            SurfacePoint {
                x: 1.0,
                y: 1.0,
                z: back.z,
            },
            SurfacePoint {
                x: -1.0,
                y: 1.0,
                z: back.z,
            },
        ],
    ];
    let mut nearest = None;
    let mut depth = f64::NEG_INFINITY;
    for wall in walls {
        for triangle in [[0, 1, 3], [1, 2, 3]] {
            let vertices = triangle.map(|i| wall[i]);
            if let Some((weights, hit_depth)) =
                surface_triangle_hit(vertices.map(|p| camera.project(p, width, height)), cursor)
            {
                if hit_depth > depth {
                    nearest = Some(SurfaceInspection {
                        index: None,
                        point: surface_interpolate(vertices, weights),
                    });
                    depth = hit_depth;
                }
            }
        }
    }
    nearest
}

fn surface_pointer_lines(
    point: SurfacePoint,
    camera: SurfaceCamera,
) -> [(SurfacePoint, SurfacePoint); 9] {
    let b = surface_back_corner(camera);
    [
        (
            SurfacePoint {
                x: point.x,
                y: -1.0,
                z: b.z,
            },
            SurfacePoint {
                x: point.x,
                y: 1.0,
                z: b.z,
            },
        ),
        (
            SurfacePoint {
                x: -1.0,
                y: point.y,
                z: b.z,
            },
            SurfacePoint {
                x: 1.0,
                y: point.y,
                z: b.z,
            },
        ),
        (
            SurfacePoint {
                x: b.x,
                y: point.y,
                z: -1.0,
            },
            SurfacePoint {
                x: b.x,
                y: point.y,
                z: 1.0,
            },
        ),
        (
            SurfacePoint {
                x: b.x,
                y: -1.0,
                z: point.z,
            },
            SurfacePoint {
                x: b.x,
                y: 1.0,
                z: point.z,
            },
        ),
        (
            SurfacePoint {
                x: point.x,
                y: b.y,
                z: -1.0,
            },
            SurfacePoint {
                x: point.x,
                y: b.y,
                z: 1.0,
            },
        ),
        (
            SurfacePoint {
                x: -1.0,
                y: b.y,
                z: point.z,
            },
            SurfacePoint {
                x: 1.0,
                y: b.y,
                z: point.z,
            },
        ),
        (point, SurfacePoint { z: b.z, ..point }),
        (point, SurfacePoint { x: b.x, ..point }),
        (point, SurfacePoint { y: b.y, ..point }),
    ]
}

/// Keyboard inspection advances through visible data vertices in source order,
/// excluding holes, disconnected samples, offscreen and occluded vertices.
fn surface_inspect_step(
    faces: &[SurfaceFace],
    width: f64,
    height: f64,
    current: Option<usize>,
    forward: bool,
) -> Option<SurfaceInspection> {
    let mut candidates: Vec<_> = faces
        .iter()
        .flat_map(|face| {
            (0..4).map(move |i| (face.indices[i], face.vertices[i], face.projected[i]))
        })
        .collect();
    candidates.sort_by_key(|v| v.0);
    candidates.dedup_by_key(|v| v.0);
    if !forward {
        candidates.reverse();
    }
    let after = candidates
        .iter()
        .position(|v| Some(v.0) == current)
        .map_or(0, |i| i + 1);
    for offset in 0..candidates.len() {
        let (index, point, projected) = candidates[(after + offset) % candidates.len()];
        if !(0.0..=width).contains(&projected.0) || !(0.0..=height).contains(&projected.1) {
            continue;
        }
        if surface_pick(faces, (projected.0, projected.1))
            .is_some_and(|hit| surface_distance(hit.point, point) < 1e-12)
        {
            return Some(SurfaceInspection {
                index: Some(index),
                point,
            });
        }
    }
    None
}

// Original Rust interaction state informed by factual OrbitControl 2.0.9
// defaults: 0.8 damping per 16ms, at most 50ms/frame; rotation/zoom retain
// momentum, while pan is consumed once. Two-touch pinch changes zoom only.
#[derive(Clone, Copy, Debug)]
struct SurfaceContact {
    id: i32,
    point: (f64, f64),
    touch: bool,
    pan: bool,
}
#[derive(Clone, Copy, Debug, Default)]
struct SurfaceOrbit {
    contacts: [Option<SurfaceContact>; 2],
    bounds: Option<SvgBounds>,
    rotation: (f64, f64), // radians per reference frame: yaw, pitch
    pan: (f64, f64),
    zoom_speed: f64, // orthographic size per reference frame
}
impl SurfaceOrbit {
    fn owns(&self, id: i32) -> bool {
        self.contacts
            .iter()
            .flatten()
            .any(|contact| contact.id == id)
    }
    fn engaged(&self) -> bool {
        self.contacts[0].is_some()
    }
    fn moving(&self) -> bool {
        self.rotation != (0.0, 0.0) || self.pan != (0.0, 0.0) || self.zoom_speed.abs() > 0.1
    }
    fn begin(&mut self, contact: SurfaceContact) -> bool {
        if self.owns(contact.id) {
            return false;
        }
        if let Some(first) = self.contacts[0] {
            if !first.touch || !contact.touch || self.contacts[1].is_some() {
                return false;
            }
            self.contacts[1] = Some(contact);
        } else {
            self.contacts[0] = Some(contact);
        }
        self.rotation = (0.0, 0.0);
        self.pan = (0.0, 0.0);
        self.zoom_speed = 0.0;
        true
    }
    fn end(&mut self, id: i32) -> bool {
        if !self.owns(id) {
            return false;
        }
        if self.contacts[0].is_some_and(|contact| contact.id == id) {
            self.contacts[0] = self.contacts[1].take();
        } else {
            self.contacts[1] = None;
        }
        // The remaining contact already stores its latest position: the next
        // single-finger move starts here, without replaying the pinch distance.
        if self.engaged() {
            self.rotation = (0.0, 0.0);
            self.pan = (0.0, 0.0);
            self.zoom_speed = 0.0;
        }
        true
    }
    fn zoom_impulse(&mut self, camera: SurfaceCamera, direction: f64) {
        if direction == 0.0 || !direction.is_finite() {
            return;
        }
        let size = 151.0 / camera.normalized().zoom;
        let speed = ((size - 21.0).min(401.0 - size) / 20.0).max(0.5);
        self.zoom_speed = -direction.signum() * speed;
    }
    fn move_contact(&mut self, id: i32, point: (f64, f64), camera: SurfaceCamera) -> bool {
        if !point.0.is_finite() || !point.1.is_finite() {
            return false;
        }
        let Some(index) = self
            .contacts
            .iter()
            .position(|contact| contact.is_some_and(|contact| contact.id == id))
        else {
            return false;
        };
        let previous = self.contacts[index].unwrap();
        self.contacts[index].as_mut().unwrap().point = point;
        let Some(bounds) = self.bounds else {
            return true;
        };
        let before = bounds.point(previous.point);
        let after = bounds.point(point);
        if let Some(other) = self.contacts[1 - index] {
            let other = bounds.point(other.point);
            let old_distance = (before.0 - other.0).hypot(before.1 - other.1);
            let distance = (after.0 - other.0).hypot(after.1 - other.1);
            if old_distance > 1.0 && distance > 1.0 && (distance - old_distance).abs() > 0.01 {
                self.zoom_impulse(camera, distance - old_distance);
            }
            self.rotation = (0.0, 0.0);
            self.pan = (0.0, 0.0);
        } else if previous.pan {
            // Source camera distance is 151 and the data cube spans 100 world
            // units. Pan has no damped tail in upstream OrbitControl.
            self.pan.0 += (after.0 - before.0) / bounds.view_width * 6.04;
            self.pan.1 -= (after.1 - before.1) / bounds.view_height * 6.04;
        } else {
            self.rotation = (
                -(after.0 - before.0) / bounds.view_height * 2.0,
                (after.1 - before.1) / bounds.view_width * 2.0,
            );
        }
        true
    }
    fn step(&mut self, camera: SurfaceCamera, elapsed_ms: f64) -> SurfaceCamera {
        let dt = elapsed_ms.clamp(0.0, 50.0);
        let damping = 0.8_f64.powf(dt / 16.0);
        let mut next = SurfaceCamera {
            yaw: camera.yaw + (self.rotation.0 * dt / 20.0).to_degrees(),
            pitch: camera.pitch + (self.rotation.1 * dt / 20.0).to_degrees(),
            zoom: 151.0 / (151.0 / camera.zoom + self.zoom_speed * dt / 20.0).clamp(21.0, 401.0),
            pan_x: camera.pan_x + self.pan.0,
            pan_y: camera.pan_y + self.pan.1,
        }
        .normalized();
        if !self.moving() {
            next = camera;
        }
        self.rotation.0 *= damping;
        self.rotation.1 *= damping;
        if self.rotation.0.hypot(self.rotation.1) < 0.0001 {
            self.rotation = (0.0, 0.0);
        }
        self.zoom_speed *= damping;
        self.pan = (0.0, 0.0);
        next
    }
}

// One Rust timer per active burst. Generation checks stop old work after a new
// gesture, reset, blur, replacement data/viewport, or unmount. DOM bridges only
// measure/capture; camera state and every gesture calculation stay in Rust.
fn surface_motion(
    mut orbit: Signal<SurfaceOrbit>,
    mut camera: Signal<SurfaceCamera>,
    generation: Rc<Cell<u64>>,
    running: Rc<Cell<bool>>,
) {
    if running.replace(true) {
        return;
    }
    let sequence = generation.get();
    spawn(async move {
        let mut previous = web_time::Instant::now();
        loop {
            futures_timer::Delay::new(std::time::Duration::from_millis(16)).await;
            if generation.get() != sequence {
                return;
            }
            let now = web_time::Instant::now();
            let elapsed = now.duration_since(previous).as_secs_f64() * 1000.0;
            previous = now;
            let Ok(current) = camera.try_peek().map(|value| *value) else {
                return;
            };
            let Ok(mut state) = orbit.try_write() else {
                return;
            };
            if !state.moving() {
                running.set(false);
                return;
            }
            let next = state.step(current, elapsed);
            drop(state);
            if let Ok(mut value) = camera.try_write() {
                *value = next;
            } else {
                return;
            }
        }
    });
}

/// Orthographic surface with captured primary-drag rotation, middle/Shift/right-drag
/// panning, wheel zoom, keyboard controls and source-coordinate tooltips.
/// Escape cancels a drag; pointer motion outside the plot continues the gesture.
#[component]
pub fn SurfaceChart(
    surface: ChartSurface,
    #[props(default = "Surface plot".to_string())] label: String,
    /// Plot viewport height in CSS pixels, also used for gesture coordinates.
    #[props(default = 600.0)]
    height: f64,
    /// Visible keyboard/pointer help. The SVG always retains accessible help.
    #[props(default = true)]
    show_instructions: bool,
) -> Element {
    let id = use_hook(|| NEXT_ADVANCED_CHART.fetch_add(1, Ordering::Relaxed));
    let width = use_signal(|| 800.0_f64);
    let mut camera = use_signal(SurfaceCamera::default);
    let mut orbit = use_signal(SurfaceOrbit::default);
    let mut mounted = use_signal(|| None::<Rc<MountedData>>);
    let epoch = use_hook(|| Rc::new(Cell::new(0_u64)));
    let motion_running = use_hook(|| Rc::new(Cell::new(false)));
    let drop_epoch = epoch.clone();
    let drop_running = motion_running.clone();
    use_drop(move || {
        drop_epoch.set(drop_epoch.get().wrapping_add(1));
        drop_running.set(false);
    });
    let cancel_epoch = epoch.clone();
    let cancel_running = motion_running.clone();
    let cancel_drag = use_callback(move |pointer: Option<i32>| {
        if pointer.is_none() || pointer.is_some_and(|pointer| orbit.peek().owns(pointer)) {
            let owners = orbit.peek().contacts;
            cancel_epoch.set(cancel_epoch.get().wrapping_add(1));
            cancel_running.set(false);
            orbit.set(SurfaceOrbit::default());
            for owner in owners.into_iter().flatten() {
                spawn(async move {
                    release_drag_pointer(&format!("ix-surface-svg-{id}"), owner.id).await;
                });
            }
        }
    });
    let mut active = use_signal(|| None::<SurfaceInspection>);
    let hover_epoch = use_hook(|| Rc::new(Cell::new(0_u64)));
    let drop_hover = hover_epoch.clone();
    use_drop(move || drop_hover.set(drop_hover.get().wrapping_add(1)));
    let clear_hover = hover_epoch.clone();
    let clear_inspection = use_callback(move |()| {
        clear_hover.set(clear_hover.get().wrapping_add(1));
        active.set(None);
    });
    let height = if height.is_finite() {
        height.max(1.0)
    } else {
        600.0
    };
    use_effect(use_reactive((&surface, &height), move |_| {
        let _ = width();
        cancel_drag.call(None);
    }));
    let move_epoch = epoch.clone();
    let move_running = motion_running.clone();
    let wheel_epoch = epoch.clone();
    let wheel_running = motion_running.clone();
    let changed_hover = hover_epoch.clone();
    use_effect(use_reactive((&surface, &height), move |_| {
        let _ = (camera(), width());
        changed_hover.set(changed_hover.get().wrapping_add(1));
        active.set(None);
    }));
    let faces = Rc::new(surface_faces(&surface, camera(), width(), height));
    let hover_faces = faces.clone();
    let keyboard_faces = faces.clone();
    let hover_camera = camera();
    let move_hover = hover_epoch.clone();
    let project = |p| camera().project(p, width(), height);
    let back = surface_back_corner(camera());
    let ticks = [-1.0, -0.5, 0.0, 0.5, 1.0];
    let grid = surface_grid(camera(), &ticks);
    let axes = [
        (
            SurfacePoint {
                x: -1.0,
                y: -back.y,
                z: back.z,
            },
            SurfacePoint {
                x: 1.0,
                y: -back.y,
                z: back.z,
            },
            "x",
        ),
        (
            SurfacePoint {
                x: -back.x,
                y: -1.0,
                z: back.z,
            },
            SurfacePoint {
                x: -back.x,
                y: 1.0,
                z: back.z,
            },
            "y",
        ),
        (
            SurfacePoint {
                x: back.x,
                y: -back.y,
                z: -1.0,
            },
            SurfacePoint {
                x: back.x,
                y: -back.y,
                z: 1.0,
            },
            "z",
        ),
    ];
    rsx! { div { class: "ix-surface-chart", role: "group", aria_label: label.clone(),
        onresize: move |event| crate::charts::resize_chart_width(width, event),
        div { class: "ix-chart-viewport",
            onmounted: move |event: MountedEvent| mounted.set(Some(event.data())),
            svg { id: "ix-surface-svg-{id}", width: "100%", height, view_box: "0 0 {width} {height}", role: "img", "aria-label": label,
                "aria-description":"Drag to rotate; middle-drag or Shift-drag to pan; scroll or pinch to zoom. Arrow keys rotate, +/− zoom, Home resets. Page Up and Page Down inspect visible data points. Escape clears inspection and ends the current gesture.",
                "aria-describedby":"ix-surface-inspection-{id}",
                "data-yaw":camera().yaw,"data-pitch":camera().pitch,"data-zoom":camera().zoom,
                "data-pan-x":camera().pan_x,"data-pan-y":camera().pan_y,
                tabindex: 0, style: "touch-action:none",
                oncontextmenu: move |event| event.prevent_default(),
                onpointerdown: move |event: PointerEvent| {
                    let touch = event.pointer_type() == "touch";
                    if !touch && !matches!(event.trigger_button(), Some(MouseButton::Primary | MouseButton::Auxiliary | MouseButton::Secondary)) { return; }
                    let Some(node) = mounted.peek().clone() else { return; };
                    let pointer = event.pointer_id();
                    let point = event.client_coordinates();
                    let first = !orbit.peek().engaged();
                    if first { cancel_drag.call(None); }
                    let accepted = orbit.write().begin(SurfaceContact { id:pointer, point:(point.x,point.y), touch,
                        pan: !touch && (event.modifiers().shift() || matches!(event.trigger_button(),Some(MouseButton::Auxiliary | MouseButton::Secondary))) });
                    if !accepted { return; }
                    event.prevent_default(); clear_inspection.call(());
                    let epoch = epoch.clone(); let sequence = epoch.get();
                    spawn(async move {
                        let chart_id = format!("ix-surface-svg-{id}");
                        if first {
                            let Some(bounds) = mounted_svg_bounds(&node, (width(), height)).await else {cancel_drag.call(Some(pointer));return;};
                            if epoch.get()!=sequence || !orbit.peek().engaged() {return;}
                            orbit.write().bounds=Some(bounds);
                        }
                        if epoch.get()!=sequence || !orbit.peek().owns(pointer) { return; }
                        if crate::capture_chart_pointer(&chart_id,pointer).await.is_err() && epoch.get()==sequence {cancel_drag.call(Some(pointer));}
                        if epoch.get()!=sequence || !orbit.peek().owns(pointer) {release_drag_pointer(&chart_id,pointer).await;}
                    });
                },
                onpointerup: move |event: PointerEvent| {
                    let pointer=event.pointer_id();
                    // pointerup itself has no movement delta; retain the last real
                    // move's velocity rather than replacing it with zero.
                    if orbit.write().end(pointer) {
                        spawn(async move {release_drag_pointer(&format!("ix-surface-svg-{id}"),pointer).await;});
                    }
                },
                onpointercancel: move |event: PointerEvent| {cancel_drag.call(Some(event.pointer_id()));clear_inspection.call(());},
                onlostpointercapture: move |event: PointerEvent| {cancel_drag.call(Some(event.pointer_id()));clear_inspection.call(());},
                onpointerleave: move |_| clear_inspection.call(()),
                onblur: move |_| {cancel_drag.call(None);clear_inspection.call(());},
                onpointermove: move |event: PointerEvent| {
                    let point = event.client_coordinates();
                    if orbit.peek().engaged() {
                        if !orbit.write().move_contact(event.pointer_id(),(point.x,point.y),camera()) {return;}
                        event.prevent_default();clear_inspection.call(());
                        surface_motion(orbit,camera,move_epoch.clone(),move_running.clone());
                    } else if !orbit.peek().moving() && event.pointer_type() != "touch" {
                        let Some(node)=mounted.peek().clone() else{return;};
                        let hover_epoch=move_hover.clone();let sequence=hover_epoch.get().wrapping_add(1);hover_epoch.set(sequence);
                        let faces=hover_faces.clone();let view_width=width();
                        spawn(async move {
                            let Some(bounds)=mounted_svg_bounds(&node,(view_width,height)).await else{return;};
                            if hover_epoch.get()!=sequence{return;}
                            let cursor=bounds.point((point.x,point.y));
                            let next=if (0.0..=view_width).contains(&cursor.0)&&(0.0..=height).contains(&cursor.1) {
                                surface_pick(&faces,cursor).or_else(||surface_wall_pick(hover_camera,view_width,height,cursor))
                            } else {None};
                            if let Ok(mut value)=active.try_write(){if *value!=next{*value=next;}}
                        });
                    }
                },
                onwheel: move |event: WheelEvent| {
                    event.prevent_default();clear_inspection.call(());
                    // Browser wheel delta is opposite the source wheelDelta sign.
                    orbit.write().zoom_impulse(camera(),-event.delta().strip_units().y);
                    surface_motion(orbit,camera,wheel_epoch.clone(),wheel_running.clone());
                },
                onkeydown: move |event: KeyboardEvent| {
                    if event.key()==Key::Escape {event.prevent_default();cancel_drag.call(None);clear_inspection.call(());return;}
                    if matches!(event.key(),Key::PageUp|Key::PageDown) {
                        event.prevent_default();cancel_drag.call(None);let current=active().and_then(|value|value.index);clear_inspection.call(());
                        active.set(surface_inspect_step(&keyboard_faces,width(),height,current,event.key()==Key::PageDown));return;
                    }
                    let mut next=camera();match event.key(){Key::ArrowLeft=>next.yaw-=5.0,Key::ArrowRight=>next.yaw+=5.0,Key::ArrowUp=>next.pitch+=5.0,Key::ArrowDown=>next.pitch-=5.0,Key::Home=>next=SurfaceCamera::default(),Key::Character(ref key) if key=="+"||key=="="=>next.zoom*=1.2,Key::Character(ref key) if key=="-"=>next.zoom/=1.2,_=>return}event.prevent_default();cancel_drag.call(None);clear_inspection.call(());camera.set(next.normalized());
                },
                g { class:"ix-surface-grid", stroke:"var(--si-sys-color-background-3,var(--ix-color-surface-2))", stroke_width:1,
                    for (start,end) in grid { {let a=project(start);let b=project(end);rsx!{line{x1:a.0,y1:a.1,x2:b.0,y2:b.1}}} }
                }
                for (start,end,name) in axes { {
                    let a=project(start);let b=project(end);
                    let center=project(SurfacePoint::default());
                    let midpoint=((a.0+b.0)*0.5,(a.1+b.1)*0.5);
                    let length=(midpoint.0-center.0).hypot(midpoint.1-center.1).max(1.0);
                    let offset=((midpoint.0-center.0)/length,(midpoint.1-center.1)/length);
                    let anchor=if offset.0 < -0.3 {"end"} else if offset.0 > 0.3 {"start"} else {"middle"};
                    rsx!{g {class:"ix-surface-axis",font_size:12,fill:"var(--si-sys-color-text-primary,var(--ix-color-text))",
                        line{x1:a.0,y1:a.1,x2:b.0,y2:b.1,stroke:"var(--si-sys-color-border-3,var(--ix-color-border))",stroke_width:2}
                        text{x:midpoint.0+offset.0*35.0,y:midpoint.1+offset.1*35.0+4.0,text_anchor:anchor,"{name}"}
                        for tick in ticks { {
                            let fraction=(tick+1.0)*0.5;
                            let p=(a.0+(b.0-a.0)*fraction,a.1+(b.1-a.1)*fraction);
                            rsx!{
                                line{x1:p.0,y1:p.1,x2:p.0+offset.0*3.0,y2:p.1+offset.1*3.0,stroke:"var(--si-sys-color-border-3,var(--ix-color-border))"}
                                text{x:p.0+offset.0*11.0,y:p.1+offset.1*11.0+4.0,text_anchor:anchor,"{tick}"}
                            }
                        } }
                    }}
                } }
                if let Some(inspection)=active() {
                    g {class:"ix-surface-axis-pointer",pointer_events:"none","data-source-index":inspection.index.map(|v|v.to_string()),"data-x":inspection.point.x,"data-y":inspection.point.y,"data-z":inspection.point.z,
                        for (start,end) in surface_pointer_lines(inspection.point,camera()) { {
                            let a=project(start);let b=project(end);
                            rsx!{line{x1:a.0,y1:a.1,x2:b.0,y2:b.1,stroke:"rgba(0,0,0,.8)",stroke_width:1}}
                        } }
                        for (start,end,name) in axes { {
                            let a=project(start);let b=project(end);let center=project(SurfacePoint::default());
                            let midpoint=((a.0+b.0)*0.5,(a.1+b.1)*0.5);let length=(midpoint.0-center.0).hypot(midpoint.1-center.1).max(1.0);
                            let offset=((midpoint.0-center.0)/length,(midpoint.1-center.1)/length);
                            let value=match name {"x"=>inspection.point.x,"y"=>inspection.point.y,_=>inspection.point.z};
                            let fraction=(value+1.0)*0.5;
                            let position=(a.0+(b.0-a.0)*fraction+offset.0*22.0,a.1+(b.1-a.1)*fraction+offset.1*22.0);
                            let text=format!("{value:.2}");let label_width=text.len() as f64*8.0+6.0;
                            rsx!{g{class:"ix-surface-pointer-label","data-axis":name,
                                rect{x:position.0-label_width*0.5,y:position.1-11.0,width:label_width,height:22,rx:3,fill:"rgba(0,0,0,.5)"}
                                text{x:position.0,y:position.1+5.0,text_anchor:"middle",font_size:14,fill:"#fff","{text}"}
                            }}
                        } }
                    }
                }
                for face in faces.iter() { polygon { class:"ix-surface-face",points: face.points.clone(),"data-source-indices":"{face.indices[0]},{face.indices[1]},{face.indices[2]},{face.indices[3]}",style:"--ix-surface-light:{face.light_color};--ix-surface-dark:{face.dark_color}",fill:"light-dark(var(--ix-surface-light),var(--ix-surface-dark))", stroke: "#0008", stroke_width: "1", stroke_linejoin:"round" } }
            }
        }
        if show_instructions {p { class: "ix-chart-instructions", "Drag to rotate; middle-drag or Shift-drag to pan; scroll or pinch to zoom. Arrow keys rotate, +/− zoom, Home resets. Page Up/Down inspect data points." }}
        output {id:"ix-surface-inspection-{id}",class:if active().is_some_and(|v|v.index.is_some()){"ix-chart-tooltip"}else{""},aria_live:"off",
            if let Some(SurfaceInspection{index:Some(_),point})=active() {"x: {point.x:.2} · y: {point.y:.2} · z: {point.z:.3}"}
        }
    } }
}

#[derive(Clone, Copy, Debug)]
enum ZoomDrag {
    Plot { initial: ChartZoom, brush: bool },
    Start(ChartZoom),
    End(ChartZoom),
}
fn zoom_drag_brush(
    value: ChartDrag<ZoomDrag>,
    left: f64,
    right: f64,
) -> Option<(f64, f64, ChartZoom)> {
    let ZoomDrag::Plot {
        initial,
        brush: true,
    } = value.kind
    else {
        return None;
    };
    let (start, current) = value.points()?;
    (start.0 >= left && start.0 <= right && start.1 >= 24.0 && start.1 <= 510.0).then_some((
        start.0,
        current.0.clamp(left, right),
        initial,
    ))
}
fn zoom_drag_window(value: ChartDrag<ZoomDrag>, left: f64, right: f64) -> Option<ChartZoom> {
    let (start, current) = value.points()?;
    let width = right - left;
    match value.kind {
        ZoomDrag::Start(initial) => Some(ChartZoom::new(
            ((current.0 - left) / width).clamp(0.0, initial.end - 0.0001),
            initial.end,
        )),
        ZoomDrag::End(initial) => Some(ChartZoom::new(
            initial.start,
            ((current.0 - left) / width).clamp(initial.start + 0.0001, 1.0),
        )),
        ZoomDrag::Plot { initial, brush } => {
            if start.0 < left || start.0 > right {
                return None;
            }
            if (548.0..=590.0).contains(&start.1) {
                Some(initial.pan((current.0 - start.0) / width))
            } else if (24.0..=510.0).contains(&start.1) && !brush {
                Some(initial.pan(-(current.0 - start.0) / width * (initial.end - initial.start)))
            } else {
                None
            }
        }
    }
}

/// A large time series with inside pan/wheel zoom, draggable overview handles,
/// keyboard-accessible bounds, area selection, history, and LTTB rendering.
#[component]
pub fn ZoomChart(
    values: Vec<f64>,
    labels: Vec<String>,
    #[props(default = "Synthetic data".to_string())] label: String,
) -> Element {
    let id = use_hook(|| NEXT_ADVANCED_CHART.fetch_add(1, Ordering::Relaxed));
    let width = use_signal(|| 800.0_f64);
    let mut zoom = use_signal(ChartZoom::default);
    let mut drag = use_signal(|| None::<ChartDrag<ZoomDrag>>);
    let mut mounted = use_signal(|| None::<Rc<MountedData>>);
    let epoch = use_hook(|| Rc::new(Cell::new(0_u64)));
    let hover_epoch = use_hook(|| Rc::new(Cell::new(0_u64)));
    let drop_epoch = epoch.clone();
    let drop_hover = hover_epoch.clone();
    use_drop(move || {
        drop_epoch.set(drop_epoch.get().wrapping_add(1));
        drop_hover.set(drop_hover.get().wrapping_add(1));
    });
    let cancel_epoch = epoch.clone();
    let cancel_hover = hover_epoch.clone();
    let cancel_drag = use_callback(move |pointer: Option<i32>| {
        if pointer.is_none()
            || drag
                .peek()
                .is_some_and(|value| Some(value.pointer) == pointer)
        {
            let owner = (*drag.peek()).map(|value| value.pointer);
            cancel_epoch.set(cancel_epoch.get().wrapping_add(1));
            cancel_hover.set(cancel_hover.get().wrapping_add(1));
            drag.set(None);
            if let Some(owner) = owner {
                spawn(async move {
                    release_drag_pointer(&format!("ix-zoom-svg-{id}"), owner).await;
                });
            }
        }
    });
    let mut brush = use_signal(|| false);
    let mut history = use_signal(Vec::<ChartZoom>::new);
    let mut active = use_signal(|| None::<usize>);
    // A previously hovered sample may leave the visible window after a keyboard,
    // overview or wheel change. Do not keep showing that stale coordinate.
    let changed_hover = hover_epoch.clone();
    use_effect(move || {
        let _ = zoom();
        changed_hover.set(changed_hover.get().wrapping_add(1));
        active.set(None);
    });
    let left = 54.0;
    let right = (width() - 20.0).max(left + 80.0);
    let plot_width = right - left;
    let top = 24.0;
    let bottom = 510.0;
    let range = zoom().indices(values.len());
    let visible = &values[range.clone()];
    let finite = visible.iter().copied().filter(|v| v.is_finite());
    let (low, high) = finite.fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), v| {
        (a.min(v), b.max(v))
    });
    let scale = if low.is_finite() {
        ChartScale::new(low, high + (high - low).max(1.0))
    } else {
        ChartScale::new(0.0, 1.0)
    };
    let sampled = chart_lttb(visible, (plot_width as usize).clamp(50, 1200));
    let position = |index: usize, value: f64| {
        (
            left + index as f64 / visible.len().saturating_sub(1).max(1) as f64 * plot_width,
            bottom - scale.fraction(value) * (bottom - top),
        )
    };
    let points: Vec<_> = sampled.iter().map(|i| position(*i, visible[*i])).collect();
    let path = chart_line_path(&points, false, false);
    let area = if let (Some(first), Some(last)) = (points.first(), points.last()) {
        format!(
            "{path} L {:.2} {bottom} L {:.2} {bottom} Z",
            last.0, first.0
        )
    } else {
        String::new()
    };
    let overview_low = values
        .iter()
        .copied()
        .filter(|v| v.is_finite())
        .fold(f64::INFINITY, f64::min);
    let overview_high = values
        .iter()
        .copied()
        .filter(|v| v.is_finite())
        .fold(f64::NEG_INFINITY, f64::max);
    let overview_span = (overview_high - overview_low).max(1.0);
    let overview_points: Vec<_> = chart_lttb(&values, (plot_width as usize).clamp(50, 1000))
        .iter()
        .map(|i| {
            (
                left + *i as f64 / values.len().saturating_sub(1).max(1) as f64 * plot_width,
                588.0 - (values[*i] - overview_low) / overview_span * 40.0,
            )
        })
        .collect();
    let overview_path = chart_line_path(&overview_points, false, false);
    let first_label = labels.get(range.start).cloned().unwrap_or_default();
    let last_label = labels
        .get(range.end.saturating_sub(1))
        .cloned()
        .unwrap_or_default();
    let pointer_count = values.len();
    let tooltip = active().and_then(|index| {
        values.get(index).map(|value| {
            format!(
                "{}: {value:.0}",
                labels.get(index).map(String::as_str).unwrap_or("")
            )
        })
    });
    let remember = use_callback(move |previous: ChartZoom| {
        let mut stack = history.write();
        if stack.last() != Some(&previous) {
            if stack.len() == 100 {
                stack.remove(0);
            }
            stack.push(previous);
        }
    });
    let shade_start = left + zoom().start * plot_width;
    let shade_end = left + zoom().end * plot_width;
    let down_hover = hover_epoch.clone();
    let down_epoch = epoch.clone();
    let start_drag = use_callback(move |(event, handle): (PointerEvent, Option<bool>)| {
        if event.trigger_button() != Some(MouseButton::Primary) || drag.peek().is_some() {
            return;
        }
        let Some(node) = mounted.peek().clone() else {
            return;
        };
        event.prevent_default();
        down_hover.set(down_hover.get().wrapping_add(1));
        active.set(None);
        let initial = zoom();
        remember.call(initial);
        let kind = match handle {
            Some(true) => ZoomDrag::Start(initial),
            Some(false) => ZoomDrag::End(initial),
            None => ZoomDrag::Plot {
                initial,
                brush: brush(),
            },
        };
        begin_drag(
            drag,
            down_epoch.clone(),
            node,
            format!("ix-zoom-svg-{id}"),
            &event,
            kind,
            (width(), 600.0),
        );
    });
    let move_hover = hover_epoch.clone();
    let leave_hover = hover_epoch.clone();
    let wheel_epoch = epoch.clone();
    let wheel_hover = hover_epoch.clone();
    rsx! { div { class: "ix-zoom-chart", role: "group", aria_label: label.clone(),
        onresize: move |event| crate::charts::resize_chart_width(width, event),
        div { class: "ix-zoom-toolbar",
            crate::IconButton { icon:"zoom-selection",label:"Select zoom area",variant:crate::ButtonVariant::SubtleTertiary,onclick:move |_|{cancel_drag.call(None);active.set(None);brush.toggle();} }
            crate::IconButton { icon:"undo",label:"Restore previous zoom",variant:crate::ButtonVariant::SubtleTertiary,disabled:history.read().is_empty(),onclick:move |_|{cancel_drag.call(None);active.set(None);if let Some(previous)=history.write().pop(){zoom.set(previous);}} }
        }
        div { class: "ix-chart-viewport",
            onmounted:move |event: MountedEvent|mounted.set(Some(event.data())),
            svg { id: "ix-zoom-svg-{id}", width:"100%",height:"600",view_box:"0 0 {width} 600",role:"img","aria-label":label.clone(),tabindex:0,style:"touch-action:none",
                onwheel:move |event:WheelEvent|{
                    event.prevent_default(); if drag.peek().is_some() {cancel_drag.call(None);}
                    wheel_hover.set(wheel_hover.get().wrapping_add(1)); active.set(None);
                    let Some(node)=mounted.peek().clone() else{return;}; let client=event.client_coordinates();let factor=(event.delta().strip_units().y*0.002).exp();let epoch=wheel_epoch.clone();let sequence=epoch.get();let view_width=width();
                    spawn(async move {let Some(bounds)=mounted_svg_bounds(&node,(view_width,600.0)).await else{return;};if epoch.get()!=sequence{return;}let (x,_)=bounds.point((client.x,client.y));let anchor=((x-left)/plot_width).clamp(0.0,1.0);let Ok(current)=zoom.try_peek().map(|value|*value) else{return;};remember.call(current);if let Ok(mut value)=zoom.try_write(){*value=current.zoom(factor,anchor);}});
                },
                onkeydown:move |event:KeyboardEvent|{if event.key()==Key::Escape{event.prevent_default();cancel_drag.call(None);brush.set(false);active.set(None);return;}let current=zoom();let next=match event.key(){Key::ArrowLeft=>current.pan(-(current.end-current.start)*0.1),Key::ArrowRight=>current.pan((current.end-current.start)*0.1),Key::Home=>ChartZoom::default(),Key::Character(ref key) if key=="+"||key=="="=>current.zoom(0.8,0.5),Key::Character(ref key) if key=="-"=>current.zoom(1.25,0.5),_=>return};event.prevent_default();cancel_drag.call(None);remember.call(current);zoom.set(next);},
                onpointerdown:move |event:PointerEvent|start_drag.call((event,None)),
                onpointermove:move |event:PointerEvent|{
                    let client=event.client_coordinates();
                    if let Some(mut value)=drag(){
                        if !value.move_pointer(event.pointer_id(),(client.x,client.y)){return;}
                        event.prevent_default();drag.set(Some(value));active.set(None);
                        if let Some(next)=zoom_drag_window(value,left,right){zoom.set(next);}
                    } else if pointer_count>0 {
                        let Some(node)=mounted.peek().clone() else{return;};let epoch=move_hover.clone();let sequence=epoch.get().wrapping_add(1);epoch.set(sequence);let view_width=width();let visible_range=range.clone();
                        spawn(async move{let Some(bounds)=mounted_svg_bounds(&node,(view_width,600.0)).await else{return;};if epoch.get()!=sequence{return;}let (x,y)=bounds.point((client.x,client.y));let next=if x>=left&&x<=right&&(top..=bottom).contains(&y){Some(visible_range.start+(((x-left)/plot_width)*(visible_range.len().saturating_sub(1))as f64).round()as usize)}else{None};if let Ok(mut value)=active.try_write(){if *value!=next{*value=next;}}});
                    }
                },
                onpointerup:move |event:PointerEvent|{
                    let Some(mut value)=drag() else{return;};let point=event.client_coordinates();if !value.move_pointer(event.pointer_id(),(point.x,point.y)){return;}
                    if let Some((start,current,initial))=zoom_drag_brush(value,left,right){if (current-start).abs()>4.0{let span=initial.end-initial.start;zoom.set(ChartZoom::new(initial.start+(start.min(current)-left)/plot_width*span,initial.start+(start.max(current)-left)/plot_width*span));}brush.set(false);}
                    else if let Some(next)=zoom_drag_window(value,left,right){zoom.set(next);}
                    cancel_drag.call(Some(event.pointer_id()));active.set(None);
                },
                onpointercancel:move |event:PointerEvent|cancel_drag.call(Some(event.pointer_id())),
                onlostpointercapture:move |event:PointerEvent|cancel_drag.call(Some(event.pointer_id())),
                onpointerleave:move |_|{leave_hover.set(leave_hover.get().wrapping_add(1));active.set(None);},
                for tick in scale.ticks(){ {let y=bottom-scale.fraction(tick)*(bottom-top);rsx!{g{line{x1:left,y1:y,x2:right,y2:y,stroke:"var(--ix-color-border)",opacity:".3"}text{x:left-8.0,y:y+4.0,text_anchor:"end",fill:"var(--ix-color-text-secondary)","{tick:.0}"}}}} }
                path { d:area,fill:chart_color(0),opacity:".12" }
                path { d:path,fill:"none",stroke:chart_color(0),stroke_width:"2" }
                text {x:left,y:534,fill:"var(--ix-color-text-secondary)","{first_label}"}
                text {x:right,y:534,text_anchor:"end",fill:"var(--ix-color-text-secondary)","{last_label}"}
                rect{x:left,y:548,width:plot_width,height:42,fill:"var(--ix-color-surface-1)",stroke:"var(--ix-color-border)"}
                path{d:overview_path,fill:"none",stroke:chart_color(0),stroke_width:"1",opacity:".6"}
                rect{class:"ix-zoom-selection",x:shade_start,y:548,width:shade_end-shade_start,height:42}
                for (start,x) in [(true,shade_start),(false,shade_end)] {
                    rect { class:"ix-zoom-handle",x:x-4.0,y:545,width:8,height:48,rx:2,role:"slider",tabindex:0,
                        "aria-label":if start{"Start of visible data"}else{"End of visible data"},"aria-valuemin":0,"aria-valuemax":100,"aria-valuenow":if start{zoom().start*100.0}else{zoom().end*100.0},
                        onpointerdown:move |event:PointerEvent|{event.stop_propagation();start_drag.call((event,Some(start)));},
                        onkeydown:move |event:KeyboardEvent|{let step=if event.modifiers().shift(){0.1}else{0.01};let amount=match event.key(){Key::ArrowLeft|Key::ArrowDown=>-step,Key::ArrowRight|Key::ArrowUp=>step,_=>return};event.prevent_default();event.stop_propagation();cancel_drag.call(None);remember.call(zoom());let current=zoom();zoom.set(if start{ChartZoom::new((current.start+amount).clamp(0.0,current.end-0.0001),current.end)}else{ChartZoom::new(current.start,(current.end+amount).clamp(current.start+0.0001,1.0))});},
                    }
                }
                if let Some((start,current,_))=drag().and_then(|value|zoom_drag_brush(value,left,right)){rect{class:"ix-zoom-selection",x:start.min(current),y:top,width:(current-start).abs(),height:bottom-top}}
            }
        }
        if let Some(tooltip)=tooltip{output{class:"ix-chart-tooltip","{tooltip}"}}
        p{class:"ix-chart-instructions","Scroll to zoom, drag to pan, or move the overview handles. Arrow keys pan; +/− zoom; Home restores the initial 10%."}
    } }
}

pub(crate) const STYLES: &str = include_str!("styles/chart_advanced.css");

#[cfg(test)]
mod tests;
