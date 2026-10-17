//! Numeric value and commit behavior, separate from the source preview fixtures.
use dioxus::prelude::*;
use dioxus_ix::*;

#[component]
pub fn NumberInputReview() -> Element {
    let mut raw = use_signal(|| "1E3".to_string());
    let mut numeric = use_signal(|| Some(1000.0));
    let mut last_commit = use_signal(|| "No committed changes".to_string());
    let mut commits = use_signal(|| 0_usize);
    let mut submits = use_signal(|| 0_usize);
    let mut allow_empty = use_signal(|| true);
    let mut suppress = use_signal(|| true);
    let mut alignment_end = use_signal(|| true);
    let numeric_text = numeric()
        .map(|value| value.to_string())
        .unwrap_or_else(|| "empty".to_string());
    rsx! {
        section { class: "demo-section", id: "number-input-events",
            h2 { "Numeric values and commits" }
            p { "Enter a number or scientific notation such as 1E6. Enter commits a change; leaving the field normalizes its notation and commits only when the number changed." }
            form { onsubmit: move |event| { event.prevent_default(); submits += 1; },
                div { class: "form-demo-grid",
                    NumberInput { id: "number-review-value", label: "Process setpoint", name: "setpoint",
                        value: raw(), step: 0.1, min: -1000000.0, max: 1000000.0,
                        show_stepper_buttons: true, allow_empty_value_change: allow_empty(),
                        suppress_submit_on_enter: suppress(), text_alignment_end: alignment_end(),
                        helper_text: "An empty field is distinct from zero when empty numeric events are enabled.",
                        onvaluechange: move |value| raw.set(value),
                        onnumericvaluechange: move |value| numeric.set(value),
                        oncommit: move |value: Option<f64>| {
                            commits += 1;
                            last_commit.set(value.map(|value| value.to_string()).unwrap_or_else(|| "empty".to_string()));
                        }
                    }
                }
                div { class: "demo-row",
                    Checkbox { label: "Allow empty numeric events", checked: allow_empty(), oncheckedchange: move |value| allow_empty.set(value) }
                    Checkbox { label: "Suppress submit on Enter", checked: suppress(), oncheckedchange: move |value| suppress.set(value) }
                    Checkbox { label: "Align number to end", checked: alignment_end(), oncheckedchange: move |value| alignment_end.set(value) }
                }
                div { class: "demo-row",
                    Button { variant: ButtonVariant::Secondary, onclick: move |_| {}, "Blur numeric field" }
                    button { r#type: "submit", class: "ix-button ix-button-secondary", "Submit numeric review" }
                }
            }
            p { class: "number-review-value", role: "status", "Numeric event: {numeric_text}. Raw controlled text: {raw}." }
            p { class: "number-review-commit", role: "status", "Commits: {commits}. Last commit: {last_commit}. Form submits: {submits}." }
        }
    }
}
