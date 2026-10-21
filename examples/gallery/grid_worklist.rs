//! Controlled maintenance worklist combining sorting, filtering, selection, and cell editing.
use dioxus::prelude::*;
use dioxus_ix::{
    Button, ButtonVariant, DataGrid, DataGridCellValueChanged, DataGridColumn, DataGridCsv,
    DataGridCsvOptions, DataGridCsvRowScope, DataGridPagination, DataGridRow,
    DataGridSortDirection, DataGridState, DataGridTextEditor, DataGridTextFilter, Textarea,
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

fn csv_scope_token(scope: DataGridCsvRowScope) -> &'static str {
    match scope {
        DataGridCsvRowScope::Displayed => "displayed",
        DataGridCsvRowScope::SelectedDisplayed => "selected-displayed",
        DataGridCsvRowScope::SelectedAll => "selected-all",
        DataGridCsvRowScope::All => "all",
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
    let mut auto_page_size = use_signal(|| false);
    let mut grid_height = use_signal(|| 420_u32);
    let mut csv_scope = use_signal(|| DataGridCsvRowScope::Displayed);
    let mut csv_snapshot = use_signal(|| None::<(DataGridCsvRowScope, DataGridCsv)>);
    let mut csv_generation = use_signal(|| 0_u64);
    let mut csv_downloading = use_signal(|| false);
    let mut csv_task = use_signal(|| None::<dioxus::core::Task>);
    let mut csv_status = use_signal(|| "empty");
    let mut csv_feedback = use_signal(|| {
        String::from("Generate a snapshot to preview or download the current worklist.")
    });
    let clear_csv = use_callback(move |(): ()| {
        let next = csv_generation.peek().wrapping_add(1);
        csv_generation.set(next);
        let pending = csv_task.write().take();
        if let Some(task) = pending {
            task.cancel();
        }
        csv_downloading.set(false);
        csv_snapshot.set(None);
        csv_status.set("empty");
        csv_feedback.set("Generate a snapshot to preview or download the current worklist.".into());
    });

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
    let pagination_enabled = current_state.pagination.is_some();
    let effective_grid_height = if pagination_enabled {
        grid_height()
    } else {
        300
    };
    let page_info = current_state.pagination_info(visible_count);
    let displayed_page = if page_info.page_count == 0 {
        0
    } else {
        page_info.page + 1
    };
    let displayed_start = if page_info.total_rows == 0 {
        0
    } else {
        page_info.range.start + 1
    };
    let snapshot = csv_snapshot();
    let snapshot_scope = snapshot
        .as_ref()
        .map(|(scope, _)| csv_scope_token(*scope))
        .unwrap_or("none");
    let snapshot_rows = snapshot
        .as_ref()
        .map(|(_, snapshot)| snapshot.row_ids.len())
        .unwrap_or(0);
    let snapshot_columns = snapshot
        .as_ref()
        .map(|(_, snapshot)| snapshot.column_ids.len())
        .unwrap_or(0);
    let snapshot_content = snapshot
        .as_ref()
        .map(|(_, snapshot)| snapshot.content.clone())
        .unwrap_or_default();
    let snapshot_row_ids = serde_json::to_string(
        snapshot
            .as_ref()
            .map(|(_, snapshot)| snapshot.row_ids.as_slice())
            .unwrap_or(&[]),
    )
    .unwrap_or_default();
    let snapshot_column_ids = serde_json::to_string(
        snapshot
            .as_ref()
            .map(|(_, snapshot)| snapshot.column_ids.as_slice())
            .unwrap_or(&[]),
    )
    .unwrap_or_default();

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
                        if let Some(pagination) = next.pagination.as_mut() { pagination.page = 0; }
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
                        auto_page_size.set(false);
                        grid_height.set(420);
                        clear_csv.call(());
                        csv_scope.set(DataGridCsvRowScope::Displayed);
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
                label { r#for: "worklist-pagination",
                    input {
                        id: "worklist-pagination", r#type: "checkbox", checked: pagination_enabled,
                        onchange: move |event: FormEvent| {
                            let mut next = state.peek().clone();
                            next.pagination = event.checked().then(|| DataGridPagination::new(0, 12));
                            state.set(next);
                        },
                    }
                    "Paginate worklist"
                }
                label { r#for: "worklist-auto-page-size",
                    input {
                        id: "worklist-auto-page-size", r#type: "checkbox", checked: auto_page_size(), disabled: !pagination_enabled,
                        onchange: move |event: FormEvent| auto_page_size.set(event.checked()),
                    }
                    "Automatic rows per page"
                }
                if pagination_enabled && auto_page_size() {
                    label { r#for: "worklist-grid-height", "View height"
                        select { id: "worklist-grid-height", class: "ix-input", value: grid_height().to_string(),
                            onchange: move |event: FormEvent| grid_height.set(match event.value().as_str() {
                                "300" => 300,
                                "560" => 560,
                                _ => 420,
                            }),
                            option { value: "300", "Compact view" }
                            option { value: "420", "Standard view" }
                            option { value: "560", "Tall view" }
                        }
                    }
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
                    height: effective_grid_height,
                    auto_page_size: auto_page_size(),
                    page_size_options: vec![12, 24, 48],
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
            p { id: "worklist-pagination-status", class: "grid-worklist-status", role: "status", hidden: !pagination_enabled,
                "data-enabled": pagination_enabled, "data-page": page_info.page, "data-page-size": page_info.page_size,
                "data-auto-page-size": pagination_enabled && auto_page_size(), "data-grid-height": effective_grid_height,
                "data-page-count": page_info.page_count, "data-total": page_info.total_rows,
                "data-start": page_info.range.start, "data-end": page_info.range.end,
                "Page {displayed_page} of {page_info.page_count} · {displayed_start}–{page_info.range.end} of {page_info.total_rows} matching records. Selection and CSV include matching records across all pages. Save or cancel a cell edit before paging."
            }
            section { id: "worklist-csv", class: "grid-worklist-csv",
                h3 { "Export worklist" }
                p { "Save any cell edits first. The CSV snapshot stays unchanged until you generate it again." }
                div { class: "grid-worklist-toolbar",
                    label { r#for: "worklist-csv-scope", "Rows"
                        select { id: "worklist-csv-scope", class: "ix-input", value: csv_scope_token(csv_scope()),
                            onchange: move |event: FormEvent| csv_scope.set(match event.value().as_str() {
                                "selected-displayed" => DataGridCsvRowScope::SelectedDisplayed,
                                "selected-all" => DataGridCsvRowScope::SelectedAll,
                                "all" => DataGridCsvRowScope::All,
                                _ => DataGridCsvRowScope::Displayed,
                            }),
                            option { value: "displayed", "Displayed rows" }
                            option { value: "selected-displayed", "Selected displayed rows" }
                            option { value: "selected-all", "All selected rows" }
                            option { value: "all", "All rows" }
                        }
                    }
                    Button { id: "worklist-csv-generate", disabled: csv_downloading(),
                        onclick: move |_| {
                            if *csv_downloading.peek() { return; }
                            let scope = *csv_scope.peek();
                            let snapshot = state.peek().export_csv(&rows.peek(), &worklist_columns(), &DataGridCsvOptions { row_scope: scope, ..Default::default() });
                            let next = csv_generation.peek().wrapping_add(1);
                            csv_generation.set(next);
                            csv_feedback.set(format!("Snapshot: {} rows, {} columns. Ready to copy or download.", snapshot.row_ids.len(), snapshot.column_ids.len()));
                            csv_snapshot.set(Some((scope, snapshot)));
                            csv_status.set("ready");
                        },
                        "Generate CSV"
                    }
                    Button { id: "worklist-csv-download", variant: ButtonVariant::Secondary,
                        disabled: snapshot.is_none() || csv_downloading(),
                        onclick: move |_| {
                            if *csv_downloading.peek() { return; }
                            let Some((_, snapshot)) = csv_snapshot.peek().clone() else { return; };
                            let generation = *csv_generation.peek();
                            csv_downloading.set(true); csv_status.set("downloading");
                            csv_feedback.set("Requesting the CSV snapshot download…".into());
                            let task = spawn(async move {
                                let result = snapshot.download("maintenance-worklist.csv").await;
                                if *csv_generation.peek() != generation { return; }
                                csv_task.set(None); csv_downloading.set(false);
                                match result {
                                    Ok(()) => { csv_status.set("dispatched"); csv_feedback.set("CSV download requested.".into()); }
                                    Err(error) => { csv_status.set("failed"); csv_feedback.set(format!("Could not request the CSV download: {error}")); }
                                }
                            });
                            csv_task.set(Some(task));
                        },
                        "Download snapshot"
                    }
                    Button { id: "worklist-csv-clear", variant: ButtonVariant::Tertiary,
                        disabled: snapshot.is_none() && !csv_downloading(),
                        onclick: move |_| clear_csv.call(()), "Clear snapshot"
                    }
                }
                Textarea { id: "worklist-csv-preview", label: "CSV preview", value: snapshot_content,
                    readonly: true, rows: 6, resize: "vertical", width: "100%", class: "grid-worklist-csv-preview",
                    placeholder: "Generate a snapshot to preview its CSV.",
                }
                p { id: "worklist-csv-status", class: "grid-worklist-status", role: "status", "aria-live": "polite",
                    "data-state": csv_status(), "data-scope": snapshot_scope,
                    "data-rows": snapshot_rows, "data-columns": snapshot_columns,
                    "data-row-ids": snapshot_row_ids, "data-column-ids": snapshot_column_ids,
                    "data-generation": csv_generation(), "data-downloading": csv_downloading(),
                    "{csv_feedback}"
                }
            }
        }
    }
}

pub const STYLES: &str = include_str!("styles/grid_worklist.css");

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
