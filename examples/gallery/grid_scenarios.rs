//! The upstream AG Grid demo data, with an independent Rust grid implementation.
use dioxus::prelude::*;
use dioxus_ix::{
    Button, ButtonVariant, DataGrid, DataGridCellValueChanged, DataGridColumn, DataGridRow,
    DataGridState, DataGridTextEditor,
};

pub const SCENARIOS: &[&str] = &["aggrid", "aggrid-striped"];

fn equipment_rows() -> Vec<DataGridRow> {
    [
        ("Equipment", "Normal", "2.0"),
        ("Positioner", "Maintenance", "1.0"),
        ("Pressure sensor", "Unknown", "N/A"),
        ("Flow meter", "Normal", "3.1"),
        ("Temperature sensor", "Warning", "2.2"),
        ("Valve", "Normal", "1.5"),
        ("Actuator", "Maintenance", "2.0"),
        ("Controller", "Normal", "4.0"),
        ("Safety relay", "Unknown", "N/A"),
        ("Power supply", "Normal", "1.8"),
    ]
    .into_iter()
    .enumerate()
    .map(|(index, (kind, status, hardware))| {
        DataGridRow::new(format!("equipment-{index}"))
            .cell("type", kind)
            .cell("status", status)
            .cell("hwVersion", hardware)
            .tooltip("type", kind)
            .tooltip("status", format!("Status: {status} - Type: {kind}"))
            .tooltip(
                "hwVersion",
                if hardware == "N/A" {
                    "Hardware version not available".to_string()
                } else {
                    format!("Hardware Version {hardware}")
                },
            )
    })
    .collect()
}

#[component]
pub fn GridScenario(name: String) -> Element {
    let mut state = use_signal(DataGridState::default);
    let mut kind = DataGridColumn::new("type", "Type");
    kind.row_drag = true;
    let mut status = DataGridColumn::new("status", "Status");
    status.sortable = true;
    status.filterable = true;
    let hardware = DataGridColumn::new("hwVersion", "HW version");
    rsx! { div { class: "source-data-grid",
        DataGrid { columns: vec![kind, status, hardware], rows: equipment_rows(), state: state(),
            onstatechange: move |next| state.set(next), striped: name == "aggrid-striped", height: 192, tooltip_delay_ms: 500,
            label: "Equipment data",
            suppress_cell_focus: true,
        }
    } }
}

fn fleet_rows() -> Vec<DataGridRow> {
    (0..10_000).map(|index| DataGridRow::new(format!("fleet-{index}"))
        .cell("type", if index == 9_999 { "Extended temperature monitoring assembly for remote high-pressure installations".to_string() } else { format!("Equipment {index:05}") })
        .cell("status", if index % 4 == 0 { "Maintenance" } else { "Normal" })
        .cell("hwVersion", format!("{}.{}", index % 5 + 1, index % 10)))
        .collect()
}

/// A scale review alongside, rather than a change to, the pinned source fixtures.
#[component]
pub fn GridExamples() -> Element {
    let mut state = use_signal(DataGridState::default);
    let rows = use_hook(fleet_rows);
    let mut kind = DataGridColumn::new("type", "Type");
    kind.row_drag = true;
    let mut status = DataGridColumn::new("status", "Status");
    status.sortable = true;
    status.filterable = true;
    let hardware = DataGridColumn::new("hwVersion", "HW version");
    rsx! {
    GridEditingExample {}
    section { class: "grid-scale-example",
        h2 { "Equipment fleet" }
        p { "10,000 illustrative records. Arrow keys, Home, End and Page Up/Down navigate the complete grid. Enter on a column separator fits all filtered values, including the long final record." }
        div { class: "grid-scale-toolbar",
            Button { variant: ButtonVariant::Secondary, onclick: move |_| state.set(DataGridState::default()), "Reset grid" }
            span { role: "status", "{state().selected.len()} selected" }
        }
        DataGrid { class: "fleet-grid", columns: vec![kind,status,hardware], rows, state: state(), onstatechange: move |next| state.set(next), height: 360,
            label: "Equipment fleet", striped: true,
        }
    } }
}

fn editable_equipment_rows() -> Vec<DataGridRow> {
    (0..250)
        .map(|index| {
            let mut row = DataGridRow::new(format!("editable-{index}"))
                .cell("asset", format!("Asset {index:03}"))
                .cell(
                    "location",
                    if index % 2 == 0 {
                        "Assembly line"
                    } else {
                        "Service bay"
                    },
                )
                .cell("serial", format!("IX-{index:05}"));
            if index == 1 {
                row.read_only_cells.insert("asset".into());
            }
            row
        })
        .collect()
}

