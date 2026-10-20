//! Application-owned native controls with composed field labels and feedback.
use dioxus::prelude::*;
use dioxus_ix::*;
use std::{collections::BTreeMap, rc::Rc};

pub const STYLES: &str = include_str!("styles/custom_field_workflow.css");
const SERVICE_CODE_PATTERN: &str = "[A-Z]{2}-[0-9]{3}";

fn valid_service_code(code: &str) -> bool {
    let bytes = code.as_bytes();
    bytes.len() == 6
        && bytes[..2].iter().all(u8::is_ascii_uppercase)
        && bytes[2] == b'-'
        && bytes[3..].iter().all(u8::is_ascii_digit)
}

#[component]
fn FieldPreparation() -> Element {
    let mut preparation = use_signal(|| String::from("Check access with the shift lead"));
    rsx! {
        Input { id: "field-composition-preparation", label: "Service preparation",
            value: preparation(), max_length: 160,
            onvaluechange: move |value| preparation.set(value),
        }
    }
}

#[component]
fn FieldWorkspace(rich: bool, locked: bool, tooltip: bool) -> Element {
    let mut code = use_signal(String::new);
    let mut note = use_signal(String::new);
    let mut records = use_signal(BTreeMap::<String, String>::new);
    let mut payload = use_signal(BTreeMap::<String, String>::new);
    let mut submissions = use_signal(|| 0_usize);
    let mut message = use_signal(|| String::from("No service notes saved yet."));
    let mut code_node = use_signal(|| None::<Rc<MountedData>>);
    let current_code = code();
    let current_note = note();
    let code_count = current_code.chars().count();
    let state = if current_code.is_empty() {
        FieldState::Default
    } else if valid_service_code(&current_code) {
        FieldState::Valid
    } else {
        FieldState::Invalid
    };
    let state_name = match state {
        FieldState::Invalid => "invalid",
        FieldState::Valid => "valid",
        _ => "default",
    };
    let label_content = rich.then(|| {
        rsx! {
            span { "Service identifier " code { "AB-123" } }
        }
    });
    let bottom_right = rsx! {
        div { class: "field-composition-footer",
            span { class: "field-composition-count", "{code_count}/6" }
            Button { id: "field-composition-clear", variant: ButtonVariant::Tertiary,
                button_type: ButtonType::Button, disabled: locked,
                onclick: move |_| {
                    if locked { return; }
                    code.set(String::new());
                    message.set("Identifier cleared. Your note and saved records are kept.".into());
                    let node = code_node.peek().clone();
                    if let Some(node) = node {
                        spawn(async move { let _ = node.set_focus(true).await; });
                    }
                },
                "Clear identifier"
            }
        }
    };
    let saved = records();
    let serialized = serde_json::to_string(&payload()).unwrap_or_default();
    rsx! {
        form { id: "field-composition-form", class: "field-composition-form",
            onsubmit: move |event| {
                event.prevent_default();
                submissions += 1;
                if locked { return; }
                let values = event.values().into_iter().filter_map(|(name, value)| match value {
                    dioxus::html::FormValue::Text(text) => Some((name, text)),
                    _ => None,
                }).collect::<BTreeMap<_, _>>();
                let Some(submitted_code) = values.get("service-code").filter(|value| valid_service_code(value)).cloned() else { return; };
                let Some(submitted_note) = values.get("service-note").cloned() else { return; };
                records.write().insert(submitted_code.clone(), submitted_note);
                payload.set(values);
                message.set(format!("Service note saved locally for {submitted_code}."));
            },
            CustomField {
                label: "Service identifier", label_content, bottom_right,
                html_for: "field-composition-code", helper_id: "field-composition-feedback",
                required: true, state,
                show_text_as_tooltip: tooltip,
                helper_text: "Use two uppercase letters, a hyphen and three digits.",
                invalid_text: "Enter an identifier such as AB-123.",
                valid_text: "Identifier format is valid.",
                input { id: "field-composition-code", class: "ix-input field-composition-code",
                    name: "service-code", r#type: "text", required: true,
                    pattern: SERVICE_CODE_PATTERN, maxlength: "6", disabled: locked,
                    autocomplete: "off", value: current_code.clone(),
                    "aria-describedby": "field-composition-feedback",
                    "aria-invalid": state == FieldState::Invalid,
                    onmounted: move |event| code_node.set(Some(event.data())),
                    oninput: move |event| code.set(event.value()),
                }
            }
            CustomField { label: "Service note", html_for: "field-composition-note",
                helper_id: "field-composition-note-help", helper_text: "Optional. Saved only in this local example.",
                textarea { id: "field-composition-note", class: "ix-input ix-textarea",
                    name: "service-note", maxlength: "240", rows: "3", disabled: locked,
                    value: current_note.clone(), "aria-describedby": "field-composition-note-help",
                    oninput: move |event| note.set(event.value()),
                }
            }
            Button { id: "field-composition-save", button_type: ButtonType::Submit,
                disabled: locked, "Save service note"
            }
        }
        p { id: "field-composition-status", class: "field-composition-status", role: "status", "aria-live": "polite",
            "data-count": saved.len(), "data-code": current_code, "data-note": current_note,
            "data-state": state_name, "data-rich": rich, "data-locked": locked,
            "data-tooltip": tooltip,
            "data-submissions": submissions(),
            "{message}"
        }
        table { id: "field-composition-records", class: "field-composition-records",
            caption { "Saved service notes" }
            thead { tr { th { scope: "col", "Identifier" } th { scope: "col", "Service note" } } }
            tbody {
                if saved.is_empty() { tr { td { colspan: "2", "No service notes saved yet." } } }
                for (identifier, saved_note) in saved.iter() {
                    tr { key: "{identifier}", id: "field-composition-record-{identifier}",
                        "data-code": identifier.clone(), "data-note": saved_note.clone(),
                        td { code { "{identifier}" } }
                        td { if saved_note.is_empty() { "No note" } else { "{saved_note}" } }
                    }
                }
            }
        }
        output { id: "field-composition-payload", hidden: true, "{serialized}" }
    }
}

