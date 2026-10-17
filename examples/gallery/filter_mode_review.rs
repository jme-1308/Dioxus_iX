//! Omitted and explicit category modes, plus live accessible control labels.
use dioxus::prelude::*;
use dioxus_ix::*;
use serde_json::{Value, json};

#[derive(Clone, Copy, PartialEq)]
enum ReviewMode {
    Absent,
    Empty,
    Populated,
}

impl ReviewMode {
    fn name(self) -> &'static str {
        match self {
            Self::Absent => "absent",
            Self::Empty => "empty",
            Self::Populated => "populated",
        }
    }

    fn categories(self) -> Option<Vec<FilterCategory>> {
        match self {
            Self::Absent => None,
            Self::Empty => Some(Vec::new()),
            Self::Populated => Some(categories()),
        }
    }
}

fn categories() -> Vec<FilterCategory> {
    vec![FilterCategory::new(
        "vendor",
        "Vendor",
        vec!["Alpha".into(), "Beta".into()],
    )]
}

fn suggestions() -> Vec<String> {
    vec!["Apple".into(), "Banana".into()]
}

fn initial_state() -> FilterState {
    FilterState {
        tokens: vec!["existing".into()],
        categories: Vec::new(),
    }
}

fn state_value(state: &FilterState) -> Value {
    json!({
        "tokens": state.tokens,
        "categories": state.categories.iter().map(|token| json!({
            "id": token.id, "value": token.value, "operator": token.operator.symbol()
        })).collect::<Vec<_>>()
    })
}

fn record(mut events: Signal<Vec<Value>>, filter: &'static str, event: &'static str, value: Value) {
    let seq = events.peek().len() + 1;
    events
        .write()
        .push(json!({"seq": seq, "filter": filter, "event": event, "value": value}));
}

#[component]
fn ModeEventLog(events: Signal<Vec<Value>>) -> Element {
    let json = serde_json::to_string(&events()).unwrap_or_else(|_| "[]".to_string());
    rsx! {
        pre { id: "b10-mode-events", style: "white-space:pre-wrap;overflow-wrap:anywhere;max-height:16rem;overflow:auto", "{json}" }
    }
}

