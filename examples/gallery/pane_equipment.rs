//! Local equipment editing with an application-owned draft and close decision.
use dioxus::prelude::*;
use dioxus_ix::*;
use std::collections::BTreeMap;

pub const STYLES: &str = include_str!("styles/pane_equipment.css");

#[derive(Clone, Debug, PartialEq, Eq)]
struct Equipment {
    name: String,
    location: String,
    note: String,
}
impl Default for Equipment {
    fn default() -> Self {
        Self {
            name: "Motor 001".into(),
            location: "Line 1 / Bay 2".into(),
            note: "Inspect coupling and guards.".into(),
        }
    }
}
#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum CloseDecision {
    #[default]
    None,
    Discard,
}
fn equipment_from_form(values: &BTreeMap<String, String>) -> Result<Equipment, &'static str> {
    let record = Equipment {
        name: values
            .get("name")
            .cloned()
            .ok_or("The form is missing the equipment name.")?,
        location: values
            .get("location")
            .cloned()
            .ok_or("The form is missing the location.")?,
        note: values
            .get("note")
            .cloned()
            .ok_or("The form is missing the maintenance note.")?,
    };
    if record.name.trim().is_empty() || record.location.trim().is_empty() {
        return Err("Enter an equipment name and a location before saving.");
    }
    Ok(record)
}

#[component]
fn EquipmentPreparation() -> Element {
    let mut note = use_signal(|| String::from("Gather isolation kit"));
    rsx! {
        div { class: "pane-equipment-preparation",
            Input { id: "pane-equipment-preparation", label: "Inspection preparation", value: note(), max_length: 160,
                onvaluechange: move |value| note.set(value),
            }
        }
    }
}

