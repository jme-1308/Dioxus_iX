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
    asset.row_drag = true;
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
    groups: Vec<DataGridPageGroup>,
}

/// One provider transaction: project the complete source, then accept its page
/// state, page records, matching total and global group summaries together.
fn accept_page(source: &[DataGridRow], mut requested: DataGridState) -> AcceptedPage {
    requested
        .pagination
        .get_or_insert(DataGridPagination::new(0, 12));
    let columns = equipment_columns();
    let groups = requested.page_group_metadata(source, &columns);
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
        groups,
    }
}

#[derive(Clone)]
struct EquipmentProvider {
    source: Vec<DataGridRow>,
    accepted: AcceptedPage,
    saved: usize,
    moves: usize,
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
            moves: 0,
            feedback: "Choose equipment across pages, or edit its name or owner.".into(),
        }
    }

    fn accept(&mut self, requested: DataGridState) {
        self.accepted = accept_page(&self.source, requested);
    }

    fn group_by(&mut self, mode: &str) {
        let mut requested = self.accepted.state.clone();
        requested.grouping = match mode {
            "status" => DataGridGrouping::new(["status"]),
            "owner" => DataGridGrouping::new(["owner"]),
            "status-owner" => DataGridGrouping::new(["status", "owner"]),
            _ => DataGridGrouping::default(),
        };
        if let Some(page) = requested.pagination.as_mut() {
            page.page = 0;
        }
        self.accept(requested);
    }

    fn select_group(&mut self, request: DataGridPageGroupSelection) {
        let mut requested = self.accepted.state.clone();
        requested.select_group(
            &self.source,
            &equipment_columns(),
            &request.path,
            request.checked,
        );
        self.accept(requested);
        self.feedback = if request.checked {
            "Selected the group's matching equipment across all pages.".into()
        } else {
            "Cleared the group's matching selection across all pages.".into()
        };
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

    fn move_row(&mut self, request: DataGridPageRowMove) -> bool {
        let mut requested = self.accepted.state.clone();
        if !request.apply_to(&mut requested, &self.source) {
            self.feedback = "The move no longer matches the current equipment order. Try again from the current page.".into();
            return false;
        }
        // The provider owns the full ordering and follows the moved record,
        // including a request crossing an unloaded page boundary.
        let page_size = requested.pagination_info(self.source.len()).page_size;
        requested.pagination = Some(DataGridPagination::new(
            request.to_index / page_size,
            page_size,
        ));
        self.accept(requested);
        self.moves += 1;
        let asset = self
            .source
            .iter()
            .find(|row| row.id == request.row_id)
            .map(|row| row.value("asset"))
            .unwrap_or("Equipment");
        self.feedback = format!(
            "Moved {asset} to position {} of {}.",
            request.to_index + 1,
            self.source.len()
        );
        true
    }
}

