//! Controlled energy chart workflows combining toolbox zoom, type switching, data editing, PNG export, and series comparison.
use dioxus::prelude::*;
use dioxus_ix::{
    Button, ButtonVariant, CartesianChart, ChartKind, ChartPng, ChartSeries, ChartToolbox,
    ChartToolboxData, ChartToolboxState,
};

const WEEKDAYS: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
const WEEK1_OUTPUT: [f64; 7] = [42.0, 48.0, 55.0, 51.0, 63.0, 58.0, 46.0];
const WEEK2_OUTPUT: [f64; 7] = [38.0, 44.0, 49.0, 60.0, 67.0, 61.0, 54.0];
const TARGET_OUTPUT: [f64; 7] = [50.0; 7];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum EnergyDataset {
    #[default]
    Week1,
    Week2,
    Empty,
}

impl EnergyDataset {
    fn week_token(self) -> &'static str {
        match self {
            Self::Week1 => "1",
            Self::Week2 => "2",
            Self::Empty => "empty",
        }
    }

    fn summary(self) -> &'static str {
        match self {
            Self::Week1 => "Dataset: Week 1 (Mon–Sun output vs. 50 kWh target).",
            Self::Week2 => "Dataset: Week 2 (alternate Mon–Sun output vs. 50 kWh target).",
            Self::Empty => {
                "Dataset: Empty (no categories or values loaded; use Load next week or Restore example to recover)."
            }
        }
    }
}

fn next_week_dataset(current: EnergyDataset) -> EnergyDataset {
    match current {
        EnergyDataset::Week1 => EnergyDataset::Week2,
        EnergyDataset::Week2 | EnergyDataset::Empty => EnergyDataset::Week1,
    }
}

fn dataset_data(dataset: EnergyDataset) -> ChartToolboxData {
    match dataset {
        EnergyDataset::Week1 => ChartToolboxData {
            categories: WEEKDAYS.into_iter().map(str::to_string).collect(),
            series: vec![
                ChartSeries::new("Output", WEEK1_OUTPUT.to_vec(), ChartKind::Line),
                ChartSeries::new("Target", TARGET_OUTPUT.to_vec(), ChartKind::Line),
            ],
        },
        EnergyDataset::Week2 => ChartToolboxData {
            categories: WEEKDAYS.into_iter().map(str::to_string).collect(),
            series: vec![
                ChartSeries::new("Output", WEEK2_OUTPUT.to_vec(), ChartKind::Line),
                ChartSeries::new("Target", TARGET_OUTPUT.to_vec(), ChartKind::Line),
            ],
        },
        EnergyDataset::Empty => ChartToolboxData {
            categories: Vec::new(),
            series: vec![
                ChartSeries::new("Output", Vec::new(), ChartKind::Line),
                ChartSeries::new("Target", Vec::new(), ChartKind::Line),
            ],
        },
    }
}

fn kind_token(data: &ChartToolboxData) -> &'static str {
    match data
        .series
        .first()
        .map(|series| series.kind)
        .unwrap_or_default()
    {
        ChartKind::Line => "line",
        ChartKind::Bar => "bar",
    }
}