/// Controlled text editing stays separate from the two unchanged source fixtures.
#[component]
fn GridEditingExample() -> Element {
    let mut rows = use_signal(editable_equipment_rows);
    let mut state = use_signal(DataGridState::default);
    let mut read_only = use_signal(|| false);
    let mut status = use_signal(|| "No edits saved.".to_string());
    let mut asset = DataGridColumn::new("asset", "Asset");
    asset.min_width = 180;
    asset.editable = true;
    asset.sortable = true;
    asset.filterable = true;
    asset.editor = DataGridTextEditor {
        required: true,
        max_length: Some(32),
    };
    let mut location = DataGridColumn::new("location", "Location");
    location.min_width = 180;
    location.editable = true;
    location.editor = DataGridTextEditor {
        required: true,
        max_length: Some(48),
    };
    let mut serial = DataGridColumn::new("serial", "Serial number");
    serial.min_width = 160;
    rsx! { section { class: "grid-editing-example", id: "grid-editing",
        h2 { "Editable equipment register" }
        p { "250 illustrative records. Double-click a cell or press Enter/F2 to edit. Enter saves, Escape cancels, and Tab/Shift+Tab saves and edits the next available cell. Asset and Location are required. Serial numbers and Asset 001 are read-only." }
        div { class: "grid-scale-toolbar",
            label { input { r#type: "checkbox", checked: read_only(), onchange: move |event: FormEvent| read_only.set(event.checked()) } "Read-only register" }
            Button { variant: ButtonVariant::Secondary, onclick: move |_| { rows.set(editable_equipment_rows()); state.set(DataGridState::default()); status.set("Register restored.".into()); }, "Restore register" }
        }
        DataGrid { class: "editable-equipment-grid", columns: vec![asset, location, serial], rows: rows(), state: state(),
            onstatechange: move |next| state.set(next), height: 280, read_only: read_only(), label: "Editable equipment register",
            oncellvaluechanged: move |change: DataGridCellValueChanged| {
                let mut next = rows.peek().clone();
                if change.apply_to(&mut next) {
                    status.set(format!("Saved {} for {}: {}", change.column_id, change.row_id, change.new_value));
                    rows.set(next);
                }
            },
        }
        p { class: "grid-editing-status", role: "status", "{status}" }
    } }
}

pub const STYLES: &str = ".grid-editing-example{margin-bottom:48px}.grid-scale-toolbar>label{display:flex;align-items:center;gap:8px}.grid-editing-status{min-height:1.5em;color:var(--ix-color-text-secondary)}.source-data-grid{width:100%;min-width:0}.grid-scale-example,.grid-editing-example{min-width:0;max-width:100%}.grid-scale-example>p,.grid-editing-example>p{max-width:72ch;line-height:1.55}.grid-scale-toolbar{display:flex;align-items:center;flex-wrap:wrap;gap:16px;margin:20px 0 12px}.grid-scale-toolbar>span{color:var(--ix-color-text-secondary);font-variant-numeric:tabular-nums}";

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn editing_review_has_virtualization_and_readonly_cells() {
        let records = editable_equipment_rows();
        assert_eq!(records.len(), 250);
        assert!(records[1].read_only_cells.contains("asset"));
        let mut dom = VirtualDom::new(GridEditingExample);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert!(html.contains("aria-label=\"Editable equipment register\""));
        assert!(html.contains("data-row-id=\"editable-0\""));
        assert!(!html.contains("data-row-id=\"editable-249\""));
        assert!(html.contains("aria-readonly=\"true\""));
        assert!(html.contains("aria-readonly=\"false\""));
        assert!(!html.contains("class=\"ix-data-grid-editor\""));
        assert!(html.contains("No edits saved."));
    }
    #[test]
    fn scale_demo_keeps_original_fixtures_separate_and_unmounted_final_text_available() {
        let rows = fleet_rows();
        assert_eq!(rows.len(), 10_000);
        assert_eq!(rows.last().unwrap().id, "fleet-9999");
        assert!(
            rows.last()
                .unwrap()
                .value("type")
                .contains("high-pressure installations")
        );
        assert_eq!(equipment_rows().len(), 10);
        let mut dom = VirtualDom::new(GridExamples);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert_eq!(html.matches("data-row-id=\"fleet-").count(), 18);
        assert!(html.contains("aria-rowcount=10001"));
        assert!(!html.contains("data-row-id=\"fleet-9999\""));
        assert!(html.contains("aria-label=\"Equipment fleet\""));
    }
    #[test]
    fn rows_preserve_exact_source_values_and_tooltip_strings() {
        let rows = equipment_rows();
        assert_eq!(rows.len(), 10);
        assert_eq!(rows[0].value("type"), "Equipment");
        assert_eq!(rows[9].value("hwVersion"), "1.8");
        assert_eq!(
            rows[2].tooltips["hwVersion"],
            "Hardware version not available"
        );
        assert_eq!(
            rows[4].tooltips["status"],
            "Status: Warning - Type: Temperature sensor"
        );
        assert_eq!(rows[3].tooltips["hwVersion"], "Hardware Version 3.1");
        let mut state = DataGridState::default();
        state.cycle_sort("status");
        assert_eq!(state.visible_rows(&rows)[0].value("type"), "Positioner");
        assert_eq!(state.visible_rows(&rows)[1].value("type"), "Actuator");
    }
    #[test]
    fn both_scenarios_render_real_grid_controls_without_an_external_runtime() {
        for name in SCENARIOS {
            let mut dom = VirtualDom::new_with_props(
                GridScenario,
                GridScenarioProps {
                    name: (*name).to_string(),
                },
            );
            dom.rebuild_in_place();
            let html = dioxus_ssr::render(&dom);
            assert!(html.contains("height:192px"));
            assert!(html.contains("Select all filtered rows"));
            assert!(html.contains("Filter Status"));
            assert!(html.contains("Resize HW version"));
            assert!(html.contains("Move column Type"));
            assert!(html.contains("Move column HW version"));
            assert!(html.contains("Enter fits its header and filtered row content"));
            assert!(html.contains("Move Equipment"));
            assert!(html.contains("Pressure sensor"));
            assert!(!html.contains("<script"));
            assert_eq!(html.matches("role=\"checkbox\"").count(), 11);
            assert_eq!(html.matches("data-row-id=").count(), 10);
            assert!(html.contains(if *name == "aggrid-striped" {
                "data-striped=\"true\""
            } else {
                "data-striped=\"false\""
            }));
        }
    }
}