#[component]
pub fn PaneEquipmentExample() -> Element {
    let mut saved = use_signal(Equipment::default);
    let mut draft = use_signal(Equipment::default);
    let mut expanded = use_signal(|| false);
    let mut floating = use_signal(|| true);
    let mut decision = use_signal(CloseDecision::default);
    let mut validation = use_signal(|| None::<String>);
    let mut status = use_signal(|| String::from("Saved equipment is ready to review."));
    let mut saves = use_signal(|| 0_usize);
    let mut payload = use_signal(BTreeMap::<String, String>::new);
    let name_field = use_input();
    let keep_name_field = name_field.clone();
    let floating_mount = use_focus_mount("pane-equipment-floating");
    let close_editor = use_callback(move |message: String| {
        draft.set(saved.peek().clone());
        expanded.set(false);
        decision.set(CloseDecision::None);
        validation.set(None);
        status.set(message);
    });
    let keep_editing = use_callback(move |_: ()| {
        decision.set(CloseDecision::None);
        status.set("Draft kept. Continue editing the equipment.".into());
        let field = keep_name_field.clone();
        spawn(async move {
            let _ = field.focus_input().await;
        });
    });
    let current = draft();
    let record = saved();
    let dirty = current != record;
    let pending = decision() == CloseDecision::Discard;
    let last_payload = serde_json::to_string(&payload()).unwrap_or_default();
    rsx! {
        style { {STYLES} }
        section { id: "pane-equipment", class: "demo-section pane-equipment-example",
            h2 { "Equipment maintenance" }
            p { class: "pane-equipment-intro", "Review a local equipment record, then edit its details in the side pane. Unsaved changes stay in the editor until you save or discard them. This example does not update a device." }
            div { class: "pane-equipment-options",
                label { onclick: move |event: MouseEvent| event.stop_propagation(),
                    input { id: "pane-equipment-floating", r#type: "checkbox", checked: floating(), onmounted: floating_mount,
                        onchange: move |event: FormEvent| floating.set(event.checked()),
                    }
                    "Floating editor"
                }
                Button { id: "pane-equipment-reset", variant: ButtonVariant::Tertiary, button_type: ButtonType::Button,
                    onclick: move |event: MouseEvent| {
                        event.stop_propagation();
                        saved.set(Equipment::default()); draft.set(Equipment::default());
                        payload.set(BTreeMap::new()); saves.set(0);
                        close_editor.call("Example reset. Your preparation note and display preference are kept.".into());
                    },
                    "Reset example"
                }
            }
            EquipmentPreparation {}
            div { class: "pane-equipment-stage",
                PaneLayout { variant: if floating() { PaneVariant::Floating } else { PaneVariant::Inline },
                    right: rsx! {
                        Pane { heading: "Edit equipment", size: "360px", expanded: expanded(), hide_on_collapse: true, close_on_click_outside: true,
                            aria_label_collapse_close_button: "Close equipment editor",
                            onbeforeexpandedchange: move |event: PaneExpandedChangeEvent| {
                                if !event.expanded && *draft.peek() != *saved.peek() {
                                    event.prevent_default();
                                    decision.set(CloseDecision::Discard);
                                    status.set("Unsaved changes. Keep editing or discard the draft.".into());
                                }
                            },
                            onexpandedchange: move |open| {
                                expanded.set(open);
                                if !open { close_editor.call("Editor closed. Saved equipment is unchanged.".into()); }
                            },
                            form { id: "pane-equipment-form", class: "pane-equipment-form",
                                onsubmit: move |event: FormEvent| {
                                    event.prevent_default();
                                    if !*expanded.peek() || *decision.peek() == CloseDecision::Discard { return; }
                                    let values: BTreeMap<String, String> = event.values().into_iter().filter_map(|(name, value)| match value {
                                        dioxus::html::FormValue::Text(text) => Some((name, text)),
                                        _ => None,
                                    }).collect();
                                    payload.set(values.clone());
                                    match equipment_from_form(&values) {
                                        Ok(record) => {
                                            saved.set(record);
                                            let next = saves.peek().saturating_add(1);
                                            saves.set(next);
                                            close_editor.call("Equipment saved locally.".into());
                                        }
                                        Err(message) => { validation.set(Some(message.into())); status.set("Equipment was not saved. Review the editor.".into()); }
                                    }
                                },
                                if pending {
                                    div { id: "pane-equipment-decision", class: "pane-equipment-decision", role: "alert",
                                        p { "Discard your unsaved changes?" }
                                        div { class: "pane-equipment-actions",
                                            Button { id: "pane-equipment-keep", variant: ButtonVariant::Primary, button_type: ButtonType::Button,
                                                onclick: move |_| keep_editing.call(()), "Keep editing"
                                            }
                                            Button { id: "pane-equipment-discard", variant: ButtonVariant::Secondary, button_type: ButtonType::Button,
                                                onclick: move |_| close_editor.call("Draft discarded. Saved equipment is unchanged.".into()), "Discard changes"
                                            }
                                        }
                                    }
                                }
                                if let Some(message) = validation() {
                                    p { id: "pane-equipment-validation", class: "pane-equipment-validation", role: "alert", "{message}" }
                                }
                                Input { id: "pane-equipment-name", name: "name", label: "Equipment name", value: current.name, required: true, autofocus: true,
                                    controller: name_field.clone(), max_length: 100,
                                    onvaluechange: move |value| { draft.write().name = value; validation.set(None); },
                                }
                                Input { id: "pane-equipment-location", name: "location", label: "Location", value: current.location, required: true, max_length: 100,
                                    onvaluechange: move |value| { draft.write().location = value; validation.set(None); },
                                }
                                Textarea { id: "pane-equipment-note", name: "note", label: "Maintenance note", value: current.note, rows: 3, resize: "vertical", max_length: 500,
                                    onvaluechange: move |value| { draft.write().note = value; validation.set(None); },
                                }
                                div { class: "pane-equipment-actions",
                                    Button { id: "pane-equipment-save", button_type: ButtonType::Submit, disabled: !dirty || pending, "Save equipment" }
                                    Button { id: "pane-equipment-cancel", variant: ButtonVariant::Secondary, button_type: ButtonType::Button,
                                        onclick: move |_| close_editor.call("Edits cancelled. Saved equipment is unchanged.".into()), "Cancel"
                                    }
                                }
                            }
                        }
                    },
                    div { class: "pane-equipment-overview",
                        h3 { "Saved equipment" }
                        dl { id: "pane-equipment-record", class: "pane-equipment-record", "data-name": record.name.clone(), "data-location": record.location.clone(), "data-note": record.note.clone(),
                            div { dt { "Equipment" } dd { id: "pane-equipment-saved-name", "{record.name}" } }
                            div { dt { "Location" } dd { id: "pane-equipment-saved-location", "{record.location}" } }
                            div { dt { "Maintenance note" } dd { id: "pane-equipment-saved-note", "{record.note}" } }
                        }
                        Button { id: "pane-equipment-edit", variant: ButtonVariant::Primary, button_type: ButtonType::Button,
                            onclick: move |event: MouseEvent| {
                                event.stop_propagation();
                                if !*expanded.peek() {
                                    draft.set(saved.peek().clone()); validation.set(None); decision.set(CloseDecision::None);
                                    expanded.set(true); status.set("Editing a draft. The saved record stays unchanged until Save.".into());
                                }
                            },
                            "Edit equipment"
                        }
                    }
                }
            }
            p { id: "pane-equipment-status", class: "pane-equipment-status", role: "status", "aria-live": "polite",
                "data-open": expanded().to_string(), "data-dirty": dirty.to_string(), "data-decision": if pending { "discard" } else { "none" }, "data-saves": saves().to_string(),
                "{status}"
            }
            output { id: "pane-equipment-payload", hidden: true, "{last_payload}" }
        }
    }
}
