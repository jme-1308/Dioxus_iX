//! Composed status, note, and action cell renderers for the controlled Rust data grid.
use dioxus::prelude::*;
use dioxus_ix::{
    Button, ButtonVariant, DataGrid, DataGridCellContext, DataGridCellEditor, DataGridCellRender,
    DataGridCellValueChanged, DataGridColumn, DataGridFilterCondition, DataGridFilterOperator,
    DataGridRow, DataGridSortDirection, DataGridState, DataGridTextEditor, DataGridTextFilter,
    Dropdown, DropdownItem, DropdownNavigationMode, Pill, StatusVariant,
};

fn maintenance_cell_rows() -> Vec<DataGridRow> {
    let asset_1 = DataGridRow::new("asset-1")
        .cell("asset", "Motor 001")
        .cell("status", "ready")
        .cell("note", "Inspect seal")
        .cell("actions", "Actions for asset-1");

    let asset_2 = DataGridRow::new("asset-2")
        .cell("asset", "Pump 002")
        .cell("status", "service")
        .cell("note", "")
        .cell("actions", "Actions for asset-2");

    let mut asset_3 = DataGridRow::new("asset-3")
        .cell("asset", "Valve 003")
        .cell("status", "ready")
        .cell("note", "Read-only note")
        .cell("actions", "Actions for asset-3");
    asset_3.read_only_cells.insert("note".into());

    vec![asset_1, asset_2, asset_3]
}

fn maintenance_cell_columns() -> Vec<DataGridColumn> {
    let mut asset = DataGridColumn::new("asset", "Asset");
    asset.min_width = 160;
    asset.editable = true;
    asset.sortable = true;
    asset.editor = DataGridTextEditor {
        required: true,
        max_length: Some(40),
    };
    asset.cell_editor = DataGridCellEditor::Text;

    let mut status = DataGridColumn::new("status", "Status");
    status.min_width = 160;
    status.filterable = true;

    let mut note = DataGridColumn::new("note", "Note");
    note.min_width = 220;
    note.editable = true;

    let mut actions = DataGridColumn::new("actions", "Actions");
    actions.min_width = 180;

    vec![asset, status, note, actions]
}

fn row_cell_value(rows: &[DataGridRow], row_id: &str, column_id: &str) -> String {
    rows.iter()
        .find(|row| row.id == row_id)
        .map(|row| row.value(column_id).to_string())
        .unwrap_or_default()
}

fn set_row_cell_value(rows: &mut [DataGridRow], row_id: &str, column_id: &str, value: String) {
    if let Some(row) = rows.iter_mut().find(|row| row.id == row_id) {
        row.cells.insert(column_id.into(), value);
    }
}

