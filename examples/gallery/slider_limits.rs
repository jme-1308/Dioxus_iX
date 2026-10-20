//! A local fan setpoint form with application-owned endpoint actions.
use dioxus::prelude::*;
use dioxus_ix::*;
use std::collections::BTreeMap;

const INITIAL_NOTE: &str = "Prepare fan inspection";

#[derive(Clone, Debug, PartialEq, Eq)]
struct SavedSetpoint {
    value: String,
    note: String,
}

fn submitted_setpoint(
    values: impl IntoIterator<Item = (String, dioxus::html::FormValue)>,
) -> Result<(SavedSetpoint, BTreeMap<String, String>), &'static str> {
    let text: BTreeMap<_, _> = values
        .into_iter()
        .filter_map(|(name, value)| match value {
            dioxus::html::FormValue::Text(text) => Some((name, text)),
            _ => None,
        })
        .collect();
    let value = text
        .get("setpoint")
        .ok_or("The form did not submit a setpoint.")?
        .clone();
    let note = text
        .get("note")
        .ok_or("The form did not submit the preparation note.")?
        .clone();
    if !value.parse::<f64>().is_ok_and(|value| {
        value.is_finite() && (0.0..=100.0).contains(&value) && value % 5.0 == 0.0
    }) {
        return Err("Choose a setpoint from 0 to 100 in steps of 5.");
    }
    Ok((SavedSetpoint { value, note }, text))
}

