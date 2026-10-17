//! Native Rust/SVG charts using the public iX Brand chart palette.
//!
//! These components do not evaluate JavaScript or load a chart runtime. Values,
//! geometry, legends, hit targets, tooltip navigation, and empty states are Rust
//! state. This is a deliberately typed chart API, rather than an ECharts options
//! interpreter. Category charts accept finite values, including negative values.
use dioxus::prelude::*;
use std::{
    cell::Cell,
    collections::BTreeMap,
    rc::Rc,
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};

static NEXT_CHART_ID: AtomicUsize = AtomicUsize::new(1);

/// The ECharts Brand palette order, resolved by the surrounding color scheme.
pub fn chart_color(index: usize) -> String {
    format!("var(--ix-chart-{})", index % 17 + 1)
}

/// A Cartesian series' geometric representation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ChartKind {
    #[default]
    Line,
    Bar,
}

/// Category-axis pointer displayed while reading a Cartesian tooltip.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ChartPointer {
    #[default]
    Line,
    Shadow,
    Cross,
}

/// Half-open category range selected by a horizontal chart brush. The indices
/// are relative to the chart's currently displayed category slice.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChartBrushSelection {
    pub start: usize,
    pub end: usize,
}
impl ChartBrushSelection {
    pub fn from_fractions(start: f64, end: f64, count: usize) -> Option<Self> {
        if count == 0 || !start.is_finite() || !end.is_finite() {
            return None;
        }
        let low = start.min(end).clamp(0.0, 1.0);
        let high = start.max(end).clamp(0.0, 1.0);
        let start = ((low * count as f64).floor() as usize).min(count - 1);
        let end = ((high * count as f64).ceil() as usize)
            .min(count)
            .max(start + 1);
        Some(Self { start, end })
    }
}

/// One Cartesian data series. Non-finite samples are treated as missing data.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ChartSeries {
    pub name: String,
    pub values: Vec<f64>,
    pub kind: ChartKind,
    /// Equal nonempty names stack bars in the same group and axis.
    pub stack: String,
    pub axis: usize,
    /// CSS color. An empty value selects the series' Brand palette color.
    pub color: String,
    pub smooth: bool,
    pub area: bool,
    pub step_end: bool,
    /// Fixed bar width in CSS pixels. `None` uses the available category space.
    pub bar_width: Option<f64>,
    /// Join this series' minimum and maximum samples with a dashed mark line.
    pub min_max_line: bool,
    /// Dim other series while this series is emphasized by pointer or keyboard.
    pub focus_series: bool,
}
impl ChartSeries {
    pub fn new(name: impl Into<String>, values: Vec<f64>, kind: ChartKind) -> Self {
        Self {
            name: name.into(),
            values,
            kind,
            ..Self::default()
        }
    }
}

/// Side on which a value axis is drawn.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ChartAxisSide {
    #[default]
    Left,
    Right,
}

/// Independent value scale, label, and unit for a Cartesian chart.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ChartAxis {
    pub name: String,
    pub unit: String,
    pub side: ChartAxisSide,
    pub offset: f64,
    pub color: String,
    pub minimum: Option<f64>,
    pub maximum: Option<f64>,
}

/// A finite plotting scale with a human-readable tick interval.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChartScale {
    pub minimum: f64,
    pub maximum: f64,
    pub step: f64,
}
impl ChartScale {
    pub fn new(minimum: f64, maximum: f64) -> Self {
        let low = if minimum.is_finite() {
            minimum.min(0.0)
        } else {
            0.0
        };
        let high = if maximum.is_finite() {
            maximum.max(0.0)
        } else {
            0.0
        };
        let span = (high - low).max(f64::EPSILON);
        let rough = if span <= f64::EPSILON {
            0.2
        } else {
            span / 5.0
        };
        let power = 10.0_f64.powf(rough.log10().floor());
        let fraction = rough / power;
        let nice = if fraction <= 1.0 {
            1.0
        } else if fraction <= 2.0 {
            2.0
        } else if fraction <= 3.0 {
            3.0
        } else if fraction <= 5.0 {
            5.0
        } else {
            10.0
        };
        let step = (nice * power).max(f64::MIN_POSITIVE);
        let minimum = (low / step).floor() * step;
        let maximum = (high / step).ceil() * step;
        Self {
            minimum,
            maximum: if maximum > minimum {
                maximum
            } else {
                minimum + 1.0
            },
            step: if high == low { 0.2 } else { step },
        }
    }
    pub fn fraction(self, value: f64) -> f64 {
        (value - self.minimum) / (self.maximum - self.minimum)
    }
    pub fn ticks(self) -> Vec<f64> {
        let count = (((self.maximum - self.minimum) / self.step).round() as usize).min(100);
        (0..=count)
            .map(|index| self.minimum + index as f64 * self.step)
            .collect()
    }
}

/// SVG line geometry; smoothing uses Catmull–Rom cubic control points.
/// Non-finite coordinates start a new subpath rather than poisoning the SVG.
pub fn chart_line_path(points: &[(f64, f64)], smooth: bool, step_end: bool) -> String {
    let mut path = String::new();
    let mut start = 0;
    while start < points.len() {
        while start < points.len() && (!points[start].0.is_finite() || !points[start].1.is_finite())
        {
            start += 1;
        }
        if start == points.len() {
            break;
        }
        let mut end = start + 1;
        while end < points.len() && points[end].0.is_finite() && points[end].1.is_finite() {
            end += 1;
        }
        path.push_str(&format!("M {:.3} {:.3}", points[start].0, points[start].1));
        for index in start + 1..end {
            let previous = points[index - 1];
            let current = points[index];
            if step_end {
                path.push_str(&format!("H {:.3} V {:.3}", current.0, current.1));
            } else if smooth {
                let before = points[index.saturating_sub(2).max(start)];
                let after = points[(index + 1).min(end - 1)];
                path.push_str(&format!(
                    "C {:.3} {:.3} {:.3} {:.3} {:.3} {:.3}",
                    previous.0 + (current.0 - before.0) / 6.0,
                    previous.1 + (current.1 - before.1) / 6.0,
                    current.0 - (after.0 - previous.0) / 6.0,
                    current.1 - (after.1 - previous.1) / 6.0,
                    current.0,
                    current.1
                ));
            } else {
                path.push_str(&format!("L {:.3} {:.3}", current.0, current.1));
            }
        }
        start = end;
    }
    path
}

/// Clockwise ring/sector path. Angles use radians; zero is twelve o'clock.
/// A complete circle uses two arcs, avoiding SVG's coincident-endpoint case.
pub fn chart_sector_path(cx: f64, cy: f64, outer: f64, inner: f64, start: f64, end: f64) -> String {
    if ![cx, cy, outer, inner, start, end]
        .into_iter()
        .all(f64::is_finite)
        || outer <= 0.0
        || end <= start
    {
        return String::new();
    }
    let inner = inner.clamp(0.0, outer);
    let point = |radius: f64, angle: f64| (cx + radius * angle.sin(), cy - radius * angle.cos());
    if end - start >= std::f64::consts::TAU - 1e-10 {
        let a = point(outer, start);
        let b = point(outer, start + std::f64::consts::PI);
        if inner > 0.0 {
            let c = point(inner, start);
            let d = point(inner, start + std::f64::consts::PI);
            return format!(
                "M {:.3} {:.3} A {outer:.3} {outer:.3} 0 1 1 {:.3} {:.3} A {outer:.3} {outer:.3} 0 1 1 {:.3} {:.3} M {:.3} {:.3} A {inner:.3} {inner:.3} 0 1 0 {:.3} {:.3} A {inner:.3} {inner:.3} 0 1 0 {:.3} {:.3} Z",
                a.0, a.1, b.0, b.1, a.0, a.1, c.0, c.1, d.0, d.1, c.0, c.1
            );
        }
        return format!(
            "M {:.3} {:.3} A {outer:.3} {outer:.3} 0 1 1 {:.3} {:.3} A {outer:.3} {outer:.3} 0 1 1 {:.3} {:.3} Z",
            a.0, a.1, b.0, b.1, a.0, a.1
        );
    }
    let sweep = end - start;
    let end = start + sweep;
    let a = point(outer, start);
    let b = point(outer, end);
    let large = usize::from(sweep > std::f64::consts::PI);
    if inner > 0.0 {
        let c = point(inner.min(outer), end);
        let d = point(inner.min(outer), start);
        format!(
            "M {:.3} {:.3} A {outer:.3} {outer:.3} 0 {large} 1 {:.3} {:.3} L {:.3} {:.3} A {inner:.3} {inner:.3} 0 {large} 0 {:.3} {:.3} Z",
            a.0, a.1, b.0, b.1, c.0, c.1, d.0, d.1
        )
    } else {
        format!(
            "M {cx:.3} {cy:.3} L {:.3} {:.3} A {outer:.3} {outer:.3} 0 {large} 1 {:.3} {:.3} Z",
            a.0, a.1, b.0, b.1
        )
    }
}