fn first_visible_value(visible: &ChartToolboxData) -> String {
    visible
        .series
        .first()
        .and_then(|series| series.values.first())
        .copied()
        .filter(|value| value.is_finite())
        .map(|value| value.to_string())
        .unwrap_or_else(|| "none".to_string())
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct ExportSummary {
    width: u32,
    height: u32,
    bytes: usize,
}

fn validate_export(png: &ChartPng) -> Option<ExportSummary> {
    let decoded = png.bytes().ok()?;
    Some(ExportSummary {
        width: png.width,
        height: png.height,
        bytes: decoded.len(),
    })
}

/// Interactive energy history toolbox and comparison chart workflow.
#[component]
pub fn ChartWorkflows() -> Element {
    let mut dataset = use_signal(EnergyDataset::default);
    let mut state = use_signal(|| ChartToolboxState::new(dataset_data(EnergyDataset::Week1)));
    let mut changes = use_signal(|| 0usize);
    let mut exported = use_signal(|| None::<ExportSummary>);
    let mut epoch = use_signal(|| 0u64);

    let current_dataset = dataset();
    let base_data = dataset_data(current_dataset);
    let current_state = state();
    let window = current_state.window();
    let visible = current_state.visible_data();
    let visible_categories = visible.categories.len();
    let first_value = first_visible_value(&visible);
    let current_kind = kind_token(current_state.data());
    let change_count = changes();
    let export_summary = exported();
    let export_width = export_summary.map_or(0, |item| item.width);
    let export_height = export_summary.map_or(0, |item| item.height);
    let export_bytes = export_summary.map_or(0, |item| item.bytes);

    rsx! {
        section { class: "demo-section chart-workflows-example", id: "energy-charts",
            h2 { "Energy history" }
            p { class: "chart-workflows-notice",
                "Illustrative data: weekly energy output and a 50 kWh target. Use the toolbox to zoom, switch chart type, edit values, restore the loaded week, or export a PNG."
            }
            div { class: "chart-workflows-toolbar",
                Button {
                    id: "energy-next",
                    variant: ButtonVariant::Primary,
                    onclick: move |_| {
                        let next_dataset = next_week_dataset(*dataset.peek());
                        dataset.set(next_dataset);
                        exported.set(None);
                        state.set(ChartToolboxState::new(dataset_data(next_dataset)));
                    },
                    "Load next week"
                }
                Button {
                    id: "energy-empty",
                    variant: ButtonVariant::Secondary,
                    onclick: move |_| {
                        let already_empty = *dataset.peek() == EnergyDataset::Empty;
                        dataset.set(EnergyDataset::Empty);
                        exported.set(None);
                        state.set(ChartToolboxState::new(dataset_data(EnergyDataset::Empty)));
                        if already_empty {
                            let next_epoch = epoch.peek().wrapping_add(1);
                            epoch.set(next_epoch);
                        }
                    },
                    "Show empty data"
                }
                Button {
                    id: "energy-reset",
                    variant: ButtonVariant::Secondary,
                    onclick: move |_| {
                        dataset.set(EnergyDataset::Week1);
                        state.set(ChartToolboxState::new(dataset_data(EnergyDataset::Week1)));
                        changes.set(0);
                        exported.set(None);
                        let next_epoch = epoch.peek().wrapping_add(1);
                        epoch.set(next_epoch);
                    },
                    "Restore example"
                }
            }
            p {
                id: "energy-dataset",
                class: "chart-workflows-status",
                role: "status",
                "data-week": current_dataset.week_token(),
                "{current_dataset.summary()}"
            }
            // Dioxus reconciles keys on fragment children; keep this lifecycle explicit.
            for generation in [epoch()] {
                div { key: "energy-charts-stage-{generation}", class: "chart-workflows-stage",
                    ChartToolbox {
                        categories: base_data.categories.clone(),
                        series: base_data.series.clone(),
                        label: "Energy history",
                        file_name: "energy-history.png",
                        onchange: move |next: ChartToolboxState| {
                            let next_changes = changes.peek().wrapping_add(1);
                            changes.set(next_changes);
                            state.set(next);
                            exported.set(None);
                        },
                        onexport: move |png: ChartPng| {
                            let summary = validate_export(&png);
                            exported.set(summary);
                        },
                    }
                    div { class: "chart-workflows-companion",
                        h3 { "Energy comparison" }
                        p {
                            "Compare the loaded week. Toggle a series in the legend, inspect samples with arrow keys, or expand the data table. Edits in the history chart leave this comparison unchanged."
                        }
                        CartesianChart {
                            categories: base_data.categories.clone(),
                            series: base_data.series.clone(),
                            label: "Energy comparison",
                            y_title: "Energy (kWh)",
                            axis_tooltip: true,
                            legend: true,
                        }
                    }
                }
            }
            output {
                id: "energy-state",
                class: "chart-workflows-status",
                role: "status",
                "data-window-start": window.start.to_string(),
                "data-window-end": window.end.to_string(),
                "data-categories": visible_categories.to_string(),
                "data-first-value": first_value.clone(),
                "data-kind": current_kind,
                "data-changes": change_count.to_string(),
                if visible_categories == 0 { "No energy samples loaded." }
                else { "Showing {visible_categories} days in the {current_kind} view." }
            }
            output {
                id: "energy-export",
                class: "chart-workflows-status",
                role: "status",
                "data-width": export_width.to_string(),
                "data-height": export_height.to_string(),
                "data-bytes": export_bytes.to_string(),
                if let Some(summary) = export_summary {
                    "Exported energy-history.png: {summary.width} × {summary.height} px ({summary.bytes} bytes)."
                } else {
                    "No PNG exported yet."
                }
            }
        }
    }
}

pub const STYLES: &str = include_str!("styles/chart_workflows.css");

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_1X1_PNG: &str = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR4nGP4z8DwHwAFAAH/iZk9HQAAAABJRU5ErkJggg==";

    #[test]
    fn datasets_and_export_validation_follow_energy_workflow_contract() {
        let week1 = dataset_data(EnergyDataset::Week1);
        assert_eq!(week1.categories.len(), 7);
        assert_eq!(week1.categories[0], "Mon");
        assert_eq!(week1.categories[6], "Sun");
        assert_eq!(week1.series.len(), 2);
        assert_eq!(week1.series[0].name, "Output");
        assert_eq!(week1.series[0].values, WEEK1_OUTPUT);
        assert_eq!(week1.series[0].kind, ChartKind::Line);
        assert_eq!(week1.series[1].name, "Target");
        assert_eq!(week1.series[1].values, TARGET_OUTPUT);
        assert_eq!(week1.series[1].kind, ChartKind::Line);

        let week2 = dataset_data(EnergyDataset::Week2);
        assert_eq!(week2.categories, week1.categories);
        assert_eq!(week2.series[0].values, WEEK2_OUTPUT);
        assert_eq!(week2.series[1].values, TARGET_OUTPUT);

        let empty = dataset_data(EnergyDataset::Empty);
        assert!(empty.categories.is_empty());
        assert!(empty.series.iter().all(|series| series.values.is_empty()));
        let empty_state = ChartToolboxState::new(empty);
        assert_eq!(empty_state.window(), 0..0);
        assert_eq!(first_visible_value(&empty_state.visible_data()), "none");

        assert_eq!(
            next_week_dataset(EnergyDataset::Week1),
            EnergyDataset::Week2
        );
        assert_eq!(
            next_week_dataset(EnergyDataset::Week2),
            EnergyDataset::Week1
        );
        assert_eq!(
            next_week_dataset(EnergyDataset::Empty),
            EnergyDataset::Week1
        );

        let valid_png = ChartPng {
            data_url: VALID_1X1_PNG.into(),
            width: 1,
            height: 1,
        };
        let summary = validate_export(&valid_png).expect("valid 1x1 PNG");
        assert_eq!(summary.width, 1);
        assert_eq!(summary.height, 1);
        assert!(summary.bytes >= 33);

        let mismatched_png = ChartPng {
            data_url: VALID_1X1_PNG.into(),
            width: 2,
            height: 1,
        };
        assert_eq!(validate_export(&mismatched_png), None);
    }

    #[test]
    fn chart_workflows_renders_toolbox_companion_chart_and_status_attributes() {
        let mut dom = VirtualDom::new(ChartWorkflows);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);

        assert!(html.contains("id=\"energy-charts\""));
        assert!(html.contains("Energy history"));
        assert!(html.contains("Illustrative data"));
        assert!(html.contains("id=\"energy-next\""));
        assert!(html.contains("Load next week"));
        assert!(html.contains("id=\"energy-empty\""));
        assert!(html.contains("Show empty data"));
        assert!(html.contains("id=\"energy-reset\""));
        assert!(html.contains("Restore example"));
        assert!(html.contains("id=\"energy-dataset\""));
        assert!(html.contains("data-week=\"1\""));
        assert!(html.contains("id=\"energy-state\""));
        assert!(html.contains("data-window-start=\"0\""));
        assert!(html.contains("data-window-end=\"7\""));
        assert!(html.contains("data-categories=\"7\""));
        assert!(html.contains("data-first-value=\"42\""));
        assert!(html.contains("data-kind=\"line\""));
        assert!(html.contains("data-changes=\"0\""));
        assert!(html.contains("id=\"energy-export\""));
        assert!(html.contains("data-width=\"0\""));
        assert!(html.contains("data-height=\"0\""));
        assert!(html.contains("data-bytes=\"0\""));
        assert!(html.contains("aria-label=\"Energy comparison\""));
        assert!(html.contains("Energy (kWh)"));
        assert!(!html.contains("data:image/png"));
        assert!(!html.contains("<script"));
    }
}
