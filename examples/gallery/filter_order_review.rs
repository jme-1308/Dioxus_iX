//! Chronological chip occurrences and controlled acknowledgements for B10.4.
use dioxus::prelude::*;
use dioxus_ix::*;
use serde_json::{Value, json};

fn categories() -> Vec<FilterCategory> {
    vec![FilterCategory::new(
        "vendor",
        "Vendor",
        vec!["Alpha".into(), "Beta".into()],
    )]
}

fn mixed_state() -> FilterState {
    FilterState {
        tokens: vec!["first".into()],
        categories: vec![
            CategoryToken {
                id: "vendor".into(),
                value: "Alpha".into(),
                operator: LogicalFilterOperator::Equal,
            },
            CategoryToken {
                id: "CW_CUSTOM_FILTER_VALUE".into(),
                value: "middle".into(),
                operator: LogicalFilterOperator::Equal,
            },
            CategoryToken {
                id: "vendor".into(),
                value: "Alpha".into(),
                operator: LogicalFilterOperator::Equal,
            },
        ],
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
fn OrderEvents(events: Signal<Vec<Value>>) -> Element {
    let json = serde_json::to_string(&events()).unwrap_or_else(|_| "[]".into());
    rsx! { pre { id: "b10-order-events", style: "white-space:pre-wrap;overflow-wrap:anywhere;max-height:16rem;overflow:auto", "{json}" } }
}

#[component]
fn OrderValues(
    local: Signal<FilterState>,
    parent: Signal<FilterState>,
    proposal: Signal<FilterState>,
    command_result: Signal<String>,
) -> Element {
    let json = json!({
        "local": state_value(&local()),
        "controlled_parent": state_value(&parent()),
        "controlled_proposal": state_value(&proposal())
    })
    .to_string();
    rsx! {
        pre { id: "b10-order-values", style: "white-space:pre-wrap;overflow-wrap:anywhere;max-height:14rem;overflow:auto", "{json}" }
        p { "Programmatic command: " output { id: "b10-order-command-result", "{command_result}" } }
    }
}

#[component]
fn FilterOrderCases() -> Element {
    let controller = use_category_filter();
    let mut local = use_signal(FilterState::default);
    let mut parent = use_signal(FilterState::default);
    let mut proposal = use_signal(FilterState::default);
    let events = use_signal(Vec::<Value>::new);
    let mut command_result = use_signal(|| "Not run".to_string());
    let mut disabled = use_signal(|| false);
    let mut readonly = use_signal(|| false);
    let mut reject = use_signal(|| false);
    rsx! {
        div { style: "display:grid;grid-template-columns:repeat(auto-fit,minmax(min(100%,22rem),1fr));gap:32px;margin-block:24px;min-width:0",
            section { id: "b10-order-local", style: "min-width:0",
                h3 { "Local filter" }
                div { class: "demo-row",
                    button { id: "b10-order-programmatic", r#type: "button", class: "ix-button ix-button-secondary",
                        onclick: { let controller = controller.clone(); move |_| {
                            let result = controller.set_filter_state(mixed_state());
                            command_result.set(match result { Ok(()) => "Ok".into(), Err(error) => format!("{error:?}") });
                        } }, "Apply mixed occurrences"
                    }
                    button { id: "b10-order-disabled", r#type: "button", class: "ix-button ix-button-secondary", "aria-pressed": disabled().to_string(),
                        onclick: move |_| disabled.toggle(), if disabled() { "Enable local filter" } else { "Disable local filter" }
                    }
                    button { id: "b10-order-read-only", r#type: "button", class: "ix-button ix-button-secondary", "aria-pressed": readonly().to_string(),
                        onclick: move |_| readonly.toggle(), if readonly() { "Allow local edits" } else { "Make local read only" }
                    }
                }
                CategoryFilter { label: "Chronological local filter", categories: categories(), controller: controller.clone(), disabled: disabled(), readonly: readonly(),
                    onchange: move |value: FilterState| { record(events, "local", "onchange", state_value(&value)); local.set(value); },
                    oninput: move |value: FilterInputState| record(events, "local", "oninput", json!({"token": value.token, "category": value.category})),
                    oncategory: move |value| record(events, "local", "oncategory", json!(value)),
                    onbeforeclear: move |_| record(events, "local", "onbeforeclear", json!("request")),
                    onclear: move |_| record(events, "local", "onclear", json!("accepted"))
                }
            }
            section { id: "b10-order-controlled", style: "min-width:0",
                h3 { "Controlled filter" }
                div { class: "demo-row",
                    button { id: "b10-order-reject", r#type: "button", class: "ix-button ix-button-secondary", "aria-pressed": reject().to_string(),
                        onclick: move |_| reject.toggle(), if reject() { "Accept changes" } else { "Reject changes" }
                    }
                    button { id: "b10-order-accept-last", r#type: "button", class: "ix-button ix-button-secondary",
                        onclick: move |_| parent.set(proposal()), "Accept last proposal"
                    }
                }
                CategoryFilter { label: "Chronological controlled filter", categories: categories(), state: parent(),
                    onchange: move |value: FilterState| { record(events, "controlled", "onchange", state_value(&value)); proposal.set(value.clone()); if !reject() { parent.set(value); } },
                    oninput: move |value: FilterInputState| record(events, "controlled", "oninput", json!({"token": value.token, "category": value.category})),
                    oncategory: move |value| record(events, "controlled", "oncategory", json!(value)),
                    onbeforeclear: move |_| record(events, "controlled", "onbeforeclear", json!("request")),
                    onclear: move |_| record(events, "controlled", "onclear", json!("accepted"))
                }
            }
        }
        button { id: "b10-order-after", r#type: "button", class: "ix-button ix-button-secondary", "Leave filter controls" }
        h3 { "Observed values" }
        OrderValues { local, parent, proposal, command_result }
        h3 { "Filter events" }
        OrderEvents { events }
    }
}

#[component]
pub fn FilterOrderReview() -> Element {
    let mut revision = use_signal(|| 0_usize);
    rsx! {
        section { id: "b10-order-review", class: "demo-section", style: "min-width:0;max-width:100%",
            h2 { "Chronological filter tokens" }
            p { "Commit plain text, choose Vendor and Alpha, then commit more plain text. Chips retain their insertion order. With an empty input, Backspace removes the newest occurrence; each close button removes its own chip." }
            p { "The mixed command inserts first, Vendor = Alpha, middle, Vendor = Alpha. Controlled changes can be rejected and accepted later without sending another filter event." }
            div { class: "demo-row",
                button { id: "b10-order-reset", r#type: "button", class: "ix-button ix-button-secondary", onclick: move |_| revision += 1, "Reset order review" }
            }
            for reset_revision in [revision()] {
                FilterOrderCases { key: "{reset_revision}" }
            }
        }
    }
}
