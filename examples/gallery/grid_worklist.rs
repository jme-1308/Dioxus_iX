//! Controlled maintenance worklist combining sorting, filtering, selection, and cell editing.
use dioxus::prelude::*;
use dioxus_ix::{
    Button, ButtonVariant, DataGrid, DataGridCellValueChanged, DataGridColumn, DataGridRow,
    DataGridSortDirection, DataGridState, DataGridTextEditor, DataGridTextFilter,
};
use std::collections::BTreeSet;

fn worklist_rows() -> Vec<DataGridRow> {
    (0..96)
        .map(|index| {
            let status = match index % 3 {
                0 => "Normal",
                1 => "Warning",
                _ => "Maintenance",
            };
            let owner = format!("Team {}", index % 4 + 1);
            let mut row = DataGridRow::new(format!("worklist-{index:03}"))
                .cell("asset", format!("Asset {index:03}"))
                .cell("status", status)
                .cell("owner", owner)
                .cell("ticket", format!("IX-{index:05}"));
            if index == 1 {
                row.read_only_cells.insert("owner".into());
            }
            row
        })
        .collect()
}

fn worklist_columns() -> Vec<DataGridColumn> {
    let mut asset = DataGridColumn::new("asset", "Asset");
    asset.min_width = 180;
    asset.editable = true;
    asset.sortable = true;
    asset.filterable = true;
    asset.editor = DataGridTextEditor {
        required: true,
        max_length: Some(40),
    };

    let mut status = DataGridColumn::new("status", "Status");
    status.min_width = 140;
    status.sortable = true;
    status.filterable = true;

    let mut owner = DataGridColumn::new("owner", "Owner");
    owner.min_width = 160;
    owner.editable = true;
    owner.sortable = true;
    owner.filterable = true;
    owner.editor = DataGridTextEditor {
        required: false,
        max_length: Some(24),
    };

    let mut ticket = DataGridColumn::new("ticket", "Ticket");
    ticket.min_width = 120;

    vec![asset, status, owner, ticket]
}

fn sort_token(state: &DataGridState) -> String {
    match &state.sort {
        Some((column, DataGridSortDirection::Ascending)) => format!("{column}-ascending"),
        Some((column, DataGridSortDirection::Descending)) => format!("{column}-descending"),
        None => "none".to_string(),
    }
}

fn mark_selected_for_service(rows: &mut [DataGridRow], state: &DataGridState) -> (usize, usize) {
    let visible_ids: BTreeSet<String> = state
        .visible_rows(rows)
        .into_iter()
        .map(|row| row.id.clone())
        .collect();
    let mut updated = 0usize;
    let mut hidden = 0usize;
    for row in rows {
        if state.selected.contains(&row.id) {
            row.cells.insert("status".into(), "Maintenance".into());
            updated += 1;
            if !visible_ids.contains(&row.id) {
                hidden += 1;
            }
        }
    }
    (updated, hidden)
}

enum WorklistEditOutcome {
    Held(String),
    Saved(String),
    Conflict(String),
}

fn apply_worklist_edit(
    rows: &mut [DataGridRow],
    change: &DataGridCellValueChanged,
    hold_edits: bool,
) -> WorklistEditOutcome {
    if hold_edits {
        return WorklistEditOutcome::Held(format!(
            "Held preview edit for {} {}: {} (stored value remains {}).",
            change.row_id, change.column_id, change.new_value, change.old_value
        ));
    }
    if change.apply_to(rows) {
        WorklistEditOutcome::Saved(format!(
            "Saved {} for {}: {}",
            change.column_id, change.row_id, change.new_value
        ))
    } else {
        WorklistEditOutcome::Conflict(format!(
            "Conflict saving {} for {}: cell is read-only, removed, or no longer matches {}.",
            change.column_id, change.row_id, change.old_value
        ))
    }
}

