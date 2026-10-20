//! Controlled spare-parts table combining text, numeric, select, and checkbox cell editors.
use dioxus::prelude::*;
use dioxus_ix::{
    Button, ButtonVariant, DataGrid, DataGridCellEditor, DataGridCellValueChanged, DataGridColumn,
    DataGridNumberEditor, DataGridRow, DataGridSelectOption, DataGridState, DataGridTextEditor,
};

fn spare_parts_rows() -> Vec<DataGridRow> {
    let part_1 = DataGridRow::new("part-1")
        .cell("asset", "Pump kit")
        .cell("quantity", "2.5")
        .cell("status", "ready")
        .cell("approved", "false");

    let part_2 = DataGridRow::new("part-2")
        .cell("asset", "Valve kit")
        .cell("quantity", "10")
        .cell("status", "service")
        .cell("approved", "true");

    let mut part_3 = DataGridRow::new("part-3")
        .cell("asset", "Sensor kit")
        .cell("quantity", "4")
        .cell("status", "ready")
        .cell("approved", "false");
    part_3.read_only_cells.insert("quantity".into());

    vec![part_1, part_2, part_3]
}

fn status_options() -> Vec<DataGridSelectOption> {
    let mut retired = DataGridSelectOption::new("retired", "Retired");
    retired.disabled = true;
    vec![
        DataGridSelectOption::new("ready", "Ready"),
        DataGridSelectOption::new("service", "Needs service"),
        retired,
    ]
}

fn spare_parts_columns() -> Vec<DataGridColumn> {
    let mut asset = DataGridColumn::new("asset", "Asset");
    asset.min_width = 160;
    asset.editable = true;
    asset.sortable = true;
    asset.filterable = true;
    asset.editor = DataGridTextEditor {
        required: true,
        max_length: Some(40),
    };
    asset.cell_editor = DataGridCellEditor::Text;

    let mut quantity = DataGridColumn::new("quantity", "Quantity");
    quantity.min_width = 120;
    quantity.editable = true;
    quantity.filterable = true;
    quantity.editor = DataGridTextEditor {
        required: true,
        max_length: None,
    };
    quantity.cell_editor = DataGridCellEditor::Number(DataGridNumberEditor {
        min: Some(0.0),
        max: Some(100.0),
        step: Some(0.5),
    });

    let mut status = DataGridColumn::new("status", "Status");
    status.min_width = 150;
    status.editable = true;
    status.sortable = true;
    status.filterable = true;
    status.editor = DataGridTextEditor {
        required: true,
        max_length: None,
    };
    status.cell_editor = DataGridCellEditor::Select(status_options());

    let mut approved = DataGridColumn::new("approved", "Approved");
    approved.min_width = 120;
    approved.editable = true;
    approved.sortable = true;
    approved.filterable = true;
    approved.editor = DataGridTextEditor {
        required: true,
        max_length: None,
    };
    approved.cell_editor = DataGridCellEditor::Checkbox;

    vec![asset, quantity, status, approved]
}

fn row_cell_value(rows: &[DataGridRow], row_id: &str, column_id: &str) -> String {
    rows.iter()
        .find(|row| row.id == row_id)
        .map(|row| row.value(column_id).to_string())
        .unwrap_or_default()
}

