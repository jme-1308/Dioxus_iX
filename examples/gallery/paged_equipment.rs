//! Controlled equipment register showcasing paginated table slices, localized labels, and cancellable page navigation.
use dioxus::prelude::*;
use dioxus_ix::*;

const TOTAL_INITIAL_ASSETS: usize = 83;
const PRUNED_ASSET_LIMIT: usize = 12;
const INITIAL_PAGE_SIZE: usize = 5;

#[derive(Clone, Debug, PartialEq, Eq)]
struct EquipmentRecord {
    id: String,
    name: String,
    location: &'static str,
    needs_service: bool,
}

impl EquipmentRecord {
    fn new(number: usize) -> Self {
        let location = match (number - 1) % 4 {
            0 => "Area A",
            1 => "Area B",
            2 => "Area C",
            _ => "Area D",
        };
        Self {
            id: format!("asset-{number}"),
            name: format!("Asset {number:03}"),
            location,
            needs_service: number.is_multiple_of(4),
        }
    }
}

fn initial_equipment_records() -> Vec<EquipmentRecord> {
    (1..=TOTAL_INITIAL_ASSETS)
        .map(EquipmentRecord::new)
        .collect()
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum EquipmentFilter {
    #[default]
    All,
    NeedsService,
    Empty,
}

impl EquipmentFilter {
    fn from_value(value: &str) -> Self {
        match value {
            "service" => Self::NeedsService,
            "empty" => Self::Empty,
            _ => Self::All,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::NeedsService => "service",
            Self::Empty => "empty",
        }
    }

    fn matches(self, record: &EquipmentRecord) -> bool {
        match self {
            Self::All => true,
            Self::NeedsService => record.needs_service,
            Self::Empty => false,
        }
    }
}

fn german_pagination_labels() -> PaginationLabels {
    PaginationLabels {
        page: "Seite".into(),
        of: "von".into(),
        items: "Einträge".into(),
        previous_page: "Vorherige Seite".into(),
        next_page: "Nächste Seite".into(),
        page_selection: "Seitenauswahl".into(),
        items_per_page: "Einträge pro Seite".into(),
        jump_to_page: "Zu Seite {page} springen".into(),
    }
}

/// Interactive equipment register with controlled pagination, direct page entry, and cancellable page changes.
#[component]
pub fn PagedEquipmentExample() -> Element {
    let mut records = use_signal(initial_equipment_records);
    let mut filter = use_signal(EquipmentFilter::default);
    let mut selected_page = use_signal(|| 0usize);
    let mut item_count = use_signal(|| INITIAL_PAGE_SIZE);
    let mut advanced = use_signal(|| true);
    let mut veto_navigation = use_signal(|| false);
    let mut hold_changes = use_signal(|| false);
    let mut german = use_signal(|| false);
    let mut hide_size = use_signal(|| false);
    let mut requests = use_signal(|| 0usize);
    let mut accepted = use_signal(|| 0usize);
    let mut epoch = use_signal(|| 0u64);

    let current_records = records();
    let current_filter = filter();
    let current_selected_page = selected_page();
    let current_size = item_count().max(1);
    let current_advanced = advanced();
    let current_veto = veto_navigation();
    let current_hold = hold_changes();
    let current_german = german();
    let current_hide_size = hide_size();
    let current_requests = requests();
    let current_accepted = accepted();
    let current_epoch = epoch();

    let filtered_records: Vec<EquipmentRecord> = current_records
        .iter()
        .filter(|record| current_filter.matches(record))
        .cloned()
        .collect();
    let total_matches = filtered_records.len();
    let page_count = if total_matches == 0 {
        0
    } else {
        total_matches.div_ceil(current_size)
    };
    let actual_page = current_selected_page.min(page_count.saturating_sub(1));
    let page_start = if total_matches == 0 {
        0
    } else {
        actual_page * current_size
    };
    let page_end = (page_start + current_size).min(total_matches);
    let page_rows = &filtered_records[page_start..page_end];

    let pagination_landmark = if current_german {
        "Geräteseiten"
    } else {
        "Equipment pages"
    };
    let pagination_labels = if current_german {
        german_pagination_labels()
    } else {
        PaginationLabels::default()
    };

    let status_summary = if total_matches == 0 {
        format!(
            "Showing 0 of 0 assets · Page 0 of 0 · Page requests: {current_accepted} of {current_requests} accepted"
        )
    } else {
        format!(
            "Showing {}–{} of {total_matches} assets · Page {} of {page_count} · Page requests: {current_accepted} of {current_requests} accepted",
            page_start + 1,
            page_end,
            actual_page + 1,
        )
    };

    rsx! {
        section {
            id: "paged-equipment",
            class: "demo-section paged-equipment-example",
            h2 { "Equipment register" }
            p {
                "Browse illustrative plant assets, filter by maintenance condition, and choose how many rows to show. "
                "Switch between direct page entry and numbered page buttons."
            }
            div { class: "paged-equipment-toolbar",
                Button {
                    id: "paged-reset",
                    variant: ButtonVariant::Secondary,
                    onclick: move |_| {
                        records.set(initial_equipment_records());
                        filter.set(EquipmentFilter::All);
                        selected_page.set(0);
                        item_count.set(INITIAL_PAGE_SIZE);
                        advanced.set(true);
                        veto_navigation.set(false);
                        hold_changes.set(false);
                        german.set(false);
                        hide_size.set(false);
                        requests.set(0);
                        accepted.set(0);
                        let next_epoch = epoch.peek().wrapping_add(1);
                        epoch.set(next_epoch);
                    },
                    "Restore register"
                }
                label { class: "paged-equipment-field", r#for: "paged-filter",
                    span { "Equipment filter" }
                    select {
                        id: "paged-filter",
                        class: "paged-equipment-select",
                        value: current_filter.as_str(),
                        onchange: move |event| {
                            filter.set(EquipmentFilter::from_value(&event.value()));
                            selected_page.set(0);
                        },
                        option { value: "all", selected: current_filter == EquipmentFilter::All, "All equipment" }
                        option { value: "service", selected: current_filter == EquipmentFilter::NeedsService, "Needs service" }
                        option { value: "empty", selected: current_filter == EquipmentFilter::Empty, "No matches" }
                    }
                }
                label { class: "paged-equipment-checkbox", r#for: "paged-advanced",
                    input {
                        id: "paged-advanced",
                        r#type: "checkbox",
                        checked: current_advanced,
                        onchange: move |event: FormEvent| advanced.set(event.checked()),
                    }
                    "Detailed controls"
                }
            }
            details {
                id: "paged-options",
                class: "paged-equipment-options",
                summary { class: "paged-equipment-options-summary", "Pagination options" }
                div { class: "paged-equipment-options-body",
                    div { class: "paged-equipment-options-row",
                        Button {
                            id: "paged-last",
                            variant: ButtonVariant::Secondary,
                            disabled: page_count == 0,
                            onclick: move |_| {
                                selected_page.set(page_count.saturating_sub(1));
                            },
                            "Last page"
                        }
                        Button {
                            id: "paged-prune",
                            variant: ButtonVariant::Secondary,
                            onclick: move |_| {
                                let mut next = records.peek().clone();
                                next.truncate(PRUNED_ASSET_LIMIT);
                                let active_filter = *filter.peek();
                                let size = (*item_count.peek()).max(1);
                                let next_total = next
                                    .iter()
                                    .filter(|record| active_filter.matches(record))
                                    .count();
                                let next_count = if next_total == 0 {
                                    0
                                } else {
                                    next_total.div_ceil(size)
                                };
                                let clamped_page = (*selected_page.peek()).min(next_count.saturating_sub(1));
                                records.set(next);
                                selected_page.set(clamped_page);
                            },
                            "Keep first 12 assets"
                        }
                    }
                    div { class: "paged-equipment-options-checks",
                        label { class: "paged-equipment-checkbox", r#for: "paged-veto",
                            input {
                                id: "paged-veto",
                                r#type: "checkbox",
                                checked: current_veto,
                                onchange: move |event: FormEvent| veto_navigation.set(event.checked()),
                            }
                            "Block navigation"
                        }
                        label { class: "paged-equipment-checkbox", r#for: "paged-hold",
                            input {
                                id: "paged-hold",
                                r#type: "checkbox",
                                checked: current_hold,
                                onchange: move |event: FormEvent| hold_changes.set(event.checked()),
                            }
                            "Hold page changes"
                        }
                        label { class: "paged-equipment-checkbox", r#for: "paged-german",
                            input {
                                id: "paged-german",
                                r#type: "checkbox",
                                checked: current_german,
                                onchange: move |event: FormEvent| german.set(event.checked()),
                            }
                            "German labels"
                        }
                        label { class: "paged-equipment-checkbox", r#for: "paged-hide-size",
                            input {
                                id: "paged-hide-size",
                                r#type: "checkbox",
                                checked: current_hide_size,
                                onchange: move |event: FormEvent| hide_size.set(event.checked()),
                            }
                            "Hide page size"
                        }
                    }
                }
            }
            div { class: "paged-equipment-table-wrap",
                table { class: "ix-table paged-equipment-table",
                    caption { "Equipment register" }
                    thead {
                        tr {
                            th { scope: "col", "Asset" }
                            th { scope: "col", "Location" }
                            th { scope: "col", "Condition" }
                        }
                    }
                    tbody {
                        for record in page_rows {
                            tr {
                                key: "{record.id}",
                                "data-equipment-id": "{record.id}",
                                td { class: "paged-equipment-name", "{record.name}" }
                                td { class: "paged-equipment-location", "{record.location}" }
                                td { class: "paged-equipment-condition",
                                    Pill {
                                        variant: if record.needs_service { StatusVariant::Warning } else { StatusVariant::Success },
                                        outline: true,
                                        icon: if record.needs_service { "warning" } else { "success" },
                                        if record.needs_service {
                                            "Needs service"
                                        } else {
                                            "Ready"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                if page_rows.is_empty() {
                    EmptyState {
                        header: "No matching equipment",
                        subheader: "Adjust the equipment filter to view assets in the register.",
                        icon: "search",
                        action: "All equipment",
                        layout: EmptyStateLayout::Compact,
                        onaction: move |_| {
                            filter.set(EquipmentFilter::All);
                            selected_page.set(0);
                        },
                    }
                }
            }
            div {
                id: "paged-navigation",
                class: "paged-equipment-navigation",
                for generation in [current_epoch] {
                    Pagination {
                        key: "equipment-pagination-{generation}",
                        count: page_count,
                        selected_page: actual_page,
                        advanced: current_advanced,
                        item_count: current_size,
                        item_count_options: vec![5, 10, 20],
                        hide_item_count: current_hide_size,
                        label: pagination_landmark.to_string(),
                        labels: pagination_labels.clone(),
                        onbeforepage: move |event: PaginationChangeEvent| {
                            let next_requests = requests.peek().saturating_add(1);
                            requests.set(next_requests);
                            if *veto_navigation.peek() {
                                event.prevent_default();
                            }
                        },
                        onpage: move |next: usize| {
                            let next_accepted = accepted.peek().saturating_add(1);
                            accepted.set(next_accepted);
                            if !*hold_changes.peek() {
                                selected_page.set(next);
                            }
                        },
                        onitemcount: move |next_size: usize| {
                            item_count.set(next_size.max(1));
                            selected_page.set(0);
                        },
                    }
                }
            }
            output {
                id: "paged-result",
                class: "paged-equipment-result",
                role: "status",
                "aria-live": "polite",
                "data-page": actual_page.to_string(),
                "data-count": page_count.to_string(),
                "data-size": current_size.to_string(),
                "data-total": total_matches.to_string(),
                "data-requests": current_requests.to_string(),
                "data-accepted": current_accepted.to_string(),
                "{status_summary}"
            }
        }
    }
}

pub const STYLES: &str = "\
.paged-equipment-example{margin-bottom:48px;min-width:0;max-width:100%}\
.paged-equipment-example>p{max-width:72ch;line-height:1.55;overflow-wrap:anywhere}\
.paged-equipment-toolbar{display:flex;align-items:center;flex-wrap:wrap;gap:12px 16px;margin:16px 0 12px;min-width:0;max-width:100%}\
.paged-equipment-field{display:inline-flex;align-items:center;flex-wrap:wrap;gap:8px;min-width:0;max-width:100%;font-size:14px}\
.paged-equipment-select{box-sizing:border-box;height:32px;min-width:136px;max-width:100%;padding:4px 8px;border:1px solid var(--ix-color-border);border-radius:var(--ix-radius);background:var(--ix-color-surface-1);color:var(--ix-color-text);font:14px/20px var(--ix-font)}\
.paged-equipment-select:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:1px}\
.paged-equipment-options{box-sizing:border-box;min-width:0;max-width:100%;margin:0 0 20px;padding:8px 12px;border:1px solid var(--ix-color-border);border-radius:var(--ix-radius);background:var(--ix-color-surface-1)}\
.paged-equipment-options-summary{cursor:pointer;font-size:14px;font-weight:600;line-height:20px;color:var(--ix-color-text);user-select:none}\
.paged-equipment-options-summary:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:2px;border-radius:var(--ix-radius)}\
.paged-equipment-options-body{display:flex;flex-direction:column;gap:12px;margin-top:12px;padding-top:12px;border-top:1px solid var(--ix-color-border);min-width:0;max-width:100%}\
.paged-equipment-options-row,.paged-equipment-options-checks{display:flex;align-items:center;flex-wrap:wrap;gap:10px 14px;min-width:0;max-width:100%}\
.paged-equipment-checkbox{display:inline-flex;align-items:center;gap:8px;min-width:0;max-width:100%;font-size:14px;cursor:pointer}\
.paged-equipment-checkbox input[type=\"checkbox\"]{width:16px;height:16px;margin:0;flex:none;accent-color:var(--ix-color-primary)}\
.paged-equipment-checkbox input[type=\"checkbox\"]:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:3px}\
.paged-equipment-table-wrap{box-sizing:border-box;min-width:0;max-width:100%;overflow-x:auto;border:1px solid var(--ix-color-border);border-radius:var(--ix-radius);background:var(--ix-color-surface-1)}\
.paged-equipment-table{width:100%;margin:0;border-collapse:collapse;border-spacing:0;color:var(--ix-color-text);font-size:14px;line-height:20px;font-variant-numeric:tabular-nums}\
.paged-equipment-table caption{padding:10px 12px;border-bottom:1px solid var(--ix-color-border);background:var(--ix-color-surface-2);color:var(--ix-color-text);font-size:14px;font-weight:600;line-height:20px;text-align:left}\
.paged-equipment-table th,.paged-equipment-table td{box-sizing:border-box;height:42px;padding:6px 12px;border-bottom:1px solid var(--ix-color-border);text-align:left;vertical-align:middle}\
.paged-equipment-table thead th{background:var(--ix-color-surface-2);color:var(--ix-color-text);font-size:12px;font-weight:600;line-height:16px}\
.paged-equipment-table tbody tr:last-child td{border-bottom:0}\
.paged-equipment-table tbody tr:hover td{background:var(--ix-color-surface-2)}\
.paged-equipment-name{font-weight:600;white-space:nowrap}\
.paged-equipment-location{color:var(--ix-color-text-secondary);white-space:nowrap}\
.paged-equipment-condition{white-space:nowrap}\
.paged-equipment-navigation{margin-top:16px;min-width:0;max-width:100%}\
.paged-equipment-navigation .ix-pagination{min-width:0;max-width:100%}\
.paged-equipment-result{display:block;min-height:1.5em;margin-top:16px;color:var(--ix-color-text-secondary);font-size:14px;line-height:20px;font-variant-numeric:tabular-nums;overflow-wrap:anywhere}\
@media(max-width:480px){.paged-equipment-toolbar,.paged-equipment-options-row,.paged-equipment-options-checks{align-items:flex-start}.paged-equipment-toolbar .ix-button,.paged-equipment-options-row .ix-button{max-width:100%}.paged-equipment-table th,.paged-equipment-table td{padding:6px 8px}}\
";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paged_equipment_example_renders_baseline_contract() {
        let mut dom = VirtualDom::new(PagedEquipmentExample);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);

        assert!(html.contains("id=\"paged-equipment\""));
        assert!(html.contains("Equipment register"));
        assert!(html.contains("id=\"paged-reset\""));
        assert!(html.contains("Restore register"));
        assert!(html.contains("id=\"paged-filter\""));
        assert!(html.contains("Equipment filter"));
        assert!(html.contains("All equipment"));
        assert!(html.contains("Needs service"));
        assert!(html.contains("No matches"));
        assert!(html.contains("id=\"paged-advanced\""));
        assert!(html.contains("Detailed controls"));
        assert!(html.contains("id=\"paged-options\""));
        assert!(html.contains("Pagination options"));
        assert!(html.contains("id=\"paged-last\""));
        assert!(html.contains("Last page"));
        assert!(html.contains("id=\"paged-prune\""));
        assert!(html.contains("Keep first 12 assets"));
        assert!(html.contains("id=\"paged-veto\""));
        assert!(html.contains("Block navigation"));
        assert!(html.contains("id=\"paged-hold\""));
        assert!(html.contains("Hold page changes"));
        assert!(html.contains("id=\"paged-german\""));
        assert!(html.contains("German labels"));
        assert!(html.contains("id=\"paged-hide-size\""));
        assert!(html.contains("Hide page size"));

        for (id, name, location) in [
            ("asset-1", "Asset 001", "Area A"),
            ("asset-2", "Asset 002", "Area B"),
            ("asset-3", "Asset 003", "Area C"),
            ("asset-4", "Asset 004", "Area D"),
            ("asset-5", "Asset 005", "Area A"),
        ] {
            assert!(
                html.contains(&format!("data-equipment-id=\"{id}\"")),
                "{id}"
            );
            assert!(html.contains(name), "{name}");
            assert!(html.contains(location), "{location}");
        }
        assert!(!html.contains("data-equipment-id=\"asset-6\""));
        assert!(!html.contains(">asset-1<"));
        assert!(html.contains("Ready"));
        assert!(html.contains("Needs service"));

        assert!(html.contains("id=\"paged-navigation\""));
        assert!(html.contains("aria-label=\"Equipment pages\""));
        assert!(html.contains("of 17"));
        assert!(html.contains("id=\"paged-result\""));
        assert!(html.contains("data-page=\"0\""));
        assert!(html.contains("data-count=\"17\""));
        assert!(html.contains("data-size=\"5\""));
        assert!(html.contains("data-total=\"83\""));
        assert!(html.contains("data-requests=\"0\""));
        assert!(html.contains("data-accepted=\"0\""));
        assert!(html.contains("Showing 1–5 of 83 assets"));
    }
}
