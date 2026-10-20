//! Local work orders whose branded controls explicitly belong to a separate form.
use dioxus::prelude::*;
use dioxus_ix::*;
use std::collections::BTreeMap;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
struct WorkOrderDraft {
    title: String,
    asset: String,
    quantity: String,
    notes: String,
    acknowledged: bool,
    urgent: bool,
    priority: f64,
    shift: String,
}

fn initial_draft() -> WorkOrderDraft {
    WorkOrderDraft {
        title: String::new(),
        asset: String::new(),
        quantity: "2".into(),
        notes: "Check drive coupling".into(),
        acknowledged: false,
        urgent: false,
        priority: 25.0,
        shift: "day".into(),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct WorkOrderRecord {
    owner: String,
    title: String,
    asset: String,
    quantity: String,
    notes: Option<String>,
    acknowledged: bool,
    urgent: bool,
    priority: String,
    shift: String,
}

fn text_payload(
    values: impl IntoIterator<Item = (String, dioxus::html::FormValue)>,
) -> BTreeMap<String, String> {
    values
        .into_iter()
        .filter_map(|(name, value)| match value {
            dioxus::html::FormValue::Text(text) => Some((name, text)),
            _ => None,
        })
        .collect()
}

fn submitted_record(
    owner: &str,
    values: &BTreeMap<String, String>,
) -> Result<WorkOrderRecord, &'static str> {
    let required = |name: &str| {
        values.get(name).filter(|text| !text.trim().is_empty()).cloned().ok_or("The submitted form is missing required work-order values. Associate the controls with this form and complete the title and asset.")
    };
    let title = required("title")?;
    let asset = required("asset")?;
    let quantity = required("quantity")?;
    let priority = required("priority")?;
    let shift = required("shift")?;
    if !matches!(asset.as_str(), "motor-001" | "pump-002")
        || !matches!(shift.as_str(), "day" | "night")
    {
        return Err("Choose one of the sample assets and a work shift before saving.");
    }
    if !quantity.parse::<f64>().is_ok_and(|value| {
        value.is_finite() && value.fract() == 0.0 && (1.0..=20.0).contains(&value)
    }) || !priority
        .parse::<f64>()
        .is_ok_and(|value| value.is_finite() && (0.0..=100.0).contains(&value))
    {
        return Err("Use a quantity from 1 to 20 and a priority from 0 to 100.");
    }
    Ok(WorkOrderRecord {
        owner: owner.into(),
        title,
        asset,
        quantity,
        notes: values.get("notes").cloned(),
        acknowledged: values.contains_key("acknowledged"),
        urgent: values.contains_key("urgent"),
        priority,
        shift,
    })
}

#[component]
fn WorkOrderPreparation() -> Element {
    let mut draft = use_signal(|| "Prepare the local work order".to_string());
    rsx! { label { class: "external-preparation", r#for: "external-preparation",
        span { "Page preparation" }
        input { id: "external-preparation", r#type: "text", maxlength: "160", value: draft(), oninput: move |event: FormEvent| draft.set(event.value()) }
    } }
}

#[component]
fn WorkOrderFields(
    draft: Signal<WorkOrderDraft>,
    association: Option<String>,
    disable_notes: bool,
) -> Element {
    let mut draft = draft;
    // These native handles belong to this keyed field scope, including resets.
    let mut title_input = use_signal(|| None::<Rc<MountedData>>);
    let mut quantity_input = use_signal(|| None::<Rc<MountedData>>);
    let mut notes_input = use_signal(|| None::<Rc<MountedData>>);
    let current = draft();
    rsx! {
        div { class: "external-toolbar",
            Button { id: "external-edit-title", variant: ButtonVariant::Secondary, button_type: ButtonType::Button,
                onclick: move |_| { if let Some(node) = title_input.peek().clone() { spawn(async move { let _ = node.set_focus(true).await; }); } }, "Edit title"
            }
            Button { id: "external-edit-quantity", variant: ButtonVariant::Secondary, button_type: ButtonType::Button,
                onclick: move |_| { if let Some(node) = quantity_input.peek().clone() { spawn(async move { let _ = node.set_focus(true).await; }); } }, "Edit inspection points"
            }
            Button { id: "external-edit-notes", variant: ButtonVariant::Secondary, button_type: ButtonType::Button, disabled: disable_notes,
                onclick: move |_| { if !disable_notes && let Some(node) = notes_input.peek().clone() { spawn(async move { let _ = node.set_focus(true).await; }); } }, "Edit notes"
            }
        }
        div { id: "external-fields", class: "external-fields",
            Input { id: "external-title", label: "Work order title", name: "title", form: association.clone(), required: true, value: current.title.clone(),
                placeholder: "For example, inspect Motor 001", onmounted: move |event: MountedEvent| title_input.set(Some(event.data())), onvaluechange: move |value| draft.write().title = value,
            }
            Select { id: "external-asset", label: "Asset", name: "asset", form: association.clone(), required: true, value: current.asset.clone(),
                options: vec![SelectOption::new("motor-001", "Motor 001"), SelectOption::new("pump-002", "Pump 002")], allow_clear: true,
                onvaluechange: move |value| draft.write().asset = value,
            }
            NumberInput { id: "external-quantity", label: "Inspection points", name: "quantity", form: association.clone(), min: 1.0, max: 20.0, step: 1.0, required: true, value: current.quantity.clone(),
                onmounted: move |event: MountedEvent| quantity_input.set(Some(event.data())),
                onvaluechange: move |value| draft.write().quantity = value,
            }
            div { id: "external-priority",
                Slider { label: "Scheduling priority", name: "priority", form: association.clone(), min: 0.0, max: 100.0, step: 5.0, value: current.priority, show_value: true,
                    onvaluechange: move |value| draft.write().priority = value,
                }
            }
            div { id: "external-shift",
                RadioGroup { label: "Work shift", name: "shift", form: association.clone(), required: true, value: current.shift.clone(),
                    options: vec![SelectOption::new("day", "Day shift"), SelectOption::new("night", "Night shift")],
                    onvaluechange: move |value| draft.write().shift = value,
                }
            }
            div { class: "external-flags",
                Checkbox { id: "external-ack", label: "Preparation reviewed", name: "acknowledged", value: "on", form: association.clone(), checked: current.acknowledged,
                    oncheckedchange: move |value| draft.write().acknowledged = value,
                }
                Toggle { id: "external-urgent", label: "Urgent work", name: "urgent", value: "on", form: association.clone(), checked: current.urgent,
                    oncheckedchange: move |value| draft.write().urgent = value,
                }
            }
            div { class: "external-notes",
                Textarea { id: "external-notes", label: "Work notes", name: "notes", form: association.clone(), disabled: disable_notes, value: current.notes.clone(),
                    onmounted: move |event: MountedEvent| notes_input.set(Some(event.data())),
                    onvaluechange: move |value| draft.write().notes = value,
                }
            }
        }
    }
}

/// Explicit native form association keeps layout and form ownership independent.
#[component]
pub fn ExternalWorkOrderExample() -> Element {
    let mut draft = use_signal(initial_draft);
    let mut owner = use_signal(|| "primary".to_string());
    let mut disable_notes = use_signal(|| false);
    let mut saved = use_signal(Vec::<WorkOrderRecord>::new);
    let mut payload = use_signal(BTreeMap::<String, String>::new);
    let mut last_form = use_signal(String::new);
    let mut saves = use_signal(|| 0_usize);
    let mut resets = use_signal(|| 0_usize);
    let mut feedback = use_signal(|| {
        "Complete the title and asset, then save with the selected form.".to_string()
    });
    let mut error = use_signal(|| false);
    let mut epoch = use_signal(|| 0_u64);
    let mut controls_epoch = use_signal(|| 0_u64);
    let submit = use_callback(move |(form, values): (String, BTreeMap<String, String>)| {
        last_form.set(form.clone());
        payload.set(values.clone());
        match submitted_record(&form, &values) {
            Ok(record) => {
                let mut next = saved.peek().clone();
                if let Some(current) = next.iter_mut().find(|record| record.owner == form) {
                    *current = record;
                } else {
                    next.push(record);
                }
                saved.set(next);
                let count = saves.peek().saturating_add(1);
                saves.set(count);
                error.set(false);
                feedback.set(format!(
                    "Work order saved from the {form} form's submitted values."
                ));
            }
            Err(message) => {
                error.set(true);
                feedback.set(message.into());
            }
        }
    });
    let reset = use_callback(move |form: String| {
        if *owner.peek() == form {
            draft.set(initial_draft());
            // The child fields own native validation-attempt state.
            let next = controls_epoch.peek().wrapping_add(1);
            controls_epoch.set(next);
            let count = resets.peek().saturating_add(1);
            resets.set(count);
            error.set(false);
            feedback.set(format!("The {form} form's draft was reset. Saved work orders and page preparation are retained."));
        } else {
            error.set(false);
            feedback.set(format!("The {form} form has no associated draft to reset."));
        }
    });
    let association = (owner() != "none").then(|| format!("external-{}-form", owner()));
    rsx! {
        section { id: "external-work-order", class: "demo-section external-work-order-example",
            h2 { "Work order form ownership" }
            p { class: "external-intro", "Prepare an illustrative local work order in a separate editing area, then submit it with its chosen form. Switching the form keeps the draft. Saved details come from the native form submission." }
            div { class: "external-toolbar",
                Button { id: "external-restore", variant: ButtonVariant::Secondary, onclick: move |_| {
                    draft.set(initial_draft()); owner.set("primary".into()); disable_notes.set(false); saved.set(Vec::new()); payload.set(BTreeMap::new()); last_form.set(String::new());
                    saves.set(0); resets.set(0); error.set(false); feedback.set("Complete the title and asset, then save with the selected form.".into());
                    let next = epoch.peek().wrapping_add(1); epoch.set(next);
                    let next = controls_epoch.peek().wrapping_add(1); controls_epoch.set(next);
                }, "Restore work order example" }
                label { r#for: "external-owner", span { "Form owner" }
                    select { id: "external-owner", value: owner(), onchange: move |event: FormEvent| {
                        let value = event.value(); if matches!(value.as_str(), "primary" | "alternate" | "none") { owner.set(value); error.set(false); feedback.set("Form ownership changed; the editing draft is retained.".into()); }
                    },
                        option { value: "primary", selected: owner() == "primary", "Primary form" }
                        option { value: "alternate", selected: owner() == "alternate", "Alternate form" }
                        option { value: "none", selected: owner() == "none", "No form owner" }
                    }
                }
            }
            details { id: "external-options", class: "external-options",
                summary { "Submission options" }
                label { input { id: "external-disable-notes", r#type: "checkbox", checked: disable_notes(), onchange: move |event: FormEvent| disable_notes.set(event.checked()) } "Disable notes and omit them from submission" }
            }
            for generation in [epoch()] { WorkOrderPreparation { key: "external-preparation-{generation}" } }
            for generation in [controls_epoch()] {
                WorkOrderFields { key: "external-controls-{generation}", draft, association: association.clone(), disable_notes: disable_notes() }
            }
            p { class: "external-owner-note", "Only the selected form submits or resets this editing area. No form owner leaves the controls independent." }
            div { class: "external-forms",
                for form in ["primary", "alternate"] {
                    form { key: "{form}", id: format!("external-{form}-form"), "data-owner": form,
                        onsubmit: move |event: FormEvent| { event.prevent_default(); submit.call((form.into(), text_payload(event.values()))); },
                        onreset: move |event: FormEvent| { event.prevent_default(); reset.call(form.into()); },
                        h3 { if form == "primary" { "Primary form" } else { "Alternate form" } }
                        div { class: "external-form-actions",
                            button { id: format!("external-save-{form}"), class: "ix-button ix-button-primary", r#type: "submit", "Save work order" }
                            button { id: format!("external-reset-{form}"), class: "ix-button ix-button-secondary", r#type: "reset", "Reset associated draft" }
                        }
                    }
                }
            }
            output { id: "external-result", class: "external-result", role: if error() { "alert" } else { "status" },
                "data-owner": owner(), "data-saves": saves().to_string(), "data-resets": resets().to_string(), "data-disable-notes": disable_notes().to_string(), "data-last-form": last_form(), "data-error": error().to_string(), "{feedback}"
            }
            div { class: "external-saved",
                h3 { "Saved local work orders" }
                if saved.read().is_empty() { p { id: "external-empty", "No work orders saved." } }
                for record in saved() {
                    dl { key: "{record.owner}", id: format!("external-record-{}", record.owner), "data-owner": record.owner.clone(),
                        dt { "Form" } dd { "{record.owner}" }
                        dt { "Title" } dd { "data-field": "title", "{record.title}" }
                        dt { "Asset" } dd { "data-field": "asset", "{record.asset}" }
                        dt { "Inspection points" } dd { "data-field": "quantity", "{record.quantity}" }
                        dt { "Notes" } dd { "data-field": "notes", if let Some(notes) = record.notes { "{notes}" } else { "Omitted from submission" } }
                        dt { "Preparation reviewed" } dd { "data-field": "acknowledged", "{record.acknowledged}" }
                        dt { "Urgent" } dd { "data-field": "urgent", "{record.urgent}" }
                        dt { "Priority" } dd { "data-field": "priority", "{record.priority}" }
                        dt { "Shift" } dd { "data-field": "shift", "{record.shift}" }
                    }
                }
            }
            details { class: "external-payload",
                summary { "Last submitted native values" }
                ul { id: "external-last-payload",
                    for (name, value) in payload() { li { key: "{name}", span { "data-name": name.clone(), "{name}: {value}" } } }
                }
            }
        }
    }
}

pub(crate) const STYLES: &str = include_str!("styles/external_work_order.css");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saved_record_requires_submitted_values_and_preserves_optional_omission() {
        assert!(submitted_record("primary", &BTreeMap::new()).is_err());
        let values = text_payload([
            (
                "title".into(),
                dioxus::html::FormValue::Text("  Inspect coupling  ".into()),
            ),
            (
                "asset".into(),
                dioxus::html::FormValue::Text("motor-001".into()),
            ),
            ("quantity".into(), dioxus::html::FormValue::Text("3".into())),
            (
                "priority".into(),
                dioxus::html::FormValue::Text("75".into()),
            ),
            (
                "shift".into(),
                dioxus::html::FormValue::Text("night".into()),
            ),
            (
                "acknowledged".into(),
                dioxus::html::FormValue::Text("on".into()),
            ),
        ]);
        let record = submitted_record("alternate", &values).expect("complete native values");
        assert_eq!(record.title, "  Inspect coupling  ");
        assert_eq!(record.owner, "alternate");
        assert_eq!(record.notes, None);
        assert!(record.acknowledged && !record.urgent);
    }

    #[test]
    fn initial_controls_explicitly_associate_with_button_only_forms() {
        let mut dom = VirtualDom::new(ExternalWorkOrderExample);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        for id in [
            "external-work-order",
            "external-title",
            "external-asset",
            "external-quantity",
            "external-notes",
            "external-ack",
            "external-urgent",
            "external-priority",
            "external-shift",
            "external-primary-form",
            "external-alternate-form",
        ] {
            assert!(html.contains(&format!("id=\"{id}\"")), "{id}");
        }
        assert!(html.matches("form=\"external-primary-form\"").count() >= 8);
        for form in ["primary", "alternate"] {
            let start = html
                .find(&format!("<form id=\"external-{form}-form\""))
                .expect("native form");
            let body = html[start..].split("</form>").next().unwrap();
            assert!(
                !body.contains("<input")
                    && !body.contains("<select")
                    && !body.contains("<textarea")
            );
        }
        assert!(
            html.contains("No work orders saved.") && html.contains("Prepare the local work order")
        );
    }
}
