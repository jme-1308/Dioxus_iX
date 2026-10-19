//! Workflow requests, transient selection and child reconciliation for G04.1.
use dioxus::prelude::*;
use dioxus_ix::*;
use serde_json::{Value, json};

fn initial_steps() -> Vec<String> {
    ["prepare", "inspect", "disabled", "finish"]
        .into_iter()
        .map(str::to_string)
        .collect()
}

fn record_request(
    mut events: Signal<Vec<Value>>,
    source: &'static str,
    event: WorkflowStepSelectedEvent,
    vetoed: bool,
) {
    let seq = events.peek().len() + 1;
    events.write().push(json!({
        "seq": seq,
        "source": source,
        "requested": event.index,
        "previous": event.previous,
        "vetoed": vetoed
    }));
    if vetoed {
        event.prevent_default();
    }
}

/// A component boundary intentionally sits between the workflow and this step.
#[component]
fn WrappedReviewStep(order: Option<usize>) -> Element {
    rsx! {
        WorkflowStep { id: "g04-step-inspect", label: "Inspection", status: WorkflowStatus::Success, order,
            span { "Inspection" }
        }
    }
}

/// Only this leaf subscribes to requests, preserving transient parent selection.
#[component]
fn WorkflowRequestLog(events: Signal<Vec<Value>>) -> Element {
    let event_json = serde_json::to_string(&events()).unwrap_or_else(|_| "[]".to_string());
    rsx! {
        pre { id: "g04-events", style: "white-space:pre-wrap;overflow-wrap:anywhere;max-height:16rem;overflow:auto", "{event_json}" }
    }
}