/// Controlled maintenance worklist combining sort, filter, stable-ID selection, and cell editing.
#[component]
pub fn GridWorklistExample() -> Element {
    let mut rows = use_signal(worklist_rows);
    let mut state = use_signal(DataGridState::default);
    let mut hold_edits = use_signal(|| false);
    let mut read_only = use_signal(|| false);
    let mut saved = use_signal(|| 0usize);
    let mut held = use_signal(|| 0usize);
    let mut last_row = use_signal(|| "none".to_string());
    let mut last_column = use_signal(|| "none".to_string());
    let mut feedback = use_signal(|| "No worklist actions or cell edits recorded.".to_string());
    let mut epoch = use_signal(|| 0u64);

    let current_rows = rows();
    let current_state = state();
    let total_count = current_rows.len();
    let visible = current_state.visible_rows(&current_rows);
    let visible_count = visible.len();
    let first_row = visible
        .first()
        .map(|row| row.id.clone())
        .unwrap_or_else(|| "none".to_string());
    let selected_count = current_state.selected.len();
    let has_active_filters = current_state
        .filters
        .values()
        .any(DataGridTextFilter::is_active);
    let current_sort = sort_token(&current_state);

    rsx! {
        section { class: "grid-worklist-example", id: "grid-worklist",
            h2 { "Maintenance worklist" }
            p { "96 illustrative maintenance records; actions update local component state only. Sort or filter columns, select records across filters, and mark all selected records (including selections hidden by active filters) for Maintenance. Double-click an editable cell or press Enter/F2 to edit; Enter saves, Escape cancels, and Tab/Shift+Tab moves to the next editable cell. Asset is required; Status, Ticket, and Asset 001's owner are read-only." }
            div { class: "grid-worklist-toolbar",
                Button {
                    id: "worklist-mark-service",
                    variant: ButtonVariant::Primary,
                    disabled: selected_count == 0,
                    onclick: move |_| {
                        let current = state.peek().clone();
                        if current.selected.is_empty() {
                            return;
                        }
                        let mut next = rows.peek().clone();
                        let (updated, hidden) = mark_selected_for_service(&mut next, &current);
                        rows.set(next);
                        feedback.set(if hidden > 0 {
                            format!(
                                "Marked {updated} selected record(s) for Maintenance ({hidden} hidden by filters)."
                            )
                        } else {
                            format!("Marked {updated} selected record(s) for Maintenance.")
                        });
                    },
                    "Mark selected for service"
                }
                Button {
                    id: "worklist-clear-selection",
                    variant: ButtonVariant::Secondary,
                    disabled: selected_count == 0,
                    onclick: move |_| {
                        let mut next = state.peek().clone();
                        let cleared = next.selected.len();
                        next.selected.clear();
                        next.selection_anchor = None;
                        state.set(next);
                        feedback.set(format!("Cleared {cleared} selected record(s)."));
                    },
                    "Clear selection"
                }
                Button {
                    id: "worklist-clear-filters",
                    variant: ButtonVariant::Secondary,
                    disabled: !has_active_filters,
                    onclick: move |_| {
                        let mut next = state.peek().clone();
                        next.filters.clear();
                        state.set(next);
                        feedback.set("Cleared active column filters.".into());
                    },
                    "Clear filters"
                }
                Button {
                    id: "worklist-restore",
                    variant: ButtonVariant::Secondary,
                    onclick: move |_| {
                        rows.set(worklist_rows());
                        state.set(DataGridState::default());
                        saved.set(0);
                        held.set(0);
                        last_row.set("none".into());
                        last_column.set("none".into());
                        hold_edits.set(false);
                        read_only.set(false);
                        let next_epoch = epoch.peek().wrapping_add(1);
                        epoch.set(next_epoch);
                        feedback.set("Worklist restored.".into());
                    },
                    "Restore worklist"
                }
                label { r#for: "worklist-hold-edits",
                    input {
                        id: "worklist-hold-edits",
                        r#type: "checkbox",
                        checked: hold_edits(),
                        onchange: move |event: FormEvent| hold_edits.set(event.checked()),
                    }
                    "Preview edits without saving"
                }
                label { r#for: "worklist-readonly",
                    input {
                        id: "worklist-readonly",
                        r#type: "checkbox",
                        checked: read_only(),
                        onchange: move |event: FormEvent| read_only.set(event.checked()),
                    }
                    "Read-only worklist"
                }
            }
            // Dioxus reconciles keys on fragment children; keep this lifecycle explicit.
            for generation in [epoch()] {
                DataGrid {
                    key: "worklist-grid-{generation}",
                    class: "maintenance-worklist-grid",
                    columns: worklist_columns(),
                    rows: current_rows.clone(),
                    state: current_state.clone(),
                    onstatechange: move |next| state.set(next),
                    height: 300,
                    striped: true,
                    read_only: read_only(),
                    label: "Maintenance worklist",
                    oncellvaluechanged: move |change: DataGridCellValueChanged| {
                        let mut next = rows.peek().clone();
                        match apply_worklist_edit(&mut next, &change, hold_edits()) {
                            WorklistEditOutcome::Held(message) => {
                                held += 1;
                                last_row.set(change.row_id);
                                last_column.set(change.column_id);
                                feedback.set(message);
                            }
                            WorklistEditOutcome::Saved(message) => {
                                saved += 1;
                                last_row.set(change.row_id);
                                last_column.set(change.column_id);
                                rows.set(next);
                                feedback.set(message);
                            }
                            WorklistEditOutcome::Conflict(message) => {
                                feedback.set(message);
                            }
                        }
                    },
                }
            }
            p {
                id: "worklist-status",
                class: "grid-worklist-status",
                role: "status",
                "data-total": total_count.to_string(),
                "data-visible": visible_count.to_string(),
                "data-selected": selected_count.to_string(),
                "data-saved": saved().to_string(),
                "data-held": held().to_string(),
                "data-last-row": last_row(),
                "data-last-column": last_column(),
                "data-first-row": first_row.clone(),
                "data-sort": current_sort.clone(),
                "{visible_count} of {total_count} visible · {selected_count} selected · {saved} saved · {held} held"
            }
            p {
                id: "worklist-feedback",
                class: "grid-worklist-feedback",
                "{feedback}"
            }
        }
    }
}

