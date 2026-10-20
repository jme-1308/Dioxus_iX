//! A wide, controlled local inspection sheet with editable measurements.
use dioxus::prelude::*;
use dioxus_ix::*;

fn inspection_columns() -> Vec<DataGridColumn> {
    let mut columns = Vec::new();
    for (id, title, min_width, editable) in [
        ("asset", "Asset", 140, true),
        ("owner", "Owner", 120, true),
        ("location", "Location", 140, false),
    ] {
        let mut column = DataGridColumn::new(id, title);
        column.min_width = min_width;
        column.sortable = true;
        column.filterable = true;
        column.editable = editable;
        column.editor = DataGridTextEditor {
            required: true,
            max_length: Some(80),
        };
        columns.push(column);
    }
    for index in 1..=32 {
        let mut column =
            DataGridColumn::new(format!("check-{index:02}"), format!("Check {index:02} (%)"));
        column.min_width = 120;
        column.sortable = true;
        column.filterable = true;
        column.editable = true;
        column.value_type = DataGridValueType::Number;
        column.editor = DataGridTextEditor {
            required: true,
            max_length: None,
        };
        column.cell_editor = DataGridCellEditor::Number(DataGridNumberEditor {
            min: Some(0.0),
            max: Some(100.0),
            step: Some(1.0),
        });
        columns.push(column);
    }
    let mut review = DataGridColumn::new("review", "Review note");
    review.min_width = 160;
    review.sortable = true;
    review.filterable = true;
    review.editable = true;
    review.editor = DataGridTextEditor {
        required: true,
        max_length: Some(120),
    };
    columns.push(review);
    columns
}

fn inspection_rows() -> Vec<DataGridRow> {
    (1..=64)
        .map(|index| {
            let asset = match (index - 1) % 3 {
                0 => "Motor",
                1 => "Pump",
                _ => "Fan",
            };
            let mut row = DataGridRow::new(format!("equipment-{index:03}"))
                .cell("asset", format!("{asset} {index:03}"))
                .cell("owner", if index % 2 == 1 { "Team A" } else { "Team B" })
                .cell(
                    "location",
                    format!("Plant A / Line {}", (index - 1) % 4 + 1),
                )
                .cell("review", "Pending review");
            for check in 1..=32 {
                row = row.cell(
                    format!("check-{check:02}"),
                    ((index * 3 + check) % 101).to_string(),
                );
            }
            row
        })
        .collect()
}

fn inspection_state() -> DataGridState {
    let mut state = DataGridState::default();
    for column in inspection_columns() {
        let width = match column.id.as_str() {
            "asset" => 180.0,
            "owner" => 150.0,
            "location" => 180.0,
            "review" => 220.0,
            _ => 132.0,
        };
        state.resize_column(&column, width);
    }
    state
}

fn stored_value(rows: &[DataGridRow], id: &str, column: &str) -> String {
    rows.iter()
        .find(|row| row.id == id)
        .map(|row| row.value(column).to_string())
        .unwrap_or_default()
}

#[component]
fn SheetPreparation() -> Element {
    let mut note = use_signal(|| "Prepare inspection sheet".to_string());
    rsx! {
        label { class: "wide-grid-note-field", r#for: "wide-grid-note",
            span { "Inspection preparation" }
            input { id: "wide-grid-note", r#type: "text", maxlength: "160", value: note(),
                oninput: move |event: FormEvent| note.set(event.value()),
            }
        }
    }
}