#[component]
pub fn WorkflowReview() -> Element {
    let mut selected = use_signal(|| 0_isize);
    let mut clickable = use_signal(|| true);
    let mut vertical = use_signal(|| false);
    let mut veto = use_signal(|| false);
    let mut error = use_signal(|| false);
    let mut tick = use_signal(|| 0_usize);
    let mut keys = use_signal(initial_steps);
    let mut next_added = use_signal(|| 1_usize);
    let mut events = use_signal(Vec::<Value>::new);
    let state_json = json!({
        "selected_index": selected(),
        "clickable": clickable(),
        "vertical": vertical(),
        "veto": veto(),
        "prepare_status": if error() { "error" } else { "open" },
        "rerender_tick": tick(),
        "keys": keys()
    })
    .to_string();
    rsx! {
        section { id: "g04-workflow-review", class: "demo-section", style: "min-width:0;max-width:100%",
            h2 { "Workflow selection and reconciliation" }
            p { "Activate a step with a pointer, Enter or Space. Requests record the previous selection before allowing or vetoing the change. Repeat activation still requests selection. Disabled steps cannot request selection." }
            div { class: "demo-row",
                button { id: "g04-reset", r#type: "button", class: "ix-button ix-button-secondary",
                    onclick: move |_| {
                        selected.set(0);
                        clickable.set(true);
                        vertical.set(false);
                        veto.set(false);
                        error.set(false);
                        keys.set(initial_steps());
                        next_added.set(1);
                        events.set(Vec::new());
                        tick += 1;
                    }, "Reset workflow"
                }
                button { id: "g04-veto", r#type: "button", class: "ix-button ix-button-secondary", "aria-pressed": veto().to_string(),
                    onclick: move |_| veto.toggle(), if veto() { "Allow requests" } else { "Veto requests" }
                }
                button { id: "g04-clickable", r#type: "button", class: "ix-button ix-button-secondary", "aria-pressed": clickable().to_string(),
                    onclick: move |_| clickable.toggle(), if clickable() { "Disable navigation" } else { "Enable navigation" }
                }
                button { id: "g04-orientation", r#type: "button", class: "ix-button ix-button-secondary", "aria-pressed": vertical().to_string(),
                    onclick: move |_| vertical.toggle(), if vertical() { "Show horizontal" } else { "Show vertical" }
                }
            }
            div { id: "g04-workflow-viewport", style: "min-width:0;max-width:100%;overflow-x:auto",
                WorkflowSteps { id: "g04-workflow", label: "Review workflow", clickable: clickable(), vertical: vertical(),
                    selected_index: selected(), class: format!("g04-render-{}", tick()),
                    onstepselected: move |event| record_request(events, "main", event, veto()),
                    for (index, step) in keys().into_iter().enumerate() {
                        if step == "inspect" {
                            WrappedReviewStep { key: "{step}", order: Some(index) }
                        } else if step == "finish" {
                            WorkflowStep { key: "{step}", id: "g04-step-finish", label: "Release", status: WorkflowStatus::Done, order: Some(index),
                                custom_icon: rsx! { Icon { name: "rocket", size: 24 } },
                                span { id: "g04-custom-content", "Release " strong { "ready" } }
                            }
                        } else {
                            WorkflowStep { key: "{step}", id: format!("g04-step-{step}"), order: Some(index),
                                label: if step == "prepare" { "Preparation".to_string() } else if step == "disabled" { "Disabled review".to_string() } else { format!("Added {}", step.trim_start_matches("added-")) },
                                disabled: step == "disabled",
                                status: if step == "prepare" && error() { WorkflowStatus::Error } else if step == "disabled" { WorkflowStatus::Warning } else { WorkflowStatus::Open }
                            }
                        }
                    }
                }
            }
            p { "The selected index stays at its programmatic value after an allowed request. Force a workflow render to restore that index. Programmatic selection may select a disabled step; an out-of-range index selects none." }
            div { class: "demo-row",
                button { id: "g04-select-zero", r#type: "button", class: "ix-button ix-button-secondary",
                    onclick: move |_| { selected.set(0); tick += 1; }, "Select index 0"
                }
                button { id: "g04-select-disabled", r#type: "button", class: "ix-button ix-button-secondary",
                    onclick: move |_| {
                        let index = keys.peek().iter().position(|key| key == "disabled").map_or(99, |index| index as isize);
                        selected.set(index);
                        tick += 1;
                    }, "Select disabled step"
                }
                button { id: "g04-select-out-of-range", r#type: "button", class: "ix-button ix-button-secondary",
                    onclick: move |_| selected.set(99), "Select index 99"
                }
                button { id: "g04-select-negative", r#type: "button", class: "ix-button ix-button-secondary",
                    onclick: move |_| selected.set(-1), "Select index −1"
                }
                button { id: "g04-rerender", r#type: "button", class: "ix-button ix-button-secondary",
                    onclick: move |_| tick += 1, "Restore programmatic index"
                }
            }
            div { class: "demo-row",
                button { id: "g04-append", r#type: "button", class: "ix-button ix-button-secondary",
                    onclick: move |_| { keys.write().push(format!("added-{}", next_added())); next_added += 1; }, "Append keyed step"
                }
                button { id: "g04-remove", r#type: "button", class: "ix-button ix-button-secondary",
                    disabled: keys().len() <= 1,
                    onclick: move |_| { if keys.peek().len() > 1 { keys.write().pop(); } }, "Remove last step"
                }
                button { id: "g04-reorder", r#type: "button", class: "ix-button ix-button-secondary",
                    onclick: move |_| keys.write().reverse(), "Reverse keyed steps"
                }
                button { id: "g04-status", r#type: "button", class: "ix-button ix-button-secondary", "aria-pressed": error().to_string(),
                    onclick: move |_| error.toggle(), if error() { "Set preparation open" } else { "Set preparation error" }
                }
            }
            h3 { "Independent workflow" }
            p { "Selection here leaves the review workflow unchanged." }
            div { style: "min-width:0;max-width:100%;overflow-x:auto",
                WorkflowSteps { id: "g04-other", label: "Independent workflow", clickable: true,
                    onstepselected: move |event| record_request(events, "other", event, false),
                    WorkflowStep { id: "g04-other-a", label: "Other first" }
                    WorkflowStep { id: "g04-other-b", label: "Other second" }
                }
            }
            h3 { "Nested workflow" }
            p { "Activate either inner step with a pointer, Enter or Space. Only the inner workflow records that request. Selecting the second outer step leaves the inner selection unchanged." }
            div { style: "min-width:0;max-width:100%;overflow-x:auto;--ix-workflow-step--height:auto",
                WorkflowSteps { id: "g04-nested-outer", label: "Outer workflow", clickable: true,
                    onstepselected: move |event| record_request(events, "outer", event, false),
                    WorkflowStep { id: "g04-outer-a", label: "Outer first",
                        span { "Outer first" }
                        WorkflowSteps { id: "g04-nested-inner", label: "Inner workflow", clickable: true,
                            onstepselected: move |event| record_request(events, "inner", event, false),
                            WorkflowStep { id: "g04-inner-a", label: "Inner first" }
                            WorkflowStep { id: "g04-inner-b", label: "Inner second" }
                        }
                    }
                    WorkflowStep { id: "g04-outer-b", label: "Outer second" }
                }
            }
            h3 { "Programmatic state" }
            pre { id: "g04-state", style: "white-space:pre-wrap;overflow-wrap:anywhere", "{state_json}" }
            h3 { "Selection requests" }
            WorkflowRequestLog { events }
        }
    }
}
