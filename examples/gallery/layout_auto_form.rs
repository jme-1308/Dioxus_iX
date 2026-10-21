//! A local inspection form composed with responsive LayoutAuto items.
use dioxus::prelude::*;
use dioxus_ix::*;
use std::{collections::BTreeMap, rc::Rc};

pub const STYLES: &str = include_str!("styles/layout_auto_form.css");

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
struct InspectionRecord {
    equipment: String,
    title: String,
    date: String,
    note: String,
}

fn inspection_record(values: &BTreeMap<String, String>) -> Option<InspectionRecord> {
    let equipment = values.get("equipment")?;
    let title = values.get("title")?;
    let date = values.get("date")?;
    if !matches!(equipment.as_str(), "pump" | "fan")
        || title.trim().is_empty()
        || title.chars().count() > 80
        || IxDate::parse_iso(date).is_none()
    {
        return None;
    }
    Some(InspectionRecord {
        equipment: equipment.clone(),
        title: title.clone(),
        date: date.clone(),
        note: values.get("note").cloned().unwrap_or_default(),
    })
}

#[component]
fn InspectionDraft(
    generation: usize,
    custom: bool,
    wide: bool,
    notes_visible: bool,
    alternate_id: bool,
    onsave: EventHandler<InspectionRecord>,
) -> Element {
    let mut equipment = use_signal(|| "pump".to_string());
    let mut title = use_signal(|| "Inspect drive coupling".to_string());
    let mut date = use_signal(|| "2026-11-03".to_string());
    let mut note = use_signal(String::new);
    let mut error = use_signal(|| false);
    let mut root_node = use_signal(|| None::<Rc<MountedData>>);
    let mut notes_node = use_signal(|| None::<Rc<MountedData>>);
    let mut note_node = use_signal(|| None::<Rc<MountedData>>);
    let mut measurement = use_signal(|| None::<(f64, f64)>);
    let mut measurement_state = use_signal(|| "idle");
    let notes_present = use_memo(move || notes_visible);
    use_effect(move || {
        if !notes_present() {
            notes_node.set(None);
            note_node.set(None);
        }
    });
    let layout = if custom {
        vec![
            AutoLayoutBreakpoint::new("0", 1),
            AutoLayoutBreakpoint::new("560px", 2),
            AutoLayoutBreakpoint::new("960px", 3),
        ]
    } else {
        vec![
            AutoLayoutBreakpoint::new("0", 1),
            AutoLayoutBreakpoint::new("48em", 2),
        ]
    };
    let root_id = if alternate_id {
        "layout-auto-inspection-alt"
    } else {
        "layout-auto-inspection"
    };
    let (width, height) = measurement().unwrap_or_default();
    rsx! {
        div { id: "layout-auto-owner", "data-generation": generation,
            div { class: "layout-auto-form-actions",
                Button { id: "layout-auto-measure", variant: ButtonVariant::Tertiary, button_type: ButtonType::Button,
                    onclick: move |_| {
                        let node = root_node.peek().clone();
                        spawn(async move {
                            let Some(node) = node else { measurement_state.set("unavailable"); return; };
                            let result = node.get_client_rect().await;
                            if !root_node.peek().as_ref().is_some_and(|current| Rc::ptr_eq(current, &node)) { return; }
                            match result {
                                Ok(rect) => { measurement.set(Some((rect.width(), rect.height()))); measurement_state.set("measured"); }
                                Err(_) => { measurement.set(None); measurement_state.set("error"); }
                            }
                        });
                    }, "Measure form"
                }
                Button { id: "layout-auto-show-notes", variant: ButtonVariant::Tertiary, button_type: ButtonType::Button, disabled: !notes_visible,
                    onclick: move |_| {
                        let (area, field) = (notes_node.peek().clone(), note_node.peek().clone());
                        spawn(async move {
                            let (Some(area), Some(field)) = (area, field) else { return; };
                            if !*notes_present.peek() { return; }
                            let _ = area.scroll_to(ScrollBehavior::Instant).await;
                            if !*notes_present.peek() || !note_node.peek().as_ref().is_some_and(|current| Rc::ptr_eq(current, &field)) { return; }
                            // Desktop may apply focus before its bool result fails
                            // to decode; actual native focus remains authoritative.
                            let _ = field.set_focus(true).await;
                        });
                    }, "Go to notes"
                }
                span { id: "layout-auto-measure-status", role: "status", "data-state": measurement_state(), "data-width": width, "data-height": height,
                    match measurement_state() {
                        "measured" => format!("Form: {width:.0} × {height:.0} px"),
                        "error" | "unavailable" => "Form measurement is unavailable.".into(),
                        _ => "Measure the current form area.".into(),
                    }
                }
            }
            form { id: "layout-auto-inspection-form",
                onsubmit: move |event: FormEvent| {
                    event.prevent_default();
                    let values = event.values().into_iter().filter_map(|(name, value)| match value {
                        dioxus::html::FormValue::Text(value) => Some((name, value)), _ => None,
                    }).collect::<BTreeMap<_, _>>();
                    if let Some(record) = inspection_record(&values) { error.set(false); onsave.call(record); }
                    else { error.set(true); }
                },
                LayoutAuto { id: root_id, class: "layout-auto-inspection-grid", layout,
                    onmounted: move |event: MountedEvent| root_node.set(Some(event.data())),
                    LayoutAutoItem { id: "layout-auto-equipment-item", colspan: 1,
                        label { class: "layout-auto-form-field", r#for: "layout-auto-equipment", "Equipment"
                            select { id: "layout-auto-equipment", name: "equipment", class: "ix-input", required: true, value: equipment(),
                                onchange: move |event: FormEvent| equipment.set(event.value()),
                                option { value: "pump", "Cooling pump" }
                                option { value: "fan", "Ventilation fan" }
                            }
                        }
                    }
                    LayoutAutoItem { id: "layout-auto-title-item", colspan: 1,
                        label { class: "layout-auto-form-field", r#for: "layout-auto-title", "Inspection title"
                            input { id: "layout-auto-title", name: "title", class: "ix-input", r#type: "text", required: true, maxlength: "80", value: title(),
                                oninput: move |event: FormEvent| { title.set(event.value()); error.set(false); },
                            }
                        }
                    }
                    LayoutAutoItem { id: "layout-auto-date-item", colspan: 1,
                        label { class: "layout-auto-form-field", r#for: "layout-auto-date", "Planned date"
                            input { id: "layout-auto-date", name: "date", class: "ix-input", r#type: "date", required: true, value: date(),
                                oninput: move |event: FormEvent| { date.set(event.value()); error.set(false); },
                            }
                        }
                    }
                    if notes_visible {
                        LayoutAutoItem { id: "layout-auto-notes-item", class: "layout-auto-notes-item", colspan: if wide { 2 } else { 1 },
                            onmounted: move |event: MountedEvent| notes_node.set(Some(event.data())),
                            label { class: "layout-auto-form-field", r#for: "layout-auto-note", "Optional notes"
                                textarea { id: "layout-auto-note", name: "note", class: "ix-input", rows: "3", value: note(),
                                    oninput: move |event: FormEvent| note.set(event.value()),
                                    onmounted: move |event: MountedEvent| note_node.set(Some(event.data())),
                                }
                            }
                            p { class: "layout-auto-form-help", "Prepare access instructions or parts to check. This form saves a local record; no inspection is booked." }
                        }
                    }
                    LayoutAutoItem { id: "layout-auto-save-item", colspan: 2,
                        Button { id: "layout-auto-save", button_type: ButtonType::Submit, "Save inspection record" }
                        if error() { p { role: "alert", "Choose equipment, an inspection title and a valid date before saving." } }
                    }
                }
            }
        }
    }
}

