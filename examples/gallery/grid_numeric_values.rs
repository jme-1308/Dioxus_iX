//! Controlled stock-planning table combining numeric sorting, numeric filtering, and number cell editors.
use dioxus::prelude::*;
use dioxus_ix::{
    Button, ButtonVariant, DataGrid, DataGridCellEditor, DataGridCellValueChanged, DataGridColumn,
    DataGridFilterCondition, DataGridFilterOperator, DataGridNumberEditor, DataGridRow,
    DataGridSortDirection, DataGridState, DataGridTextEditor, DataGridTextFilter,
    DataGridValueType,
};

fn stock_planning_rows() -> Vec<DataGridRow> {
    let stock_1 = DataGridRow::new("stock-1")
        .cell("asset", "Motor kit")
        .cell("units", "2.5")
        .cell("cost", "1200");

    let stock_2 = DataGridRow::new("stock-2")
        .cell("asset", "Valve kit")
        .cell("units", "10")
        .cell("cost", "75");

    let stock_3 = DataGridRow::new("stock-3")
        .cell("asset", "Sensor kit")
        .cell("units", "2")
        .cell("cost", "150");

    let stock_4 = DataGridRow::new("stock-4")
        .cell("asset", "Seal pack")
        .cell("units", "0")
        .cell("cost", "0.5");

    let stock_5 = DataGridRow::new("stock-5")
        .cell("asset", "Pump kit")
        .cell("units", "100")
        .cell("cost", "2500");

    let mut stock_6 = DataGridRow::new("stock-6")
        .cell("asset", "Uncounted")
        .cell("units", "")
        .cell("cost", "");
    stock_6.read_only_cells.insert("units".into());
    stock_6.read_only_cells.insert("cost".into());

    let mut stock_7 = DataGridRow::new("stock-7")
        .cell("asset", "Pending count")
        .cell("units", "pending")
        .cell("cost", "unknown");
    stock_7.read_only_cells.insert("units".into());
    stock_7.read_only_cells.insert("cost".into());

    vec![
        stock_1, stock_2, stock_3, stock_4, stock_5, stock_6, stock_7,
    ]
}

fn stock_planning_columns() -> Vec<DataGridColumn> {
    let mut asset = DataGridColumn::new("asset", "Asset");
    asset.min_width = 180;
    asset.value_type = DataGridValueType::Text;
    asset.sortable = true;
    asset.filterable = true;

    let mut units = DataGridColumn::new("units", "Units");
    units.min_width = 140;
    units.value_type = DataGridValueType::Number;
    units.sortable = true;
    units.filterable = true;
    units.editable = true;
    units.editor = DataGridTextEditor {
        required: true,
        max_length: None,
    };
    units.cell_editor = DataGridCellEditor::Number(DataGridNumberEditor {
        min: Some(0.0),
        max: Some(1000.0),
        step: Some(0.5),
    });

    let mut cost = DataGridColumn::new("cost", "Unit cost");
    cost.min_width = 160;
    cost.value_type = DataGridValueType::Number;
    cost.sortable = true;
    cost.filterable = true;
    cost.editable = true;
    cost.editor = DataGridTextEditor {
        required: true,
        max_length: None,
    };
    cost.cell_editor = DataGridCellEditor::Number(DataGridNumberEditor {
        min: Some(0.0),
        max: None,
        step: Some(0.01),
    });

    vec![asset, units, cost]
}

fn row_cell_value(rows: &[DataGridRow], row_id: &str, column_id: &str) -> String {
    rows.iter()
        .find(|row| row.id == row_id)
        .map(|row| row.value(column_id).to_string())
        .unwrap_or_default()
}

fn sort_summary(state: &DataGridState) -> String {
    match &state.sort {
        None => "none".to_string(),
        Some((column, DataGridSortDirection::Ascending)) if column == "units" => "asc".to_string(),
        Some((column, DataGridSortDirection::Descending)) if column == "units" => {
            "desc".to_string()
        }
        Some((column, DataGridSortDirection::Ascending)) => format!("{column}:asc"),
        Some((column, DataGridSortDirection::Descending)) => format!("{column}:desc"),
    }
}

