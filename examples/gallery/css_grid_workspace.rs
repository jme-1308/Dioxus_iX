//! Responsive inspection layout with Rust-owned drafts and native form submission.
use dioxus::prelude::*;
use dioxus_ix::*;
use std::{collections::BTreeMap, rc::Rc};

pub const STYLES: &str = include_str!("styles/css_grid_workspace.css");

#[derive(Clone, Debug, PartialEq, Eq)]
struct InspectionRecord {
    equipment: String,
    title: String,
    note: String,
}

fn inspection_record(values: &BTreeMap<String, String>) -> Option<InspectionRecord> {
    let equipment = values.get("equipment")?;
    let title = values.get("title")?;
    if !matches!(equipment.as_str(), "pump" | "fan") || title.trim().is_empty() {
        return None;
    }
    Some(InspectionRecord {
        equipment: equipment.clone(),
        title: title.clone(),
        note: values.get("note")?.clone(),
    })
}

fn workspace_templates(editor_first: bool, renamed: bool) -> GridTemplates {
    let activity = if renamed { "notes" } else { "activity" };
    let (large, medium, small) = if editor_first {
        (
            vec![
                vec!["overview", "overview"],
                vec!["editor", "equipment"],
                vec!["editor", activity],
            ],
            vec![
                vec!["overview", "overview"],
                vec!["editor", "editor"],
                vec!["equipment", activity],
            ],
            vec![
                vec!["overview"],
                vec!["editor"],
                vec!["equipment"],
                vec![activity],
            ],
        )
    } else {
        (
            vec![
                vec!["overview", "overview"],
                vec!["equipment", "editor"],
                vec![activity, "editor"],
            ],
            vec![
                vec!["overview", "overview"],
                vec!["equipment", activity],
                vec!["editor", "editor"],
            ],
            vec![
                vec!["overview"],
                vec!["equipment"],
                vec!["editor"],
                vec![activity],
            ],
        )
    };
    let template = |rows: Vec<Vec<&str>>| {
        GridTemplate::new(
            rows.into_iter()
                .map(|row| row.into_iter().map(str::to_owned).collect())
                .collect(),
        )
        .expect("inspection areas form rectangular templates")
    };
    GridTemplates {
        sm: Some(template(small)),
        md: Some(template(medium)),
        lg: Some(template(large)),
    }
}

#[component]
fn InspectionEditor(
    disabled: bool,
    ontitlemounted: EventHandler<MountedEvent>,
    ontitlefocused: EventHandler<()>,
) -> Element {
    let mut title = use_signal(|| String::from("Inspect inlet seals"));
    let mut note = use_signal(String::new);
    rsx! {
        h3 { "Inspection note" }
        Input { id: "css-grid-title", name: "title", label: "Inspection title", required: true,
            disabled, value: title(), max_length: 120, onmounted: ontitlemounted,
            onfocus: move |_| ontitlefocused.call(()),
            onvaluechange: move |value| title.set(value),
        }
        Textarea { id: "css-grid-note", name: "note", label: "Observations", disabled,
            value: note(), rows: 4, max_length: 400, resize: "vertical",
            onvaluechange: move |value| note.set(value),
        }
        p { class: "css-grid-hint", "Drafts stay here when you choose another asset or rearrange the workspace." }
        Button { id: "css-grid-save", button_type: ButtonType::Submit, disabled, "Save inspection" }
    }
}

#[component]
fn InspectionActivity() -> Element {
    let mut preparation = use_signal(|| String::from("Bring the inspection kit"));
    rsx! {
        h3 { "Before you start" }
        p { "Local checklist for this workspace. Preparation is kept separate from saved inspection notes." }
        Input { id: "css-grid-preparation", label: "Preparation", value: preparation(), max_length: 160,
            onvaluechange: move |value| preparation.set(value),
        }
    }
}

