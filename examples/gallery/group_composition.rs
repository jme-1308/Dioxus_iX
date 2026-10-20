//! Local inspection assignments with selectable labels and independent editing.
use dioxus::prelude::*;
use dioxus_ix::*;

#[derive(Clone, Debug, PartialEq, Eq)]
struct Assignment {
    asset: String,
    note: String,
}

fn asset_name(asset: &str) -> &'static str {
    match asset {
        "motor" => "Motor 001",
        "pump" => "Pump 002",
        _ => "No asset selected",
    }
}

#[component]
fn AssignmentEditor(onsave: EventHandler<String>) -> Element {
    let mut note = use_signal(|| "Inspect coupling before assignment".to_string());
    rsx! {
        div { class: "group-compose-editor",
            Input { id: "group-compose-note", label: "Assignment note", value: note(), max_length: 160,
                onvaluechange: move |value| note.set(value),
            }
            Button { id: "group-compose-save", button_type: ButtonType::Button,
                onclick: move |_| onsave.call(note.peek().clone()), "Save assignment"
            }
        }
    }
}

#[component]
fn AssignmentPreparation() -> Element {
    let mut preparation = use_signal(|| "Prepare local assignments".to_string());
    rsx! { Input { id: "group-compose-preparation", class: "group-compose-preparation", label: "Page preparation", value: preparation(), max_length: 160,
        onvaluechange: move |value| preparation.set(value),
    } }
}