/// Controlled stock-planning table demonstrating numeric sorting, numeric range filtering, and number editing.
#[component]
pub fn GridNumericValuesExample() -> Element {
    let mut rows = use_signal(stock_planning_rows);
    let mut state = use_signal(DataGridState::default);
    let mut read_only = use_signal(|| false);
    let mut edits = use_signal(|| 0usize);
    let mut epoch = use_signal(|| 0u64);

    let columns = stock_planning_columns();
    let current_rows = rows();
    let current_state = state();
    let current_edits = edits();
    let visible = current_state.visible_rows_with_columns(&current_rows, &columns);
    let visible_count = visible.len();
    let visible_order = visible
        .iter()
        .map(|row| row.id.as_str())
        .collect::<Vec<_>>()
        .join(",");
    let selected_count = current_state.selected.len();
    let qty1 = row_cell_value(&current_rows, "stock-1", "units");
    let current_sort = sort_summary(&current_state);
    let has_active_filters = current_state
        .filters
        .values()
        .any(DataGridTextFilter::is_active);

    rsx! {
        section { class: "grid-numeric-values-example", id: "grid-numeric-values",
            h2 { "Stock planning" }
            p {
                "Illustrative stock data. Sort quantities and prices numerically, or combine two filters with AND/OR. "
                "Blank matches empty cells; blank and unknown values sort last."
            }
            p {
                "Double-click a number or press F2/Enter to edit. Enter saves, Escape cancels, and Tab/Shift+Tab moves between editable cells. "
                "Uncounted and Pending count are read-only."
            }
            div { class: "grid-numeric-values-toolbar",
                Button {
                    id: "numeric-reset",
                    variant: ButtonVariant::Secondary,
                    onclick: move |_| {
                        rows.set(stock_planning_rows());
                        state.set(DataGridState::default());
                        read_only.set(false);
                        edits.set(0);
                        let next_epoch = epoch.peek().wrapping_add(1);
                        epoch.set(next_epoch);
                    },
                    "Restore stock"
                }
                Button {
                    id: "numeric-low-stock",
                    variant: ButtonVariant::Secondary,
                    onclick: move |_| {
                        let mut next = state.peek().clone();
                        next.filters.insert(
                            "units".into(),
                            DataGridTextFilter {
                                first: DataGridFilterCondition {
                                    operator: DataGridFilterOperator::LessThan,
                                    value: "10".into(),
                                },
                                second: DataGridFilterCondition::default(),
                                match_any: false,
                            },
                        );
                        state.set(next);
                    },
                    "Low stock (<10)"
                }
                Button {
                    id: "numeric-clear-filters",
                    variant: ButtonVariant::Secondary,
                    disabled: !has_active_filters,
                    onclick: move |_| {
                        let mut next = state.peek().clone();
                        next.filters.clear();
                        state.set(next);
                    },
                    "Clear filters"
                }
                label { r#for: "numeric-readonly",
                    input {
                        id: "numeric-readonly",
                        r#type: "checkbox",
                        checked: read_only(),
                        onchange: move |event: FormEvent| read_only.set(event.checked()),
                    }
                    "Read-only stock"
                }
            }
            // Dioxus reconciles keys on fragment children; keep this lifecycle explicit.
            for generation in [epoch()] {
                DataGrid {
                    key: "numeric-stock-grid-{generation}",
                    class: "numeric-stock-grid",
                    columns: columns.clone(),
                    rows: current_rows.clone(),
                    state: current_state.clone(),
                    onstatechange: move |next| state.set(next),
                    height: 300,
                    row_height: 42,
                    striped: true,
                    read_only: read_only(),
                    label: "Stock planning",
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
                id: "numeric-result",
                class: "grid-numeric-values-result",
                role: "status",
                "data-order": visible_order.clone(),
                "data-visible": visible_count.to_string(),
                "data-selected": selected_count.to_string(),
                "data-edits": current_edits.to_string(),
                "data-qty1": qty1.clone(),
                "data-sort": current_sort.clone(),
                "{visible_count} visible · {selected_count} selected · {current_edits} edits · Motor kit: {qty1} units"
            }
        }
    }
}

pub const STYLES: &str = ".grid-numeric-values-example{margin-bottom:48px;min-width:0;max-width:100%}.grid-numeric-values-example>p{max-width:72ch;line-height:1.55;overflow-wrap:anywhere}.grid-numeric-values-toolbar{display:flex;align-items:center;flex-wrap:wrap;gap:12px 16px;margin:20px 0 12px;min-width:0;max-width:100%}.grid-numeric-values-toolbar>label{display:inline-flex;align-items:center;gap:8px;max-width:100%;font-size:14px;cursor:pointer}.grid-numeric-values-toolbar input[type=\"checkbox\"]{width:16px;height:16px;margin:0;flex:none;accent-color:var(--ix-color-primary)}.grid-numeric-values-toolbar input[type=\"checkbox\"]:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:3px}.numeric-stock-grid{width:100%;min-width:0;max-width:100%}.grid-numeric-values-result{min-height:1.5em;margin:12px 0 0;color:var(--ix-color-text-secondary);font-variant-numeric:tabular-nums;overflow-wrap:anywhere}@media(max-width:420px){.grid-numeric-values-toolbar{align-items:flex-start;gap:10px 12px}.grid-numeric-values-toolbar .ix-button{max-width:100%}}";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numeric_values_example_renders_baseline_contract() {
        let mut dom = VirtualDom::new(GridNumericValuesExample);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert!(html.contains("id=\"grid-numeric-values\""));
        assert!(html.contains("aria-label=\"Stock planning\""));
        assert!(html.contains("id=\"numeric-reset\""));
        assert!(html.contains("id=\"numeric-low-stock\""));
        assert!(html.contains("id=\"numeric-clear-filters\""));
        assert!(html.contains("id=\"numeric-readonly\""));
        assert!(html.contains("id=\"numeric-result\""));
        assert!(
            html.contains("data-order=\"stock-1,stock-2,stock-3,stock-4,stock-5,stock-6,stock-7\"")
        );
        assert!(html.contains("data-visible=\"7\""));
        assert!(html.contains("data-selected=\"0\""));
        assert!(html.contains("data-edits=\"0\""));
        assert!(html.contains("data-qty1=\"2.5\""));
        assert!(html.contains("data-sort=\"none\""));
        assert!(html.contains("data-row-id=\"stock-1\""));
        assert!(html.contains("data-row-id=\"stock-6\""));
        assert!(html.contains("data-row-id=\"stock-7\""));
    }
}