/// Controlled spare-parts inventory showcasing typed text, number, select, and checkbox cell editors.
#[component]
pub fn GridTypedEditorsExample() -> Element {
    let mut rows = use_signal(spare_parts_rows);
    let mut state = use_signal(DataGridState::default);
    let mut read_only = use_signal(|| false);
    let mut hold_edits = use_signal(|| false);
    let mut count = use_signal(|| 0usize);
    let mut last = use_signal(|| "none".to_string());
    let mut epoch = use_signal(|| 0u64);

    let current_rows = rows();
    let current_state = state();
    let current_count = count();
    let current_last = last();
    let part1_quantity = row_cell_value(&current_rows, "part-1", "quantity");
    let part1_status = row_cell_value(&current_rows, "part-1", "status");
    let part1_approved = row_cell_value(&current_rows, "part-1", "approved");
    let part2_quantity = row_cell_value(&current_rows, "part-2", "quantity");

    rsx! {
        section { class: "grid-typed-editors-example", id: "grid-typed-editors",
            h2 { "Spare-parts editor" }
            p {
                "Three illustrative spare-parts records kept in local component state with no backend service. "
                "Double-click an editable cell or press F2/Enter to edit; Enter saves, Escape cancels, and Tab/Shift+Tab saves and moves to the next editable cell. "
                "Asset is required text; Quantity accepts numbers from 0 to 100 in steps of 0.5 (Sensor kit's quantity is read-only); Status offers Ready and Needs service while Retired is disabled; Approved toggles between true and false."
            }
            div { class: "grid-typed-editors-toolbar",
                Button {
                    id: "grid-typed-reset",
                    variant: ButtonVariant::Secondary,
                    onclick: move |_| {
                        rows.set(spare_parts_rows());
                        state.set(DataGridState::default());
                        count.set(0);
                        last.set("none".into());
                        read_only.set(false);
                        hold_edits.set(false);
                        let next_epoch = epoch.peek().wrapping_add(1);
                        epoch.set(next_epoch);
                    },
                    "Restore parts"
                }
                label { r#for: "grid-typed-readonly",
                    input {
                        id: "grid-typed-readonly",
                        r#type: "checkbox",
                        checked: read_only(),
                        onchange: move |event: FormEvent| read_only.set(event.checked()),
                    }
                    "Read-only parts"
                }
                label { r#for: "grid-typed-hold",
                    input {
                        id: "grid-typed-hold",
                        r#type: "checkbox",
                        checked: hold_edits(),
                        onchange: move |event: FormEvent| hold_edits.set(event.checked()),
                    }
                    "Hold edit requests"
                }
            }
            // Dioxus reconciles keys on fragment children; keep this lifecycle explicit.
            for generation in [epoch()] {
                DataGrid {
                    key: "typed-parts-grid-{generation}",
                    class: "typed-parts-grid",
                    columns: spare_parts_columns(),
                    rows: current_rows.clone(),
                    state: current_state.clone(),
                    onstatechange: move |next| state.set(next),
                    height: 240,
                    row_height: 42,
                    striped: true,
                    read_only: read_only(),
                    label: "Spare-parts editor",
                    oncellvaluechanged: move |change: DataGridCellValueChanged| {
                        let next_count = count.peek().saturating_add(1);
                        count.set(next_count);
                        last.set(format!(
                            "{}:{}:{}->{}",
                            change.row_id, change.column_id, change.old_value, change.new_value
                        ));
                        if !hold_edits() {
                            let mut next = rows.peek().clone();
                            if change.apply_to(&mut next) {
                                rows.set(next);
                            }
                        }
                    },
                }
            }
            p {
                id: "grid-typed-result",
                class: "grid-typed-editors-result",
                role: "status",
                "data-count": current_count.to_string(),
                "data-last": current_last.clone(),
                "data-part1-quantity": part1_quantity.clone(),
                "data-part1-status": part1_status.clone(),
                "data-part1-approved": part1_approved.clone(),
                "data-part2-quantity": part2_quantity.clone(),
                "Requests: {current_count} · Last: {current_last} · Pump kit: qty {part1_quantity}, status {part1_status}, approved {part1_approved} · Valve kit: qty {part2_quantity}"
            }
        }
    }
}

pub const STYLES: &str = include_str!("styles/grid_typed_editors.css");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typed_editors_example_renders_baseline_contract() {
        let mut dom = VirtualDom::new(GridTypedEditorsExample);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert!(html.contains("id=\"grid-typed-editors\""));
        assert!(html.contains("aria-label=\"Spare-parts editor\""));
        assert!(html.contains("id=\"grid-typed-reset\""));
        assert!(html.contains("id=\"grid-typed-readonly\""));
        assert!(html.contains("id=\"grid-typed-hold\""));
        assert!(html.contains("id=\"grid-typed-result\""));
        assert!(html.contains("data-count=\"0\""));
        assert!(html.contains("data-last=\"none\""));
        assert!(html.contains("data-part1-quantity=\"2.5\""));
        assert!(html.contains("data-part1-status=\"ready\""));
        assert!(html.contains("data-part1-approved=\"false\""));
        assert!(html.contains("data-part2-quantity=\"10\""));
        assert!(html.contains("data-row-id=\"part-1\""));
        assert!(html.contains("data-row-id=\"part-2\""));
        assert!(html.contains("data-row-id=\"part-3\""));
    }
}
