//! Local equipment filtering and assignments in responsive, content-sized columns.
use dioxus::prelude::*;
use dioxus_ix::*;
use std::collections::BTreeMap;

pub const STYLES: &str = include_str!("styles/equipment_search_layout.css");

#[derive(Clone, Copy)]
struct Equipment {
    id: &'static str,
    name: &'static str,
    line: &'static str,
    archived: bool,
}
static EQUIPMENT: [Equipment; 4] = [
    Equipment {
        id: "motor",
        name: "Motor 001",
        line: "Line 1",
        archived: false,
    },
    Equipment {
        id: "pump",
        name: "Pump 002",
        line: "Line 2",
        archived: false,
    },
    Equipment {
        id: "compressor",
        name: "Compressor 003",
        line: "Line 1",
        archived: false,
    },
    Equipment {
        id: "archive",
        name: "Archived 004",
        line: "Line 3",
        archived: true,
    },
];
fn equipment_for(id: &str) -> Option<&'static Equipment> {
    EQUIPMENT.iter().find(|equipment| equipment.id == id)
}
fn matches_filter(equipment: &Equipment, query: &str, line: &str) -> bool {
    (line == "all" || equipment.line == line)
        && format!("{} {}", equipment.name, equipment.line)
            .to_lowercase()
            .contains(&query.to_lowercase())
}
fn text_values(event: &FormEvent) -> BTreeMap<String, String> {
    event
        .values()
        .into_iter()
        .filter_map(|(name, value)| match value {
            dioxus::html::FormValue::Text(text) => Some((name, text)),
            _ => None,
        })
        .collect()
}
#[derive(Clone)]
struct SearchModel {
    query: String,
    line: String,
    line_draft: String,
    selected: Option<String>,
    assignments: BTreeMap<String, String>,
    filter_payload: BTreeMap<String, String>,
    assignment_payload: BTreeMap<String, String>,
    query_reset: u64,
    note_reset: u64,
    message: String,
}
impl Default for SearchModel {
    fn default() -> Self {
        Self {
            query: String::new(),
            line: "all".into(),
            line_draft: "all".into(),
            selected: None,
            assignments: BTreeMap::new(),
            filter_payload: BTreeMap::new(),
            assignment_payload: BTreeMap::new(),
            query_reset: 0,
            note_reset: 0,
            message: "Choose equipment to prepare a local assignment.".into(),
        }
    }
}
#[component]
fn SearchPreparation() -> Element {
    let mut preparation = use_signal(|| String::from("Check access with the shift lead"));
    rsx! { div { class: "equipment-search-preparation",
        Input { id: "equipment-search-preparation", label: "Inspection preparation", value: preparation(), max_length: 160,
            onvaluechange: move |value| preparation.set(value), }
    } }
}
#[component]
fn SearchDraftField(id: String, name: String, label: String, reset: u64) -> Element {
    let mut draft = use_signal(String::new);
    use_effect(use_reactive((&reset,), move |(_reset,)| {
        let has_value = !draft.peek().is_empty();
        if has_value {
            draft.set(String::new());
        }
    }));
    rsx! { div { class: "equipment-search-draft",
        Input { id, name, label, value: draft(), max_length: 160, onvaluechange: move |value| draft.set(value) }
    } }
}