/// A composed row remains a native selectable button; editing stays in its own row.
#[component]
pub fn GroupCompositionExample() -> Element {
    let mut selected = use_signal(String::new);
    let mut paused = use_signal(|| false);
    let mut motor_unavailable = use_signal(|| false);
    let mut records = use_signal(Vec::<Assignment>::new);
    let mut error = use_signal(|| false);
    let mut feedback =
        use_signal(|| "Choose an available asset and save an assignment note.".to_string());
    let mut epoch = use_signal(|| 0_u64);
    let save = use_callback(move |note: String| {
        let asset = selected.peek().clone();
        if asset != "pump" && (asset != "motor" || *motor_unavailable.peek()) {
            error.set(true);
            feedback.set(
                "Choose an available Motor 001 or Pump 002 before saving an assignment.".into(),
            );
            return;
        }
        let mut next = records.peek().clone();
        next.push(Assignment {
            asset: asset.clone(),
            note,
        });
        records.set(next);
        error.set(false);
        feedback.set(format!(
            "Assignment saved for {}. The note remains available for another assignment.",
            asset_name(&asset)
        ));
    });
    rsx! {
        section { id: "group-composition", class: "demo-section group-composition-example",
            h2 { "Inspection assignments" }
            p { class: "group-compose-intro", "Choose an illustrative local asset using its structured row label, then save an assignment note. The editable note is independent of row selection. This example does not contact equipment or an assignment service." }
            div { class: "group-compose-toolbar",
                Button { id: "group-compose-reset", variant: ButtonVariant::Secondary, button_type: ButtonType::Button,
                    onclick: move |_| {
                        selected.set(String::new()); paused.set(false); motor_unavailable.set(false); records.set(Vec::new()); error.set(false);
                        feedback.set("Choose an available asset and save an assignment note.".into()); let next = epoch.peek().wrapping_add(1); epoch.set(next);
                    }, "Reset assignment example"
                }
            }
            div { class: "group-compose-options",
                label { input { id: "group-compose-paused", r#type: "checkbox", checked: paused(), onchange: move |event: FormEvent| paused.set(event.checked()) } "Pause row selection" }
                label { input { id: "group-compose-motor-unavailable", r#type: "checkbox", checked: motor_unavailable(), onchange: move |event: FormEvent| motor_unavailable.set(event.checked()) } "Motor unavailable" }
            }
            for generation in [epoch()] { AssignmentPreparation { key: "group-compose-preparation-{generation}" } }
            for generation in [epoch()] {
                Group { key: "group-compose-{generation}", id: "group-compose-group", header: "Inspection assignment", subheader: "Local equipment", default_expanded: true, suppress_header_selection: true,
                    GroupItem { id: "group-compose-motor", class: "group-compose-selectable", icon: "cogwheel", selectable_children: true,
                        selected: selected() == "motor", disabled: motor_unavailable(), suppress_selection: paused(), aria_label: "Assign Motor 001",
                        onselect: move |_| { if !*paused.peek() && !*motor_unavailable.peek() { selected.set("motor".into()); error.set(false); } },
                        span { class: "group-compose-label",
                            span { class: "group-compose-name-line", strong { "Motor 001" } span { if motor_unavailable() { "Unavailable" } else { "Available" } } }
                            span { "Drive train · Line 1" }
                        }
                    }
                    GroupItem { id: "group-compose-pump", class: "group-compose-selectable", icon: "bulb", selectable_children: true,
                        selected: selected() == "pump", suppress_selection: paused(), aria_label: "Assign Pump 002",
                        onselect: move |_| { if !*paused.peek() { selected.set("pump".into()); error.set(false); } },
                        span { class: "group-compose-label",
                            span { class: "group-compose-name-line", strong { "Pump 002" } span { "Available" } }
                            span { "Cooling loop · Line 2" }
                        }
                    }
                    GroupItem { id: "group-compose-archive", class: "group-compose-selectable", icon: "lock", selectable_children: true, disabled: true, aria_label: "Archived 004 unavailable",
                        span { class: "group-compose-label",
                            span { class: "group-compose-name-line", strong { "Archived 004" } span { "Archived" } }
                            span { "Retired inspection fixture" }
                        }
                    }
                    GroupItem { id: "group-compose-editor", aria_label: "Independent assignment controls", AssignmentEditor { onsave: save } }
                }
            }
            output { id: "group-compose-result", class: "group-compose-result", role: if error() { "alert" } else { "status" },
                "data-selected": selected(), "data-saves": records.read().len().to_string(), "data-paused": paused().to_string(), "data-motor-unavailable": motor_unavailable().to_string(), "data-error": error().to_string(),
                span { "Selected: {asset_name(&selected())}." } span { "{feedback}" }
            }
            table { id: "group-compose-records", class: "group-compose-records",
                caption { "Saved local assignments" }
                thead { tr { th { scope: "col", "Asset" } th { scope: "col", "Assignment note" } } }
                tbody {
                    for (index, record) in records().into_iter().enumerate() {
                        tr { key: "{index}", id: format!("group-compose-record-{}", index + 1), "data-asset": record.asset.clone(),
                            th { scope: "row", "data-field": "asset", "{asset_name(&record.asset)}" }
                            td { "data-field": "note", "{record.note}" }
                        }
                    }
                    if records.read().is_empty() { tr { td { colspan: "2", "No assignments saved." } } }
                }
            }
        }
    }
}

pub(crate) const STYLES: &str = include_str!("styles/group_composition.css");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn composed_assignment_rows_and_independent_editor_have_native_semantics() {
        let mut dom = VirtualDom::new(GroupCompositionExample);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        for id in [
            "group-composition",
            "group-compose-motor",
            "group-compose-pump",
            "group-compose-archive",
            "group-compose-note",
            "group-compose-save",
            "group-compose-preparation",
            "group-compose-records",
        ] {
            assert!(html.contains(&format!("id=\"{id}\"")), "{id}");
        }
        assert!(
            html.contains("aria-label=\"Assign Motor 001\"")
                && html.contains("Drive train · Line 1")
                && html.contains("Cooling loop · Line 2")
        );
        let archive = html
            .split("id=\"group-compose-archive\"")
            .nth(1)
            .unwrap()
            .split('>')
            .next()
            .unwrap();
        assert!(
            archive.contains("disabled")
                && archive.contains("aria-label=\"Archived 004 unavailable\"")
        );
        let mut depth = 0;
        for tag in html.split('<').skip(1) {
            if tag.starts_with("button ") {
                assert_eq!(
                    depth, 0,
                    "independent controls must not be nested in selectable rows"
                );
                depth += 1;
            } else if tag.starts_with("/button>") {
                depth -= 1;
            }
        }
        assert_eq!(depth, 0);
        assert!(
            html.contains("Inspect coupling before assignment")
                && html.contains("data-selected=\"\"")
                && html.contains("No assignments saved.")
        );
    }
}
