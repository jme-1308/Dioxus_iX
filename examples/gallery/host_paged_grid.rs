//! An application-owned page provider over a complete local equipment catalog.
use dioxus::prelude::*;
use dioxus_ix::*;

pub const STYLES: &str = include_str!("styles/host_paged_grid.css");

fn equipment_rows() -> Vec<DataGridRow> {
    (0..137)
        .map(|index| {
            DataGridRow::new(format!("host-equipment-{index:03}"))
                .cell("asset", format!("Equipment {index:03}"))
                .cell(
                    "status",
                    match index % 3 {
                        0 => "Normal",
                        1 => "Warning",
                        _ => "Maintenance",
                    },
                )
                .cell("owner", format!("Team {}", index % 4 + 1))
        })
        .collect()
}

fn equipment_columns() -> Vec<DataGridColumn> {
    let mut asset = DataGridColumn::new("asset", "Equipment");
    asset.min_width = 180;
    asset.sortable = true;
    asset.filterable = true;
    asset.editable = true;
    asset.editor = DataGridTextEditor {
        required: true,
        max_length: Some(64),
    };
    let mut status = DataGridColumn::new("status", "Status");
    status.min_width = 140;
    status.sortable = true;
    status.filterable = true;
    let mut owner = DataGridColumn::new("owner", "Owner");
    owner.min_width = 160;
    owner.sortable = true;
    owner.filterable = true;
    owner.editable = true;
    owner.editor = DataGridTextEditor {
        required: false,
        max_length: Some(32),
    };
    vec![asset, status, owner]
}

#[derive(Clone)]
struct AcceptedPage {
    state: DataGridState,
    rows: Vec<DataGridRow>,
    total: usize,
}

/// One provider transaction: project the complete source, then accept its page
/// state, page records and matching total together.
fn accept_page(source: &[DataGridRow], mut requested: DataGridState) -> AcceptedPage {
    requested
        .pagination
        .get_or_insert(DataGridPagination::new(0, 12));
    let columns = equipment_columns();
    let visible = requested.visible_rows_with_columns(source, &columns);
    let total = visible.len();
    let page = requested.pagination_info(total);
    let rows = visible[page.range.clone()]
        .iter()
        .map(|row| (*row).clone())
        .collect();
    requested.pagination = Some(DataGridPagination::new(page.page, page.page_size));
    AcceptedPage {
        state: requested,
        rows,
        total,
    }
}

#[derive(Clone)]
struct EquipmentProvider {
    source: Vec<DataGridRow>,
    accepted: AcceptedPage,
    saved: usize,
    feedback: String,
}

impl EquipmentProvider {
    fn new() -> Self {
        let source = equipment_rows();
        let state = DataGridState {
            pagination: Some(DataGridPagination::new(0, 12)),
            ..Default::default()
        };
        let accepted = accept_page(&source, state);
        Self {
            source,
            accepted,
            saved: 0,
            feedback: "Choose equipment across pages, or edit its name or owner.".into(),
        }
    }

    fn accept(&mut self, requested: DataGridState) {
        self.accepted = accept_page(&self.source, requested);
    }

    fn mark_selected(&mut self) {
        let mut changed = 0;
        for row in &mut self.source {
            if self.accepted.state.selected.contains(&row.id) {
                row.cells.insert("status".into(), "Maintenance".into());
                changed += 1;
            }
        }
        self.accept(self.accepted.state.clone());
        self.feedback = format!(
            "Marked {changed} selected equipment record(s) for Maintenance, including selections on other pages."
        );
    }

    fn save_edit(&mut self, change: DataGridCellValueChanged) {
        if change.apply_to(&mut self.source) {
            self.saved += 1;
            self.accept(self.accepted.state.clone());
            self.feedback = format!(
                "Saved {} for {}: {}",
                change.column_id, change.row_id, change.new_value
            );
        } else {
            self.feedback = "The edit could not be applied because its original value no longer matches the source.".into();
        }
    }
}

