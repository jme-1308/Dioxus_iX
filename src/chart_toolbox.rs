//! Rust chart toolbox state, editable data view, zoom history and PNG export.
//! The DOM bridge performs native pointer capture and rasterization only; all
//! data parsing, zoom selection, type changes and restore behavior live here.
use crate::{
    CartesianChart, ChartBrushSelection, ChartKind, ChartPng, ChartSeries, export_chart_png,
};
use dioxus::prelude::*;
use std::{
    fmt,
    ops::Range,
    rc::Rc,
    sync::atomic::{AtomicUsize, Ordering},
};

static NEXT_TOOLBOX_ID: AtomicUsize = AtomicUsize::new(1);

/// Original category labels and series metadata used by a chart toolbox.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ChartToolboxData {
    pub categories: Vec<String>,
    pub series: Vec<ChartSeries>,
}

/// A data-view validation error. A failed refresh preserves the current chart.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChartDataViewError(pub String);
impl fmt::Display for ChartDataViewError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
impl std::error::Error for ChartDataViewError {}

/// Pure Rust business state shared by the reusable chart toolbox and its tests.
/// Zoom ranges use original category indices; a brush is relative to the current
/// visible window. Restore reinstates the initial data, types and full range.
#[derive(Clone, Debug, PartialEq)]
pub struct ChartToolboxState {
    original: ChartToolboxData,
    data: ChartToolboxData,
    window: Range<usize>,
    history: Vec<Range<usize>>,
}
impl ChartToolboxState {
    pub fn new(data: ChartToolboxData) -> Self {
        let window = 0..data.categories.len();
        Self {
            original: data.clone(),
            data,
            window,
            history: Vec::new(),
        }
    }
    pub fn data(&self) -> &ChartToolboxData {
        &self.data
    }
    pub fn window(&self) -> Range<usize> {
        self.window.clone()
    }
    pub fn can_zoom_back(&self) -> bool {
        !self.history.is_empty()
    }
    pub fn visible_data(&self) -> ChartToolboxData {
        let range = self.window.start.min(self.data.categories.len())
            ..self.window.end.min(self.data.categories.len());
        ChartToolboxData {
            categories: self.data.categories[range.clone()].to_vec(),
            series: self
                .data
                .series
                .iter()
                .map(|series| ChartSeries {
                    values: range
                        .clone()
                        .map(|index| series.values.get(index).copied().unwrap_or(f64::NAN))
                        .collect(),
                    ..series.clone()
                })
                .collect(),
        }
    }
    pub fn zoom(&mut self, selection: ChartBrushSelection) -> bool {
        let count = self.window.len();
        if count == 0 {
            return false;
        }
        let start = selection.start.min(count - 1);
        let end = selection.end.min(count).max(start + 1);
        let next = self.window.start + start..self.window.start + end;
        if next == self.window {
            return false;
        }
        self.history.push(self.window.clone());
        self.window = next;
        true
    }
    pub fn zoom_back(&mut self) -> bool {
        let Some(previous) = self.history.pop() else {
            return false;
        };
        self.window = previous;
        true
    }
    pub fn set_kind(&mut self, kind: ChartKind) {
        for series in &mut self.data.series {
            series.kind = kind;
        }
    }
    pub fn restore(&mut self) {
        self.data = self.original.clone();
        self.window = 0..self.data.categories.len();
        self.history.clear();
    }
    /// Category-axis data view in the upstream tab-separated header/row format.
    /// The first header cell is intentionally blank, followed by series names.
    pub fn data_view(&self) -> String {
        let mut lines = vec![format!(
            " \t{}",
            self.data
                .series
                .iter()
                .map(|series| series.name.as_str())
                .collect::<Vec<_>>()
                .join("\t")
        )];
        for (index, category) in self.data.categories.iter().enumerate() {
            let values = self
                .data
                .series
                .iter()
                .map(|series| {
                    series
                        .values
                        .get(index)
                        .filter(|value| value.is_finite())
                        .map(|value| value.to_string())
                        .unwrap_or_default()
                })
                .collect::<Vec<_>>()
                .join("\t");
            lines.push(format!("{category}\t{values}"));
        }
        lines.join("\n")
    }
    /// Apply an edited table atomically. Empty trailing lines are ignored.
    /// Existing series retain their style, axis, stack, and interpolation flags.
    pub fn refresh_data_view(&mut self, text: &str) -> Result<(), ChartDataViewError> {
        let mut lines = text.lines();
        let header = lines.next().ok_or_else(|| {
            ChartDataViewError(
                "Keep a header row followed by tab-separated categories and values.".into(),
            )
        })?;
        let headers: Vec<&str> = header.split('\t').collect();
        if headers.len() < 2 {
            return Err(ChartDataViewError(
                "Separate the header cells with tabs; the first cell is the category column."
                    .into(),
            ));
        }
        let mut next = ChartToolboxData {
            categories: Vec::new(),
            series: headers[1..]
                .iter()
                .enumerate()
                .map(|(index, name)| ChartSeries {
                    name: name.trim().into(),
                    values: Vec::new(),
                    ..self.data.series.get(index).cloned().unwrap_or_else(|| {
                        ChartSeries::new(
                            "",
                            Vec::new(),
                            self.data
                                .series
                                .first()
                                .map(|series| series.kind)
                                .unwrap_or_default(),
                        )
                    })
                })
                .collect(),
        };
        for (row, line) in lines.enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            let cells: Vec<&str> = line.trim_end_matches('\r').split('\t').collect();
            if cells.len() != next.series.len() + 1 {
                return Err(ChartDataViewError(format!(
                    "Row {} needs one category and {} value column(s), separated by tabs.",
                    row + 2,
                    next.series.len()
                )));
            }
            let category = cells[0].trim();
            if category.is_empty() {
                return Err(ChartDataViewError(format!(
                    "Enter a category label on row {}.",
                    row + 2
                )));
            }
            next.categories.push(category.into());
            for (index, value) in cells[1..].iter().enumerate() {
                let value = value
                    .trim()
                    .parse::<f64>()
                    .ok()
                    .filter(|value| value.is_finite())
                    .ok_or_else(|| {
                        ChartDataViewError(format!(
                            "Enter a finite number in row {}, value column {}.",
                            row + 2,
                            index + 1
                        ))
                    })?;
                next.series[index].values.push(value);
            }
        }
        let count = next.categories.len();
        fn clamp_window(range: Range<usize>, count: usize) -> Range<usize> {
            if count == 0 {
                0..0
            } else {
                let start = range.start.min(count - 1);
                start..range.end.min(count).max(start + 1)
            }
        }
        self.window = clamp_window(self.window.clone(), count);
        self.history = self
            .history
            .iter()
            .cloned()
            .map(|range| clamp_window(range, count))
            .filter(|range| !range.is_empty())
            .collect();
        self.data = next;
        Ok(())
    }
}

