//! Observable text commits and feedback lifecycles for the coordinated G03 pass.
use dioxus::prelude::*;
use dioxus_ix::*;
use serde_json::{Value, json};

fn record(mut events: Signal<Vec<Value>>, field: &'static str, event: &'static str, value: String) {
    let next = events.peek().len() + 1;
    events.write().push(json!({
        "seq": next, "field": field, "event": event, "value": value
    }));
}

fn record_enter(mut events: Signal<Vec<Value>>, field: &'static str, value: String) {
    let next = events.peek().len() + 1;
    events.write().push(json!({
        "seq": next, "field": field, "event": "onkeydown", "value": value, "key": "Enter"
    }));
}

#[component]
pub fn TextCommitReview() -> Element {
    let mut input = use_signal(|| "initial".to_string());
    let mut textarea = use_signal(|| "initial".to_string());
    let mut tooltip_input = use_signal(|| "Feedback input".to_string());
    let mut tooltip_textarea = use_signal(|| "Feedback notes".to_string());
    let mut events = use_signal(Vec::<Value>::new);
    let mut submits = use_signal(|| 0_usize);
    let mut mounted = use_signal(|| true);
    let mut invalid = use_signal(|| true);
    let feedback_state = if invalid() {
        FieldState::Invalid
    } else {
        FieldState::Default
    };
    let event_json = serde_json::to_string(&events()).unwrap_or_else(|_| "[]".to_string());
    rsx! {
        section { class: "demo-section", id: "text-commit-review", style: "min-width:0",
            h2 { "Text commits and feedback" }
            p { "Edit a field, press Enter, then leave it. A changed input commits once; leaving it also reports blur. Textarea Enter inserts a new line. Programmatic values do not commit." }
            div { class: "demo-row",
                button { id: "g03-reset", r#type: "button", class: "ix-button ix-button-secondary",
                    onclick: move |_| {
                        input.set("initial".to_string());
                        textarea.set("initial".to_string());
                        tooltip_input.set("Feedback input".to_string());
                        tooltip_textarea.set("Feedback notes".to_string());
                        events.set(Vec::new());
                        submits.set(0);
                        mounted.set(true);
                        invalid.set(true);
                    }, "Reset review"
                }
                button { id: "g03-programmatic", r#type: "button", class: "ix-button ix-button-secondary",
                    onclick: move |_| {
                        input.set("programmatic".to_string());
                        textarea.set("programmatic".to_string());
                    }, "Set programmatic values"
                }
                button { id: "g03-unmount-toggle", r#type: "button", class: "ix-button ix-button-secondary",
                    onclick: move |_| mounted.toggle(),
                    if mounted() { "Unmount tooltip fields" } else { "Mount tooltip fields" }
                }
                button { id: "g03-feedback-toggle", r#type: "button", class: "ix-button ix-button-secondary",
                    onclick: move |_| invalid.toggle(),
                    if invalid() { "Show helper feedback" } else { "Show error feedback" }
                }
            }
            form { id: "g03-form", onsubmit: move |event| { event.prevent_default(); submits += 1; },
                div { style: "display:grid;grid-template-columns:repeat(auto-fit,minmax(min(100%,18rem),1fr));gap:24px;margin-block:24px;min-width:0",
                    Input { id: "g03-input", name: "g03-input", label: "Input commit review", value: input(),
                        suppress_submit_on_enter: true, helper_text: "Enter commits without submitting this form.",
                        onvaluechange: move |value: String| { record(events, "input", "onvaluechange", value.clone()); input.set(value); },
                        onchange: move |value| record(events, "input", "onchange", value),
                        onixblur: move |_| record(events, "input", "onixblur", input()),
                        onblur: move |_| record(events, "input", "onblur", input()),
                        onfocus: move |_| record(events, "input", "onfocus", input()),
                        onkeydown: move |event: KeyboardEvent| if event.key() == Key::Enter { record_enter(events, "input", input()); }
                    }
                    Textarea { id: "g03-textarea", name: "g03-textarea", label: "Textarea commit review", value: textarea(), rows: 3,
                        helper_text: "A new line is an edit; leaving the field commits its value.",
                        onvaluechange: move |value: String| { record(events, "textarea", "onvaluechange", value.clone()); textarea.set(value); },
                        onchange: move |value| record(events, "textarea", "onchange", value),
                        onixblur: move |_| record(events, "textarea", "onixblur", textarea()),
                        onblur: move |_| record(events, "textarea", "onblur", textarea()),
                        onfocus: move |_| record(events, "textarea", "onfocus", textarea()),
                        onkeydown: move |event: KeyboardEvent| if event.key() == Key::Enter { record_enter(events, "textarea", textarea()); }
                    }
                }
                button { id: "g03-after", r#type: "button", class: "ix-button ix-button-secondary", "Leave text fields" }
                button { id: "g03-submit", r#type: "submit", class: "ix-button ix-button-secondary", "Submit review form" }
            }
            p { "Form submits: " output { id: "g03-submits", "{submits}" } }
            h3 { "Feedback tooltips" }
            p { "Hover or focus a feedback field to read its message. Switching feedback updates the message; unmounting removes the tooltip. Character counts remain visible. Whitespace feedback creates no tooltip." }
            if mounted() {
                div { style: "display:grid;grid-template-columns:repeat(auto-fit,minmax(min(100%,18rem),1fr));gap:24px;margin-block:24px;min-width:0",
                    Input { id: "g03-tooltip-input", label: "Input feedback tooltip", value: tooltip_input(),
                        state: feedback_state, helper_text: "Input helper feedback.", invalid_text: "Input error feedback.",
                        show_text_as_tooltip: true, max_length: 32,
                        onvaluechange: move |value: String| { record(events, "tooltip-input", "onvaluechange", value.clone()); tooltip_input.set(value); },
                        onchange: move |value| record(events, "tooltip-input", "onchange", value),
                        onixblur: move |_| record(events, "tooltip-input", "onixblur", tooltip_input()),
                        onblur: move |_| record(events, "tooltip-input", "onblur", tooltip_input()),
                        onfocus: move |_| record(events, "tooltip-input", "onfocus", tooltip_input()),
                        onkeydown: move |event: KeyboardEvent| if event.key() == Key::Enter { record_enter(events, "tooltip-input", tooltip_input()); }
                    }
                    Textarea { id: "g03-tooltip-textarea", label: "Textarea feedback tooltip", value: tooltip_textarea(), rows: 3,
                        state: feedback_state, helper_text: "Textarea helper feedback.", invalid_text: "Textarea error feedback.",
                        show_text_as_tooltip: true, max_length: 48,
                        onvaluechange: move |value: String| { record(events, "tooltip-textarea", "onvaluechange", value.clone()); tooltip_textarea.set(value); },
                        onchange: move |value| record(events, "tooltip-textarea", "onchange", value),
                        onixblur: move |_| record(events, "tooltip-textarea", "onixblur", tooltip_textarea()),
                        onblur: move |_| record(events, "tooltip-textarea", "onblur", tooltip_textarea()),
                        onfocus: move |_| record(events, "tooltip-textarea", "onfocus", tooltip_textarea()),
                        onkeydown: move |event: KeyboardEvent| if event.key() == Key::Enter { record_enter(events, "tooltip-textarea", tooltip_textarea()); }
                    }
                    Input { id: "g03-blank-tooltip", label: "Whitespace feedback", value: "No feedback message", readonly: true,
                        state: feedback_state, helper_text: "   ", invalid_text: " \n ", show_text_as_tooltip: true, max_length: 32
                    }
                }
                div { style: "display:grid;grid-template-columns:repeat(auto-fit,minmax(min(100%,14rem),1fr));gap:24px;margin-block:24px;min-width:0",
                Input { id: "g03-valid-tooltip", label: "Valid feedback", value: "Ready", readonly: true, state: FieldState::Valid,
                    valid_text: "The value is ready.", show_text_as_tooltip: true }
                Input { id: "g03-warning-tooltip", label: "Warning feedback", value: "Check value", readonly: true, state: FieldState::Warning,
                    warning_text: "Check this value before continuing.", show_text_as_tooltip: true }
                Input { id: "g03-info-tooltip", label: "Information feedback", value: "Reference", readonly: true, state: FieldState::Info,
                    info_text: "This value is a reference.", show_text_as_tooltip: true }
                }
            }
            h3 { "Event order" }
            pre { id: "g03-events", style: "white-space:pre-wrap;overflow-wrap:anywhere;max-height:20rem;overflow:auto", "{event_json}" }
        }
    }
}
