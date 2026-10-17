//! Mount-scoped filter commands and source-like property application for B10.3.
use dioxus::prelude::*;
use dioxus_ix::*;
use serde_json::{Value, json};

fn plain_state(text: &str) -> FilterState {
    FilterState {
        tokens: vec![text.into()],
        categories: Vec::new(),
    }
}

fn categories() -> Vec<FilterCategory> {
    vec![FilterCategory::new(
        "vendor",
        "Vendor",
        vec!["Alpha".into(), "Beta".into()],
    )]
}

fn command_state() -> FilterState {
    FilterState {
        tokens: vec!["  Applied  ".into(), String::new(), "Applied".into()],
        categories: vec![
            CategoryToken {
                id: "vendor".into(),
                value: " Alpha ".into(),
                operator: LogicalFilterOperator::NotEqual,
            },
            CategoryToken {
                id: "vendor".into(),
                value: "Alpha".into(),
                operator: LogicalFilterOperator::NotEqual,
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

fn apply(
    controller: &CategoryFilterController,
    state: FilterState,
    filter: &'static str,
    command: &'static str,
    mut results: Signal<Vec<Value>>,
) {
    let result = controller.set_filter_state(state);
    let seq = results.peek().len() + 1;
    results.write().push(json!({
        "seq": seq, "filter": filter, "command": command,
        "result": match result { Ok(()) => "Ok".to_string(), Err(error) => format!("{error:?}") }
    }));
}

#[component]
fn StateEventLog(events: Signal<Vec<Value>>) -> Element {
    let json = serde_json::to_string(&events()).unwrap_or_else(|_| "[]".into());
    rsx! { pre { id: "b10-state-events", style: "white-space:pre-wrap;overflow-wrap:anywhere;max-height:16rem;overflow:auto", "{json}" } }
}

#[component]
fn CommandResults(results: Signal<Vec<Value>>) -> Element {
    let json = serde_json::to_string(&results()).unwrap_or_else(|_| "[]".into());
    rsx! { pre { id: "b10-state-results", style: "white-space:pre-wrap;overflow-wrap:anywhere;max-height:12rem;overflow:auto", "{json}" } }
}

#[component]
fn StateValues(
    local: Signal<FilterState>,
    prop: Signal<FilterState>,
    controlled_parent: Signal<FilterState>,
    controlled_proposal: Signal<FilterState>,
    mounted: Signal<bool>,
    reject: Signal<bool>,
) -> Element {
    let json = json!({
        "local": state_value(&local()),
        "prop": state_value(&prop()),
        "controlled_parent": state_value(&controlled_parent()),
        "controlled_proposal": state_value(&controlled_proposal()),
        "local_mounted": mounted(), "reject_controlled": reject()
    })
    .to_string();
    rsx! { pre { id: "b10-state-values", style: "white-space:pre-wrap;overflow-wrap:anywhere;max-height:18rem;overflow:auto", "{json}" } }
}

#[component]
fn FilterStateCases() -> Element {
    let local_controller = use_category_filter();
    let controlled_controller = use_category_filter();
    let mut local = use_signal(|| plain_state("existing"));
    let mut prop_value = use_signal(|| plain_state("Initial prop"));
    let mut prop_observed = use_signal(FilterState::default);
    let mut parent = use_signal(|| plain_state("parent"));
    let mut proposal = use_signal(|| plain_state("parent"));
    let mut mounted = use_signal(|| true);
    let mut reject = use_signal(|| false);
    let events = use_signal(Vec::<Value>::new);
    let results = use_signal(Vec::<Value>::new);
    rsx! {
        div { style: "min-width:0",
            h3 { "Local state commands" }
            p { "Apply trims text, skips empty values and follows the source duplicate rules. Repeat sends the same command again. Empty preserves an uncommitted draft; accepted tokens clear it. Commands report one change without requesting input focus." }
            div { class: "demo-row",
                button { id: "b10-state-apply", r#type: "button", class: "ix-button ix-button-secondary",
                    onclick: { let controller = local_controller.clone(); move |_| apply(&controller, command_state(), "local", "apply", results) }, "Apply values"
                }
                button { id: "b10-state-equal", r#type: "button", class: "ix-button ix-button-secondary",
                    onclick: { let controller = local_controller.clone(); move |_| apply(&controller, command_state(), "local", "equal", results) }, "Repeat same values"
                }
                button { id: "b10-state-empty", r#type: "button", class: "ix-button ix-button-secondary",
                    onclick: { let controller = local_controller.clone(); move |_| apply(&controller, FilterState::default(), "local", "empty", results) }, "Apply empty state"
                }
                button { id: "b10-state-many", r#type: "button", class: "ix-button ix-button-secondary",
                    onclick: { let controller = local_controller.clone(); move |_| apply(&controller, FilterState { tokens: (1..=40).map(|index| format!("token-{index:02}")).collect(), categories: Vec::new() }, "local", "many", results) }, "Apply 40 tokens"
                }
                button { id: "b10-state-mount", r#type: "button", class: "ix-button ix-button-secondary", "aria-pressed": mounted().to_string(),
                    onclick: move |_| { mounted.toggle(); if mounted() { local.set(plain_state("existing")); } },
                    if mounted() { "Unmount local filter" } else { "Mount local filter" }
                }
            }
            div { id: "b10-state-local", style: "min-width:0;max-width:48rem",
                if mounted() {
                    CategoryFilter { label: "Programmatic local filter", categories: categories(), default_state: plain_state("existing"), controller: local_controller.clone(),
                        onchange: move |value: FilterState| { record(events, "local", "onchange", state_value(&value)); local.set(value); },
                        oninput: move |value: FilterInputState| record(events, "local", "oninput", json!({"token": value.token, "category": value.category})),
                        oncategory: move |value| record(events, "local", "oncategory", json!(value)),
                        onbeforeclear: move |_| record(events, "local", "onbeforeclear", json!("request")),
                        onclear: move |_| record(events, "local", "onclear", json!("accepted"))
                    }
                }
            }
            div { style: "display:grid;grid-template-columns:repeat(auto-fit,minmax(min(100%,22rem),1fr));gap:32px;margin-block:32px;min-width:0",
                section { id: "b10-state-prop", style: "min-width:0",
                    h3 { "Property application" }
                    p { "The initial value applies after mounting. Updating the property applies a new value to the same filter." }
                    div { class: "demo-row",
                        button { id: "b10-state-prop-update", r#type: "button", class: "ix-button ix-button-secondary",
                            onclick: move |_| { let next = if prop_value.peek().tokens.first().is_some_and(|token| token == "Initial prop") { "Updated prop" } else { "Initial prop" }; prop_value.set(plain_state(next)); }, "Update property"
                        }
                    }
                    CategoryFilter { label: "Programmatic prop filter", categories: categories(), filter_state: prop_value(),
                        onchange: move |value: FilterState| { record(events, "prop", "onchange", state_value(&value)); prop_observed.set(value); },
                        oninput: move |value: FilterInputState| record(events, "prop", "oninput", json!({"token": value.token, "category": value.category})),
                        oncategory: move |value| record(events, "prop", "oncategory", json!(value)),
                        onbeforeclear: move |_| record(events, "prop", "onbeforeclear", json!("request")),
                        onclear: move |_| record(events, "prop", "onclear", json!("accepted"))
                    }
                }
                section { id: "b10-state-controlled", style: "min-width:0",
                    h3 { "Controlled parent state" }
                    p { "A command proposes a change. The parent can accept it or keep its current criteria." }
                    div { class: "demo-row",
                        button { id: "b10-state-controlled-apply", r#type: "button", class: "ix-button ix-button-secondary",
                            onclick: { let controller = controlled_controller.clone(); move |_| {
                                let mut state = plain_state("controlled command");
                                state.tokens.extend((2..=40).map(|index| format!("token-{index:02}")));
                                apply(&controller, state, "controlled", "apply", results);
                            } }, "Apply controlled values"
                        }
                        button { id: "b10-state-reject", r#type: "button", class: "ix-button ix-button-secondary", "aria-pressed": reject().to_string(),
                            onclick: move |_| reject.toggle(), if reject() { "Accept proposals" } else { "Reject proposals" }
                        }
                        button { id: "b10-state-accept-last", r#type: "button", class: "ix-button ix-button-secondary",
                            onclick: move |_| parent.set(proposal()), "Accept last proposal"
                        }
                    }
                    CategoryFilter { label: "Programmatic controlled filter", categories: categories(), state: parent(), controller: controlled_controller.clone(),
                        onchange: move |value: FilterState| { record(events, "controlled", "onchange", state_value(&value)); proposal.set(value.clone()); if !reject() { parent.set(value); } },
                        oninput: move |value: FilterInputState| record(events, "controlled", "oninput", json!({"token": value.token, "category": value.category})),
                        oncategory: move |value| record(events, "controlled", "oncategory", json!(value)),
                        onbeforeclear: move |_| record(events, "controlled", "onbeforeclear", json!("request")),
                        onclear: move |_| record(events, "controlled", "onclear", json!("accepted"))
                    }
                }
            }
            button { id: "b10-state-after", r#type: "button", class: "ix-button ix-button-secondary", "Leave filter controls" }
            h3 { "Observed values" }
            StateValues { local, prop: prop_observed, controlled_parent: parent, controlled_proposal: proposal, mounted, reject }
            h3 { "Command results" }
            CommandResults { results }
            h3 { "Filter events" }
            StateEventLog { events }
        }
    }
}

#[component]
pub fn FilterStateReview() -> Element {
    let mut revision = use_signal(|| 0_usize);
    rsx! {
        section { id: "b10-state-review", class: "demo-section", style: "min-width:0;max-width:100%",
            h2 { "Programmatic filter state" }
            div { class: "demo-row",
                button { id: "b10-state-reset", r#type: "button", class: "ix-button ix-button-secondary", onclick: move |_| revision += 1, "Reset state review" }
            }
            for reset_revision in [revision()] {
                FilterStateCases { key: "{reset_revision}" }
            }
        }
    }
}
