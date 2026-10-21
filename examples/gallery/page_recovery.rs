//! Local equipment navigation, retained drafts and a useful missing-page recovery.
use dioxus::prelude::*;
use dioxus_ix::*;
use std::{collections::BTreeMap, rc::Rc};

pub const STYLES: &str = include_str!("styles/page_recovery.css");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Equipment {
    id: &'static str,
    label: &'static str,
    description: &'static str,
    initial_title: &'static str,
}

const EQUIPMENT: [Equipment; 2] = [
    Equipment {
        id: "pump",
        label: "Pump P-101",
        description: "Cooling water circuit · Inlet inspection",
        initial_title: "Inspect inlet seals",
    },
    Equipment {
        id: "fan",
        label: "Fan F-204",
        description: "Ventilation line · Guard inspection",
        initial_title: "Check fan guard",
    },
];

fn equipment_by_id(id: &str) -> Option<Equipment> {
    EQUIPMENT.iter().find(|asset| asset.id == id).copied()
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct InspectionDraft {
    title: String,
    note: String,
}

fn initial_drafts() -> BTreeMap<String, InspectionDraft> {
    EQUIPMENT
        .iter()
        .map(|asset| {
            (
                asset.id.to_owned(),
                InspectionDraft {
                    title: asset.initial_title.into(),
                    note: String::new(),
                },
            )
        })
        .collect()
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SavedInspection {
    equipment: String,
    title: String,
    note: String,
}

fn submitted_inspection(values: &BTreeMap<String, String>) -> Option<SavedInspection> {
    let equipment = values.get("equipment")?;
    equipment_by_id(equipment)?;
    let title = values.get("title")?;
    if title.trim().is_empty() {
        return None;
    }
    Some(SavedInspection {
        equipment: equipment.clone(),
        title: title.clone(),
        note: values.get("note")?.clone(),
    })
}

#[component]
fn RecoveryPreparation() -> Element {
    let mut preparation = use_signal(|| String::from("Bring the inspection kit"));
    rsx! {
        div { class: "page-recovery-preparation",
            Input { id: "page-recovery-preparation", label: "Preparation", value: preparation(), max_length: 160,
                helper_text: "Your checklist stays here while you look up equipment.",
                onvaluechange: move |value| preparation.set(value),
            }
        }
    }
}

#[component]
fn EquipmentPage(
    asset: Equipment,
    draft: InspectionDraft,
    disabled: bool,
    scrollable: bool,
    ontitlechange: EventHandler<String>,
    onnotechange: EventHandler<String>,
    onsubmit: EventHandler<FormEvent>,
    oncontentmounted: EventHandler<MountedEvent>,
    onbodymounted: EventHandler<MountedEvent>,
    ontitlemounted: EventHandler<MountedEvent>,
    onreview: EventHandler<()>,
    onformstart: EventHandler<()>,
) -> Element {
    let header = rsx! {
        ContentHeader { title: asset.label, subtitle: asset.description,
            Button { id: "page-recovery-review", button_type: ButtonType::Button, variant: ButtonVariant::Tertiary,
                onclick: move |_| onreview.call(()), "Review equipment"
            }
            if scrollable {
                Button { id: "page-recovery-form-start", button_type: ButtonType::Button, variant: ButtonVariant::Secondary,
                    disabled, onclick: move |_| onformstart.call(()), "Form start"
                }
                Button { id: "page-recovery-save", button_type: ButtonType::Submit, form: "page-recovery-editor-form",
                    disabled, "Save inspection"
                }
            }
        }
    };
    rsx! {
        Content { id: "page-recovery-content", class: "page-recovery-content",
            style: if scrollable { "min-height:0;height:var(--page-recovery-editor-height,380px)" } else { "min-height:0" }, tabindex: "-1",
            "aria-label": "Equipment editor", "data-equipment": asset.id,
            onmounted: oncontentmounted, onbodymounted, scrollable, header,
            form { id: "page-recovery-editor-form", onsubmit,
                input { r#type: "hidden", name: "equipment", value: asset.id, disabled }
                Input { id: "page-recovery-title", name: "title", label: "Inspection title", required: true,
                    max_length: 120, value: draft.title, disabled,
                    onmounted: ontitlemounted, onvaluechange: ontitlechange,
                }
                Textarea { id: "page-recovery-note", name: "note", label: "Observations", rows: 3,
                    max_length: 400, value: draft.note, disabled, resize: "vertical", onvaluechange: onnotechange,
                }
                p { class: "page-recovery-hint", "Each asset keeps its own draft. Saved inspections are snapshots of the fields submitted below." }
                if !scrollable {
                    Button { id: "page-recovery-save", button_type: ButtonType::Submit, disabled, "Save inspection" }
                }
            }
        }
    }
}

#[component]
fn RecoveryWorkspace(generation: usize) -> Element {
    let mut query = use_signal(|| String::from("pump"));
    let mut route = use_signal(|| String::from("pump"));
    let mut last_known = use_signal(|| String::from("pump"));
    let mut drafts = use_signal(initial_drafts);
    let mut disabled = use_signal(|| false);
    let mut scrollable = use_signal(|| false);
    let mut saved = use_signal(Vec::<SavedInspection>::new);
    let mut payload = use_signal(BTreeMap::<String, String>::new);
    let mut message = use_signal(|| String::from("No inspections saved yet."));
    let mut focus_target = use_signal(|| None::<(String, Rc<MountedData>)>);
    let mut content_node = use_signal(|| None::<(String, Rc<MountedData>)>);
    let mut body_node = use_signal(|| None::<(String, Rc<MountedData>)>);
    let mut focus_requested = use_signal(|| false);
    let navigate = use_callback(move |requested: String| {
        let requested = requested.trim().to_ascii_lowercase();
        query.set(requested.clone());
        if equipment_by_id(&requested).is_some() {
            last_known.set(requested.clone());
        }
        if *route.peek() == requested {
            let node = focus_target
                .peek()
                .as_ref()
                .filter(|(owner, _)| owner == &requested)
                .map(|(_, node)| node.clone());
            if let Some(node) = node {
                spawn(async move {
                    if *route.peek() == requested {
                        let _ = node.set_focus(true).await;
                    }
                });
            }
            return;
        }
        focus_target.set(None);
        content_node.set(None);
        body_node.set(None);
        focus_requested.set(true);
        route.set(requested);
    });
    let register_target = use_callback(move |(owner, event): (String, MountedEvent)| {
        if *route.peek() != owner {
            return;
        }
        let node = event.data();
        focus_target.set(Some((owner.clone(), node.clone())));
        if *focus_requested.peek() {
            focus_requested.set(false);
            spawn(async move {
                if *route.peek() == owner {
                    let _ = node.set_focus(true).await;
                }
            });
        }
    });
    let current_route = route();
    let asset = equipment_by_id(&current_route);
    let records = saved();
    let serialized = serde_json::to_string(&payload()).unwrap_or_default();
    rsx! {
        div { id: "page-recovery-owner", "data-generation": generation,
            form { id: "page-recovery-lookup-form", class: "page-recovery-lookup",
                onsubmit: move |event: FormEvent| {
                    event.prevent_default();
                    let query = event.values().into_iter().find_map(|(name, value)| match value {
                        dioxus::html::FormValue::Text(value) if name == "equipment-query" => Some(value),
                        _ => None,
                    });
                    if let Some(value) = query.filter(|value| !value.trim().is_empty()) { navigate.call(value); }
                },
                Input { id: "page-recovery-query", name: "equipment-query", label: "Equipment identifier", required: true,
                    value: query(), max_length: 48, helper_text: "Try pump or fan. Another identifier opens the recovery page.",
                    onvaluechange: move |value| query.set(value),
                }
                Button { id: "page-recovery-lookup", button_type: ButtonType::Submit, "Open equipment" }
            }
            RecoveryPreparation {}
            label { class: "page-recovery-option",
                input { id: "page-recovery-disabled", r#type: "checkbox", checked: disabled(),
                    onchange: move |event: FormEvent| disabled.set(event.checked()),
                }
                "Pause editing"
            }
            label { class: "page-recovery-option page-recovery-scroll-option",
                input { id: "page-recovery-scrollable", r#type: "checkbox", checked: scrollable(),
                    onchange: move |event: FormEvent| scrollable.set(event.checked()),
                }
                "Keep header visible"
            }
            div { class: "page-recovery-page-area",
                if let Some(asset) = asset {
                    for owner in [asset.id] {
                        EquipmentPage { key: "{owner}", asset,
                            draft: drafts().get(owner).cloned().expect("known assets have a draft"), disabled: disabled(), scrollable: scrollable(),
                            ontitlechange: move |value| { if let Some(draft) = drafts.write().get_mut(owner) { draft.title = value; } },
                            onnotechange: move |value| { if let Some(draft) = drafts.write().get_mut(owner) { draft.note = value; } },
                            ontitlemounted: move |event: MountedEvent| register_target.call((owner.into(), event)),
                            oncontentmounted: move |event: MountedEvent| {
                                if route.peek().as_str() == owner { content_node.set(Some((owner.into(), event.data()))); }
                            },
                            onbodymounted: move |event: MountedEvent| {
                                if route.peek().as_str() == owner { body_node.set(Some((owner.into(), event.data()))); }
                            },
                            onreview: move |_| {
                                let node = content_node.peek().as_ref().filter(|(current, _)| current == owner).map(|(_, node)| node.clone());
                                if let Some(node) = node { spawn(async move { if route.peek().as_str() == owner { let _ = node.set_focus(true).await; } }); }
                            },
                            onformstart: move |_| {
                                if *disabled.peek() || route.peek().as_str() != owner { return; }
                                let body = body_node.peek().as_ref().filter(|(current, _)| current == owner).map(|(_, node)| node.clone());
                                let title = focus_target.peek().as_ref().filter(|(current, _)| current == owner).map(|(_, node)| node.clone());
                                if let (Some(body), Some(title)) = (body, title) {
                                    spawn(async move {
                                        if *disabled.peek() || route.peek().as_str() != owner { return; }
                                        if !body_node.peek().as_ref().is_some_and(|(current, node)| current == owner && Rc::ptr_eq(node, &body)) { return; }
                                        let _ = body.scroll(dioxus::html::geometry::PixelsVector2D::zero(), ScrollBehavior::Instant).await;
                                        if *disabled.peek() || route.peek().as_str() != owner { return; }
                                        if !focus_target.peek().as_ref().is_some_and(|(current, node)| current == owner && Rc::ptr_eq(node, &title)) { return; }
                                        let _ = title.set_focus(true).await;
                                    });
                                }
                            },
                            onsubmit: move |event: FormEvent| {
                                event.prevent_default();
                                if *disabled.peek() || route.peek().as_str() != owner { return; }
                                let values = event.values().into_iter().filter_map(|(name, value)| match value {
                                    dioxus::html::FormValue::Text(value) => Some((name, value)), _ => None,
                                }).collect::<BTreeMap<_, _>>();
                                let Some(record) = submitted_inspection(&values).filter(|record| record.equipment == owner) else {
                                    message.set("Enter an inspection title before saving.".into()); return;
                                };
                                saved.write().push(record);
                                payload.set(values);
                                message.set(format!("Inspection saved locally for {}.", asset.label));
                            },
                        }
                    }
                } else {
                    for missing in [current_route.clone()] {
                        InfoPage { key: "{missing}", id: "page-recovery-missing", class: "page-recovery-missing", style: "min-height:0", tabindex: "-1",
                            "aria-label": "Equipment unavailable", "data-missing-id": missing.clone(),
                            onmounted: { let owner = missing.clone(); move |event: MountedEvent| register_target.call((owner.clone(), event)) },
                            title_text: "Equipment not found",
                            copy_text: format!("There is no equipment named “{missing}” in this local list."),
                            instructions: "Check the identifier, return to your previous equipment or open Pump P-101. Your drafts and preparation are kept.",
                            actions: rsx! {
                                Button { id: "page-recovery-return", button_type: ButtonType::Button,
                                    onclick: move |_| navigate.call(last_known()), "Return to previous equipment"
                                }
                                Button { id: "page-recovery-open-pump", button_type: ButtonType::Button, variant: ButtonVariant::Secondary,
                                    onclick: move |_| navigate.call("pump".into()), "Open Pump P-101"
                                }
                            },
                        }
                    }
                }
            }
            p { id: "page-recovery-status", class: "page-recovery-status", role: "status", "aria-live": "polite",
                "data-route": current_route, "data-state": if asset.is_some() { "ready" } else { "missing" },
                "data-equipment": last_known(), "data-saved-count": records.len(), "data-payload": serialized,
                "data-disabled": disabled(), "data-scrollable": scrollable(), "{message}"
            }
            div { class: "page-recovery-saved",
                h3 { "Saved inspections" }
                if records.is_empty() { p { "Saved notes appear here for this session. Resetting the workspace or leaving this page clears them." } }
                ol { id: "page-recovery-saved-records",
                    for (index, record) in records.iter().enumerate() {
                        li { key: "{index}", "data-equipment": record.equipment.clone(), "data-title": record.title.clone(), "data-note": record.note.clone(),
                            strong { "{record.title}" }
                            span { if record.equipment == "pump" { "Pump P-101" } else { "Fan F-204" } }
                            if !record.note.is_empty() { p { "{record.note}" } }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn PageRecoveryExample() -> Element {
    let mut generation = use_signal(|| 0_usize);
    rsx! {
        style { {STYLES} }
        section { id: "page-recovery", class: "demo-section page-recovery-example",
            h2 { "Equipment lookup and recovery" }
            p { class: "page-recovery-intro", "Find an asset, keep its inspection draft and recover from an unknown identifier. This example uses two local records; it does not contact equipment or a server." }
            for owner in [generation()] { RecoveryWorkspace { key: "{owner}", generation: owner } }
            Button { id: "page-recovery-reset", button_type: ButtonType::Button, variant: ButtonVariant::Tertiary,
                onclick: move |_| generation += 1, "Reset workspace"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saved_inspections_validate_equipment_and_capture_submitted_fields() {
        let mut values = BTreeMap::from([
            ("equipment".into(), "fan".into()),
            ("title".into(), "Inspect motor guard".into()),
            ("note".into(), "Tighten the lower fastener".into()),
        ]);
        let saved = submitted_inspection(&values).unwrap();
        values.insert("title".into(), "Next draft".into());
        assert_eq!(saved.title, "Inspect motor guard");
        assert_eq!(saved.note, "Tighten the lower fastener");
        assert_eq!(saved.equipment, "fan");
        values.insert("equipment".into(), "unknown".into());
        assert!(submitted_inspection(&values).is_none());
        values.insert("equipment".into(), "pump".into());
        values.insert("title".into(), "  ".into());
        assert!(submitted_inspection(&values).is_none());
        values.remove("title");
        assert!(submitted_inspection(&values).is_none());
    }
}
