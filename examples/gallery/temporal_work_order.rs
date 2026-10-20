//! Native temporal form ownership for an illustrative local service appointment.
use dioxus::prelude::*;
use dioxus_ix::*;
use std::collections::BTreeMap;
use std::rc::Rc;

const INITIAL_REMINDER: &str = "2026/10/21 09:30:00";

fn today() -> IxDate {
    IxDate::new(2026, 10, 20).unwrap()
}
fn first_date() -> IxDate {
    IxDate::new(2026, 10, 1).unwrap()
}
fn last_date() -> IxDate {
    IxDate::new(2026, 10, 31).unwrap()
}
fn first_time() -> IxTime {
    IxTime::new(8, 0, 0, 0).unwrap()
}
fn last_time() -> IxTime {
    IxTime::new(18, 0, 0, 0).unwrap()
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct AppointmentDraft {
    date: String,
    time: String,
    datetime: String,
}

fn initial_draft() -> AppointmentDraft {
    AppointmentDraft {
        date: String::new(),
        time: String::new(),
        datetime: INITIAL_REMINDER.into(),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct AppointmentRecord {
    owner: String,
    date: String,
    time: String,
    datetime: Option<String>,
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
) -> Result<AppointmentRecord, &'static str> {
    let date = values
        .get("date")
        .filter(|value| !value.trim().is_empty())
        .ok_or(
            "The submitted form has no service date. Choose this form as owner and enter a date.",
        )?;
    let time = values
        .get("time")
        .filter(|value| !value.trim().is_empty())
        .ok_or("The submitted form has no start time. Enter a start time before saving.")?;
    let parsed_date = DateFormat::YmdSlash
        .parse(date)
        .ok_or("Use YYYY/MM/DD for the service date.")?;
    let parsed_time = TimeFormat::Hms
        .parse(time)
        .ok_or("Use HH:MM:SS for the start time.")?;
    if parsed_date < first_date() || parsed_date > last_date() {
        return Err("Choose a service date in October 2026.");
    }
    if parsed_time < first_time() || parsed_time > last_time() {
        return Err("Choose a start time from 08:00:00 through 18:00:00.");
    }
    let datetime = values.get("datetime").cloned();
    if let Some(value) = datetime.as_ref().filter(|value| !value.trim().is_empty()) {
        let reminder = IxDateTime::parse(value, DateFormat::YmdSlash, TimeFormat::Hms)
            .ok_or("Use YYYY/MM/DD HH:MM:SS for the reminder, or leave it empty.")?;
        let first = IxDateTime {
            date: first_date(),
            time: first_time(),
        };
        let last = IxDateTime {
            date: last_date(),
            time: last_time(),
        };
        if reminder < first || reminder > last {
            return Err("Keep the reminder between 1 October 08:00 and 31 October 18:00, 2026.");
        }
    }
    Ok(AppointmentRecord {
        owner: owner.into(),
        date: date.clone(),
        time: time.clone(),
        datetime,
    })
}

#[component]
fn TemporalPreparation() -> Element {
    let mut draft = use_signal(|| "Prepare Motor 001 service".to_string());
    rsx! { label { class: "temporal-preparation", r#for: "temporal-preparation",
        span { "Page preparation" }
        input { id: "temporal-preparation", r#type: "text", maxlength: "160", value: draft(), oninput: move |event: FormEvent| draft.set(event.value()) }
    } }
}

#[component]
fn TemporalAppointmentFields(
    draft: Signal<AppointmentDraft>,
    association: Option<String>,
    reminder_disabled: bool,
) -> Element {
    let mut draft = draft;
    // These native handles share the keyed fields' lifetime, including resets.
    let mut date_input = use_signal(|| None::<Rc<MountedData>>);
    let mut time_input = use_signal(|| None::<Rc<MountedData>>);
    let mut reminder_input = use_signal(|| None::<Rc<MountedData>>);
    let current = draft();
    rsx! {
        div { id: "temporal-fields", class: "temporal-fields",
            div { class: "temporal-edit-actions",
                button { id: "temporal-edit-date", class: "ix-button ix-button-secondary", r#type: "button",
                    onclick: move |_| { if let Some(node) = date_input.peek().clone() { spawn(async move { let _ = node.set_focus(true).await; }); } }, "Edit date"
                }
                button { id: "temporal-edit-time", class: "ix-button ix-button-secondary", r#type: "button",
                    onclick: move |_| { if let Some(node) = time_input.peek().clone() { spawn(async move { let _ = node.set_focus(true).await; }); } }, "Edit time"
                }
                button { id: "temporal-edit-datetime", class: "ix-button ix-button-secondary", r#type: "button", disabled: reminder_disabled,
                    onclick: move |_| { if !reminder_disabled && let Some(node) = reminder_input.peek().clone() { spawn(async move { let _ = node.set_focus(true).await; }); } }, "Edit reminder"
                }
            }
            DateInput { id: "temporal-date", name: "date", form: association.clone(), label: "Service date", today: today(), value: current.date.clone(), format: DateFormat::YmdSlash,
                min_date: first_date(), max_date: last_date(), required: true, calendar_button_label: "Choose service date", placeholder: "YYYY/MM/DD", helper_text: "1–31 October 2026.",
                onmounted: move |event: MountedEvent| date_input.set(Some(event.data())),
                onvaluechange: move |value| draft.write().date = value,
            }
            TimeInput { id: "temporal-time", name: "time", form: association.clone(), label: "Start time", value: current.time.clone(), format: TimeFormat::Hms,
                min_time: first_time(), max_time: last_time(), required: true, time_button_label: "Choose start time", placeholder: "HH:MM:SS", helper_text: "08:00:00–18:00:00.",
                onmounted: move |event: MountedEvent| time_input.set(Some(event.data())),
                onvaluechange: move |value| draft.write().time = value,
            }
            div { class: "temporal-reminder",
                DateTimeInput { id: "temporal-datetime", name: "datetime", form: association.clone(), label: "Follow-up reminder", today: today(), value: current.datetime.clone(),
                    date_format: DateFormat::YmdSlash, time_format: TimeFormat::Hms, min_date: first_date(), max_date: last_date(), min_time: first_time(), max_time: last_time(),
                    disabled: reminder_disabled, calendar_button_label: "Choose follow-up reminder", placeholder: "YYYY/MM/DD HH:MM:SS", helper_text: "Optional: between 1 October 08:00 and 31 October 18:00, 2026.",
                    onmounted: move |event: MountedEvent| reminder_input.set(Some(event.data())),
                    onvaluechange: move |value| draft.write().datetime = value,
                }
            }
        }
    }
}

/// The same date/time controls can submit to either explicitly named native form.
#[component]
pub fn TemporalWorkOrderExample() -> Element {
    let mut draft = use_signal(initial_draft);
    let mut owner = use_signal(|| "primary".to_string());
    let mut disable_datetime = use_signal(|| false);
    let mut saved = use_signal(Vec::<AppointmentRecord>::new);
    let mut payload = use_signal(BTreeMap::<String, String>::new);
    let mut last_form = use_signal(String::new);
    let mut saves = use_signal(|| 0_usize);
    let mut resets = use_signal(|| 0_usize);
    let mut error = use_signal(|| false);
    let mut feedback = use_signal(|| {
        "Choose a service date and start time, then save with the selected form.".to_string()
    });
    let mut controls_epoch = use_signal(|| 0_u64);
    let mut preparation_epoch = use_signal(|| 0_u64);
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
                    "Motor 001 service appointment saved from the {form} form's submitted values."
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
            // Reset also closes and discards the fields' owned picker drafts.
            let next = controls_epoch.peek().wrapping_add(1);
            controls_epoch.set(next);
            let count = resets.peek().saturating_add(1);
            resets.set(count);
            error.set(false);
            feedback.set(format!("The {form} form's draft was reset. Saved appointments and page preparation are retained."));
        } else {
            error.set(false);
            feedback.set(format!(
                "The {form} form has no associated appointment draft to reset."
            ));
        }
    });
    let association = (owner() != "none").then(|| format!("temporal-{}-form", owner()));
    rsx! {
        section { id: "temporal-work-order", class: "demo-section temporal-work-order-example",
            h2 { "Service appointment forms" }
            p { class: "temporal-intro", "Prepare an illustrative local Motor 001 service appointment in October 2026, starting between 08:00 and 18:00. Choose which form receives the appointment; switching ownership keeps the editing draft. No service is contacted." }
            div { class: "temporal-toolbar",
                Button { id: "temporal-restore", variant: ButtonVariant::Secondary, onclick: move |_| {
                    draft.set(initial_draft()); owner.set("primary".into()); disable_datetime.set(false); saved.set(Vec::new()); payload.set(BTreeMap::new()); last_form.set(String::new());
                    saves.set(0); resets.set(0); error.set(false); feedback.set("Choose a service date and start time, then save with the selected form.".into());
                    let next = controls_epoch.peek().wrapping_add(1); controls_epoch.set(next);
                    let next = preparation_epoch.peek().wrapping_add(1); preparation_epoch.set(next);
                }, "Restore appointment example" }
                label { r#for: "temporal-owner", span { "Form owner" }
                    select { id: "temporal-owner", value: owner(), onchange: move |event: FormEvent| {
                        let value = event.value(); if matches!(value.as_str(), "primary" | "alternate" | "none") { owner.set(value); error.set(false); feedback.set("Form ownership changed; the appointment draft is retained.".into()); }
                    },
                        option { value: "primary", selected: owner() == "primary", "Primary form" }
                        option { value: "alternate", selected: owner() == "alternate", "Alternate form" }
                        option { value: "none", selected: owner() == "none", "No form owner" }
                    }
                }
            }
            details { id: "temporal-options", class: "temporal-options",
                summary { "Appointment options" }
                label { input { id: "temporal-disable-datetime", r#type: "checkbox", checked: disable_datetime(), onchange: move |event: FormEvent| disable_datetime.set(event.checked()) } "Disable the reminder and omit it from submission" }
            }
            for generation in [preparation_epoch()] { TemporalPreparation { key: "temporal-preparation-{generation}" } }
            for generation in [controls_epoch()] {
                TemporalAppointmentFields { key: "temporal-controls-{generation}", draft, association: association.clone(), reminder_disabled: disable_datetime() }
            }
            p { class: "temporal-owner-note", "Only the selected form submits or resets these appointment fields. No form owner leaves the fields independent." }
            div { class: "temporal-forms",
                for form in ["primary", "alternate"] {
                    form { key: "{form}", id: format!("temporal-{form}-form"), "data-owner": form,
                        onsubmit: move |event: FormEvent| { event.prevent_default(); submit.call((form.into(), text_payload(event.values()))); },
                        onreset: move |event: FormEvent| { event.prevent_default(); reset.call(form.into()); },
                        h3 { if form == "primary" { "Primary form" } else { "Alternate form" } }
                        div { class: "temporal-form-actions",
                            button { id: format!("temporal-save-{form}"), class: "ix-button ix-button-primary", r#type: "submit", "Save appointment" }
                            button { id: format!("temporal-reset-{form}"), class: "ix-button ix-button-secondary", r#type: "reset", "Reset associated draft" }
                        }
                    }
                }
            }
            output { id: "temporal-result", class: "temporal-result", role: if error() { "alert" } else { "status" },
                "data-owner": owner(), "data-saves": saves().to_string(), "data-resets": resets().to_string(), "data-error": error().to_string(), "data-last-form": last_form(), "data-disable-datetime": disable_datetime().to_string(), "{feedback}"
            }
            div { class: "temporal-saved",
                h3 { "Saved local appointments" }
                if saved.read().is_empty() { p { id: "temporal-empty", "No appointments saved." } }
                for record in saved() {
                    dl { key: "{record.owner}", id: format!("temporal-record-{}", record.owner), "data-owner": record.owner.clone(),
                        dt { "Asset" } dd { "Motor 001" }
                        dt { "Form" } dd { "{record.owner}" }
                        dt { "Service date" } dd { "data-field": "date", "{record.date}" }
                        dt { "Start time" } dd { "data-field": "time", "{record.time}" }
                        dt { "Reminder" } dd { "data-field": "datetime", if let Some(value) = record.datetime { if value.trim().is_empty() { "Not specified" } else { "{value}" } } else { "Omitted from submission" } }
                    }
                }
            }
            details { class: "temporal-payload",
                summary { "Last submitted native values" }
                ul { id: "temporal-last-payload", for (name, value) in payload() { li { key: "{name}", span { "data-name": name.clone(), "{name}: {value}" } } } }
            }
        }
    }
}

pub(crate) const STYLES: &str = r#"
.temporal-work-order-example{min-width:0;max-width:100%}.temporal-work-order-example .temporal-intro{max-width:72ch;line-height:1.55}.temporal-work-order-example .temporal-toolbar{display:flex;align-items:center;flex-wrap:wrap;gap:12px 20px;margin:20px 0 16px}.temporal-work-order-example .temporal-toolbar .ix-button{max-width:100%;white-space:normal}.temporal-work-order-example .temporal-toolbar>label{display:flex;align-items:center;flex-wrap:wrap;gap:8px;min-width:0;font-size:14px;line-height:20px}.temporal-work-order-example .temporal-toolbar select{box-sizing:border-box;min-height:32px;max-width:100%;padding:4px 8px;border:1px solid var(--ix-color-border);border-radius:var(--ix-radius);background:var(--ix-color-surface-1);color:var(--ix-color-text);font:14px/20px var(--ix-font)}.temporal-work-order-example .temporal-options{margin-bottom:24px;border:1px solid var(--ix-color-border);border-radius:var(--ix-radius)}.temporal-work-order-example .temporal-options>summary{padding:10px 12px;font-size:14px;font-weight:600;cursor:pointer}.temporal-work-order-example .temporal-options>label{display:flex;align-items:center;gap:8px;padding:12px;font-size:14px;line-height:20px}.temporal-work-order-example .temporal-options input{flex:none;width:16px;height:16px;margin:0;accent-color:var(--ix-color-primary)}.temporal-work-order-example .temporal-preparation{display:flex;flex-direction:column;gap:8px;width:100%;max-width:480px;min-width:0;margin-bottom:32px;font-size:14px;line-height:20px}.temporal-work-order-example .temporal-preparation input{box-sizing:border-box;width:100%;min-width:0;height:32px;padding:4px 8px;border:1px solid var(--ix-color-border);border-radius:var(--ix-radius);background:var(--ix-color-surface-1);color:var(--ix-color-text);font:14px/20px var(--ix-font);caret-color:var(--ix-color-primary)}.temporal-work-order-example .temporal-fields{display:grid;grid-template-columns:repeat(auto-fit,minmax(min(100%,280px),1fr));align-items:start;gap:24px 32px;max-width:880px;min-width:0}.temporal-work-order-example .temporal-fields>div{min-width:0}.temporal-work-order-example .temporal-reminder{grid-column:1/-1;max-width:480px}.temporal-work-order-example .temporal-owner-note{max-width:72ch;margin:24px 0;font-size:14px;line-height:20px}.temporal-work-order-example .temporal-forms{display:flex;flex-wrap:wrap;gap:24px 40px;max-width:880px}.temporal-work-order-example .temporal-forms>form{min-width:0;max-width:100%}.temporal-work-order-example .temporal-forms h3{margin:0 0 12px}.temporal-work-order-example .temporal-form-actions{display:flex;flex-wrap:wrap;gap:12px}.temporal-work-order-example .temporal-form-actions .ix-button{max-width:100%;white-space:normal}.temporal-work-order-example .temporal-result{display:block;max-width:72ch;margin-top:24px;font-size:14px;line-height:20px;overflow-wrap:anywhere;color:var(--ix-color-text)}.temporal-work-order-example .temporal-saved{margin-top:40px;max-width:880px}.temporal-work-order-example .temporal-saved h3{margin:0 0 12px}.temporal-work-order-example .temporal-saved p{font-size:14px;line-height:20px}.temporal-work-order-example .temporal-saved dl{display:grid;grid-template-columns:minmax(100px,1fr) minmax(0,3fr);gap:8px 16px;margin:0 0 24px;font-size:14px;line-height:20px}.temporal-work-order-example .temporal-saved dt{font-weight:600}.temporal-work-order-example .temporal-saved dd{margin:0;overflow-wrap:anywhere}.temporal-work-order-example .temporal-payload{margin-top:24px;max-width:880px;font-size:14px;line-height:20px}.temporal-work-order-example .temporal-payload>summary{cursor:pointer;min-height:24px}.temporal-work-order-example .temporal-payload li{overflow-wrap:anywhere}.temporal-work-order-example .temporal-preparation input:focus-visible,.temporal-work-order-example .temporal-toolbar select:focus-visible,.temporal-work-order-example .temporal-options>summary:focus-visible,.temporal-work-order-example .temporal-payload>summary:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:2px}.temporal-work-order-example .temporal-options input:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:3px}
.temporal-work-order-example .temporal-edit-actions{display:flex;flex-wrap:wrap;gap:12px;grid-column:1/-1}.temporal-work-order-example .temporal-edit-actions .ix-button{max-width:100%;white-space:normal}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn submitted_dates_are_required_bounded_and_saved_without_draft_substitution() {
        assert!(submitted_record("primary", &BTreeMap::new()).is_err());
        let mut values = text_payload([
            (
                "date".into(),
                dioxus::html::FormValue::Text("2026/10/22".into()),
            ),
            (
                "time".into(),
                dioxus::html::FormValue::Text("10:15:30".into()),
            ),
        ]);
        let record = submitted_record("alternate", &values).expect("complete native values");
        assert_eq!(record.date, "2026/10/22");
        assert_eq!(record.time, "10:15:30");
        assert_eq!(record.datetime, None);
        values.insert("datetime".into(), INITIAL_REMINDER.into());
        assert_eq!(
            submitted_record("primary", &values)
                .unwrap()
                .datetime
                .as_deref(),
            Some(INITIAL_REMINDER)
        );
        values.insert("date".into(), "2026/11/01".into());
        assert!(submitted_record("primary", &values).is_err());
        values.insert("date".into(), "2026/10/22".into());
        values.insert("time".into(), "07:59:59".into());
        assert!(submitted_record("primary", &values).is_err());
    }

    #[test]
    fn initial_temporal_controls_are_external_and_explicitly_owned() {
        let mut dom = VirtualDom::new(TemporalWorkOrderExample);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        for id in [
            "temporal-work-order",
            "temporal-edit-date",
            "temporal-edit-time",
            "temporal-edit-datetime",
            "temporal-date",
            "temporal-time",
            "temporal-datetime",
            "temporal-fields",
            "temporal-primary-form",
            "temporal-alternate-form",
            "temporal-preparation",
        ] {
            assert!(html.contains(&format!("id=\"{id}\"")), "{id}");
        }
        assert_eq!(html.matches("form=\"temporal-primary-form\"").count(), 3);
        for form in ["primary", "alternate"] {
            let start = html
                .find(&format!("<form id=\"temporal-{form}-form\""))
                .expect("native form");
            let body = html[start..].split("</form>").next().unwrap();
            assert!(
                !body.contains("<input")
                    && !body.contains("<select")
                    && !body.contains("<textarea")
            );
        }
        assert!(
            html.contains(INITIAL_REMINDER)
                && html.contains("No appointments saved.")
                && html.contains("Prepare Motor 001 service")
        );
    }
}