fn focus_node(node: Signal<Option<Rc<MountedData>>>) {
    let Some(node) = node.try_peek().ok().and_then(|node| node.clone()) else {
        return;
    };
    spawn(async move {
        let _ = node.set_focus(true).await;
    });
}

fn invalidate_chart_image(
    mut generation: Signal<u64>,
    mut image: Signal<Option<ChartPng>>,
    mut status: Signal<String>,
    mut busy: Signal<bool>,
) {
    let next = (*generation.peek()).wrapping_add(1);
    generation.set(next);
    image.set(None);
    status.set(String::new());
    busy.set(false);
}

fn editor_focus_index(current: usize, reverse: bool) -> usize {
    (current % 3 + if reverse { 2 } else { 1 }) % 3
}

/// Category chart with a horizontal zoom brush/back stack, restore, line/bar
/// switch, editable data view and an actual PNG export of the rendered plot.
/// `onexport` receives a ready PNG; requesting a browser download does not prove
/// that the host saved a file. A native Download PNG link remains available.
/// Replacing the input dataset resets zoom and discards an open data-view draft
/// without emitting `onchange`. Restore then uses that replacement as its baseline.
#[component]
pub fn ChartToolbox(
    categories: Vec<String>,
    series: Vec<ChartSeries>,
    #[props(default = "Chart".into())] label: String,
    #[props(default = "chart.png".into())] file_name: String,
    #[props(default)] onchange: EventHandler<ChartToolboxState>,
    #[props(default)] onexport: EventHandler<ChartPng>,
) -> Element {
    let id = use_hook(|| NEXT_TOOLBOX_ID.fetch_add(1, Ordering::Relaxed));
    let input = ChartToolboxData { categories, series };
    let mut state = use_signal(|| ChartToolboxState::new(input.clone()));
    let mut brush = use_signal(|| false);
    let mut editor = use_signal(|| false);
    let mut draft = use_signal(String::new);
    let mut error = use_signal(String::new);
    let mut export_status = use_signal(String::new);
    let mut exporting = use_signal(|| false);
    let mut last_png = use_signal(|| None::<ChartPng>);
    let export_generation = use_signal(|| 0_u64);
    let mut editor_trigger = use_signal(|| None::<Rc<MountedData>>);
    let mut editor_textarea = use_signal(|| None::<Rc<MountedData>>);
    let mut editor_close = use_signal(|| None::<Rc<MountedData>>);
    let mut editor_refresh = use_signal(|| None::<Rc<MountedData>>);
    let mut editor_focus = use_signal(|| 0_usize);
    let mut restore_editor_focus = use_signal(|| false);
    use_effect(use_reactive!(|input| {
        if state.peek().original != input {
            state.set(ChartToolboxState::new(input));
            // A draft belongs to the input dataset that opened it. Replacing
            // that input must not let Refresh overwrite the new data with it.
            // Leave focus with the host control that requested the update.
            editor.set(false);
            draft.set(String::new());
            error.set(String::new());
            brush.set(false);
            restore_editor_focus.set(false);
            invalidate_chart_image(export_generation, last_png, export_status, exporting);
        }
    }));
    use_effect(move || {
        if !editor() && restore_editor_focus() {
            restore_editor_focus.set(false);
            focus_node(editor_trigger);
        }
    });
    let visible = state.read().visible_data();
    let line = state
        .read()
        .data
        .series
        .iter()
        .all(|series| series.kind == ChartKind::Line);
    let bar = !state.read().data.series.is_empty()
        && state
            .read()
            .data
            .series
            .iter()
            .all(|series| series.kind == ChartKind::Bar);
    let export_id = format!("ix-chart-toolbox-plot-{id}");
    let image_id = export_id.clone();
    let download_file_name = file_name.clone();
    let back_disabled = !state.read().can_zoom_back();
    rsx! {
        div { class: "ix-chart-toolbox",
            div { class: "ix-chart-toolbox-content", inert: editor().then_some(""),
            div { class: "ix-chart-toolbox-controls", role: "toolbar", "aria-label": "Chart toolbox", "data-chart-export-exclude": "true",
                button { r#type: "button", title: "Zoom", "aria-label": "Zoom", "aria-pressed": brush(), onclick: move |_| { brush.toggle(); }, ToolboxIcon { kind: ToolboxGlyph::Zoom } }
                button { r#type: "button", title: "Zoom back", "aria-label": "Zoom back", disabled: back_disabled, onclick: move |_| { if state.write().zoom_back() { invalidate_chart_image(export_generation,last_png,export_status,exporting); onchange.call(state.read().clone()); } }, ToolboxIcon { kind: ToolboxGlyph::Back } }
                button { r#type: "button", title: "Restore", "aria-label": "Restore", onclick: move |_| { state.write().restore(); brush.set(false); error.set(String::new()); invalidate_chart_image(export_generation,last_png,export_status,exporting); onchange.call(state.read().clone()); }, ToolboxIcon { kind: ToolboxGlyph::Restore } }
                button { r#type: "button", title: "Save as image", "aria-label": "Save as image", disabled: exporting(), "aria-busy": exporting(), onclick: move |_| {
                    invalidate_chart_image(export_generation,last_png,export_status,exporting);
                    let generation = *export_generation.peek();
                    exporting.set(true); export_status.set("Creating PNG…".into()); let image_id = image_id.clone(); let file_name = file_name.clone();
                    spawn(async move {
                        let result = export_chart_png(&image_id, 1.0).await;
                        if export_generation.try_peek().map(|value| *value != generation).unwrap_or(true) { return; }
                        match result {
                            Ok(png) => {
                                if let Ok(mut image) = last_png.try_write() { *image = Some(png.clone()); }
                                if let Ok(mut status) = export_status.try_write() { *status = format!("PNG ready ({} × {}).",png.width,png.height); }
                                let download = png.download(&file_name).await;
                                if export_generation.try_peek().map(|value| *value != generation).unwrap_or(true) { return; }
                                match download { Ok(()) => { if let Ok(mut status) = export_status.try_write() { *status = format!("PNG ready ({} × {}). Download requested.", png.width, png.height); } }, Err(error) => { if let Ok(mut status) = export_status.try_write() { *status = format!("PNG ready. Use Download PNG to save it. Automatic request failed: {error}"); } } }
                                onexport.call(png);
                            }
                            Err(error) => { if let Ok(mut status) = export_status.try_write() { *status = format!("Could not create PNG: {error}"); } }
                        }
                        if let Ok(mut busy) = exporting.try_write() { *busy = false; }
                    });
                }, ToolboxIcon { kind: ToolboxGlyph::Save } }
                button { r#type: "button", title: "Switch to line chart", "aria-label": "Switch to line chart", "aria-pressed": line, onclick: move |_| { state.write().set_kind(ChartKind::Line); invalidate_chart_image(export_generation,last_png,export_status,exporting); onchange.call(state.read().clone()); }, ToolboxIcon { kind: ToolboxGlyph::Line } }
                button { r#type: "button", title: "Switch to bar chart", "aria-label": "Switch to bar chart", "aria-pressed": bar, onclick: move |_| { state.write().set_kind(ChartKind::Bar); invalidate_chart_image(export_generation,last_png,export_status,exporting); onchange.call(state.read().clone()); }, ToolboxIcon { kind: ToolboxGlyph::Bar } }
                button { r#type: "button", title: "Data view", "aria-label": "Data view", onmounted: move |event| editor_trigger.set(Some(event.data())), onclick: move |_| { draft.set(state.read().data_view()); error.set(String::new()); editor.set(true); brush.set(false); }, ToolboxIcon { kind: ToolboxGlyph::Data } }
            }
            p { class: "ix-chart-toolbox-brush-hint", hidden: !brush(), "Drag horizontally to zoom, or use Enter and arrow keys. Escape cancels." }
            div { id: export_id, class: "ix-chart-toolbox-plot",
                CartesianChart { categories: visible.categories, series: visible.series, label, legend: false, brush_enabled: brush(),
                    onbrush: move |selection| { if state.write().zoom(selection) { invalidate_chart_image(export_generation,last_png,export_status,exporting); onchange.call(state.read().clone()); } },
                    onbrushcancel: move |_| brush.set(false),
                }
            }
            p { class: "ix-chart-toolbox-export-status", role: "status", "aria-live": "polite", hidden: export_status.read().is_empty(), "{export_status}" }
            if let Some(png) = last_png() { a { class: "ix-chart-toolbox-download", download: download_file_name, href: png.data_url, "Download PNG" } }
            }
            if editor() {
                div { class: "ix-chart-data-editor", role: "dialog", "aria-label": "Data view", "data-chart-export-exclude": "true",
                    onmounted: move |event| { spawn(async move { let _ = event.scroll_to_with_options(dioxus::html::ScrollToOptions { behavior: dioxus::html::ScrollBehavior::Instant, vertical: dioxus::html::ScrollLogicalPosition::Start, horizontal: dioxus::html::ScrollLogicalPosition::Nearest }).await; }); },
                    onkeydown: move |event: KeyboardEvent| {
                        if event.key() == Key::Escape { event.prevent_default(); editor.set(false); restore_editor_focus.set(true); }
                        else if event.key() == Key::Tab { event.prevent_default(); let next = editor_focus_index(editor_focus(),event.modifiers().shift()); editor_focus.set(next); focus_node(match next { 0=>editor_textarea,1=>editor_close,_=>editor_refresh }); }
                    },
                    h4 { "Data view" }
                    p { id: "ix-chart-data-help-{id}", "Edit category labels and values separated by tabs." }
                    textarea { "aria-label": "Chart data", "aria-describedby": "ix-chart-data-help-{id}", spellcheck: false, value: draft(),
                        onmounted: move |event| { editor_textarea.set(Some(event.data())); spawn(async move { let _ = event.set_focus(true).await; }); },
                        onfocus: move |_| editor_focus.set(0),
                        oninput: move |event| { draft.set(event.value()); error.set(String::new()); },
                    }
                    p { class: "ix-chart-data-error", role: "alert", hidden: error.read().is_empty(), "{error}" }
                    div { class: "ix-chart-data-editor-actions",
                        button { r#type: "button", onmounted: move |event| editor_close.set(Some(event.data())), onfocus: move |_| editor_focus.set(1), onclick: move |_| { editor.set(false); error.set(String::new()); restore_editor_focus.set(true); }, "Close" }
                        button { r#type: "button", onmounted: move |event| editor_refresh.set(Some(event.data())), onfocus: move |_| editor_focus.set(2), onclick: move |_| { let result = state.write().refresh_data_view(&draft.read()); match result { Ok(()) => { editor.set(false); error.set(String::new()); invalidate_chart_image(export_generation,last_png,export_status,exporting); onchange.call(state.read().clone()); restore_editor_focus.set(true); }, Err(problem) => error.set(problem.to_string()) } }, "Refresh" }
                    }
                }
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum ToolboxGlyph {
    Zoom,
    Back,
    Restore,
    Save,
    Line,
    Bar,
    Data,
}
#[component]
fn ToolboxIcon(kind: ToolboxGlyph) -> Element {
    let path = match kind {
        ToolboxGlyph::Zoom => "M3 7h8M7 3v8M13 7h7v13H7v-7",
        ToolboxGlyph::Back => "M9 3 4 8l5 5M5 8h15v12H5v-5",
        ToolboxGlyph::Restore => {
            "M20 8a8 8 0 0 0-14-3L3 8m0-5v5h5M4 16a8 8 0 0 0 14 3l3-3m0 5v-5h-5"
        }
        ToolboxGlyph::Save => "M12 2v12m-4-4 4 4 4-4M4 14v7h16v-7",
        ToolboxGlyph::Line => "M3 20V4m0 16h18M5 16l5-7 5 3 5-7",
        ToolboxGlyph::Bar => "M3 20h18M5 18v-7h3v7m3 0V5h3v13m3 0V8h3v10",
        ToolboxGlyph::Data => "M5 2h10l4 4v16H5Zm10 0v5h4M8 11h8M8 15h8M8 19h8",
    };
    rsx! { svg { view_box: "0 0 24 24", width: "20", height: "20", fill: "none", stroke: "currentColor", stroke_width: "1.4", "aria-hidden": "true", path { d: path } } }
}

pub(crate) const STYLES: &str = include_str!("styles/chart_toolbox.css");

#[cfg(test)]
#[path = "chart_toolbox_tests.rs"]
mod lifecycle_tests;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn editor_tab_focus_wraps_only_its_three_controls() {
        assert_eq!(editor_focus_index(0, false), 1);
        assert_eq!(editor_focus_index(1, false), 2);
        assert_eq!(editor_focus_index(2, false), 0);
        assert_eq!(editor_focus_index(0, true), 2);
        assert_eq!(editor_focus_index(2, true), 1);
        assert_eq!(editor_focus_index(1, true), 0);
    }

    #[test]
    fn chart_edits_clear_ready_images_and_invalidate_pending_exports() {
        type ExportSignals = (
            Signal<u64>,
            Signal<Option<ChartPng>>,
            Signal<String>,
            Signal<bool>,
        );
        type Capture = Rc<std::cell::RefCell<Option<ExportSignals>>>;
        fn app(capture: Capture) -> Element {
            let generation = use_signal(|| 5);
            let image = use_signal(|| {
                Some(ChartPng {
                    data_url: "data:image/png;base64,old".into(),
                    width: 1,
                    height: 1,
                })
            });
            let status = use_signal(|| "PNG ready".to_string());
            let busy = use_signal(|| true);
            *capture.borrow_mut() = Some((generation, image, status, busy));
            rsx! { div {} }
        }
        let capture = Rc::new(std::cell::RefCell::new(None));
        let mut dom = VirtualDom::new_with_props(app, capture.clone());
        dom.rebuild_in_place();
        let (generation, image, status, busy) = capture.borrow().unwrap();
        dom.in_scope(ScopeId::ROOT, || {
            let pending_generation = *generation.peek();
            invalidate_chart_image(generation, image, status, busy);
            assert_ne!(*generation.peek(), pending_generation);
            assert!(image.read().is_none());
            assert!(status.read().is_empty());
            assert!(!busy());
        });
    }
    fn data() -> ChartToolboxData {
        ChartToolboxData {
            categories: ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]
                .into_iter()
                .map(str::to_string)
                .collect(),
            series: vec![ChartSeries {
                step_end: true,
                ..ChartSeries::new(
                    "",
                    vec![150.0, 230.0, 224.0, 218.0, 135.0, 147.0, 260.0],
                    ChartKind::Line,
                )
            }],
        }
    }

    #[test]
    fn nested_brush_windows_are_relative_and_back_reverses_each_step() {
        let mut state = ChartToolboxState::new(data());
        assert!(state.zoom(ChartBrushSelection { start: 1, end: 6 }));
        assert_eq!(state.window(), 1..6);
        assert!(state.zoom(ChartBrushSelection { start: 1, end: 3 }));
        assert_eq!(state.window(), 2..4);
        assert_eq!(state.visible_data().series[0].values, vec![224.0, 218.0]);
        assert!(state.zoom_back());
        assert_eq!(state.window(), 1..6);
        assert!(state.zoom_back());
        assert_eq!(state.window(), 0..7);
        assert!(!state.zoom_back());
        assert!(!state.zoom(ChartBrushSelection { start: 0, end: 7 }));
        assert_eq!(
            ChartBrushSelection::from_fractions(0.8, 0.2, 7),
            Some(ChartBrushSelection { start: 1, end: 6 })
        );
        assert_eq!(ChartBrushSelection::from_fractions(f64::NAN, 1.0, 7), None);
    }

    #[test]
    fn edits_are_atomic_and_restore_recovers_original_types_data_and_zoom() {
        let original = data();
        let mut state = ChartToolboxState::new(original.clone());
        let table = state.data_view();
        assert!(table.starts_with(" \t\nMon\t150\nTue\t230"));
        state
            .refresh_data_view(&table.replace("Mon\t150", "Monday\t199.5"))
            .unwrap();
        assert_eq!(state.data().categories[0], "Monday");
        assert_eq!(state.data().series[0].values[0], 199.5);
        let before = state.clone();
        assert!(
            state
                .refresh_data_view(&table.replace("Mon\t150", "Mon\tnot-a-number"))
                .is_err()
        );
        assert_eq!(state, before);
        state.set_kind(ChartKind::Bar);
        assert_eq!(state.data().series[0].kind, ChartKind::Bar);
        assert!(state.data().series[0].step_end);
        state.zoom(ChartBrushSelection { start: 2, end: 5 });
        state.restore();
        assert_eq!(state.data(), &original);
        assert_eq!(state.window(), 0..7);
        assert!(!state.can_zoom_back());
    }

    #[test]
    fn data_refresh_preserves_zoom_and_supports_new_rows_and_series() {
        let mut state = ChartToolboxState::new(data());
        state.zoom(ChartBrushSelection { start: 1, end: 4 });
        let mut text = state.data_view();
        text.push_str("\nHoliday\t42\n");
        state.refresh_data_view(&text).unwrap();
        assert_eq!(state.window(), 1..4);
        assert_eq!(state.data().categories.len(), 8);
        state
            .refresh_data_view("Category\tA\tB\nFirst\t-1.5\t2e2\nSecond\t8\t9")
            .unwrap();
        assert_eq!(state.data().series.len(), 2);
        assert_eq!(state.data().series[1].name, "B");
        assert_eq!(state.data().series[1].values, vec![200.0, 9.0]);
        assert_eq!(state.window(), 1..2);
    }

    #[test]
    fn toolbox_has_real_plot_and_all_seven_tools_without_eager_export() {
        fn app() -> Element {
            let data = data();
            rsx! { ChartToolbox { categories:data.categories,series:data.series } }
        }
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        for label in [
            "Zoom",
            "Zoom back",
            "Restore",
            "Save as image",
            "Switch to line chart",
            "Switch to bar chart",
            "Data view",
        ] {
            assert!(html.contains(&format!("aria-label=\"{label}\"")), "{label}");
        }
        assert!(html.contains("ix-chart-toolbox-plot-"));
        assert!(html.contains("ix-chart-svg-"));
        assert!(!html.contains("data:image/png"));
        assert!(
            !html.contains(" inert"),
            "The closed chart must remain interactive"
        );
    }
}