#[component]
pub fn CustomFieldWorkflowExample() -> Element {
    let mut rich = use_signal(|| true);
    let mut locked = use_signal(|| false);
    let mut tooltip = use_signal(|| false);
    let mut epoch = use_signal(|| 0_u64);
    rsx! {
        style { {STYLES} }
        section { id: "field-composition-workflow", class: "field-composition-workflow",
            h2 { "Record a service note" }
            p { class: "field-composition-intro", "Use a native input with a composed label and field feedback. Identifiers such as AB-123 are illustrative; saved notes stay in this example." }
            div { class: "field-composition-preparation", FieldPreparation {} }
            div { class: "field-composition-options",
                label { input { id: "field-composition-rich", r#type: "checkbox", checked: rich(),
                    onchange: move |event| rich.set(event.checked()), } "Show the format in the label" }
                label { input { id: "field-composition-lock", r#type: "checkbox", checked: locked(),
                    onchange: move |event| locked.set(event.checked()), } "Lock editing" }
                label { input { id: "field-composition-tooltip", r#type: "checkbox", checked: tooltip(),
                    onchange: move |event| tooltip.set(event.checked()), } "Show identifier feedback as a tooltip" }
            }
            for generation in [epoch()] {
                FieldWorkspace { key: "field-workspace-{generation}", rich: rich(), locked: locked(), tooltip: tooltip() }
            }
            Button { id: "field-composition-reset", class: "field-composition-reset",
                variant: ButtonVariant::Secondary, button_type: ButtonType::Button,
                onclick: move |_| epoch += 1,
                "Reset notes and draft"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_identifier_preserves_required_pattern_quantifiers() {
        let mut dom = VirtualDom::new(CustomFieldWorkflowExample);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        let input = html
            .split("<input")
            .skip(1)
            .filter_map(|tail| tail.split('>').next())
            .find(|attributes| attributes.contains("id=\"field-composition-code\""))
            .expect("service identifier native input");
        assert!(input.contains(" required"), "{input}");
        assert!(input.contains("pattern=\"[A-Z]{2}-[0-9]{3}\""), "{input}");
        assert!(!input.contains("pattern=\"[A-Z]2-[0-9]3\""), "{input}");
    }
}