#[component]
fn HostPageWorkspace(generation: usize) -> Element {
    let mut provider = use_signal(EquipmentProvider::new);
    let current = provider();
    let page = current
        .accepted
        .state
        .pagination_info(current.accepted.total);
    let selected: Vec<_> = current
        .source
        .iter()
        .filter(|row| current.accepted.state.selected.contains(&row.id))
        .collect();
    let selected_ids: Vec<_> = selected.iter().map(|row| row.id.as_str()).collect();
    let serialized_selection = serde_json::to_string(&selected_ids).unwrap_or_default();
    let mut selected_summary = selected
        .iter()
        .take(5)
        .map(|row| row.value("asset"))
        .collect::<Vec<_>>()
        .join(", ");
    if selected.len() > 5 {
        selected_summary.push_str(&format!(", and {} more", selected.len() - 5));
    }
    if selected.is_empty() {
        selected_summary = "No equipment selected.".into();
    }
    let has_filters = current
        .accepted
        .state
        .filters
        .values()
        .any(DataGridTextFilter::is_active);
    let first = current
        .accepted
        .rows
        .first()
        .map(|row| row.id.as_str())
        .unwrap_or("none");
    let last = current
        .accepted
        .rows
        .last()
        .map(|row| row.id.as_str())
        .unwrap_or("none");
    let display_start = if current.accepted.total == 0 {
        0
    } else {
        page.range.start + 1
    };
    rsx! {
        div { id: "host-paged-owner", "data-generation": generation,
            div { class: "host-paged-actions",
                Button { id: "host-paged-mark-service", button_type: ButtonType::Button, disabled: selected.is_empty(),
                    onclick: move |_| { let mut next = provider.peek().clone(); next.mark_selected(); provider.set(next); },
                    "Mark selected for maintenance"
                }
                Button { id: "host-paged-clear-selection", variant: ButtonVariant::Secondary, button_type: ButtonType::Button,
                    disabled: selected.is_empty(),
                    onclick: move |_| {
                        let mut next = provider.peek().clone();
                        let mut query = next.accepted.state.clone();
                        query.selected.clear(); query.selection_anchor = None;
                        next.accept(query); next.feedback = "Cleared selection across all pages.".into(); provider.set(next);
                    }, "Clear selection"
                }
                Button { id: "host-paged-clear-filters", variant: ButtonVariant::Secondary, button_type: ButtonType::Button,
                    disabled: !has_filters,
                    onclick: move |_| {
                        let mut next = provider.peek().clone();
                        let mut query = next.accepted.state.clone();
                        query.filters.clear();
                        if let Some(pagination) = query.pagination.as_mut() { pagination.page = 0; }
                        next.accept(query); next.feedback = "Cleared filters and returned to the first page.".into(); provider.set(next);
                    }, "Clear filters"
                }
            }
            DataGrid { class: "host-paged-equipment-grid", label: "Host-paged equipment",
                columns: equipment_columns(), rows: current.accepted.rows.clone(), state: current.accepted.state.clone(),
                row_model: DataGridRowModel::Page { total_rows: current.accepted.total },
                height: 420, striped: true, page_size_options: vec![12, 24, 48],
                onstatechange: move |requested: DataGridState| {
                    let mut next = provider.peek().clone(); next.accept(requested); provider.set(next);
                },
                oncellvaluechanged: move |change: DataGridCellValueChanged| {
                    let mut next = provider.peek().clone(); next.save_edit(change); provider.set(next);
                },
            }
            p { id: "host-paged-status", class: "host-paged-status", role: "status", "aria-live": "polite",
                "data-page": page.page, "data-page-size": page.page_size, "data-page-count": page.page_count,
                "data-total": current.accepted.total, "data-loaded": current.accepted.rows.len(), "data-source-total": current.source.len(),
                "data-start": page.range.start, "data-end": page.range.end, "data-first-row": first, "data-last-row": last,
                "data-selected": selected.len(), "data-selected-ids": serialized_selection.clone(), "data-saved": current.saved,
                "Loaded records {display_start}–{page.range.end} of {current.accepted.total} matching equipment · {selected.len()} selected · {current.saved} saved edits"
            }
            p { id: "host-paged-selection", class: "host-paged-selection", "data-selected-ids": serialized_selection,
                "{selected_summary}"
            }
            p { id: "host-paged-feedback", class: "host-paged-feedback", "{current.feedback}" }
        }
    }
}