/// Controlled maintenance table demonstrating display and interactive Dioxus cell composition.
#[component]
pub fn GridCellRenderersExample() -> Element {
    let mut rows = use_signal(maintenance_cell_rows);
    let mut state = use_signal(DataGridState::default);
    let mut read_only = use_signal(|| false);
    let mut inspections = use_signal(|| 0usize);
    let mut last = use_signal(|| "none".to_string());
    let mut edits = use_signal(|| 0usize);
    let mut epoch = use_signal(|| 0u64);

    let current_rows = rows();
    let current_state = state();
    let current_inspections = inspections();
    let current_last = last();
    let current_edits = edits();
    let note1 = row_cell_value(&current_rows, "asset-1", "note");
    let status1 = row_cell_value(&current_rows, "asset-1", "status");
    let selected_count = current_state.selected.len();
    let visible_count = current_state.visible_rows(&current_rows).len();
    let service_only = current_state.filters.get("status").is_some_and(|filter| {
        filter.first.operator == DataGridFilterOperator::Equals
            && filter.first.value.eq_ignore_ascii_case("service")
            && !filter.second.is_active()
    });
    let reverse_order = matches!(
        &current_state.sort,
        Some((column, DataGridSortDirection::Descending)) if column == "asset"
    );

    rsx! {
        section { class: "grid-cell-renderers-example", id: "grid-cell-renderers",
            h2 { "Composed maintenance cells" }
            p {
                "Three illustrative maintenance assets kept in local component state with no remote service. "
                "Focus a cell and press F2/Enter to edit, or double-click its text. Tab from a Note cell reaches the inline input; notes update as you type, and Escape returns to the cell. Actions opens a menu for inspecting or updating an asset. Valve 003's note is read-only."
            }
            div { class: "grid-cell-renderers-toolbar",
                Button {
                    id: "composed-reset",
                    variant: ButtonVariant::Secondary,
                    onclick: move |_| {
                        rows.set(maintenance_cell_rows());
                        state.set(DataGridState::default());
                        read_only.set(false);
                        inspections.set(0);
                        last.set("none".into());
                        edits.set(0);
                        let next_epoch = epoch.peek().wrapping_add(1);
                        epoch.set(next_epoch);
                    },
                    "Restore assets"
                }
                label { r#for: "composed-readonly",
                    input {
                        id: "composed-readonly",
                        r#type: "checkbox",
                        checked: read_only(),
                        onchange: move |event: FormEvent| read_only.set(event.checked()),
                    }
                    "Read-only cells"
                }
                label { r#for: "composed-service-filter",
                    input {
                        id: "composed-service-filter",
                        r#type: "checkbox",
                        checked: service_only,
                        onchange: move |event: FormEvent| {
                            let mut next = state.peek().clone();
                            if event.checked() {
                                next.filters.insert(
                                    "status".into(),
                                    DataGridTextFilter {
                                        first: DataGridFilterCondition {
                                            operator: DataGridFilterOperator::Equals,
                                            value: "service".into(),
                                        },
                                        second: DataGridFilterCondition::default(),
                                        match_any: false,
                                    },
                                );
                            } else {
                                next.filters.remove("status");
                            }
                            state.set(next);
                        },
                    }
                    "Show service only"
                }
                label { r#for: "composed-sort",
                    input {
                        id: "composed-sort",
                        r#type: "checkbox",
                        checked: reverse_order,
                        onchange: move |event: FormEvent| {
                            let mut next = state.peek().clone();
                            next.sort = if event.checked() {
                                Some(("asset".into(), DataGridSortDirection::Descending))
                            } else {
                                None
                            };
                            state.set(next);
                        },
                    }
                    "Reverse asset order"
                }
            }
            // Dioxus reconciles keys on fragment children; keep this lifecycle explicit.
            for generation in [epoch()] {
                DataGrid {
                    key: "composed-cells-grid-{generation}",
                    class: "composed-cells-grid",
                    columns: maintenance_cell_columns(),
                    rows: current_rows.clone(),
                    state: current_state.clone(),
                    onstatechange: move |next| state.set(next),
                    height: 240,
                    row_height: 48,
                    striped: true,
                    read_only: read_only(),
                    label: "Composed maintenance cells",
                    render_cell: move |ctx: DataGridCellContext| match ctx.column.id.as_str() {
                        "asset" => None,
                        "status" => {
                            let is_service = ctx.value == "service";
                            let variant = if is_service {
                                StatusVariant::Warning
                            } else {
                                StatusVariant::Success
                            };
                            let label = if is_service { "Needs service" } else { "Ready" };
                            let selected = ctx.selected;
                            Some(DataGridCellRender::display(rsx! {
                                div { class: "composed-status-cell", "data-row-index": ctx.row_index.to_string(),
                                    Pill { variant, "{label}" }
                                    if selected {
                                        span { class: "composed-status-selected", "Selected" }
                                    }
                                }
                            }))
                        }
                        "note" => {
                            let row_id = ctx.row.id.clone();
                                                        let input_id = format!("composed-note-{row_id}");
                            let aria_label = format!("Note for {row_id}");
                            let value = ctx.value.clone();
                            let editable = ctx.editable;
                            let mut input_rows = rows;
                            Some(DataGridCellRender::interactive(rsx! {
                                input {
                                    id: input_id,
                                    class: "composed-note-input",
                                    r#type: "text",
                                    value,
                                    readonly: !editable,
                                    "aria-label": aria_label,
                                    oninput: move |event: FormEvent| {
                                        if !editable {
                                            return;
                                        }
                                        let mut next = input_rows.peek().clone();
                                        if next
                                            .iter()
                                            .find(|row| row.id == row_id)
                                            .is_some_and(|row| !row.read_only_cells.contains("note"))
                                        {
                                            set_row_cell_value(&mut next, &row_id, "note", event.value());
                                            input_rows.set(next);
                                        }
                                    },
                                }
                            }))
                        }
                        "actions" => {
                            let row_id = ctx.row.id.clone();
                            let inspect_row_id = row_id.clone();
                            let service_row_id = row_id.clone();
                            let clear_row_id = row_id.clone();
                            let dropdown_label = format!("Actions for {row_id}");
                            let grid_read_only = ctx.grid_read_only;
                            let note_read_only =
                                grid_read_only || ctx.row.read_only_cells.contains("note");
                            let mut inspect_count = inspections;
                            let mut inspect_last = last;
                            let mut service_rows = rows;
                            let mut clear_rows = rows;
                            Some(DataGridCellRender::interactive(rsx! {
                                div { class: "composed-actions-cell",
                                    Dropdown {
                                        label: dropdown_label,
                                        navigation_mode: DropdownNavigationMode::RovingTabindex,
                                        enable_top_layer: true,
                                        DropdownItem {
                                            id: format!("composed-inspect-{row_id}"),
                                            label: "Inspect",
                                            onitemclick: move |_| {
                                                let next = inspect_count.peek().saturating_add(1);
                                                inspect_count.set(next);
                                                inspect_last.set(inspect_row_id.clone());
                                            },
                                        }
                                        DropdownItem {
                                            id: format!("composed-service-{row_id}"),
                                            label: "Mark for service",
                                            disabled: grid_read_only,
                                            onitemclick: move |_| {
                                                if grid_read_only {
                                                    return;
                                                }
                                                let mut next = service_rows.peek().clone();
                                                set_row_cell_value(
                                                    &mut next,
                                                    &service_row_id,
                                                    "status",
                                                    "service".into(),
                                                );
                                                service_rows.set(next);
                                            },
                                        }
                                        DropdownItem {
                                            id: format!("composed-clear-note-{row_id}"),
                                            label: "Clear note",
                                            disabled: note_read_only,
                                            onitemclick: move |_| {
                                                if note_read_only {
                                                    return;
                                                }
                                                let mut next = clear_rows.peek().clone();
                                                if next
                                                    .iter()
                                                    .find(|row| row.id == clear_row_id)
                                                    .is_some_and(|row| !row.read_only_cells.contains("note"))
                                                {
                                                    set_row_cell_value(
                                                        &mut next,
                                                        &clear_row_id,
                                                        "note",
                                                        String::new(),
                                                    );
                                                    clear_rows.set(next);
                                                }
                                            },
                                        }
                                        DropdownItem {
                                            id: format!("composed-archive-{row_id}"),
                                            label: "Archive",
                                            disabled: true,
                                        }
                                    }
                                }
                            }))
                        }
                        _ => None,
                    },
                    oncellvaluechanged: move |change: DataGridCellValueChanged| {
                        let mut next = rows.peek().clone();
                        if change.apply_to(&mut next) {
                            let next_edits = edits.peek().saturating_add(1);
                            edits.set(next_edits);
                            rows.set(next);
                        }
                    },
                }
            }
            p {
                id: "composed-result",
                class: "grid-cell-renderers-result",
                role: "status",
                "data-inspections": current_inspections.to_string(),
                "data-last": current_last.clone(),
                "data-note1": note1.clone(),
                "data-status1": status1.clone(),
                "data-selected": selected_count.to_string(),
                "data-visible": visible_count.to_string(),
                "data-edits": current_edits.to_string(),
                "Inspections: {current_inspections} · Last: {current_last} · Motor 001: status {status1}, note \"{note1}\" · {selected_count} selected · {visible_count} visible · {current_edits} edits"
            }
        }
    }
}