#[component]
pub fn SliderLimitsExample() -> Element {
    let mut value = use_signal(|| 25.0);
    let mut note = use_signal(|| INITIAL_NOTE.to_string());
    let mut disabled = use_signal(|| false);
    let mut saved = use_signal(|| None::<SavedSetpoint>);
    let mut payload = use_signal(BTreeMap::<String, String>::new);
    let mut saves = use_signal(|| 0_usize);
    let mut resets = use_signal(|| 0_usize);
    let mut error = use_signal(|| false);
    let mut feedback = use_signal(|| "No local setpoint saved.".to_string());
    rsx! {
        section { id: "slider-limits", class: "demo-section slider-limits-example",
            h2 { "Fan setpoint limits" }
            p { class: "slider-limits-intro", "Prepare an illustrative local fan setpoint. Use the range or an endpoint action, keep a preparation note, then save. This example does not control a device." }
            label { class: "slider-limits-disable",
                input { id: "slider-limits-disabled", r#type: "checkbox", checked: disabled(), onchange: move |event: FormEvent| disabled.set(event.checked()) }
                "Disable setpoint form"
            }
            form { id: "slider-limits-form", class: "slider-limits-form",
                onsubmit: move |event: FormEvent| {
                    event.prevent_default(); if *disabled.peek() { return; }
                    match submitted_setpoint(event.values()) {
                        Ok((record, values)) => {
                            saved.set(Some(record)); payload.set(values); let count = saves.peek().saturating_add(1); saves.set(count); error.set(false);
                            feedback.set("Fan setpoint saved from the form's submitted values. Further adjustments remain drafts.".into());
                        }
                        Err(message) => { error.set(true); feedback.set(message.into()); }
                    }
                },
                onreset: move |event: FormEvent| {
                    event.prevent_default(); value.set(25.0); note.set(INITIAL_NOTE.into()); error.set(false);
                    let count = resets.peek().saturating_add(1); resets.set(count);
                    feedback.set("Setpoint and note reset. The saved local record is retained.".into());
                },
                Slider { label: "Fan setpoint (%)", name: "setpoint", min: 0.0, max: 100.0, step: 5.0, value: value(), disabled: disabled(), show_value: true,
                    label_start: "Idle setpoint", label_end: "Rated setpoint",
                    label_start_content: rsx! {
                        div { class: "slider-limits-endpoint",
                            Button { id: "slider-limits-idle", variant: ButtonVariant::Secondary, button_type: ButtonType::Button, disabled: disabled(),
                                onclick: move |_| { if !*disabled.peek() { value.set(0.0); } }, "Set idle"
                            }
                            span { "0% idle" }
                        }
                    },
                    label_end_content: rsx! {
                        div { class: "slider-limits-endpoint slider-limits-endpoint-end",
                            Button { id: "slider-limits-rated", variant: ButtonVariant::Secondary, button_type: ButtonType::Button, disabled: disabled(),
                                onclick: move |_| { if !*disabled.peek() { value.set(100.0); } }, "Set rated"
                            }
                            span { "100% rated" }
                        }
                    },
                    onvaluechange: move |next| { if !*disabled.peek() { value.set(next); } },
                }
                Input { id: "slider-limits-note", label: "Preparation note", name: "note", value: note(), disabled: disabled(), max_length: 160,
                    onvaluechange: move |next| { if !*disabled.peek() { note.set(next); } },
                }
                div { class: "slider-limits-actions",
                    Button { id: "slider-limits-save", variant: ButtonVariant::Primary, button_type: ButtonType::Submit, disabled: disabled(), "Save setpoint" }
                    Button { id: "slider-limits-reset", variant: ButtonVariant::Secondary, button_type: ButtonType::Reset, "Reset draft" }
                }
            }
            div { class: "slider-limits-saved",
                h3 { "Saved local setpoint" }
                if let Some(record) = saved() {
                    dl { id: "slider-limits-saved",
                        dt { "Setpoint (%)" } dd { "data-field": "setpoint", "{record.value}" }
                        dt { "Preparation note" } dd { "data-field": "note", "{record.note}" }
                    }
                } else { p { id: "slider-limits-empty", "No setpoint saved. Choose a value and select Save setpoint." } }
            }
            output { id: "slider-limits-result", class: "slider-limits-result", role: if error() { "alert" } else { "status" },
                "data-value": value().to_string(), "data-disabled": disabled().to_string(), "data-saved": saved.read().is_some().to_string(),
                "data-saves": saves().to_string(), "data-resets": resets().to_string(), "data-error": error().to_string(), "{feedback}"
            }
            details { class: "slider-limits-payload",
                summary { "Last submitted native values" }
                ul { id: "slider-limits-payload", for (name, value) in payload() { li { key: "{name}", span { "data-name": name.clone(), "{name}: {value}" } } } }
            }
        }
    }
}

pub(crate) const STYLES: &str = r#"
.slider-limits-example{min-width:0;max-width:100%}.slider-limits-example .slider-limits-intro{max-width:72ch;line-height:1.55}.slider-limits-example .slider-limits-disable{display:flex;align-items:center;gap:8px;margin:20px 0 24px;font-size:14px;line-height:20px}.slider-limits-example .slider-limits-disable input{flex:none;width:16px;height:16px;margin:0;accent-color:var(--ix-color-primary)}.slider-limits-example .slider-limits-form{display:flex;flex-direction:column;gap:28px;max-width:640px;min-width:0}.slider-limits-example .slider-limits-endpoint{display:flex;flex-direction:column;align-items:flex-start;gap:8px;min-width:0;font-size:14px;line-height:20px}.slider-limits-example .slider-limits-endpoint-end{align-items:flex-end}.slider-limits-example .slider-limits-endpoint .ix-button{max-width:100%;white-space:normal}.slider-limits-example .slider-limits-actions{display:flex;flex-wrap:wrap;gap:12px}.slider-limits-example .slider-limits-actions .ix-button{max-width:100%;white-space:normal}.slider-limits-example .slider-limits-saved{margin-top:40px;max-width:640px}.slider-limits-example .slider-limits-saved h3{margin:0 0 12px}.slider-limits-example .slider-limits-saved p{font-size:14px;line-height:20px}.slider-limits-example .slider-limits-saved dl{display:grid;grid-template-columns:minmax(100px,1fr) minmax(0,3fr);gap:8px 16px;margin:0;font-size:14px;line-height:20px}.slider-limits-example .slider-limits-saved dt{font-weight:600}.slider-limits-example .slider-limits-saved dd{margin:0;overflow-wrap:anywhere}.slider-limits-example .slider-limits-result{display:block;max-width:72ch;margin-top:24px;font-size:14px;line-height:20px;color:var(--ix-color-text);overflow-wrap:anywhere}.slider-limits-example .slider-limits-payload{margin-top:24px;font-size:14px;line-height:20px}.slider-limits-example .slider-limits-payload>summary{cursor:pointer;min-height:24px}.slider-limits-example .slider-limits-payload li{overflow-wrap:anywhere}.slider-limits-example .slider-limits-disable input:focus-visible,.slider-limits-example .slider-limits-payload>summary:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:3px}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_setpoint_form_has_real_composed_endpoint_actions_and_named_values() {
        let mut dom = VirtualDom::new(SliderLimitsExample);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        for id in [
            "slider-limits",
            "slider-limits-form",
            "slider-limits-idle",
            "slider-limits-rated",
            "slider-limits-note",
            "slider-limits-save",
            "slider-limits-reset",
        ] {
            assert!(html.contains(&format!("id=\"{id}\"")), "{id}");
        }
        assert!(
            html.contains("name=\"setpoint\"")
                && html.contains("name=\"note\"")
                && html.contains(INITIAL_NOTE)
        );
        assert!(
            html.contains("Set idle")
                && html.contains("Set rated")
                && html.contains("0% idle")
                && html.contains("100% rated")
        );
        assert!(!html.contains("Idle setpoint") && !html.contains("Rated setpoint"));
        assert!(html.contains("data-value=\"25\"") && html.contains("No setpoint saved."));
    }
}