#[component]
fn HostPageWorkspace(generation: usize) -> Element {
    let mut provider = use_signal(EquipmentProvider::new);
    let mut auto_page_size = use_signal(|| false);
    let mut tab_across_pages = use_signal(|| false);
    let mut navigate_across_pages = use_signal(|| false);
    let mut row_move = use_signal(|| false);
    let mut drag_across_pages = use_signal(|| false);
    let on_page_row_move = row_move().then(|| {
        EventHandler::new(move |request: DataGridPageRowMove| {
            let mut next = provider.peek().clone();
            next.move_row(request);
            provider.set(next);
        })
    });
    let current = provider();
    let group_mode = match current.accepted.state.grouping.columns.as_slice() {
        [status, owner] if status == "status" && owner == "owner" => "status-owner",
        [column] if column == "status" => "status",
        [column] if column == "owner" => "owner",
        _ => "none",
    };
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
    let ordered_ids: Vec<_> = if current.accepted.state.row_order.is_empty() {
        current.source.iter().map(|row| row.id.clone()).collect()
    } else {
        current.accepted.state.row_order.clone()
    };
    let serialized_order = serde_json::to_string(&ordered_ids).unwrap_or_default();
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
                label { r#for: "host-paged-grouping", "Group equipment by"
                    select { id: "host-paged-grouping", class: "ix-input", value: group_mode,
                        onchange: move |event: FormEvent| {
                            let mut next = provider.peek().clone(); next.group_by(&event.value()); provider.set(next);
                        },
                        option { value: "none", "None" }
                        option { value: "status", "Status" }
                        option { value: "owner", "Owner" }
                        option { value: "status-owner", "Status then Owner" }
                    }
                }
                label { r#for: "host-paged-navigate-across-pages",
                    input { id: "host-paged-navigate-across-pages", r#type: "checkbox", checked: navigate_across_pages(),
                        onchange: move |event: FormEvent| navigate_across_pages.set(event.checked()),
                    }
                    "Navigate across pages"
                }
                label { r#for: "host-paged-tab-across-pages",
                    input { id: "host-paged-tab-across-pages", r#type: "checkbox", checked: tab_across_pages(),
                        onchange: move |event: FormEvent| tab_across_pages.set(event.checked()),
                    }
                    "Continue editing across pages with Tab"
                }
                label { r#for: "host-paged-auto",
                    input { id: "host-paged-auto", r#type: "checkbox", checked: auto_page_size(),
                        onchange: move |event: FormEvent| auto_page_size.set(event.checked()),
                    }
                    "Fit equipment page to height"
                }
                label { r#for: "host-paged-row-move",
                    input { id: "host-paged-row-move", r#type: "checkbox", checked: row_move(),
                        onchange: move |event: FormEvent| row_move.set(event.checked()),
                    }
                    "Reorder equipment"
                }
                label { r#for: "host-paged-drag-across-pages",
                    input { id: "host-paged-drag-across-pages", r#type: "checkbox", checked: drag_across_pages(), disabled: !row_move(),
                        onchange: move |event: FormEvent| drag_across_pages.set(event.checked()),
                    }
                    "Drag across pages"
                }
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
                page_groups: current.accepted.groups.clone(),
                height: 420, striped: true, page_size_options: vec![12, 24, 48],
                auto_page_size: auto_page_size(),
                tab_across_pages: tab_across_pages(),
                    navigate_across_pages: navigate_across_pages(),
                onstatechange: move |requested: DataGridState| {
                    let mut next = provider.peek().clone(); next.accept(requested); provider.set(next);
                },
                oncellvaluechanged: move |change: DataGridCellValueChanged| {
                    let mut next = provider.peek().clone(); next.save_edit(change); provider.set(next);
                },
                onpagegroupselect: move |request: DataGridPageGroupSelection| {
                    let mut next = provider.peek().clone(); next.select_group(request); provider.set(next);
                },
                onpagerowmove: on_page_row_move,
                row_drag_across_pages: row_move() && drag_across_pages(),
            }
            p { id: "host-paged-status", class: "host-paged-status", role: "status", "aria-live": "polite",
                "data-page": page.page, "data-page-size": page.page_size, "data-page-count": page.page_count,
                "data-total": current.accepted.total, "data-loaded": current.accepted.rows.len(), "data-source-total": current.source.len(),
                "data-start": page.range.start, "data-end": page.range.end, "data-first-row": first, "data-last-row": last,
                "data-selected": selected.len(), "data-selected-ids": serialized_selection.clone(), "data-saved": current.saved,
                "data-navigate-across-pages": navigate_across_pages(),
                "data-tab-across-pages": tab_across_pages(),
                "data-group-mode": group_mode, "data-groups": current.accepted.groups.len(), "data-auto-page-size": auto_page_size(),
                "data-row-move": row_move(), "data-moves": current.moves, "data-row-order": serialized_order,
                "data-row-drag-across-pages": row_move() && drag_across_pages(),
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
            p { class: "host-paged-intro", "The page provider filters, sorts and groups all 137 local records, then supplies the accepted page, matching total and global group summaries. An application can use the same state request to fetch its data; this example makes no network requests." }
            p { class: "host-paged-intro", "Select equipment on different pages, then mark the full selection for maintenance. Group checkboxes select matching equipment across pages; the page header selects only loaded records. Closing a group keeps its selection. Double-click Equipment or Owner to edit; save with Enter or cancel with Escape before paging." }
            p { class: "host-paged-intro", "Enable reordering, then drag an Equipment handle or use Alt+Up/Down on it. The provider follows the moved record across pages. Clear sorting, filters and grouping before moving equipment." }
            p { class: "host-paged-intro", "With Drag across pages enabled, hold the dragged equipment near the top or bottom of the grid to change page, then drop before or after a row." }
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
    fn provider_accepts_loaded_page_change_before_moving_original_source_row() {
        let mut provider = EquipmentProvider::new();
        let mut query = provider.accepted.state.clone();
        query.selected.insert("host-equipment-002".into());
        provider.accept(query);
        provider.save_edit(DataGridCellValueChanged {
            row_id: "host-equipment-002".into(),
            column_id: "owner".into(),
            old_value: "Team 3".into(),
            new_value: "Inspection team".into(),
        });
        // Edge paging accepts a new partial page before the drop. The provider
        // still validates source identity against its complete managed order.
        let mut query = provider.accepted.state.clone();
        query.pagination.as_mut().unwrap().page = 1;
        provider.accept(query);
        assert!(
            !provider
                .accepted
                .rows
                .iter()
                .any(|row| row.id == "host-equipment-002")
        );
        let loaded_before_drop = provider.accepted.rows.clone();
        assert!(provider.move_row(DataGridPageRowMove {
            row_id: "host-equipment-002".into(),
            from_index: 2,
            to_index: 14,
            total_rows: 137,
        }));
        assert_eq!(provider.accepted.state.pagination.unwrap().page, 1);
        assert_eq!(provider.accepted.rows[2].id, "host-equipment-002");
        assert_eq!(provider.accepted.rows[2].value("owner"), "Inspection team");
        assert_eq!(provider.accepted.state.row_order[14], "host-equipment-002");
        assert_eq!(provider.accepted.state.row_order.len(), 137);
        assert!(
            provider
                .accepted
                .state
                .selected
                .contains("host-equipment-002")
        );
        assert_eq!(provider.moves, 1);
        assert_eq!(provider.saved, 1);
        assert_eq!(loaded_before_drop[0].id, "host-equipment-012");
        assert_eq!(loaded_before_drop[2].id, "host-equipment-014");
    }

    #[test]
    fn provider_moves_full_order_across_pages_without_losing_committed_data_or_selection() {
        let mut provider = EquipmentProvider::new();
        let mut query = provider.accepted.state.clone();
        query
            .selected
            .extend(["host-equipment-011".into(), "host-equipment-100".into()]);
        provider.accept(query);
        provider.save_edit(DataGridCellValueChanged {
            row_id: "host-equipment-011".into(),
            column_id: "owner".into(),
            old_value: "Team 4".into(),
            new_value: "On-call team".into(),
        });
        let previous = provider.accepted.clone();
        assert!(provider.move_row(DataGridPageRowMove {
            row_id: "host-equipment-011".into(),
            from_index: 11,
            to_index: 12,
            total_rows: 137,
        }));
        assert_eq!(provider.accepted.state.pagination.unwrap().page, 1);
        assert_eq!(provider.accepted.rows[0].id, "host-equipment-011");
        assert_eq!(provider.accepted.rows[0].value("owner"), "On-call team");
        assert_eq!(provider.accepted.state.row_order.len(), 137);
        assert_eq!(
            &provider.accepted.state.row_order[10..14],
            [
                "host-equipment-010",
                "host-equipment-012",
                "host-equipment-011",
                "host-equipment-013"
            ]
        );
        assert_eq!(provider.accepted.state.selected, previous.state.selected);
        assert_eq!(previous.rows[11].id, "host-equipment-011");
        assert!(previous.state.row_order.is_empty());
        assert_eq!(provider.moves, 1);
        assert_eq!(provider.saved, 1);
        assert!(provider.move_row(DataGridPageRowMove {
            row_id: "host-equipment-011".into(),
            from_index: 12,
            to_index: 0,
            total_rows: 137,
        }));
        assert_eq!(provider.accepted.state.pagination.unwrap().page, 0);
        assert_eq!(provider.accepted.rows[0].id, "host-equipment-011");
        assert_eq!(provider.accepted.state.selected, previous.state.selected);
        provider.group_by("owner");
        let group = DataGridGroupPath::new([("owner", "On-call team")]);
        assert_eq!(summary(&provider, &group).count, 1);
        assert_eq!(summary(&provider, &group).selected_count, 1);
        assert_eq!(provider.moves, 2);
    }

    #[test]
    fn stale_provider_move_preserves_accepted_page_order_and_selection() {
        let mut provider = EquipmentProvider::new();
        let mut query = provider.accepted.state.clone();
        query.selected.insert("host-equipment-000".into());
        query.pagination.as_mut().unwrap().page = 1;
        provider.accept(query);
        let request = DataGridPageRowMove {
            row_id: "host-equipment-012".into(),
            from_index: 12,
            to_index: 11,
            total_rows: 137,
        };
        assert!(provider.move_row(request.clone()));
        let accepted = provider.accepted.clone();
        assert!(!provider.move_row(request));
        assert_eq!(provider.moves, 1);
        assert_eq!(provider.accepted.state, accepted.state);
        assert_eq!(
            provider
                .accepted
                .rows
                .iter()
                .map(|row| &row.id)
                .collect::<Vec<_>>(),
            accepted.rows.iter().map(|row| &row.id).collect::<Vec<_>>()
        );
        assert_eq!(provider.accepted.total, 137);
        assert!(provider.feedback.contains("no longer matches"));
        // Existing grouping remains a supported provider workflow, but it must
        // not accidentally admit a delayed managed move from the flat view.
        provider.group_by("status");
        let grouped = provider.accepted.clone();
        assert!(!provider.move_row(DataGridPageRowMove {
            row_id: "host-equipment-012".into(),
            from_index: 11,
            to_index: 12,
            total_rows: 137,
        }));
        assert_eq!(provider.accepted.state, grouped.state);
        assert_eq!(provider.accepted.groups.len(), grouped.groups.len());
        assert_eq!(provider.moves, 1);
    }

    fn summary<'a>(
        provider: &'a EquipmentProvider,
        path: &DataGridGroupPath,
    ) -> &'a DataGridPageGroup {
        provider
            .accepted
            .groups
            .iter()
            .find(|group| &group.path == path)
            .unwrap()
    }

    #[test]
    fn grouped_provider_selection_counts_matching_records_beyond_the_loaded_page() {
        let mut provider = EquipmentProvider::new();
        provider.group_by("status");
        let normal = DataGridGroupPath::new([("status", "Normal")]);
        let warning = DataGridGroupPath::new([("status", "Warning")]);
        assert_eq!(provider.accepted.total, 137);
        assert_eq!(provider.accepted.groups.len(), 3);
        assert_eq!(summary(&provider, &normal).count, 46);
        assert_eq!(provider.accepted.rows.len(), 12);
        provider.select_group(DataGridPageGroupSelection {
            path: normal.clone(),
            checked: true,
        });
        assert_eq!(provider.accepted.state.selected.len(), 46);
        assert_eq!(summary(&provider, &normal).selected_count, 46);
        assert!(
            provider
                .accepted
                .state
                .selected
                .contains("host-equipment-135")
        );
        let mut query = provider.accepted.state.clone();
        query.pagination.as_mut().unwrap().page = 3;
        query.set_group_expanded(&normal, false);
        provider.accept(query);
        assert_eq!(provider.accepted.state.pagination.unwrap().page, 3);
        assert_eq!(summary(&provider, &normal).selected_count, 46);
        let mut query = provider.accepted.state.clone();
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
        query.pagination.as_mut().unwrap().page = 0;
        provider.accept(query);
        assert_eq!(provider.accepted.total, 46);
        assert_eq!(provider.accepted.groups.len(), 1);
        assert_eq!(summary(&provider, &warning).selected_count, 0);
        provider.select_group(DataGridPageGroupSelection {
            path: warning.clone(),
            checked: true,
        });
        assert_eq!(provider.accepted.state.selected.len(), 92);
        assert_eq!(summary(&provider, &warning).selected_count, 46);
        provider.select_group(DataGridPageGroupSelection {
            path: warning,
            checked: false,
        });
        assert_eq!(provider.accepted.state.selected.len(), 46);
        assert!(
            provider
                .accepted
                .state
                .selected
                .contains("host-equipment-135")
        );
        assert!(
            !provider
                .accepted
                .state
                .selected
                .contains("host-equipment-136")
        );
    }

    #[test]
    fn committed_owner_edit_regroups_global_summaries_without_changing_selection() {
        let mut provider = EquipmentProvider::new();
        provider.group_by("status-owner");
        let old_group = DataGridGroupPath::new([("status", "Normal"), ("owner", "Team 1")]);
        assert_eq!(summary(&provider, &old_group).count, 12);
        let previous = provider.accepted.clone();
        provider.select_group(DataGridPageGroupSelection {
            path: old_group.clone(),
            checked: true,
        });
        provider.save_edit(DataGridCellValueChanged {
            row_id: "host-equipment-000".into(),
            column_id: "owner".into(),
            old_value: "Team 1".into(),
            new_value: "On-call team".into(),
        });
        let new_group = DataGridGroupPath::new([("status", "Normal"), ("owner", "On-call team")]);
        assert_eq!(summary(&provider, &old_group).count, 11);
        assert_eq!(summary(&provider, &old_group).selected_count, 11);
        assert_eq!(summary(&provider, &new_group).count, 1);
        assert_eq!(summary(&provider, &new_group).selected_count, 1);
        assert_eq!(
            previous
                .groups
                .iter()
                .find(|group| group.path == old_group)
                .unwrap()
                .count,
            12
        );
        assert_eq!(previous.rows[0].value("owner"), "Team 1");
        let selected = provider.accepted.state.selected.clone();
        provider.group_by("owner");
        let on_call = DataGridGroupPath::new([("owner", "On-call team")]);
        assert_eq!(summary(&provider, &on_call).selected_count, 1);
        assert_eq!(provider.accepted.state.selected, selected);
        let mut query = provider.accepted.state.clone();
        query.filters.insert(
            "owner".into(),
            DataGridTextFilter {
                first: DataGridFilterCondition {
                    operator: DataGridFilterOperator::Equals,
                    value: "On-call team".into(),
                },
                ..Default::default()
            },
        );
        provider.accept(query);
        assert_eq!(provider.accepted.total, 1);
        assert_eq!(provider.accepted.rows[0].id, "host-equipment-000");
        assert_eq!(provider.accepted.groups.len(), 1);
        assert_eq!(provider.accepted.state.selected, selected);
        provider.group_by("none");
        assert!(provider.accepted.groups.is_empty());
        assert_eq!(provider.accepted.total, 1);
        assert_eq!(provider.accepted.state.selected, selected);
        assert_eq!(provider.saved, 1);
    }

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