fn value_label(value: f64) -> String {
    if !value.is_finite() {
        return "—".into();
    }
    let rounded = (value * 100.0).round() / 100.0;
    if rounded.fract().abs() < 0.000_001 {
        format!("{rounded:.0}")
    } else {
        format!("{rounded:.2}").trim_end_matches('0').to_string()
    }
}

fn axis_scales(
    series: &[ChartSeries],
    visible: &[bool],
    axes: &[ChartAxis],
    categories: usize,
) -> Vec<ChartScale> {
    let count = axes
        .len()
        .max(series.iter().map(|s| s.axis + 1).max().unwrap_or(1));
    (0..count)
        .map(|axis| {
            let mut low: f64 = 0.0;
            let mut high: f64 = 0.0;
            for category in 0..categories {
                let mut stacks: BTreeMap<&str, (f64, f64)> = BTreeMap::new();
                for (index, series) in series.iter().enumerate().filter(|(_, s)| s.axis == axis) {
                    if !visible.get(index).copied().unwrap_or(true) {
                        continue;
                    }
                    let Some(value) = series
                        .values
                        .get(category)
                        .copied()
                        .filter(|v| v.is_finite())
                    else {
                        continue;
                    };
                    if series.kind == ChartKind::Bar && !series.stack.is_empty() {
                        let entry = stacks.entry(&series.stack).or_default();
                        if value < 0.0 {
                            entry.0 += value;
                        } else {
                            entry.1 += value;
                        }
                        low = low.min(entry.0);
                        high = high.max(entry.1);
                    } else {
                        low = low.min(value);
                        high = high.max(value);
                    }
                }
            }
            let mut scale = ChartScale::new(low, high);
            if let Some(config) = axes.get(axis) {
                if let Some(minimum) = config.minimum.filter(|v| v.is_finite()) {
                    scale.minimum = minimum;
                }
                if let Some(maximum) = config
                    .maximum
                    .filter(|v| v.is_finite() && *v > scale.minimum)
                {
                    scale.maximum = maximum;
                }
            }
            scale
        })
        .collect()
}

// MountedData is the common Rust document interface on web and desktop. A scoped
// task samples only this chart's bounds and is cancelled automatically on unmount.
// Unlike a JS ResizeObserver bridge this also works with the desktop renderer.
pub(crate) fn observe_chart_width(mut width: Signal<f64>, event: MountedEvent) {
    spawn(async move {
        loop {
            if width.try_peek().is_err() {
                return;
            }
            if let Ok(bounds) = event.get_client_rect().await {
                if !bounds.size.width.is_finite() || bounds.size.width <= 0.0 {
                    futures_timer::Delay::new(Duration::from_millis(500)).await;
                    continue;
                }
                let measured = bounds.size.width.max(240.0);
                let Ok(current) = width.try_peek().map(|value| *value) else {
                    return;
                };
                if (current - measured).abs() > 1.0 {
                    let Ok(mut value) = width.try_write() else {
                        return;
                    };
                    *value = measured;
                }
            }
            futures_timer::Delay::new(Duration::from_millis(500)).await;
        }
    });
}

fn chart_keyboard_target(
    points: &[(usize, usize)],
    current: Option<(usize, usize)>,
    key: &Key,
) -> Option<(usize, usize)> {
    let current = current
        .filter(|point| points.contains(point))
        .or_else(|| points.first().copied())?;
    let same_series = |point: &&(usize, usize)| point.0 == current.0;
    Some(match key {
        Key::ArrowRight => points
            .iter()
            .filter(same_series)
            .find(|point| point.1 > current.1)
            .copied()
            .unwrap_or(current),
        Key::ArrowLeft => points
            .iter()
            .filter(same_series)
            .rev()
            .find(|point| point.1 < current.1)
            .copied()
            .unwrap_or(current),
        Key::ArrowDown => points
            .iter()
            .find(|point| point.1 == current.1 && point.0 > current.0)
            .copied()
            .unwrap_or(current),
        Key::ArrowUp => points
            .iter()
            .rev()
            .find(|point| point.1 == current.1 && point.0 < current.0)
            .copied()
            .unwrap_or(current),
        Key::Home => points
            .iter()
            .find(|point| point.0 == current.0)
            .copied()
            .unwrap_or(current),
        Key::End => points
            .iter()
            .rev()
            .find(|point| point.0 == current.0)
            .copied()
            .unwrap_or(current),
        Key::Enter => current,
        Key::Character(value) if value == " " => current,
        _ => return None,
    })
}

fn focus_chart_host(mounted: Signal<Option<Rc<MountedData>>>) {
    let Some(node) = mounted.try_peek().ok().and_then(|node| node.clone()) else {
        return;
    };
    spawn(async move {
        let _ = node.set_focus(true).await;
    });
}

fn scroll_chart_category(
    mounted: Signal<Option<Rc<MountedData>>>,
    coordinate: f64,
    viewport: f64,
    content: f64,
) {
    if content <= viewport + 1.0 {
        return;
    }
    let Some(node) = mounted.try_peek().ok().and_then(|node| node.clone()) else {
        return;
    };
    let offset = (coordinate - viewport / 2.0).clamp(0.0, (content - viewport).max(0.0));
    spawn(async move {
        let _ = node
            .scroll(
                dioxus::html::geometry::PixelsVector2D::new(offset, 0.0),
                dioxus::html::ScrollBehavior::Instant,
            )
            .await;
    });
}

#[derive(Clone, Copy, Debug)]
struct PlotBounds {
    left: f64,
    right: f64,
    top: f64,
    bottom: f64,
}
impl PlotBounds {
    fn width(self) -> f64 {
        self.right - self.left
    }
    fn height(self) -> f64 {
        self.bottom - self.top
    }
}