#[component]
pub fn EquipmentSearchLayoutExample() -> Element {
    let mut model = use_signal(SearchModel::default);
    let mut auto = use_signal(|| true);
    let line_mount = use_focus_mount("equipment-search-line");
    let auto_mount = use_focus_mount("equipment-search-auto");
    let current = model();
    let results = EQUIPMENT
        .iter()
        .filter(|equipment| matches_filter(equipment, &current.query, &current.line))
        .collect::<Vec<_>>();
    let selected_equipment = current.selected.as_deref().and_then(equipment_for);
    let action_label = selected_equipment.map_or_else(
        || "Select equipment".to_string(),
        |equipment| format!("Assign {}", equipment.name),
    );
    let content_widths = ColWidths {
        base: Some(if auto() {
            ColWidth::Auto
        } else {
            ColWidth::Span(3)
        }),
        sm: Some(ColWidth::Span(12)),
        ..ColWidths::default()
    };
    let flexible_widths = ColWidths {
        base: Some(ColWidth::Flexible),
        sm: Some(ColWidth::Span(12)),
        ..ColWidths::default()
    };
    let filter_payload = serde_json::to_string(&current.filter_payload).unwrap_or_default();
    let assignment_payload = serde_json::to_string(&current.assignment_payload).unwrap_or_default();
    rsx! {
        style { {STYLES} }
        section { id: "equipment-search-layout", class: "equipment-search-example",
            h2 { "Find equipment for inspection" }
            p { class: "equipment-search-intro", "Filter these illustrative local records, then save an assignment note. Action columns follow their content while search fields use the remaining space." }
            SearchPreparation {}
            label { class: "equipment-search-option",
                input { id: "equipment-search-auto", r#type: "checkbox", checked: auto(), onmounted: auto_mount, onchange: move |event| auto.set(event.checked()) }
                "Size filter and action columns to their content"
            }
            form { id: "equipment-search-filter-form", class: "equipment-search-form",
                onsubmit: move |event| {
                    event.prevent_default();
                    let values = text_values(&event);
                    let Some(query) = values.get("query").cloned() else { return; };
                    let Some(line) = values.get("line").filter(|line| ["all", "Line 1", "Line 2", "Line 3"].contains(&line.as_str())).cloned() else { return; };
                    let mut state = model.write();
                    if state.selected.as_deref().and_then(equipment_for).is_some_and(|equipment| !matches_filter(equipment, &query, &line)) { state.selected = None; }
                    state.query = query;
                    state.line = line.clone();
                    state.line_draft = line;
                    state.filter_payload = values;
                    state.message = "Equipment results updated.".into();
                },
                onreset: move |event| {
                    event.prevent_default();
                    let mut state = model.write();
                    state.query.clear(); state.line = "all".into(); state.line_draft = "all".into();
                    state.query_reset += 1; state.filter_payload.clear();
                    state.message = "Filters cleared. Assignment notes are kept.".into();
                },
                LayoutGrid { gap: 16,
                    Row {
                        Col { widths: content_widths,
                            label { class: "equipment-search-location", r#for: "equipment-search-line", "Location"
                                select { id: "equipment-search-line", name: "line", value: current.line_draft.clone(), onmounted: line_mount,
                                    onchange: move |event| model.write().line_draft = event.value(),
                                    for (value, label) in [("all", "All locations"), ("Line 1", "Line 1"), ("Line 2", "Line 2"), ("Line 3", "Line 3")] {
                                        option { value, selected: current.line_draft == value, "{label}" }
                                    }
                                }
                            }
                        }
                        Col { widths: flexible_widths,
                            SearchDraftField { id: "equipment-search-query", name: "query", label: "Search equipment", reset: current.query_reset }
                        }
                        Col { widths: content_widths,
                            div { class: "equipment-search-action-group", span { "Filter results" }
                                div { class: "equipment-search-actions",
                                    Button { id: "equipment-search-apply", button_type: ButtonType::Submit, "Apply filters" }
                                    Button { id: "equipment-search-clear", button_type: ButtonType::Reset, variant: ButtonVariant::Secondary, "Clear filters" }
                                }
                            }
                        }
                    }
                }
            }
            table { id: "equipment-search-results", class: "equipment-search-table",
                caption { "Equipment results" }
                thead { tr { th { scope: "col", "Equipment" } th { scope: "col", "Location" } th { scope: "col", "Selection" } } }
                tbody {
                    if results.is_empty() { tr { td { colspan: "3", "No equipment matches these filters. Clear filters to try again." } } }
                    for equipment in results.iter().copied() {
                        tr { key: "{equipment.id}", id: "equipment-search-row-{equipment.id}", "data-equipment": equipment.id,
                            "data-selected": current.selected.as_deref() == Some(equipment.id),
                            td { "{equipment.name}" } td { "{equipment.line}" }
                            td { Button { id: "equipment-select-{equipment.id}", variant: ButtonVariant::Secondary, button_type: ButtonType::Button,
                                disabled: equipment.archived, aria_label: format!("Select {}", equipment.name),
                                onclick: move |_| { if !equipment.archived { let mut state = model.write(); state.selected = Some(equipment.id.into()); state.message = format!("{} selected for assignment.", equipment.name); } },
                                if equipment.archived { "Unavailable" } else if current.selected.as_deref() == Some(equipment.id) { "Selected" } else { "Select" }
                            } }
                        }
                    }
                }
            }
            form { id: "equipment-search-assignment-form", class: "equipment-search-form equipment-search-assignment",
                onsubmit: move |event| {
                    event.prevent_default();
                    let selected = model.peek().selected.clone();
                    let Some(equipment) = selected.as_deref().and_then(equipment_for).filter(|equipment| !equipment.archived) else { return; };
                    let values = text_values(&event);
                    let Some(note) = values.get("assignment-note").cloned() else { return; };
                    let mut state = model.write();
                    state.assignments.insert(equipment.id.into(), note);
                    state.assignment_payload = values;
                    state.message = format!("Assignment saved for {}.", equipment.name);
                },
                LayoutGrid { gap: 16,
                    Row {
                        Col { widths: flexible_widths,
                            SearchDraftField { id: "equipment-search-note", name: "assignment-note", label: "Assignment note", reset: current.note_reset }
                        }
                        Col { widths: content_widths,
                            div { class: "equipment-search-action-group", span { "Selected equipment" }
                                Button { id: "equipment-search-assign", button_type: ButtonType::Submit, disabled: selected_equipment.is_none(), "{action_label}" }
                            }
                        }
                    }
                }
            }
            Button { id: "equipment-search-reset", class: "equipment-search-reset", variant: ButtonVariant::Tertiary, button_type: ButtonType::Button,
                onclick: move |_| { let query_reset = model.peek().query_reset + 1; let note_reset = model.peek().note_reset + 1;
                    model.set(SearchModel { query_reset, note_reset, message: "Search reset. Preparation and the width setting are kept.".into(), ..SearchModel::default() }); },
                "Reset search and assignments"
            }
            p { id: "equipment-search-status", class: "equipment-search-status", role: "status", "aria-live": "polite",
                "data-query": current.query, "data-line": current.line, "data-selected": current.selected.unwrap_or_default(),
                "data-visible": results.len(), "data-assigned": current.assignments.len(), "data-auto": auto(), "{current.message}"
            }
            table { id: "equipment-search-assignments", class: "equipment-search-table",
                caption { "Saved assignments" }
                thead { tr { th { scope: "col", "Equipment" } th { scope: "col", "Assignment note" } } }
                tbody {
                    if current.assignments.is_empty() { tr { td { colspan: "2", "No assignments saved yet." } } }
                    for (id, note) in current.assignments.iter() {
                        tr { key: "{id}", id: "equipment-search-assignment-{id}", "data-equipment": id.clone(), "data-note": note.clone(),
                            td { {equipment_for(id).map_or("Equipment", |equipment| equipment.name)} }
                            td { if note.is_empty() { "No note" } else { "{note}" } }
                        }
                    }
                }
            }
            output { id: "equipment-search-filter-payload", hidden: true, "{filter_payload}" }
            output { id: "equipment-search-assignment-payload", hidden: true, "{assignment_payload}" }
        }
    }
}