#[component]
pub fn LayoutAutoFormExample() -> Element {
    let mut custom = use_signal(|| false);
    let mut wide = use_signal(|| true);
    let mut notes_visible = use_signal(|| true);
    let mut alternate_id = use_signal(|| false);
    let mut preparation = use_signal(|| "Check access to the equipment".to_string());
    let mut generation = use_signal(|| 0_usize);
    let mut records = use_signal(Vec::<InspectionRecord>::new);
    let current = records();
    let payload = current
        .last()
        .map(|record| serde_json::to_string(record).unwrap_or_default())
        .unwrap_or_default();
    rsx! {
        style { {STYLES} }
        div { id: "layout-auto-form-example", class: "layout-auto-form-example",
            h3 { "Prepare an inspection" }
            p { class: "layout-auto-form-intro", "Resize the page or change the layout. The same inspection draft stays in place, and saved records keep the submitted values." }
            label { class: "layout-auto-form-field layout-auto-preparation", r#for: "layout-auto-preparation", "Independent preparation"
                input { id: "layout-auto-preparation", class: "ix-input", r#type: "text", value: preparation(),
                    oninput: move |event: FormEvent| preparation.set(event.value()),
                }
            }
            div { class: "layout-auto-form-options",
                label { input { id: "layout-auto-custom", r#type: "checkbox", checked: custom(), onchange: move |event: FormEvent| custom.set(event.checked()) } "Custom viewport columns" }
                label { input { id: "layout-auto-wide", r#type: "checkbox", checked: wide(), onchange: move |event: FormEvent| wide.set(event.checked()) } "Wide notes" }
                label { input { id: "layout-auto-notes-visible", r#type: "checkbox", checked: notes_visible(), onchange: move |event: FormEvent| notes_visible.set(event.checked()) } "Show optional notes" }
                label { input { id: "layout-auto-root-id", r#type: "checkbox", checked: alternate_id(), onchange: move |event: FormEvent| alternate_id.set(event.checked()) } "Alternate form identifier" }
            }
            p { class: "layout-auto-form-help", if custom() { "Custom layout: one column, two from 560px, three from 960px." } else { "Default layout: one column, two from 48em." } }
            for owner in [generation()] {
                InspectionDraft { key: "{owner}", generation: owner, custom: custom(), wide: wide(), notes_visible: notes_visible(), alternate_id: alternate_id(),
                    onsave: move |record| records.write().push(record),
                }
            }
            Button { id: "layout-auto-reset", variant: ButtonVariant::Tertiary, button_type: ButtonType::Button, onclick: move |_| generation += 1, "Reset inspection draft" }
            p { id: "layout-auto-result", class: "layout-auto-result", role: "status", "data-count": current.len(), "data-payload": payload,
                "{current.len()} local inspection record(s) saved. Reset keeps saved records, preparation and display options."
            }
            ol { id: "layout-auto-records", class: "layout-auto-records",
                for (index, record) in current.iter().enumerate() {
                    li { key: "{index}", "data-equipment": record.equipment.clone(), "data-title": record.title.clone(), "data-date": record.date.clone(), "data-note": record.note.clone(),
                        strong { "{record.title}" }
                        span { "{record.equipment} · {record.date}" }
                        if !record.note.is_empty() { span { "{record.note}" } }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fields() -> BTreeMap<String, String> {
        [
            ("equipment", "fan"),
            ("title", " Inspect fan & guard "),
            ("date", "2026-11-03"),
            ("note", "Bring <spare> filter\nKeep access clear"),
        ]
        .into_iter()
        .map(|(key, value)| (key.into(), value.into()))
        .collect()
    }
    #[test]
    fn submitted_record_keeps_exact_values_independently_of_later_draft_changes() {
        let mut values = fields();
        let record = inspection_record(&values).unwrap();
        values.insert("title".into(), "Changed later".into());
        values.insert("note".into(), String::new());
        assert_eq!(record.title, " Inspect fan & guard ");
        assert_eq!(record.note, "Bring <spare> filter\nKeep access clear");
        assert_eq!(record.equipment, "fan");
        assert_eq!(record.date, "2026-11-03");
    }
    #[test]
    fn unavailable_notes_are_optional_but_required_and_invalid_fields_cannot_save() {
        let mut values = fields();
        values.remove("note");
        assert_eq!(inspection_record(&values).unwrap().note, "");
        values.insert("date".into(), "2026-02-30".into());
        assert!(inspection_record(&values).is_none());
        values.insert("date".into(), "2026-11-03".into());
        values.insert("title".into(), "   ".into());
        assert!(inspection_record(&values).is_none());
        values.insert("title".into(), "Inspect guard".into());
        values.insert("equipment".into(), "unknown".into());
        assert!(inspection_record(&values).is_none());
    }
}
