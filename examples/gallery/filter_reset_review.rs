//! Category-filter reset requests and retained draft state for B10.1.
use dioxus::prelude::*;
use dioxus_ix::*;
use serde_json::{Value, json};

fn initial_state() -> FilterState {
    FilterState {
        tokens: vec!["existing".to_string()],
        categories: Vec::new(),
    }
}

fn categories() -> Vec<FilterCategory> {
    vec![
        FilterCategory::new("vendor", "Vendor", vec!["Alpha".into(), "Beta".into()]),
        FilterCategory::new("plant", "Plant", vec!["North".into(), "South".into()]),
    ]
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

/// Reading the log here keeps local filter edits from rerendering its ancestors.
#[component]
fn ResetEventLog(events: Signal<Vec<Value>>) -> Element {
    let json = serde_json::to_string(&events()).unwrap_or_else(|_| "[]".to_string());
    rsx! {
        pre { id: "b10-events", style: "white-space:pre-wrap;overflow-wrap:anywhere;max-height:18rem;overflow:auto", "{json}" }
    }
}

#[component]
fn ResetFilterCases(
    events: Signal<Vec<Value>>,
    veto: Signal<bool>,
    reject: Signal<bool>,
) -> Element {
    let mut controlled = use_signal(initial_state);
    let controlled_json = state_value(&controlled()).to_string();
    rsx! {
        div { style: "display:grid;grid-template-columns:repeat(auto-fit,minmax(min(100%,22rem),1fr));gap:32px;min-width:0;margin-block:24px",
            section { id: "b10-local-filter", style: "min-width:0",
                h3 { "Local state · inline options" }
                CategoryFilter { label: "Local reset filter", categories: categories(), default_state: initial_state(),
                    onbeforeclear: move |event: FilterClearEvent| {
                        record(events, "local", "onbeforeclear", json!("request"));
                        if veto() { event.prevent_default(); }
                    },
                    oncategory: move |value| record(events, "local", "oncategory", json!(value)),
                    onchange: move |value: FilterState| record(events, "local", "onchange", state_value(&value)),
                    onclear: move |_| record(events, "local", "onclear", json!("accepted")),
                    oninput: move |value: FilterInputState| record(events, "local", "oninput", json!({"token": value.token, "category": value.category}))
                }
            }
            section { id: "b10-controlled-filter", style: "min-width:0",
                h3 { "Controlled state · top layer options" }
                CategoryFilter { label: "Controlled reset filter", categories: categories(), state: controlled(), enable_top_layer: true,
                    onbeforeclear: move |event: FilterClearEvent| {
                        record(events, "controlled", "onbeforeclear", json!("request"));
                        if veto() { event.prevent_default(); }
                    },
                    oncategory: move |value| record(events, "controlled", "oncategory", json!(value)),
                    onchange: move |value: FilterState| {
                        record(events, "controlled", "onchange", state_value(&value));
                        if !reject() { controlled.set(value); }
                    },
                    onclear: move |_| record(events, "controlled", "onclear", json!("accepted")),
                    oninput: move |value: FilterInputState| record(events, "controlled", "oninput", json!({"token": value.token, "category": value.category}))
                }
                p { "Parent state:" }
                pre { id: "b10-controlled-state", style: "white-space:pre-wrap;overflow-wrap:anywhere", "{controlled_json}" }
            }
        }
    }
}

#[component]
pub fn FilterResetReview() -> Element {
    let mut revision = use_signal(|| 0_usize);
    let mut veto = use_signal(|| false);
    let mut reject = use_signal(|| false);
    let mut events = use_signal(Vec::<Value>::new);
    rsx! {
        section { id: "b10-reset-review", class: "demo-section", style: "min-width:0;max-width:100%",
            h2 { "Filter reset and draft retention" }
            p { "Choose a category, switch its comparison operator, then type an unmatched draft without committing it. Clear filters removes committed criteria and the category preview while retaining the draft text and operator. Clearing closes options without moving focus back to the input." }
            p { "Veto preserves the whole filter. The controlled example can reject proposed state changes while still exposing the reset event sequence. A reset emits no input event; a category event is emitted only when a preview was active." }
            div { class: "demo-row",
                button { id: "b10-reset", r#type: "button", class: "ix-button ix-button-secondary",
                    onclick: move |_| { revision += 1; veto.set(false); reject.set(false); events.set(Vec::new()); }, "Reset review"
                }
                button { id: "b10-veto", r#type: "button", class: "ix-button ix-button-secondary", "aria-pressed": veto().to_string(),
                    onclick: move |_| veto.toggle(), if veto() { "Allow filter reset" } else { "Veto filter reset" }
                }
                button { id: "b10-reject", r#type: "button", class: "ix-button ix-button-secondary", "aria-pressed": reject().to_string(),
                    onclick: move |_| reject.toggle(), if reject() { "Accept controlled changes" } else { "Reject controlled changes" }
                }
            }
            for reset_revision in [revision()] {
                ResetFilterCases { key: "{reset_revision}", events, veto, reject }
            }
            button { id: "b10-after", r#type: "button", class: "ix-button ix-button-secondary", "Leave filter controls" }
            h3 { "Filter events" }
            ResetEventLog { events }
        }
    }
}