pub const STYLES: &str = ".grid-worklist-example{margin-bottom:48px;min-width:0;max-width:100%}.grid-worklist-example>p{max-width:72ch;line-height:1.55;overflow-wrap:anywhere}.grid-worklist-toolbar{display:flex;align-items:center;flex-wrap:wrap;gap:12px 16px;margin:20px 0 12px;min-width:0;max-width:100%}.grid-worklist-toolbar>label{display:inline-flex;align-items:center;gap:8px;max-width:100%;font-size:14px;cursor:pointer}.grid-worklist-toolbar input[type=\"checkbox\"]{width:16px;height:16px;margin:0;flex:none;accent-color:var(--ix-color-primary)}.grid-worklist-toolbar input[type=\"checkbox\"]:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:3px}.maintenance-worklist-grid{width:100%;min-width:0;max-width:100%}.grid-worklist-status,.grid-worklist-feedback{min-height:1.5em;margin:12px 0 0;color:var(--ix-color-text-secondary);font-variant-numeric:tabular-nums;overflow-wrap:anywhere}@media(max-width:420px){.grid-worklist-toolbar{align-items:flex-start;gap:10px 12px}.grid-worklist-toolbar .ix-button{max-width:100%}}";

#[cfg(test)]
mod tests {
    use super::*;
    use dioxus_ix::{DataGridFilterCondition, DataGridFilterOperator};

    #[test]
    fn worklist_renders_contract_controls_and_applies_bulk_and_cell_edits() {
        let mut records = worklist_rows();
        let mut state = DataGridState::default();
        state.select_row(&records, "worklist-000", true, false);
        state.select_row(&records, "worklist-001", true, false);
        state.filters.insert(
            "status".into(),
            DataGridTextFilter {
                first: DataGridFilterCondition {
                    operator: DataGridFilterOperator::Equals,
                    value: "Warning".into(),
                },
                second: DataGridFilterCondition::default(),
                match_any: false,
            },
        );
        assert_eq!(state.visible_rows(&records).len(), 32);
        let (updated, hidden) = mark_selected_for_service(&mut records, &state);
        assert_eq!((updated, hidden), (2, 1));
        assert_eq!(records[0].value("status"), "Maintenance");
        assert_eq!(records[1].value("status"), "Maintenance");

        let change = DataGridCellValueChanged {
            row_id: "worklist-000".into(),
            column_id: "asset".into(),
            old_value: "Asset 000".into(),
            new_value: "Primary feed pump".into(),
        };
        assert!(matches!(
            apply_worklist_edit(&mut records, &change, true),
            WorklistEditOutcome::Held(_)
        ));
        assert_eq!(records[0].value("asset"), "Asset 000");
        assert!(matches!(
            apply_worklist_edit(&mut records, &change, false),
            WorklistEditOutcome::Saved(_)
        ));
        assert_eq!(records[0].value("asset"), "Primary feed pump");
        assert!(matches!(
            apply_worklist_edit(&mut records, &change, false),
            WorklistEditOutcome::Conflict(_)
        ));

        let mut dom = VirtualDom::new(GridWorklistExample);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert!(html.contains("aria-label=\"Maintenance worklist\""));
        assert!(html.contains("data-visible=\"96\""));
        assert!(html.contains("data-selected=\"0\""));
        assert!(html.contains("data-row-id=\"worklist-000\""));
        assert!(!html.contains("data-row-id=\"worklist-095\""));
    }
}