/// Responsive category chart with independent value axes and stack-aware scales.
///
/// Tab into the chart and use the arrow keys to read neighboring samples. One
/// HTML focus host exposes the current SVG option through aria-activedescendant.
/// legend is composed of native toggle buttons; hiding a series recalculates its
/// value scale and stack. A data table exposes every unrounded source sample.
/// `minimum_width` preserves plot density in a horizontal scroll area when
/// several axes or dense categories would otherwise consume the viewport.
#[component]
pub fn CartesianChart(
    categories: Vec<String>,
    series: Vec<ChartSeries>,
    #[props(default = "Chart".into())] label: String,
    #[props(default)] x_title: String,
    #[props(default)] y_title: String,
    #[props(default)] horizontal: bool,
    #[props(default = true)] legend: bool,
    #[props(default = true)] tooltip: bool,
    #[props(default)] axis_tooltip: bool,
    #[props(default)] pointer: ChartPointer,
    #[props(default)] dashed_grid: bool,
    #[props(default)] mark_extremes: bool,
    #[props(default)] average_line: bool,
    #[props(default)] axes: Vec<ChartAxis>,
    #[props(default)] minimum_width: f64,
    #[props(default)] brush_enabled: bool,
    #[props(default)] onbrush: EventHandler<ChartBrushSelection>,
    #[props(default)] onbrushcancel: EventHandler<()>,
) -> Element {
    let id = use_hook(|| NEXT_CHART_ID.fetch_add(1, Ordering::Relaxed));
    let width = use_signal(|| 720.0_f64);
    let mut hidden = use_signal(Vec::<usize>::new);
    let mut active = use_signal(|| None::<(usize, usize)>);
    let mut keyboard_point = use_signal(|| None::<(usize, usize)>);
    let mut canvas_focused = use_signal(|| false);
    let mut hover_position = use_signal(|| None::<(f64, f64)>);
    let mut mounted = use_signal(|| None::<Rc<MountedData>>);
    let mut plot_mounted = use_signal(|| None::<Rc<MountedData>>);
    let pointer_epoch = use_hook(|| Rc::new(Cell::new(0_u64)));
    let mut brush_drag = use_signal(|| None::<(f64, f64)>);
    let mut brush_bounds = use_signal(|| None::<(f64, f64)>);
    let mut brush_keyboard_anchor = use_signal(|| None::<usize>);
    let brush_epoch = use_hook(|| Rc::new(Cell::new(0_u64)));
    let brush_disable_epoch = brush_epoch.clone();
    use_effect(use_reactive!(|brush_enabled| {
        if !brush_enabled {
            brush_disable_epoch.set(brush_disable_epoch.get().wrapping_add(1));
            brush_drag.set(None);
            brush_bounds.set(None);
            brush_keyboard_anchor.set(None);
        }
    }));
    let visible: Vec<bool> = (0..series.len())
        .map(|index| !hidden.read().contains(&index))
        .collect();
    let valid_points: Vec<(usize, usize)> = series
        .iter()
        .enumerate()
        .filter(|(index, _)| visible[*index])
        .flat_map(|(series, item)| {
            item.values
                .iter()
                .take(categories.len())
                .enumerate()
                .filter(|(_, value)| value.is_finite())
                .map(move |(category, _)| (series, category))
        })
        .collect();
    let keyboard_entry = keyboard_point()
        .filter(|point| valid_points.contains(point))
        .or_else(|| valid_points.first().copied());
    let axes = if axes.is_empty() {
        vec![ChartAxis {
            name: y_title,
            ..ChartAxis::default()
        }]
    } else {
        axes
    };
    let scales = axis_scales(&series, &visible, &axes, categories.len());
    let viewport_width = width();
    let chart_width = if minimum_width.is_finite() {
        viewport_width.max(minimum_width)
    } else {
        viewport_width
    };
    let right_axes = axes
        .iter()
        .filter(|axis| axis.side == ChartAxisSide::Right)
        .count();
    let right_margin = if right_axes > 1 {
        140.0_f64.min((chart_width - 138.0).max(80.0))
    } else if right_axes == 1 {
        66.0
    } else {
        28.0
    };
    let axis_offset = |axis: &ChartAxis| axis.offset.min((right_margin - 65.0).max(0.0));
    let plot = PlotBounds {
        left: if horizontal { 55.0 } else { 58.0 },
        right: chart_width - right_margin,
        top: if axes.len() > 1 && chart_width < 480.0 {
            85.0
        } else {
            55.0
        },
        bottom: 535.0,
    };
    let count = categories.len().max(1);
    let category_span = if horizontal {
        plot.height()
    } else {
        plot.width()
    } / count as f64;
    let category_position = move |index: usize| {
        if horizontal {
            plot.bottom - (index as f64 + 0.5) * category_span
        } else {
            plot.left + (index as f64 + 0.5) * category_span
        }
    };
    let value_position = |value: f64, axis: usize| {
        let fraction = scales
            .get(axis)
            .copied()
            .unwrap_or(scales[0])
            .fraction(value);
        if horizontal {
            plot.left + fraction * plot.width()
        } else {
            plot.bottom - fraction * plot.height()
        }
    };
    let mut groups: Vec<(usize, String)> = Vec::new();
    for (index, item) in series
        .iter()
        .enumerate()
        .filter(|(index, s)| visible[*index] && s.kind == ChartKind::Bar)
    {
        let key = (
            item.axis,
            if item.stack.is_empty() {
                format!("series-{index}")
            } else {
                item.stack.clone()
            },
        );
        if !groups.contains(&key) {
            groups.push(key);
        }
    }
    let group_width = category_span * 0.72 / groups.len().max(1) as f64;
    let bar_geometry = |series_index: usize, item: &ChartSeries| {
        let key = (
            item.axis,
            if item.stack.is_empty() {
                format!("series-{series_index}")
            } else {
                item.stack.clone()
            },
        );
        let group_index = groups
            .iter()
            .position(|entry| *entry == key)
            .unwrap_or_default();
        // An explicit width on one stacked series applies to its entire stack.
        let explicit = series
            .iter()
            .enumerate()
            .filter(|(index, candidate)| {
                visible[*index]
                    && candidate.kind == ChartKind::Bar
                    && candidate.axis == item.axis
                    && if item.stack.is_empty() {
                        *index == series_index
                    } else {
                        candidate.stack == item.stack
                    }
            })
            .find_map(|(_, candidate)| candidate.bar_width);
        let bar_width = explicit
            .unwrap_or(group_width * 0.9)
            .max(1.0)
            .min(group_width);
        let offset = -category_span * 0.36 + (group_index as f64 + 0.5) * group_width;
        (offset, bar_width)
    };
    let mut marks = Vec::<Element>::new();
    let mut stacks: BTreeMap<(usize, String, usize, bool), f64> = BTreeMap::new();
    for (series_index, item) in series.iter().enumerate() {
        if !visible[series_index] {
            continue;
        }
        let color = if item.color.is_empty() {
            chart_color(series_index)
        } else {
            item.color.clone()
        };
        let (bar_offset, bar_width) = bar_geometry(series_index, item);
        let points: Vec<(f64, f64)> = item
            .values
            .iter()
            .take(categories.len())
            .enumerate()
            .map(|(index, value)| {
                let offset = if item.kind == ChartKind::Bar {
                    bar_offset
                } else {
                    0.0
                };
                if horizontal {
                    (
                        value_position(*value, item.axis),
                        category_position(index) + offset,
                    )
                } else {
                    (
                        category_position(index) + offset,
                        value_position(*value, item.axis),
                    )
                }
            })
            .collect();
        if item.kind == ChartKind::Line {
            let path = chart_line_path(&points, item.smooth, item.step_end);
            if item.area
                && !points.is_empty()
                && points.iter().all(|(x, y)| x.is_finite() && y.is_finite())
            {
                let (first, last) = (points[0], points[points.len() - 1]);
                let baseline = value_position(0.0, item.axis);
                let area = if horizontal {
                    format!(
                        "{path} L {baseline:.3} {:.3} L {baseline:.3} {:.3} Z",
                        last.1, first.1
                    )
                } else {
                    format!(
                        "{path} L {:.3} {baseline:.3} L {:.3} {baseline:.3} Z",
                        last.0, first.0
                    )
                };
                marks.push(rsx! { path { d: area, fill: "url(#ix-chart-area-{id}-{series_index})", stroke: "none" } });
            }
            marks.push(rsx! { path { class: "ix-chart-line", d: path, fill: "none", stroke: color.clone(), stroke_width: if item.smooth { "4" } else { "2" } } });
        }
        for (category_index, value) in item
            .values
            .iter()
            .copied()
            .take(categories.len())
            .enumerate()
            .filter(|(_, value)| value.is_finite())
        {
            let category = &categories[category_index];
            let name = if item.name.is_empty() {
                category.clone()
            } else {
                format!("{category}, {}", item.name)
            };
            let accessible = format!(
                "{name}: {}{}",
                value_label(value),
                axes.get(item.axis).map(|a| a.unit.as_str()).unwrap_or("")
            );
            let selected = active() == Some((series_index, category_index));
            let dimmed = active().is_some_and(|(selected, _)| {
                selected != series_index
                    && series
                        .get(selected)
                        .is_some_and(|series| series.focus_series)
            });
            let at = category_position(category_index);
            let shape = if item.kind == ChartKind::Bar {
                let position = at + bar_offset - bar_width / 2.0;
                let baseline = if item.stack.is_empty() {
                    0.0
                } else {
                    *stacks
                        .entry((item.axis, item.stack.clone(), category_index, value < 0.0))
                        .or_default()
                };
                let end = baseline + value;
                if !item.stack.is_empty() {
                    stacks.insert(
                        (item.axis, item.stack.clone(), category_index, value < 0.0),
                        end,
                    );
                }
                let a = value_position(baseline, item.axis);
                let b = value_position(end, item.axis);
                if horizontal {
                    rsx! { rect { x: a.min(b), y: position, width: (a-b).abs(), height: bar_width, fill: color.clone() } }
                } else {
                    rsx! { rect { x: position, y: a.min(b), width: bar_width, height: (a-b).abs(), fill: color.clone() } }
                }
            } else {
                let (x, y) = points[category_index];
                rsx! { circle { cx: x, cy: y, r: if selected { "5" } else { "2" }, fill: "var(--ix-color-surface)", stroke: color.clone(), stroke_width: "2" }
                    circle { cx: x, cy: y, r: "10", fill: "transparent", stroke: "none" }
                }
            };
            marks.push(rsx! { g {
                class: format!("ix-chart-mark{}{}", if selected { " is-active" } else { "" }, if keyboard_entry == Some((series_index, category_index)) { " is-keyboard-active" } else { "" }),
                opacity: if dimmed { "0.15" } else { "1" },
                id: "ix-chart-{id}-{series_index}-{category_index}",
                role: "option", "aria-selected": keyboard_entry == Some((series_index, category_index)), "aria-label": accessible.clone(),
                onmouseenter: move |_| active.set(Some((series_index, category_index))),
                onmouseleave: move |_| active.set(None),
                onclick: move |_| { keyboard_point.set(Some((series_index, category_index))); active.set(Some((series_index, category_index))); focus_chart_host(mounted); },
                title { "{accessible}" }
                {shape}
            } });
        }
        if item.min_max_line {
            let extrema = finite_extrema(&item.values);
            if let Some((minimum, maximum)) = extrema {
                if let (Some(a), Some(b)) = (points.get(minimum), points.get(maximum)) {
                    marks.push(rsx! { line { x1: a.0, y1: a.1, x2: b.0, y2: b.1, stroke: color.clone(), stroke_dasharray: "5 4", stroke_width: "1.5" } });
                }
            }
        }
        if average_line && !item.values.is_empty() {
            let finite: Vec<f64> = item
                .values
                .iter()
                .copied()
                .filter(|v| v.is_finite())
                .collect();
            if !finite.is_empty() {
                let average = finite.iter().sum::<f64>() / finite.len() as f64;
                let position = value_position(average, item.axis);
                marks.push(rsx! { line { x1: plot.left, y1: position, x2: plot.right, y2: position, stroke: color.clone(), stroke_dasharray: "5 4" }
                    text { x: plot.right, y: position - 7.0, text_anchor: "end", fill: color.clone(), "Avg: {value_label(average)}" }
                });
            }
        }
        if mark_extremes {
            if let Some((minimum, maximum)) = finite_extrema(&item.values) {
                for index in [minimum, maximum] {
                    if let Some(&(x, y)) = points.get(index) {
                        marks.push(rsx! { g { class: "ix-chart-extreme", circle { cx: x, cy: y, r: "30", fill: color.clone() }
                            text { x, y: y + 4.0, text_anchor: "middle", fill: "var(--ix-color-on-primary)", font_weight: "bold", "{value_label(item.values[index])}" }
                        } });
                    }
                }
            }
        }
    }
    let active_point =
        active().filter(|(s, c)| *s < series.len() && *c < categories.len() && visible[*s]);
    let tooltip_rows: Vec<(String, String, String)> = active_point
        .map(|(selected, category)| {
            series
                .iter()
                .enumerate()
                .filter(|(i, s)| {
                    visible[*i]
                        && (axis_tooltip || *i == selected)
                        && s.values.get(category).is_some_and(|v| v.is_finite())
                })
                .map(|(index, s)| {
                    (
                        if s.name.is_empty() {
                            categories[category].clone()
                        } else {
                            s.name.clone()
                        },
                        format!(
                            "{}{}",
                            value_label(s.values[category]),
                            axes.get(s.axis).map(|a| a.unit.as_str()).unwrap_or("")
                        ),
                        if s.color.is_empty() {
                            chart_color(index)
                        } else {
                            s.color.clone()
                        },
                    )
                })
                .collect()
        })
        .unwrap_or_default();
    let label_spacing = (categories
        .iter()
        .map(|category| category.chars().count())
        .max()
        .unwrap_or(0) as f64
        * 6.2
        + 12.0)
        .clamp(45.0, 120.0);
    let label_count = ((plot.width() / label_spacing).floor() as usize)
        .max(2)
        .min(categories.len());
    let label_indices: Vec<usize> = (0..label_count)
        .map(|tick| {
            (tick as f64 * categories.len().saturating_sub(1) as f64
                / label_count.saturating_sub(1).max(1) as f64)
                .round() as usize
        })
        .collect();
    let pointer_values = series.clone();
    let pointer_visible = visible.clone();
    let move_epoch = pointer_epoch.clone();
    let pointer_move = move |event: MouseEvent| {
        if !axis_tooltip || brush_enabled {
            return;
        }
        let Some(node) = plot_mounted.try_peek().ok().and_then(|node| node.clone()) else {
            return;
        };
        let client = event.client_coordinates();
        let values = pointer_values.clone();
        let visible = pointer_visible.clone();
        let epoch = move_epoch.clone();
        let sequence = epoch.get().wrapping_add(1);
        epoch.set(sequence);
        spawn(async move {
            let Ok(bounds) = node.get_client_rect().await else {
                return;
            };
            let scroll = node.get_scroll_offset().await.unwrap_or_default();
            if epoch.get() != sequence {
                return;
            }
            let x = client.x - bounds.origin.x + scroll.x;
            let y = client.y - bounds.origin.y + scroll.y;
            if x < plot.left || x > plot.right || y < plot.top || y > plot.bottom {
                return;
            }
            let fraction = if horizontal {
                (plot.bottom - y) / plot.height()
            } else {
                (x - plot.left) / plot.width()
            };
            let category = ((fraction * count as f64).floor() as usize).min(count - 1);
            let available = |index: &usize| {
                visible.get(*index).copied().unwrap_or(false)
                    && values
                        .get(*index)
                        .and_then(|s| s.values.get(category))
                        .is_some_and(|v| v.is_finite())
            };
            let Ok(current_active) = active.try_peek().map(|value| *value) else {
                return;
            };
            let selected = current_active
                .map(|(series, _)| series)
                .filter(available)
                .or_else(|| (0..values.len()).find(available));
            if let Some(series) = selected {
                if current_active != Some((series, category)) {
                    let Ok(mut value) = active.try_write() else {
                        return;
                    };
                    *value = Some((series, category));
                }
                if let Ok(mut value) = hover_position.try_write() {
                    *value = Some((x, y));
                }
            }
        });
    };
    let keyboard_epoch = pointer_epoch.clone();
    let focus_points = valid_points.clone();
    let tooltip_space = 48 + 24 * if axis_tooltip { series.len().max(1) } else { 1 };
    let brush_start_epoch = brush_epoch.clone();
    let brush_finish_epoch = brush_epoch.clone();
    let brush_cancel_epoch = brush_epoch.clone();
    let selection = if brush_enabled {
        if let (Some((start, end)), Some((left, scroll))) = (brush_drag(), brush_bounds()) {
            Some((
                (start.min(end) - left + scroll).clamp(plot.left, plot.right),
                (start.max(end) - left + scroll).clamp(plot.left, plot.right),
            ))
        } else if let (Some(start), Some((_, end))) = (brush_keyboard_anchor(), keyboard_entry) {
            Some((
                (category_position(start.min(end)) - category_span / 2.0).max(plot.left),
                (category_position(start.max(end)) + category_span / 2.0).min(plot.right),
            ))
        } else {
            None
        }
    } else {
        None
    };
    rsx! {
        figure { class: "ix-chart ix-chart-cartesian", "aria-label": label.clone(),
            p { class: "ix-chart-scroll-hint", hidden: chart_width <= viewport_width + 1.0, "Scroll horizontally or use arrow keys to view the full chart." }
            div { class: "ix-chart-canvas", role: "listbox", tabindex: if valid_points.is_empty() { "-1" } else { "0" }, "aria-label": label.clone(),
                "aria-activedescendant": if canvas_focused() { keyboard_entry.map(|(series, category)| format!("ix-chart-{id}-{series}-{category}")) } else { None },
                onmounted: move |event| { mounted.set(Some(event.data())); observe_chart_width(width, event); },
                onfocus: move |_| { canvas_focused.set(true); let target = (*keyboard_point.peek()).filter(|point| focus_points.contains(point)).or(keyboard_entry); if let Some(point) = target { keyboard_point.set(Some(point)); active.set(Some(point)); } },
                onblur: move |_| { canvas_focused.set(false); active.set(None); },
                onkeydown: move |event: KeyboardEvent| {
                    if event.key() == Key::Escape { event.prevent_default(); keyboard_epoch.set(keyboard_epoch.get().wrapping_add(1)); brush_epoch.set(brush_epoch.get().wrapping_add(1)); active.set(None); brush_keyboard_anchor.set(None); brush_drag.set(None); if brush_enabled { onbrushcancel.call(()); } return; }
                    if brush_enabled && event.key() == Key::Enter {
                        event.prevent_default();
                        if let Some((_, end)) = keyboard_entry { if let Some(start) = brush_keyboard_anchor() { brush_keyboard_anchor.set(None); onbrush.call(ChartBrushSelection { start: start.min(end), end: start.max(end) + 1 }); } else { brush_keyboard_anchor.set(Some(end)); } }
                        return;
                    }
                    if let Some(target) = chart_keyboard_target(&valid_points, keyboard_entry, &event.key()) {
                        event.prevent_default(); keyboard_epoch.set(keyboard_epoch.get().wrapping_add(1)); hover_position.set(None); keyboard_point.set(Some(target)); active.set(Some(target));
                        scroll_chart_category(plot_mounted, if horizontal { plot.left } else { category_position(target.1) }, viewport_width, chart_width);
                    }
                },
                onmouseleave: move |_| { pointer_epoch.set(pointer_epoch.get().wrapping_add(1)); active.set(None); hover_position.set(None); },
                div { class: if tooltip { "ix-chart-tooltip-region" } else { "ix-chart-tooltip-region is-disabled" }, style: "--ix-chart-tooltip-space:{tooltip_space}px",
                    if tooltip && !tooltip_rows.is_empty() {
                        div { class: "ix-chart-tooltip", role: "status", "aria-live": "polite",
                            if let Some((_, category)) = active_point { strong { "{categories[category]}" } }
                            for (name, value, color) in tooltip_rows { div { span { class: "ix-chart-swatch", style: "background:{color}" } span { "{name}" } b { "{value}" } } }
                        }
                    } else {
                        span { class: "ix-chart-tooltip-placeholder", "aria-hidden": "true", "Select a data point to inspect values." }
                    }
                }
                div { class: "ix-chart-plot-scroll", onmounted: move |event| plot_mounted.set(Some(event.data())),
                svg { id: "ix-chart-svg-{id}", width: chart_width, height: "600", view_box: "0 0 {chart_width} 600", role: "group", "aria-label": label.clone(), onmousemove: pointer_move,
                    style: if brush_enabled { "touch-action:none;cursor:crosshair" } else { "" },
                    onpointerdown: move |event: PointerEvent| {
                        if !brush_enabled || !event.is_primary() || event.trigger_button().is_some_and(|button| button != dioxus::html::input_data::MouseButton::Primary) { return; }
                        event.prevent_default();
                        let point = event.client_coordinates(); let pointer = event.pointer_id();
                        brush_keyboard_anchor.set(None); brush_drag.set(Some((point.x, point.x))); brush_bounds.set(None); active.set(None);
                        let Some(node) = plot_mounted.try_peek().ok().and_then(|node| node.clone()) else { return; };
                        let epoch = brush_start_epoch.clone(); let sequence = epoch.get().wrapping_add(1); epoch.set(sequence);
                        spawn(async move {
                            let Ok(bounds) = node.get_client_rect().await else { return; };
                            let scroll = node.get_scroll_offset().await.unwrap_or_default();
                            if epoch.get() != sequence { return; }
                            if brush_drag.try_peek().map(|value| value.is_none()).unwrap_or(true) { return; }
                            let x = point.x - bounds.origin.x + scroll.x;
                            let y = point.y - bounds.origin.y + scroll.y;
                            if x < plot.left || x > plot.right || y < plot.top || y > plot.bottom { if let Ok(mut value) = brush_drag.try_write() { *value = None; } return; }
                            if let Ok(mut value) = brush_bounds.try_write() { *value = Some((bounds.origin.x, scroll.x)); }
                            let _ = crate::capture_chart_pointer(&format!("ix-chart-svg-{id}"), pointer).await;
                        });
                    },
                    onpointermove: move |event: PointerEvent| { if brush_enabled { if let Some((start, _)) = brush_drag() { event.prevent_default(); brush_drag.set(Some((start, event.client_coordinates().x))); } } },
                    onpointerup: move |event: PointerEvent| {
                        if !brush_enabled { return; }
                        let selection = brush_drag.write().take(); brush_bounds.set(None);
                        let epoch = brush_finish_epoch.clone(); let sequence = epoch.get().wrapping_add(1); epoch.set(sequence);
                        let Some((start, _)) = selection else { return; }; let end = event.client_coordinates().x;
                        if (end - start).abs() < 4.0 { return; }
                        event.prevent_default();
                        let Some(node) = plot_mounted.try_peek().ok().and_then(|node| node.clone()) else { return; };
                        spawn(async move {
                            let Ok(bounds) = node.get_client_rect().await else { return; }; let scroll = node.get_scroll_offset().await.unwrap_or_default();
                            if epoch.get() != sequence { return; }
                            if let Some(selection) = ChartBrushSelection::from_fractions((start - bounds.origin.x + scroll.x - plot.left) / plot.width(), (end - bounds.origin.x + scroll.x - plot.left) / plot.width(), count) { onbrush.call(selection); }
                        });
                    },
                    onpointercancel: move |_| { brush_cancel_epoch.set(brush_cancel_epoch.get().wrapping_add(1)); brush_drag.set(None); brush_bounds.set(None); },
                    defs { for (index, item) in series.iter().enumerate() { linearGradient { id: "ix-chart-area-{id}-{index}", x1: "0", y1: "0", x2: "0", y2: "1",
                        stop { offset: "0", stop_color: if item.color.is_empty() { chart_color(index) } else { item.color.clone() }, stop_opacity: "0.65" }
                        stop { offset: "1", stop_color: if item.color.is_empty() { chart_color(index) } else { item.color.clone() }, stop_opacity: "0" }
                    } } }
                    for (axis_index, axis) in axes.iter().enumerate() {
                        for tick in scales[axis_index].ticks() {
                            if horizontal {
                                line { class: "ix-chart-grid", x1: value_position(tick, axis_index), y1: plot.top, x2: value_position(tick, axis_index), y2: plot.bottom, stroke_dasharray: if dashed_grid { "4 4" } else { "none" } }
                                text { x: value_position(tick, axis_index), y: plot.bottom + 22.0, text_anchor: "middle", "{value_label(tick)}{axis.unit}" }
                            } else {
                                if axis_index == 0 { line { class: "ix-chart-grid", x1: plot.left, y1: value_position(tick, axis_index), x2: plot.right, y2: value_position(tick, axis_index), stroke_dasharray: if dashed_grid { "4 4" } else { "none" } } }
                                text { x: if axis.side == ChartAxisSide::Left { plot.left - 10.0 } else { plot.right + 10.0 + axis_offset(axis) }, y: value_position(tick, axis_index) + 4.0, text_anchor: if axis.side == ChartAxisSide::Left { "end" } else { "start" }, fill: if axis.color.is_empty() { "var(--ix-color-text)".into() } else { axis.color.clone() }, "{value_label(tick)}{axis.unit}" }
                            }
                        }
                        if !axis.name.is_empty() {
                            if horizontal { text { x: plot.left - 10.0, y: plot.top - 18.0, text_anchor: "end", "{axis.name}" } }
                            else { text { x: if axis.side == ChartAxisSide::Left { plot.left } else { plot.right + axis_offset(axis) }, y: if axes.len() > 1 && chart_width < 480.0 { 15.0 + axis_index as f64 * 20.0 } else if axes.len() > 1 && axis.side == ChartAxisSide::Left { 14.0 } else { 30.0 - (axis_index % 2) as f64 * 16.0 }, text_anchor: if axis.side == ChartAxisSide::Left { "start" } else { "end" }, fill: if axis.color.is_empty() { "var(--ix-color-text)".into() } else { axis.color.clone() },
                                for (line, text) in axis.name.lines().enumerate() { tspan { x: if axis.side == ChartAxisSide::Left { plot.left } else { plot.right + axis_offset(axis) }, dy: if line == 0 { "0" } else { "14" }, "{text}" } }
                            } }
                        }
                    }
                    line { class: "ix-chart-axis", x1: plot.left, y1: plot.bottom, x2: plot.right, y2: plot.bottom }
                    line { class: "ix-chart-axis", x1: plot.left, y1: plot.top, x2: plot.left, y2: plot.bottom }
                    for (index, category) in categories.iter().enumerate() {
                        g { key: "category-{index}",
                            if horizontal { text { x: plot.left - 10.0, y: category_position(index) + 4.0, text_anchor: "end", "{category}" } }
                            else if label_indices.contains(&index) { text { x: category_position(index), y: plot.bottom + 24.0, text_anchor: "middle", "{category}" } }
                        }
                    }
                    if !x_title.is_empty() { text { x: (plot.left + plot.right) / 2.0, y: 590.0, text_anchor: "middle", "{x_title}" } }
                    if axis_tooltip { if let Some((selected, category)) = active_point {
                        if pointer == ChartPointer::Shadow {
                            if horizontal { rect { class: "ix-chart-shadow", x: plot.left, y: category_position(category) - category_span / 2.0, width: plot.width(), height: category_span } }
                            else { rect { class: "ix-chart-shadow", x: category_position(category) - category_span / 2.0, y: plot.top, width: category_span, height: plot.height() } }
                        } else {
                            if horizontal { line { class: "ix-chart-pointer", x1: plot.left, x2: plot.right, y1: category_position(category), y2: category_position(category) } }
                            else { line { class: "ix-chart-pointer", x1: category_position(category), x2: category_position(category), y1: plot.top, y2: plot.bottom } }
                            if pointer == ChartPointer::Cross {
                                if let Some(value) = series[selected].values.get(category) {
                                    if horizontal { line { class: "ix-chart-pointer", x1: hover_position().map(|p| p.0).unwrap_or_else(|| value_position(*value, series[selected].axis)), x2: hover_position().map(|p| p.0).unwrap_or_else(|| value_position(*value, series[selected].axis)), y1: plot.top, y2: plot.bottom } }
                                    else { line { class: "ix-chart-pointer", x1: plot.left, x2: plot.right, y1: hover_position().map(|p| p.1).unwrap_or_else(|| value_position(*value, series[selected].axis)), y2: hover_position().map(|p| p.1).unwrap_or_else(|| value_position(*value, series[selected].axis)) } }
                                }
                            }
                        }
                    } }
                    {marks.into_iter()}
                    g { "data-chart-export-exclude": "true",
                        if brush_enabled { rect { class: "ix-chart-brush-hit", x: plot.left, y: plot.top, width: plot.width(), height: plot.height(), fill: "transparent" } }
                        if let Some((left, right)) = selection { rect { class: "ix-chart-brush-selection", x: left, y: plot.top, width: (right-left).max(1.0), height: plot.height() } }
                    }
                }
                }
            }
            if legend { div { class: "ix-chart-legend", "aria-label": "Chart series",
                for (index, item) in series.iter().enumerate().filter(|(_, item)| !item.name.is_empty()) {
                    button { r#type: "button", "aria-pressed": visible[index], class: if visible[index] { "" } else { "is-hidden" }, onclick: move |_| { let mut values = hidden.write(); if let Some(position) = values.iter().position(|value| *value == index) { values.remove(position); } else { values.push(index); } active.set(None); },
                        span { class: "ix-chart-swatch", style: format!("background:{}", if item.color.is_empty() { chart_color(index) } else { item.color.clone() }) }
                        "{item.name}"
                    }
                }
            } }
            details { class: "ix-chart-data", summary { "View chart data" }
                div { class: "ix-chart-table-scroll",
                    table {
                        caption { "{label}" }
                        thead { tr { th { scope: "col", "Category" } for item in &series { th { scope: "col", "{item.name}" } } } }
                        tbody { for (category, name) in categories.iter().enumerate() {
                            tr { th { scope: "row", "{name}" }
                                for item in &series { td { {item.values.get(category).filter(|v| v.is_finite()).map(|v| v.to_string()).unwrap_or_else(|| "—".into())} } }
                            }
                        } }
                    }
                }
            }
        }
    }
}