pub const STYLES: &str = ".grid-cell-renderers-example{margin-bottom:48px;min-width:0;max-width:100%}.grid-cell-renderers-example>p{max-width:72ch;line-height:1.55;overflow-wrap:anywhere}.grid-cell-renderers-toolbar{display:flex;align-items:center;flex-wrap:wrap;gap:12px 16px;margin:20px 0 12px;min-width:0;max-width:100%}.grid-cell-renderers-toolbar>label{display:inline-flex;align-items:center;gap:8px;max-width:100%;font-size:14px;cursor:pointer}.grid-cell-renderers-toolbar input[type=\"checkbox\"]{width:16px;height:16px;margin:0;flex:none;accent-color:var(--ix-color-primary)}.grid-cell-renderers-toolbar input[type=\"checkbox\"]:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:3px}.composed-cells-grid{width:100%;min-width:0;max-width:100%}.composed-status-cell{display:flex;flex-wrap:wrap;align-items:center;gap:2px 8px;min-width:0;max-width:100%}.composed-status-selected{font-size:12px;color:var(--ix-color-text-secondary);white-space:nowrap}.composed-note-input{box-sizing:border-box;width:100%;min-width:0;height:32px;padding:4px 8px;border:1px solid var(--ix-color-border);border-radius:var(--ix-radius);background:var(--ix-color-surface-1);color:var(--ix-color-text);font:inherit;caret-color:var(--ix-color-primary)}.composed-note-input[readonly]{background:var(--ix-color-surface);color:var(--ix-color-text-secondary)}.composed-note-input:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:1px}.composed-actions-cell{display:flex;align-items:center;min-width:0;max-width:100%}.grid-cell-renderers-result{min-height:1.5em;margin:12px 0 0;color:var(--ix-color-text-secondary);font-variant-numeric:tabular-nums;overflow-wrap:anywhere}@media(max-width:420px){.grid-cell-renderers-toolbar{align-items:flex-start;gap:10px 12px}.grid-cell-renderers-toolbar .ix-button{max-width:100%}}";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cell_renderers_example_renders_baseline_contract() {
        let mut dom = VirtualDom::new(GridCellRenderersExample);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert!(html.contains("id=\"grid-cell-renderers\""));
        assert!(html.contains("aria-label=\"Composed maintenance cells\""));
        assert!(html.contains("id=\"composed-reset\""));
        assert!(html.contains("id=\"composed-readonly\""));
        assert!(html.contains("id=\"composed-service-filter\""));
        assert!(html.contains("id=\"composed-sort\""));
        assert!(html.contains("id=\"composed-result\""));
        assert!(html.contains("id=\"composed-note-asset-1\""));
        assert!(html.contains("id=\"composed-note-asset-2\""));
        assert!(html.contains("id=\"composed-note-asset-3\""));
        assert!(html.contains("aria-label=\"Note for asset-1\""));
        assert!(html.contains("Actions for asset-1"));
        assert!(html.contains("data-inspections=\"0\""));
        assert!(html.contains("data-last=\"none\""));
        assert!(html.contains("data-note1=\"Inspect seal\""));
        assert!(html.contains("data-status1=\"ready\""));
        assert!(html.contains("data-selected=\"0\""));
        assert!(html.contains("data-visible=\"3\""));
        assert!(html.contains("data-edits=\"0\""));
    }
}