/// An illustrative local sheet with full controlled records, widths and column order.
#[component]
pub fn WideEquipmentGridExample() -> Element {
    let mut columns = use_signal(inspection_columns);
    let mut rows = use_signal(inspection_rows);
    let mut state = use_signal(inspection_state);
    let mut full_columns = use_signal(|| false);
    let mut read_only = use_signal(|| false);
    let mut edits = use_signal(|| 0_usize);
    let mut epoch = use_signal(|| 0_u64);
    let current_columns = columns();
    let current_rows = rows();
    let current_state = state();
    let visible = current_state.visible_rows_with_columns(&current_rows, &current_columns);
    let visible_count = visible.len();
    let visible_order = visible
        .iter()
        .map(|row| row.id.as_str())
        .collect::<Vec<_>>()
        .join(",");
    let column_order = current_state
        .ordered_columns(&current_columns)
        .iter()
        .map(|column| column.id.as_str())
        .collect::<Vec<_>>()
        .join(",");
    let selected_count = current_state.selected.len();
    let has_team_filter = current_state.filters.get("owner").is_some_and(|filter| {
        filter.first.operator == DataGridFilterOperator::Equals && filter.first.value == "Team A"
    });
    let asset_width = current_state
        .column_widths
        .get("asset")
        .copied()
        .unwrap_or(180);
    let sort = current_state.sort.as_ref().map_or_else(
        || "none".to_string(),
        |(column, direction)| {
            format!(
                "{column}:{}",
                if *direction == DataGridSortDirection::Ascending {
                    "asc"
                } else {
                    "desc"
                }
            )
        },
    );

    rsx! {
        section { id: "wide-equipment-grid", class: "wide-equipment-grid-example",
            h2 { "Equipment inspection sheet" }
            p { class: "wide-grid-intro", "Edit 64 illustrative local equipment records across 36 columns. The percentage checks are example inspection scores, not live measurements. Sort or filter from a column header, select records, and resize or reorder columns." }
            p { class: "wide-grid-instructions", "Scroll horizontally to reach later checks and Review note. Ctrl+Right/Left moves to the last/first logical column. On a column header, Left/Right moves between columns; Tab reaches its controls. F2 or Enter edits a cell; Enter saves, Escape cancels, and Tab saves and continues to the next editable cell." }
            div { class: "wide-grid-toolbar",
                Button { id: "wide-grid-reset", variant: ButtonVariant::Secondary,
                    onclick: move |_| {
                        columns.set(inspection_columns()); rows.set(inspection_rows()); state.set(inspection_state());
                        full_columns.set(false); read_only.set(false); edits.set(0); let next = epoch.peek().wrapping_add(1); epoch.set(next);
                    }, "Restore inspection sheet"
                }
                Button { id: "wide-grid-team", variant: ButtonVariant::Secondary,
                    onclick: move |_| {
                        let mut next = state.peek().clone();
                        if has_team_filter { next.filters.remove("owner"); }
                        else { next.filters.insert("owner".into(), DataGridTextFilter {
                            first: DataGridFilterCondition { operator: DataGridFilterOperator::Equals, value: "Team A".into() },
                            second: DataGridFilterCondition::default(), match_any: false,
                        }); }
                        state.set(next);
                    },
                    if has_team_filter { "All owners" } else { "Team A only" }
                }
                Button { id: "wide-grid-select", variant: ButtonVariant::Secondary,
                    onclick: move |_| { let mut next = state.peek().clone(); next.select_filtered_with_columns(&rows.peek(), &columns.peek(), true); state.set(next); },
                    "Select visible"
                }
                Button { id: "wide-grid-width", variant: ButtonVariant::Secondary,
                    onclick: move |_| {
                        let mut next = state.peek().clone();
                        if let Some(column) = columns.peek().iter().find(|column| column.id == "asset") {
                            next.resize_column(column, if asset_width < 260 { 260.0 } else { 180.0 });
                        }
                        state.set(next);
                    },
                    if asset_width < 260 { "Widen asset" } else { "Restore asset width" }
                }
                Button { id: "wide-grid-reverse", variant: ButtonVariant::Secondary,
                    onclick: move |_| {
                        let mut next = state.peek().clone();
                        let mut order: Vec<_> = next.ordered_columns(&columns.peek()).iter().map(|column| column.id.clone()).collect();
                        order.reverse(); next.column_order = order; state.set(next);
                    }, "Reverse columns"
                }
            }
            details { id: "wide-grid-options", class: "wide-grid-options",
                summary { "Sheet options" }
                div { class: "wide-grid-option-fields",
                    label { input { id: "wide-grid-full-columns", r#type: "checkbox", checked: full_columns(), onchange: move |event: FormEvent| full_columns.set(event.checked()) } "Mount all columns" }
                    label { input { id: "wide-grid-readonly", r#type: "checkbox", checked: read_only(), onchange: move |event: FormEvent| read_only.set(event.checked()) } "Read-only sheet" }
                }
            }
            for generation in [epoch()] { SheetPreparation { key: "wide-grid-preparation-{generation}" } }
            div { id: "wide-grid-host", class: "wide-grid-host",
                for generation in [epoch()] {
                    DataGrid { key: "wide-inspection-grid-{generation}", class: "wide-inspection-grid", label: "Equipment inspection sheet",
                        columns: current_columns.clone(), rows: current_rows.clone(), state: current_state.clone(), onstatechange: move |next| state.set(next),
                        height: 320, row_height: 42, striped: true, read_only: read_only(), suppress_column_virtualisation: full_columns(),
                        oncellvaluechanged: move |change: DataGridCellValueChanged| {
                            if *read_only.peek() { return; }
                            let mut next = rows.peek().clone();
                            if change.apply_to(&mut next) { rows.set(next); let count = edits.peek().saturating_add(1); edits.set(count); }
                        },
                    }
                }
            }
            table { id: "wide-grid-records", class: "wide-grid-records",
                caption { "Saved first and last equipment records" }
                thead { tr { th { scope: "col", "Asset" } th { scope: "col", "Saved inspection details" } } }
                tbody {
                    for row_id in ["equipment-001", "equipment-064"] {
                        tr { key: "{row_id}", id: format!("wide-grid-record-{row_id}"), "data-row-id": row_id,
                            th { scope: "row", "data-field": "asset", {stored_value(&current_rows, row_id, "asset")} }
                            td {
                                span { "Owner: " span { "data-field": "owner", {stored_value(&current_rows, row_id, "owner")} } }
                                span { "Check 01: " span { "data-field": "check-01", {stored_value(&current_rows, row_id, "check-01")} } "%; Check 32: " span { "data-field": "check-32", {stored_value(&current_rows, row_id, "check-32")} } "%" }
                                span { "data-field": "review", {stored_value(&current_rows, row_id, "review")} }
                            }
                        }
                    }
                }
            }
            output { id: "wide-grid-result", class: "wide-grid-result", role: "status",
                "data-total": current_rows.len().to_string(), "data-columns": current_columns.len().to_string(), "data-visible": visible_count.to_string(),
                "data-selected": selected_count.to_string(), "data-edits": edits().to_string(), "data-column-order": column_order,
                "data-row-order": visible_order, "data-sort": sort, "data-full-columns": full_columns().to_string(), "data-readonly": read_only().to_string(),
                "data-asset-width": asset_width.to_string(),
                "{visible_count} visible equipment records · {selected_count} selected · Saved edits: {edits}"
            }
        }
    }
}

pub(crate) const STYLES: &str = include_str!("styles/wide_equipment_grid.css");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wide_sheet_renders_bounded_headers_and_real_first_and_last_records() {
        let mut dom = VirtualDom::new(WideEquipmentGridExample);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        for id in [
            "wide-equipment-grid",
            "wide-grid-host",
            "wide-grid-reset",
            "wide-grid-note",
            "wide-grid-full-columns",
            "wide-grid-record-equipment-001",
            "wide-grid-record-equipment-064",
            "wide-grid-result",
        ] {
            assert!(html.contains(&format!("id=\"{id}\"")), "{id}");
        }
        assert!(
            html.contains("Equipment inspection sheet")
                && html.contains("Prepare inspection sheet")
        );
        assert!(
            html.contains("Motor 001")
                && html.contains("Motor 064")
                && html.contains("Pending review")
        );
        assert!(
            html.contains("data-total=\"64\"")
                && html.contains("data-columns=\"36\"")
                && html.contains("data-edits=\"0\"")
        );
        let headers = html.matches("role=\"columnheader\"").count();
        assert!(
            headers > 0 && headers < 37,
            "initial wide sheet mounts a bounded column window: {headers}"
        );
        assert!(
            html.contains("data-field=\"check-32\">22"),
            "last stored row remains part of the actual model"
        );
    }
}