fn finite_extrema(values: &[f64]) -> Option<(usize, usize)> {
    let finite: Vec<(usize, &f64)> = values
        .iter()
        .enumerate()
        .filter(|(_, value)| value.is_finite())
        .collect();
    Some((
        finite.iter().min_by(|a, b| a.1.total_cmp(b.1))?.0,
        finite.iter().max_by(|a, b| a.1.total_cmp(b.1))?.0,
    ))
}

/// A positive slice in a pie or doughnut chart.
#[derive(Clone, Debug, PartialEq)]
pub struct ChartSlice {
    pub name: String,
    pub value: f64,
}

/// Pie/doughnut with direct labels, toggleable legend, and focusable slices.
#[component]
pub fn PieChart(
    slices: Vec<ChartSlice>,
    #[props(default)] donut: bool,
    #[props(default = "Chart".into())] label: String,
) -> Element {
    let id = use_hook(|| NEXT_CHART_ID.fetch_add(1, Ordering::Relaxed));
    let mut hidden = use_signal(Vec::<usize>::new);
    let mut active = use_signal(|| None::<usize>);
    let mut keyboard_point = use_signal(|| None::<usize>);
    let mut canvas_focused = use_signal(|| false);
    let mut mounted = use_signal(|| None::<Rc<MountedData>>);
    let width = use_signal(|| 720.0_f64);
    let valid_points: Vec<(usize, usize)> = slices
        .iter()
        .enumerate()
        .filter(|(index, slice)| {
            !hidden.read().contains(index) && slice.value.is_finite() && slice.value > 0.0
        })
        .map(|(index, _)| (0, index))
        .collect();
    let keyboard_entry = keyboard_point()
        .filter(|index| valid_points.contains(&(0, *index)))
        .or_else(|| valid_points.first().map(|point| point.1));
    let total: f64 = slices
        .iter()
        .enumerate()
        .filter(|(i, slice)| {
            !hidden.read().contains(i) && slice.value.is_finite() && slice.value > 0.0
        })
        .map(|(_, slice)| slice.value)
        .sum();
    let chart_width = width();
    let radius = (chart_width * 0.36).min(if donut { 240.0 } else { 224.0 });
    let center = (chart_width / 2.0, 290.0);
    let mut start = 0.0;
    let segments: Vec<Element> = slices.iter().enumerate().filter_map(|(index, slice)| {
        if hidden.read().contains(&index) || slice.value <= 0.0 || !slice.value.is_finite() || total <= 0.0 { return None; }
        let end = start + slice.value / total * std::f64::consts::TAU;
        let middle = (start + end) / 2.0;
        let outer = if active() == Some(index) { radius + 5.0 } else { radius };
        let path = chart_sector_path(center.0, center.1, outer, if donut { radius * 2.0 / 3.0 } else { 0.0 }, start, end);
        start = end;
        let connector_start = (center.0 + radius * middle.sin(), center.1 - radius * middle.cos());
        let mut connector_end = (center.0 + (radius + 14.0) * middle.sin(), center.1 - (radius + 14.0) * middle.cos());
        let right = middle.sin() >= 0.0;
        let text_width = slice.name.chars().count() as f64 * 7.0;
        let label_x = if right { (connector_end.0 + 6.0).clamp(8.0, (chart_width - 8.0 - text_width).max(8.0)) } else { (connector_end.0 - 6.0).clamp((text_width + 8.0).min(chart_width - 8.0), chart_width - 8.0) };
        connector_end.0 = label_x + if right { -4.0 } else { 4.0 };
        let accessible = format!("{}: {} ({}%)", slice.name, value_label(slice.value), value_label(slice.value / total * 100.0));
        Some(rsx! { g { class: if keyboard_entry == Some(index) { "ix-chart-slice is-keyboard-active" } else { "ix-chart-slice" }, role: "option", id: "ix-chart-pie-{id}-{index}", "aria-selected": keyboard_entry == Some(index), "aria-label": accessible.clone(), onmouseenter: move |_| active.set(Some(index)), onmouseleave: move |_| active.set(None), onclick: move |_| { keyboard_point.set(Some(index)); active.set(Some(index)); focus_chart_host(mounted); },
            title { "{accessible}" }
            path { d: path, fill: chart_color(index) }
            line { x1: connector_start.0, y1: connector_start.1, x2: connector_end.0, y2: connector_end.1, stroke: chart_color(index) }
            text { x: label_x, y: connector_end.1 + 4.0, text_anchor: if right { "start" } else { "end" }, "{slice.name}" }
        } })
    }).collect();
    let focus_points = valid_points.clone();
    rsx! { figure { class: "ix-chart ix-chart-pie", "aria-label": label.clone(),
        div { class: "ix-chart-canvas", role: "listbox", tabindex: if valid_points.is_empty() { "-1" } else { "0" }, "aria-label": label.clone(),
            "aria-activedescendant": if canvas_focused() { keyboard_entry.map(|index| format!("ix-chart-pie-{id}-{index}")) } else { None },
            onmounted: move |event| { mounted.set(Some(event.data())); observe_chart_width(width, event); },
            onfocus: move |_| { canvas_focused.set(true); let target = (*keyboard_point.peek()).filter(|index| focus_points.contains(&(0,*index))).or(keyboard_entry); if let Some(index) = target { keyboard_point.set(Some(index)); active.set(Some(index)); } },
            onblur: move |_| { canvas_focused.set(false); active.set(None); },
            onkeydown: move |event: KeyboardEvent| {
                if event.key() == Key::Escape { event.prevent_default(); active.set(None); return; }
                let key = match event.key() { Key::ArrowUp => Key::ArrowLeft, Key::ArrowDown => Key::ArrowRight, key => key };
                if let Some((_, index)) = chart_keyboard_target(&valid_points, keyboard_entry.map(|index| (0,index)), &key) { event.prevent_default(); keyboard_point.set(Some(index)); active.set(Some(index)); }
            },
            div { class: "ix-chart-tooltip-region", style: "--ix-chart-tooltip-space:72px",
                if let Some(index) = active() { if let Some(slice) = slices.get(index) { div { class: "ix-chart-tooltip", role: "status", strong { "{label}" } div { span { class: "ix-chart-swatch", style: "background:{chart_color(index)}" } span { "{slice.name}" } b { "{value_label(slice.value)} ({value_label(slice.value / total * 100.0)}%)" } } } } }
                else { span { class: "ix-chart-tooltip-placeholder", "aria-hidden": "true", "Select a data point to inspect values." } }
            }
            svg { width: "100%", height: "580", view_box: "0 0 {chart_width} 580", role: "group", "aria-label": label.clone(), {segments.into_iter()} }
        }
        div { class: "ix-chart-legend", "aria-label": "Chart series", for (index, slice) in slices.iter().enumerate() { button { r#type: "button", "aria-pressed": !hidden.read().contains(&index), class: if hidden.read().contains(&index) { "is-hidden" } else { "" }, onclick: move |_| { let mut values = hidden.write(); if let Some(position) = values.iter().position(|value| *value == index) { values.remove(position); } else { values.push(index); } active.set(None); }, span { class: "ix-chart-swatch", style: "background:{chart_color(index)}" } "{slice.name}" } } }
        details { class: "ix-chart-data",
            summary { "View chart data" }
            table { caption { "{label}" }
                thead { tr { th { scope: "col", "Category" } th { scope: "col", "Value" } } }
                tbody { for slice in &slices { tr { th { scope: "row", "{slice.name}" } td { "{value_label(slice.value)}" } } } }
            }
        }
    } }
}

/// The source's speed gauge, 220-degree progress arc, or full progress circle.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GaugeVariant {
    #[default]
    Speed,
    Arc,
    Circle,
}

/// A fixed-value gauge. The value is clamped to 0–100 for the arc geometry;
/// the exact finite value remains available to assistive technology and text.
#[component]
pub fn GaugeChart(value: f64, #[props(default)] variant: GaugeVariant) -> Element {
    let width = use_signal(|| 720.0_f64);
    let chart_width = width();
    let safe = if value.is_finite() { value } else { 0.0 };
    let fraction = safe.clamp(0.0, 100.0) / 100.0;
    let (start, span) = match variant {
        GaugeVariant::Speed => (-std::f64::consts::FRAC_PI_2, std::f64::consts::PI),
        GaugeVariant::Arc => (-110.0_f64.to_radians(), 220.0_f64.to_radians()),
        GaugeVariant::Circle => (0.0, std::f64::consts::TAU),
    };
    let (center, radius) = if variant == GaugeVariant::Speed {
        ((chart_width / 2.0, 360.0), (chart_width * 0.375).min(225.0))
    } else {
        (
            (chart_width / 2.0, 155.0),
            (chart_width / 2.0 - 20.0).min(128.0),
        )
    };
    let track = chart_sector_path(
        center.0,
        center.1,
        radius,
        radius - 15.0,
        start,
        start + span,
    );
    let progress = chart_sector_path(
        center.0,
        center.1,
        radius + 10.0,
        radius - 25.0,
        start,
        start + span * fraction,
    );
    let color = if variant != GaugeVariant::Speed || safe > 60.0 {
        "var(--ix-color-success)"
    } else if safe > 25.0 {
        "var(--ix-color-warning)"
    } else {
        "var(--ix-color-danger)"
    };
    let label = if variant == GaugeVariant::Speed {
        format!("{}Mbps Network Speed", value_label(safe))
    } else {
        format!("{} / 100 completed", value_label(safe))
    };
    rsx! { figure { class: if variant == GaugeVariant::Speed { "ix-chart ix-chart-gauge" } else { "ix-chart ix-chart-progress" }, onmounted: move |event| observe_chart_width(width, event),
        svg { width: "100%", height: if variant == GaugeVariant::Speed { "600" } else { "310" }, view_box: format!("0 0 {chart_width} {}", if variant == GaugeVariant::Speed { 600 } else { 310 }), role: "img", "aria-label": label.clone(),
            title { "{label}" }
            path { d: track, fill: "var(--ix-chart-track)" }
            path { d: progress, fill: color }
            if variant == GaugeVariant::Speed {
                for (a, b, fill) in [(0.0, 0.25, "var(--ix-color-danger)"), (0.25, 0.6, "var(--ix-color-warning)"), (0.6, 1.0, "var(--ix-color-success)")] {
                    path { d: chart_sector_path(center.0, center.1, radius + 15.0, radius + 10.0, start + span * a, start + span * b), fill }
                }
                text { x: center.0 - radius, y: center.1 + 32.0, text_anchor: "middle", "0" }
                text { x: center.0 + radius, y: center.1 + 32.0, text_anchor: "middle", "100" }
                text { x: center.0, y: center.1 - 65.0, text_anchor: "middle", font_size: "24", "{value_label(safe)}Mbps" }
                text { x: center.0, y: center.1 - 30.0, text_anchor: "middle", font_size: "24", "Network Speed" }
            } else {
                text { x: center.0, y: 152, text_anchor: "middle", font_size: "32", "{value_label(safe)} / 100" }
                text { x: center.0, y: 188, text_anchor: "middle", font_size: "24", "completed" }
            }
        }
    } }
}

/// Public Brand chart palette from packages/echarts/src/themes/brand-{light,dark}.ts.
/// The surrounding IxTheme selects a scheme; all chart text follows its tokens.
pub const STYLES: &str = r#"
.ix-theme{--ix-chart-1:#009999;--ix-chart-2:#005159;--ix-chart-3:#00004A;--ix-chart-4:#1A747D;--ix-chart-5:#3664C6;--ix-chart-6:#002949;--ix-chart-7:#7353E5;--ix-chart-8:#553BA3;--ix-chart-9:#740089;--ix-chart-10:#BE5925;--ix-chart-11:#4F153D;--ix-chart-12:#C04774;--ix-chart-13:#00237A;--ix-chart-14:#801100;--ix-chart-15:#805800;--ix-chart-16:#5E5E4A;--ix-chart-17:#4C4C68;--ix-chart-track:var(--si-sys-color-data-sequential-deep-blue-4,#00237a)}
.ix-theme[data-ix-color-schema=dark]{--ix-chart-1:#00C1B6;--ix-chart-2:#85E9D2;--ix-chart-3:#3664C6;--ix-chart-4:#00FFB9;--ix-chart-5:#6895F6;--ix-chart-6:#CCF5F5;--ix-chart-7:#805CFF;--ix-chart-8:#BFB0F3;--ix-chart-9:#B95CC9;--ix-chart-10:#BE5925;--ix-chart-11:#FF98C4;--ix-chart-12:#E5659B;--ix-chart-13:#97C7FF;--ix-chart-14:#FFBC66;--ix-chart-15:#FFF7D6;--ix-chart-16:#AAAA96;--ix-chart-17:#7D8099;--ix-chart-track:var(--si-sys-color-data-sequential-deep-blue-4,#3664c6)}
@media(prefers-color-scheme:dark){.ix-theme[data-ix-color-schema=system]{--ix-chart-1:#00C1B6;--ix-chart-2:#85E9D2;--ix-chart-3:#3664C6;--ix-chart-4:#00FFB9;--ix-chart-5:#6895F6;--ix-chart-6:#CCF5F5;--ix-chart-7:#805CFF;--ix-chart-8:#BFB0F3;--ix-chart-9:#B95CC9;--ix-chart-10:#BE5925;--ix-chart-11:#FF98C4;--ix-chart-12:#E5659B;--ix-chart-13:#97C7FF;--ix-chart-14:#FFBC66;--ix-chart-15:#FFF7D6;--ix-chart-16:#AAAA96;--ix-chart-17:#7D8099;--ix-chart-track:var(--si-sys-color-data-sequential-deep-blue-4,#3664c6)}}
.ix-chart{position:relative;width:100%;min-width:0;margin:0;padding-top:16px;color:var(--ix-color-text);font:12px var(--ix-font);font-variant-numeric:tabular-nums}
.ix-chart-canvas{position:relative;width:100%;min-width:0}.ix-chart svg{display:block;overflow:visible;max-width:100%;font:12px var(--ix-font);fill:var(--ix-color-text)}
.ix-chart-grid{stroke:var(--ix-color-text);stroke-opacity:.1;stroke-width:1}.ix-chart-axis{stroke:var(--ix-color-text);stroke-opacity:.3;stroke-width:1}.ix-chart-pointer{stroke:var(--ix-color-text);stroke-opacity:.45;stroke-dasharray:4 4;pointer-events:none}
.ix-chart-shadow{fill:var(--ix-color-text);fill-opacity:.1;pointer-events:none}
.ix-chart-brush-selection{fill:var(--ix-color-primary);fill-opacity:.18;stroke:var(--ix-color-primary);stroke-width:1;pointer-events:none}.ix-chart-brush-hit{cursor:crosshair}
.ix-chart-mark{cursor:crosshair}.ix-chart-mark:hover,.ix-chart-mark.is-active{filter:brightness(1.15)}.ix-theme .ix-chart-canvas:focus-visible{outline:none}.ix-chart-canvas:focus-visible .is-keyboard-active{outline:2px solid var(--ix-color-focus);outline-offset:4px}.ix-chart-line{pointer-events:none}.ix-chart-extreme{pointer-events:none}
.ix-chart-plot-scroll{max-width:100%;overflow-x:auto;overflow-y:hidden;overscroll-behavior-inline:contain}.ix-chart-plot-scroll>svg{max-width:none}.ix-chart-scroll-hint{margin:0 0 12px;font:12px var(--ix-font);color:var(--ix-color-text-secondary)}.ix-chart-scroll-hint[hidden]{display:none}
.ix-chart-tooltip-region{position:absolute;top:16px;right:12px;z-index:2;max-width:calc(100% - 24px);pointer-events:none}.ix-chart-tooltip-region.is-disabled{display:none}.ix-chart-tooltip-placeholder{display:none}.ix-chart-tooltip{padding:10px 14px;background:var(--ix-color-surface-1);border:1px solid var(--ix-color-border);box-shadow:0 4px 12px #0003;font-size:14px;line-height:1.6}.ix-chart-tooltip>div{display:flex;align-items:center;gap:8px}.ix-chart-tooltip b{margin-left:auto;padding-left:12px}.ix-chart-tooltip strong{font-weight:600}
.ix-chart-swatch{display:inline-block;flex:none;width:14px;height:10px;vertical-align:middle}.ix-chart-legend{display:flex;flex-wrap:wrap;gap:4px 12px;justify-content:center;padding:4px 16px 16px}.ix-chart-legend button{display:inline-flex;align-items:center;gap:6px;min-height:28px;padding:2px;background:transparent;border:0;color:inherit;font:inherit;cursor:pointer}.ix-chart-legend button.is-hidden{opacity:.5}.ix-chart-legend button:hover{color:var(--ix-color-primary)}.ix-chart-legend button.is-hidden .ix-chart-swatch{background:var(--ix-color-border)!important}
.ix-chart-data{font:13px var(--ix-font);margin-top:8px}.ix-chart-data summary{cursor:pointer;color:var(--ix-color-primary);padding:8px 0}.ix-chart-table-scroll{max-width:100%;overflow:auto}.ix-chart-data table{border-collapse:collapse;width:100%;text-align:left}.ix-chart-data th,.ix-chart-data td{border-bottom:1px solid var(--ix-color-border);padding:8px}.ix-chart-data caption{text-align:left;padding:8px 0;font-weight:bold}
.ix-chart-slice{cursor:crosshair}.ix-chart-progress{padding-top:48px}
@media(max-width:480px){.ix-chart-tooltip-region{position:relative;top:auto;right:auto;width:100%;max-width:none;min-height:var(--ix-chart-tooltip-space,72px);margin-bottom:12px;box-sizing:border-box}.ix-chart-tooltip{font-size:12px;box-sizing:border-box;width:100%;box-shadow:none}.ix-chart-tooltip-placeholder{display:block;padding:12px 0;color:var(--ix-color-text-secondary);font:12px var(--ix-font)}.ix-chart-legend{padding-inline:0;gap:2px 8px}.ix-chart-pie text{font-size:11px}}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    struct TestChartMount;
    struct TestChartConverter;
    macro_rules! test_converter {
        (enum Event { $(#[convert = $convert:ident] #[events = [$($events:tt)*]] $(#[raw = [$($raw:ident),* $(,)?]])? $group:ident($data:ident),)* }) => {
            impl dioxus::html::HtmlEventConverter for TestChartConverter { $(fn $convert(&self, _: &dioxus::html::PlatformEventData) -> dioxus::html::$data { test_converter!(@data $group) })* }
        };
        (@data Mounted) => { MountedData::new(TestChartMount) };
        (@data $other:ident) => { panic!("Unexpected non-mounted test event") };
    }
    dioxus::html::with_html_event_groups!(test_converter);
    impl dioxus::html::RenderedElementBacking for TestChartMount {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
        fn get_client_rect(
            &self,
        ) -> std::pin::Pin<
            Box<
                dyn std::future::Future<
                        Output = dioxus::html::MountedResult<dioxus::html::geometry::PixelsRect>,
                    >,
            >,
        > {
            Box::pin(async {
                Ok(dioxus::html::geometry::PixelsRect::new(
                    dioxus::html::geometry::euclid::Point2D::new(0.0, 0.0),
                    dioxus::html::geometry::PixelsSize::new(309.0, 600.0),
                ))
            })
        }
    }

    #[test]
    fn cartesian_mobile_mount_measures_and_rerenders() {
        let _event_converter_guard = crate::test_event_converter_guard();
        fn app() -> Element {
            rsx! { CartesianChart { categories: vec!["Product A".into(),"Product B".into(),"Product C".into(),"Product D".into(),"Product E".into(),"Product F".into()], series: vec![ChartSeries::new("",vec![10.3,9.2,7.3,6.4,6.2,4.4],ChartKind::Bar)], x_title: "Product", y_title: "Number of sold products\n (in Mio)" } }
        }
        let mut dom = VirtualDom::new(app);
        dioxus::html::set_event_converter(Box::new(TestChartConverter));
        let mutations = dom.rebuild_to_vec();
        let mut current_id = dioxus::dioxus_core::ElementId::ROOT;
        for edit in mutations.edits {
            match edit {
                dioxus::dioxus_core::Mutation::SetId { id }
                | dioxus::dioxus_core::Mutation::PushId { id } => current_id = id,
                dioxus::dioxus_core::Mutation::NewEventListener { name } if name == "mounted" => {
                    dom.runtime().handle_event(
                        "mounted",
                        Event::new(
                            Rc::new(dioxus::html::PlatformEventData::new(Box::new(()))),
                            false,
                        ),
                        current_id,
                    )
                }
                _ => {}
            }
        }
        dom.render_immediate_to_vec();
        let html = dioxus_ssr::render(&dom);
        assert!(
            html.contains("0 0 309 600"),
            "mobile measured viewBox missing: {html}"
        );
        assert_eq!(
            html.matches("tabindex=\"0\"").count(),
            1,
            "one keyboard focus host per chart"
        );
        assert_eq!(html.matches("role=\"option\"").count(), 6);
    }

    #[test]
    fn keyboard_navigation_skips_missing_points_and_stays_in_the_focused_series() {
        let points = [(0, 0), (0, 2), (1, 0), (1, 2), (2, 1)];
        assert_eq!(
            chart_keyboard_target(&points, Some((0, 0)), &Key::ArrowRight),
            Some((0, 2))
        );
        assert_eq!(
            chart_keyboard_target(&points, Some((0, 2)), &Key::ArrowDown),
            Some((1, 2))
        );
        assert_eq!(
            chart_keyboard_target(&points, Some((1, 2)), &Key::ArrowDown),
            Some((1, 2))
        );
        assert_eq!(
            chart_keyboard_target(&points, Some((1, 2)), &Key::ArrowUp),
            Some((0, 2))
        );
        assert_eq!(
            chart_keyboard_target(&points, Some((0, 2)), &Key::Home),
            Some((0, 0))
        );
        assert_eq!(
            chart_keyboard_target(&points, Some((0, 0)), &Key::End),
            Some((0, 2))
        );
        assert_eq!(chart_keyboard_target(&[], None, &Key::ArrowRight), None);
    }

    #[test]
    fn stacks_share_a_scale_and_negative_values_do_not_cancel_positive_stacks() {
        let values = vec![
            ChartSeries {
                stack: "a".into(),
                ..ChartSeries::new("A", vec![8.0, -3.0], ChartKind::Bar)
            },
            ChartSeries {
                stack: "a".into(),
                ..ChartSeries::new("B", vec![9.0, -7.0], ChartKind::Bar)
            },
            ChartSeries::new("C", vec![15.0, 1.0], ChartKind::Bar),
        ];
        let scale = axis_scales(&values, &[true, true, true], &[], 2)[0];
        assert!(scale.maximum >= 17.0 && scale.minimum <= -10.0);
        let hidden = axis_scales(&values, &[true, false, false], &[], 2)[0];
        assert!(hidden.maximum < scale.maximum);
        assert_eq!(ChartScale::new(0.0, 0.0).maximum, 1.0);
        assert!(
            ChartScale::new(f64::NAN, f64::INFINITY)
                .fraction(0.0)
                .is_finite()
        );
    }

    #[test]
    fn missing_points_break_line_paths_and_steps_end_at_the_next_category() {
        let path = chart_line_path(
            &[(0.0, 4.0), (10.0, 6.0), (f64::NAN, 2.0), (30.0, 1.0)],
            false,
            true,
        );
        assert_eq!(path, "M 0.000 4.000H 10.000 V 6.000M 30.000 1.000");
        assert!(chart_line_path(&[(0.0, 4.0), (10.0, 6.0)], true, false).contains('C'));
        assert!(!path.contains("NaN"));
    }

    #[test]
    fn pie_paths_cover_large_and_full_sectors_without_nan_or_zero_endpoint_arcs() {
        let path = chart_sector_path(100.0, 100.0, 80.0, 50.0, 0.0, std::f64::consts::TAU);
        assert!(path.contains("0 1 1"));
        assert!(path.contains("0 1 0"));
        assert_eq!(chart_sector_path(0.0, 0.0, 1.0, 0.0, 0.0, 0.0), "");
        assert_eq!(
            chart_sector_path(0.0, 0.0, f64::INFINITY, 0.0, 0.0, 1.0),
            ""
        );
    }
}