#[component]
fn InspectionWorkspace(generation: usize) -> Element {
    let mut equipment = use_signal(|| String::from("pump"));
    let mut editor_first = use_signal(|| false);
    let mut alternate_id = use_signal(|| false);
    let mut renamed = use_signal(|| false);
    let mut disabled = use_signal(|| false);
    let mut records = use_signal(Vec::<InspectionRecord>::new);
    let mut payload = use_signal(BTreeMap::<String, String>::new);
    let mut message = use_signal(|| String::from("No inspections saved yet."));
    let mut root_node = use_signal(|| None::<Rc<MountedData>>);
    let mut editor_node = use_signal(|| None::<Rc<MountedData>>);
    let mut title_node = use_signal(|| None::<Rc<MountedData>>);
    let mut dimensions = use_signal(|| None::<(f64, f64)>);
    let mut measurement_state = use_signal(|| "ready");
    let mut editor_state = use_signal(|| "ready");
    let root_id = if alternate_id() {
        "css-grid-workspace-grid-alternate"
    } else {
        "css-grid-workspace-grid"
    };
    let selected = equipment();
    let selected_label = if selected == "pump" {
        "Pump P-101"
    } else {
        "Ventilation fan F-204"
    };
    let current_records = records();
    let serialized = serde_json::to_string(&payload()).unwrap_or_default();
    let (width, height) = dimensions().unwrap_or_default();
    rsx! {
        div { id: "css-grid-workspace-owner", "data-generation": generation,
            div { class: "css-grid-workspace-options",
                label { input { id: "css-grid-editor-first", r#type: "checkbox", checked: editor_first(),
                    onchange: move |event: FormEvent| { editor_first.set(event.checked()); measurement_state.set("ready"); dimensions.set(None); },
                } "Editor first" }
                label { input { id: "css-grid-alternate-id", r#type: "checkbox", checked: alternate_id(),
                    onchange: move |event: FormEvent| alternate_id.set(event.checked()),
                } "Use alternate grid ID" }
                label { input { id: "css-grid-rename-area", r#type: "checkbox", checked: renamed(),
                    onchange: move |event: FormEvent| renamed.set(event.checked()),
                } "Rename checklist area" }
                label { input { id: "css-grid-disabled", r#type: "checkbox", checked: disabled(),
                    onchange: move |event: FormEvent| disabled.set(event.checked()),
                } "Pause editing" }
            }
            div { class: "css-grid-workspace-actions",
                Button { id: "css-grid-show-editor", variant: ButtonVariant::Secondary, button_type: ButtonType::Button, disabled: disabled(),
                    onclick: move |_| {
                        let area = editor_node.peek().clone();
                        let title = title_node.peek().clone();
                        spawn(async move {
                            let (Some(area), Some(title)) = (area, title) else { editor_state.set("unavailable"); return; };
                            editor_state.set("requested");
                            // Pinned desktop can apply an operation before its boolean
                            // response fails to decode. Attempt both mounted operations;
                            // the normal input focus event confirms actual focus.
                            let _ = area.scroll_to(ScrollBehavior::Instant).await;
                            if *disabled.peek() { return; }
                            let _ = title.set_focus(true).await;
                        });
                    },
                    "Show editor"
                }
                Button { id: "css-grid-check-space", variant: ButtonVariant::Tertiary, button_type: ButtonType::Button,
                    onclick: move |_| {
                        let node = root_node.peek().clone();
                        spawn(async move {
                            let Some(node) = node else { measurement_state.set("unavailable"); return; };
                            match node.get_client_rect().await {
                                Ok(rect) => { dimensions.set(Some((rect.width(), rect.height()))); measurement_state.set("measured"); }
                                Err(_) => { dimensions.set(None); measurement_state.set("error"); }
                            }
                        });
                    },
                    "Check space"
                }
                span { id: "css-grid-space-status", class: "css-grid-space-status", role: "status", "aria-live": "polite",
                    "data-state": measurement_state(), "data-width": width.to_string(), "data-height": height.to_string(),
                    match measurement_state() {
                        "measured" => format!("Workspace: {width:.0} × {height:.0} px"),
                        "error" | "unavailable" => "Workspace measurement is unavailable.".into(),
                        _ => "Check the current workspace size.".into(),
                    }
                }
            }
            form { id: "css-grid-inspection-form",
                onsubmit: move |event: FormEvent| {
                    event.prevent_default();
                    if *disabled.peek() { return; }
                    let values = event.values().into_iter().filter_map(|(name, value)| match value {
                        dioxus::html::FormValue::Text(value) => Some((name, value)),
                        _ => None,
                    }).collect::<BTreeMap<_, _>>();
                    let Some(record) = inspection_record(&values) else {
                        message.set("Enter an inspection title before saving.".into()); return;
                    };
                    message.set(format!("Inspection saved locally for {}.", if record.equipment == "pump" { "Pump P-101" } else { "Ventilation fan F-204" }));
                    records.write().push(record);
                    payload.set(values);
                },
                CssGrid { id: root_id, class: "css-grid-inspection-layout", style: "gap:16px;align-items:start",
                    templates: workspace_templates(editor_first(), renamed()),
                    onmounted: move |event: MountedEvent| root_node.set(Some(event.data())),
                    GridArea { id: "css-grid-overview", name: "overview", class: "css-grid-overview",
                        h3 { "Equipment round" }
                        p { "Two assets on this local inspection list. Choose an asset, record your observations and save a note for this session." }
                        p { class: "css-grid-selected", "Selected: " strong { "{selected_label}" } }
                    }
                    GridArea { id: "css-grid-equipment", name: "equipment", class: "css-grid-equipment",
                        h3 { "Choose equipment" }
                        div { class: "css-grid-equipment-list", role: "group", "aria-label": "Equipment",
                            Button { id: "css-grid-pump", button_type: ButtonType::Button,
                                variant: if selected == "pump" { ButtonVariant::Primary } else { ButtonVariant::Secondary },
                                onclick: move |_| equipment.set("pump".into()), "Pump P-101"
                            }
                            Button { id: "css-grid-fan", button_type: ButtonType::Button,
                                variant: if selected == "fan" { ButtonVariant::Primary } else { ButtonVariant::Secondary },
                                onclick: move |_| equipment.set("fan".into()), "Fan F-204"
                            }
                        }
                        input { r#type: "hidden", name: "equipment", value: selected.clone(), disabled: disabled() }
                    }
                    GridArea { id: "css-grid-editor", name: "editor", class: "css-grid-editor",
                        onmounted: move |event: MountedEvent| editor_node.set(Some(event.data())),
                        InspectionEditor { disabled: disabled(), ontitlemounted: move |event: MountedEvent| title_node.set(Some(event.data())),
                            ontitlefocused: move |_| editor_state.set("focused"),
                        }
                    }
                    GridArea { id: "css-grid-activity", name: if renamed() { "notes" } else { "activity" }, class: "css-grid-activity",
                        InspectionActivity {}
                    }
                }
            }
            p { id: "css-grid-status", class: "css-grid-status", role: "status", "aria-live": "polite",
                "data-saved-count": current_records.len(), "data-equipment": selected,
                "data-payload": serialized, "data-editor-state": editor_state(), "data-disabled": disabled(),
                "{message}"
            }
            div { class: "css-grid-saved",
                h3 { "Saved inspections" }
                if current_records.is_empty() { p { "No records yet. Saved notes are kept here until you reset the workspace or leave this page." } }
                ol { id: "css-grid-saved-records",
                    for (index, record) in current_records.iter().enumerate() {
                        li { key: "{index}", "data-equipment": record.equipment.clone(), "data-title": record.title.clone(), "data-note": record.note.clone(),
                            strong { "{record.title}" }
                            span { if record.equipment == "pump" { "Pump P-101" } else { "Ventilation fan F-204" } }
                            if !record.note.is_empty() { p { "{record.note}" } }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn CssGridWorkspaceExample() -> Element {
    let mut generation = use_signal(|| 0_usize);
    rsx! {
        style { {STYLES} }
        section { id: "css-grid-workspace", class: "demo-section css-grid-workspace-example",
            h2 { "Responsive inspection workspace" }
            p { class: "css-grid-intro", "Keep an inspection draft beside its equipment list. The layout adapts to available screen space; changes stay in this local example and are never sent to a server." }
            for owner in [generation()] {
                InspectionWorkspace { key: "{owner}", generation: owner }
            }
            Button { id: "css-grid-reset", variant: ButtonVariant::Tertiary, button_type: ButtonType::Button,
                onclick: move |_| generation += 1, "Reset workspace"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inspection_save_validates_native_payload_and_keeps_a_snapshot() {
        let mut values = BTreeMap::from([
            ("equipment".into(), "fan".into()),
            ("title".into(), "Inspect motor guard".into()),
            ("note".into(), "Tighten the lower fastener".into()),
        ]);
        let saved = inspection_record(&values).expect("complete inspection payload");
        values.insert("title".into(), "Next inspection".into());
        assert_eq!(saved.title, "Inspect motor guard");
        assert_eq!(saved.equipment, "fan");
        assert_eq!(saved.note, "Tighten the lower fastener");
        values.insert("equipment".into(), "unknown".into());
        assert!(inspection_record(&values).is_none());
        values.insert("equipment".into(), "pump".into());
        values.insert("title".into(), "  ".into());
        assert!(inspection_record(&values).is_none());
        values.remove("title");
        assert!(inspection_record(&values).is_none());
    }

    #[test]
    fn every_layout_keeps_four_areas_and_updates_the_checklist_name() {
        for editor_first in [false, true] {
            for renamed in [false, true] {
                let templates = workspace_templates(editor_first, renamed);
                for template in [templates.lg, templates.md, templates.sm] {
                    let css = template.unwrap().css();
                    for name in [
                        "overview",
                        "equipment",
                        "editor",
                        if renamed { "notes" } else { "activity" },
                    ] {
                        assert!(css.split(['"', ' ']).any(|area| area == name));
                    }
                    assert!(!css.contains(if renamed { "activity" } else { "notes" }));
                }
            }
        }
    }
}
