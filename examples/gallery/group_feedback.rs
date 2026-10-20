//! Illustrative local inspection with explicit group validation and saved choices.
use dioxus::prelude::*;
use dioxus_ix::*;

#[derive(Clone, Debug, PartialEq, Eq)]
struct InspectionRecord {
    isolated: bool,
    guard: bool,
    next_step: String,
}

fn next_step_label(value: &str) -> &'static str {
    match value {
        "inspect" => "Inspect in place",
        "service" => "Arrange service",
        _ => "Not selected",
    }
}

fn inspection_record(isolated: bool, guard: bool, next_step: &str) -> Option<InspectionRecord> {
    (isolated && guard && matches!(next_step, "inspect" | "service")).then(|| InspectionRecord {
        isolated,
        guard,
        next_step: next_step.into(),
    })
}

#[component]
fn InspectionPreparation() -> Element {
    let mut draft = use_signal(|| "Prepare Motor 001 inspection".to_string());
    rsx! {
        label { class: "inspection-preparation", r#for: "group-feedback-preparation",
            span { "Page preparation" }
            input { id: "group-feedback-preparation", r#type: "text", maxlength: "160", value: draft(),
                oninput: move |event: FormEvent| draft.set(event.value()),
            }
        }
    }
}

/// Choices are drafts until Save; presentation changes preserve the same form.
#[component]
pub fn GroupFeedbackExample() -> Element {
    let mut isolated = use_signal(|| false);
    let mut guard = use_signal(|| false);
    let mut next_step = use_signal(String::new);
    let mut attempted = use_signal(|| false);
    let mut saved = use_signal(|| None::<InspectionRecord>);
    let mut saves = use_signal(|| 0_usize);
    let mut tooltip = use_signal(|| false);
    let mut disabled = use_signal(|| false);
    let mut horizontal = use_signal(|| false);
    let mut epoch = use_signal(|| 0_u64);
    let checks_ready = isolated() && guard();
    let step_ready = matches!(next_step().as_str(), "inspect" | "service");
    let checks_state = if !attempted() {
        FieldState::Default
    } else if checks_ready {
        FieldState::Valid
    } else {
        FieldState::Invalid
    };
    let step_state = if !attempted() {
        FieldState::Default
    } else if step_ready {
        FieldState::Valid
    } else {
        FieldState::Invalid
    };
    rsx! {
        section { id: "group-feedback", class: "demo-section group-feedback-example",
            h2 { "Safety inspection" }
            p { class: "inspection-intro", "Record an illustrative local inspection for Motor 001. Confirm both safety checks and choose a next step, then save. The example does not control equipment." }
            div { class: "inspection-toolbar",
                Button { id: "group-feedback-restore", variant: ButtonVariant::Secondary, onclick: move |_| {
                    isolated.set(false); guard.set(false); next_step.set(String::new()); attempted.set(false); saved.set(None); saves.set(0);
                    tooltip.set(false); disabled.set(false); horizontal.set(false); let next = epoch.peek().wrapping_add(1); epoch.set(next);
                }, "Restore inspection example" }
            }
            details { id: "group-feedback-options", class: "inspection-options",
                summary { "Feedback and layout options" }
                div {
                    label { input { id: "group-feedback-tooltip", r#type: "checkbox", checked: tooltip(), onchange: move |event: FormEvent| tooltip.set(event.checked()) } "Show feedback as a tooltip" }
                    label { input { id: "group-feedback-disabled", r#type: "checkbox", checked: disabled(), onchange: move |event: FormEvent| disabled.set(event.checked()) } "Disable inspection choices" }
                    label { input { id: "group-feedback-horizontal", r#type: "checkbox", checked: horizontal(), onchange: move |event: FormEvent| horizontal.set(event.checked()) } "Horizontal choices" }
                }
            }
            for generation in [epoch()] { InspectionPreparation { key: "inspection-preparation-{generation}" } }
            // Owner validation treats the two checks as one requirement, rather
            // than inferring group validity from arbitrary descendant controls.
            form { id: "group-feedback-form", class: "inspection-form", novalidate: true,
                onsubmit: move |event: FormEvent| {
                    event.prevent_default(); if *disabled.peek() { return; }
                    attempted.set(true);
                    if let Some(record) = inspection_record(*isolated.peek(), *guard.peek(), &next_step.peek()) {
                        saved.set(Some(record)); let count = saves.peek().saturating_add(1); saves.set(count);
                    }
                },
                onreset: move |event: FormEvent| {
                    event.prevent_default(); isolated.set(false); guard.set(false); next_step.set(String::new()); attempted.set(false);
                },
                div { class: "inspection-groups",
                    div { id: "inspection-checks",
                        CheckboxGroup { label: "Safety checks", required: true, disabled: disabled(), horizontal: horizontal(), state: checks_state, show_text_as_tooltip: tooltip(),
                            helper_text: "Confirm both checks before continuing.", invalid_text: "Confirm power isolation and secure the guard.", valid_text: "Safety checks are complete.",
                            Checkbox { id: "inspection-isolated", label: "Power isolated", name: "isolated", checked: isolated(), disabled: disabled(),
                                oncheckedchange: move |value| { if !*disabled.peek() { isolated.set(value); } },
                            }
                            Checkbox { id: "inspection-guard", label: "Guard secured", name: "guard", checked: guard(), disabled: disabled(),
                                oncheckedchange: move |value| { if !*disabled.peek() { guard.set(value); } },
                            }
                        }
                    }
                    div { id: "inspection-next-step",
                        RadioGroup { label: "Next step", name: "inspection-next-step", required: true,
                            value: next_step(), options: vec![SelectOption::new("inspect", "Inspect in place"), SelectOption::new("service", "Arrange service")],
                            disabled: disabled(), horizontal: horizontal(), state: step_state, show_text_as_tooltip: tooltip(),
                            helper_text: "Choose how to continue the inspection.", invalid_text: "Choose a next step.", valid_text: "The next step is selected.",
                            onvaluechange: move |value: String| { if !*disabled.peek() && matches!(value.as_str(), "inspect" | "service") { next_step.set(value); } },
                        }
                    }
                }
                div { class: "inspection-form-actions",
                    button { id: "group-feedback-save", class: "ix-button ix-button-primary", r#type: "submit", disabled: disabled(), "aria-disabled": disabled().to_string(), "Save inspection" }
                    button { id: "group-feedback-reset", class: "ix-button ix-button-secondary", r#type: "reset", "Reset choices" }
                }
            }
            div { class: "inspection-saved",
                h3 { "Saved inspection" }
                if let Some(record) = saved() {
                    dl { id: "group-feedback-saved", "data-asset": "motor-001",
                        dt { "Asset" } dd { "Motor 001" }
                        dt { "Safety checks" } dd { "data-field": "checks", if record.isolated && record.guard { "Power isolated; Guard secured" } }
                        dt { "Next step" } dd { "data-field": "next-step", "{next_step_label(&record.next_step)}" }
                    }
                } else { p { id: "group-feedback-empty", "No inspection saved. Complete the choices and select Save inspection." } }
            }
            output { id: "group-feedback-result", class: "inspection-result", role: "status",
                "data-isolated": isolated().to_string(), "data-guard": guard().to_string(), "data-next-step": next_step(), "data-attempted": attempted().to_string(),
                "data-ready": (checks_ready && step_ready).to_string(), "data-saved": saved.read().is_some().to_string(), "data-saves": saves().to_string(),
                "data-tooltip": tooltip().to_string(), "data-disabled": disabled().to_string(), "data-horizontal": horizontal().to_string(),
                if attempted() && !(checks_ready && step_ready) { "Complete the missing choices before saving." }
                else if saved.read().is_some() { "A local inspection is saved. Changes to choices remain drafts until you save again." }
                else { "No local inspection has been saved." }
            }
        }
    }
}

pub(crate) const STYLES: &str = include_str!("styles/group_feedback.css");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn incomplete_choices_do_not_create_an_inspection() {
        assert!(inspection_record(false, true, "inspect").is_none());
        assert!(inspection_record(true, true, "").is_none());
        assert!(inspection_record(true, true, "unknown").is_none());
        let saved = inspection_record(true, true, "service").expect("complete inspection");
        assert!(saved.isolated && saved.guard);
        assert_eq!(next_step_label(&saved.next_step), "Arrange service");
    }

    #[test]
    fn initial_form_has_real_choices_and_each_inline_helper_once() {
        let mut dom = VirtualDom::new(GroupFeedbackExample);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        for id in [
            "group-feedback",
            "group-feedback-form",
            "inspection-isolated",
            "inspection-guard",
            "inspection-next-step",
            "group-feedback-save",
            "group-feedback-reset",
            "group-feedback-preparation",
            "group-feedback-empty",
        ] {
            assert!(html.contains(&format!("id=\"{id}\"")), "{id}");
        }
        assert!(
            html.contains("Power isolated")
                && html.contains("Guard secured")
                && html.contains("Inspect in place")
                && html.contains("Arrange service")
        );
        assert_eq!(
            html.matches("Confirm both checks before continuing.")
                .count(),
            1
        );
        assert_eq!(
            html.matches("Choose how to continue the inspection.")
                .count(),
            1
        );
        assert!(
            html.contains("Prepare Motor 001 inspection") && html.contains("data-saved=\"false\"")
        );
        assert!(!html.contains("id=\"group-feedback-saved\""));
    }
}