#[component]
fn FilterModeCases(
    mode: Signal<ReviewMode>,
    translated: Signal<bool>,
    events: Signal<Vec<Value>>,
) -> Element {
    let labels = translated();
    let input_label = labels.then(|| "Filtertekst".to_string());
    let reset_label = labels.then(|| "Nullstill filtre".to_string());
    let operator_label = labels.then(|| "Bytt sammenligningsoperator".to_string());
    let mode_name = mode().name();
    let state_json = json!({
        "mode": mode_name,
        "labels": if labels { "norwegian" } else { "defaults" },
        "input_label": input_label.as_deref().unwrap_or("Filter"),
        "reset_label": reset_label.as_deref().unwrap_or("Clear filters"),
        "operator_label": operator_label.as_deref().unwrap_or("Toggle comparison operator")
    })
    .to_string();
    rsx! {
        div { style: "min-width:0",
            h3 { "Switch modes without remounting" }
            p { "Choose Vendor, switch its operator, and enter a draft. Changing the mode or labels keeps the same filter instance and committed criteria." }
            div { id: "b10-mode-dynamic", "data-mode": mode_name, "data-translated": labels.to_string(), style: "min-width:0;max-width:48rem",
                CategoryFilter { categories: mode().categories(), suggestions: suggestions(), default_state: initial_state(), enable_top_layer: true,
                    aria_label_filter_input: input_label,
                    aria_label_reset_button: reset_label,
                    aria_label_operator_button: operator_label,
                    onbeforeclear: move |_| record(events, "dynamic", "onbeforeclear", json!("request")),
                    oncategory: move |value| record(events, "dynamic", "oncategory", json!(value)),
                    onchange: move |value: FilterState| record(events, "dynamic", "onchange", state_value(&value)),
                    onclear: move |_| record(events, "dynamic", "onclear", json!("accepted")),
                    oninput: move |value: FilterInputState| record(events, "dynamic", "oninput", json!({"token": value.token, "category": value.category}))
                }
            }
            pre { id: "b10-mode-state", style: "white-space:pre-wrap;overflow-wrap:anywhere", "{state_json}" }
            h3 { "Compare initial modes" }
            p { "All three receive Apple and Banana suggestions. Suggestions appear only when categories are omitted. Free text can still be committed in every mode." }
            div { style: "display:grid;grid-template-columns:repeat(auto-fit,minmax(min(100%,18rem),1fr));gap:24px;margin-block:24px;min-width:0",
                section { id: "b10-mode-absent-case", style: "min-width:0",
                    h4 { "Categories omitted" }
                    CategoryFilter { label: "Omitted categories filter", suggestions: suggestions(),
                        onchange: move |value: FilterState| record(events, "absent", "onchange", state_value(&value)),
                        oninput: move |value: FilterInputState| record(events, "absent", "oninput", json!({"token": value.token, "category": value.category}))
                    }
                }
                section { id: "b10-mode-empty-case", style: "min-width:0",
                    h4 { "Categories explicitly empty" }
                    CategoryFilter { label: "Empty categories filter", categories: Vec::<FilterCategory>::new(), suggestions: suggestions(),
                        onchange: move |value: FilterState| record(events, "empty", "onchange", state_value(&value)),
                        oninput: move |value: FilterInputState| record(events, "empty", "oninput", json!({"token": value.token, "category": value.category}))
                    }
                }
                section { id: "b10-mode-populated-case", style: "min-width:0",
                    h4 { "Vendor category" }
                    CategoryFilter { label: "Populated categories filter", categories: categories(), suggestions: suggestions(),
                        oncategory: move |value| record(events, "populated", "oncategory", json!(value)),
                        onchange: move |value: FilterState| record(events, "populated", "onchange", state_value(&value)),
                        oninput: move |value: FilterInputState| record(events, "populated", "oninput", json!({"token": value.token, "category": value.category}))
                    }
                }
            }
        }
    }
}

#[component]
pub fn FilterModeReview() -> Element {
    let mut mode = use_signal(|| ReviewMode::Populated);
    let mut translated = use_signal(|| false);
    let mut revision = use_signal(|| 0_usize);
    let mut events = use_signal(Vec::<Value>::new);
    rsx! {
        section { id: "b10-mode-review", class: "demo-section", style: "min-width:0;max-width:100%",
            h2 { "Filter modes and accessible labels" }
            div { class: "demo-row",
                button { id: "b10-mode-absent", r#type: "button", class: "ix-button ix-button-secondary", "aria-pressed": (mode() == ReviewMode::Absent).to_string(),
                    onclick: move |_| mode.set(ReviewMode::Absent), "Omit categories"
                }
                button { id: "b10-mode-empty", r#type: "button", class: "ix-button ix-button-secondary", "aria-pressed": (mode() == ReviewMode::Empty).to_string(),
                    onclick: move |_| mode.set(ReviewMode::Empty), "Use empty categories"
                }
                button { id: "b10-mode-populated", r#type: "button", class: "ix-button ix-button-secondary", "aria-pressed": (mode() == ReviewMode::Populated).to_string(),
                    onclick: move |_| mode.set(ReviewMode::Populated), "Use Vendor category"
                }
                button { id: "b10-mode-labels", r#type: "button", class: "ix-button ix-button-secondary", "aria-pressed": translated().to_string(),
                    onclick: move |_| translated.toggle(), if translated() { "Use default labels" } else { "Use Norwegian labels" }
                }
                button { id: "b10-mode-reset", r#type: "button", class: "ix-button ix-button-secondary",
                    onclick: move |_| { mode.set(ReviewMode::Populated); translated.set(false); revision += 1; events.set(Vec::new()); }, "Reset mode review"
                }
            }
            for reset_revision in [revision()] {
                FilterModeCases { key: "{reset_revision}", mode, translated, events }
            }
            button { id: "b10-mode-after", r#type: "button", class: "ix-button ix-button-secondary", "Leave filter controls" }
            h3 { "Mode review events" }
            ModeEventLog { events }
        }
    }
}