#[component]
pub fn HostPagedGridExample() -> Element {
    let mut generation = use_signal(|| 0_usize);
    rsx! {
        style { {STYLES} }
        section { id: "host-paged-grid", class: "demo-section host-paged-grid-example",
            h2 { "Equipment supplied one page at a time" }
            p { class: "host-paged-intro", "The page provider filters and sorts all 137 local records, then supplies only the accepted page and matching total to the grid. An application can use the same state request to fetch its data; this example makes no network requests." }
            p { class: "host-paged-intro", "Select equipment on different pages, then mark the full selection for maintenance. A page's header checkbox selects only its loaded records. Double-click Equipment or Owner to edit; save with Enter or cancel with Escape before paging." }
            for owner in [generation()] { HostPageWorkspace { key: "{owner}", generation: owner } }
            Button { id: "host-paged-reset", variant: ButtonVariant::Tertiary, button_type: ButtonType::Button,
                onclick: move |_| generation += 1, "Reset equipment provider"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_filters_and_sorts_full_source_before_accepting_partial_page() {
        let source = equipment_rows();
        let mut query = DataGridState {
            pagination: Some(DataGridPagination::new(1, 24)),
            ..Default::default()
        };
        query.sort = Some(("asset".into(), DataGridSortDirection::Descending));
        query.filters.insert(
            "status".into(),
            DataGridTextFilter {
                first: DataGridFilterCondition {
                    operator: DataGridFilterOperator::Equals,
                    value: "Warning".into(),
                },
                ..Default::default()
            },
        );
        let accepted = accept_page(&source, query);
        assert_eq!(accepted.total, 46);
        assert_eq!(accepted.rows.len(), 22);
        assert_eq!(accepted.rows.first().unwrap().id, "host-equipment-064");
        assert_eq!(accepted.rows.last().unwrap().id, "host-equipment-001");
        assert_eq!(accepted.state.pagination.unwrap().page, 1);
        let empty = accept_page(&[], accepted.state);
        assert_eq!(empty.total, 0);
        assert!(empty.rows.is_empty());
        assert_eq!(empty.state.pagination.unwrap().page, 0);
    }

    #[test]
    fn source_selection_and_committed_edits_survive_page_changes() {
        let mut provider = EquipmentProvider::new();
        let mut query = provider.accepted.state.clone();
        query
            .selected
            .extend(["host-equipment-000".into(), "host-equipment-012".into()]);
        query.pagination.as_mut().unwrap().page = 1;
        provider.accept(query);
        provider.mark_selected();
        assert_eq!(provider.source[0].value("status"), "Maintenance");
        assert_eq!(provider.source[12].value("status"), "Maintenance");
        assert_eq!(provider.source[1].value("status"), "Warning");
        assert_eq!(provider.accepted.state.selected.len(), 2);
        let previous_page = provider.accepted.rows.clone();
        let edit = DataGridCellValueChanged {
            row_id: "host-equipment-012".into(),
            column_id: "owner".into(),
            old_value: "Team 1".into(),
            new_value: "On-call team".into(),
        };
        provider.save_edit(edit.clone());
        assert_eq!(provider.saved, 1);
        assert_eq!(provider.accepted.rows[0].value("owner"), "On-call team");
        assert_eq!(previous_page[0].value("owner"), "Team 1");
        provider.save_edit(edit);
        assert_eq!(provider.saved, 1);
        let mut query = provider.accepted.state.clone();
        query.pagination.as_mut().unwrap().page = 0;
        provider.accept(query);
        assert_eq!(provider.accepted.rows[0].value("status"), "Maintenance");
        assert!(
            provider
                .accepted
                .state
                .selected
                .contains("host-equipment-012")
        );
    }
}
