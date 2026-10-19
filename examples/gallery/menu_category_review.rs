//! Expanded category panels and collapsed flyout review for G04.4.
use dioxus::prelude::*;
use dioxus_ix::*;
use serde_json::{Value, json};

fn initial_order() -> Vec<&'static str> {
    vec!["one", "disabled-child", "two", "wrapped"]
}

fn record(mut events: Signal<Vec<Value>>, event: &'static str, value: Value) {
    let seq = events.peek().len() + 1;
    events
        .write()
        .push(json!({"seq": seq, "event": event, "value": value}));
}

#[component]
fn CategoryReviewEntry(
    entry: &'static str,
    order: Option<usize>,
    disabled: bool,
    #[props(default)] hidden: bool,
    one_active: bool,
    events: Signal<Vec<Value>>,
) -> Element {
    match entry {
        "one" => {
            rsx! { MenuItem { label: "One", order, hidden, active: one_active, onclick: move |_| record(events, "select", json!("one")) } }
        }
        "disabled-child" => {
            rsx! { MenuItem { label: "Disabled child", order, disabled, hidden, onclick: move |_| record(events, "select", json!("disabled-child")) } }
        }
        "two" => {
            rsx! { MenuItem { label: "Two", order, hidden, onclick: move |_| record(events, "select", json!("two")) } }
        }
        _ => rsx! {
            div { id: "g04-category-wrapper",
                MenuItem { label: "Wrapped child", order, hidden, href: "#g04-category-stage", onclick: move |_| record(events, "select", json!("wrapped")) }
            }
        },
    }
}

#[component]
fn CategoryEvents(events: Signal<Vec<Value>>) -> Element {
    let json = serde_json::to_string(&events()).unwrap_or_else(|_| "[]".into());
    rsx! { pre { id: "g04-category-events", style: "white-space:pre-wrap;overflow-wrap:anywhere;max-height:14rem;overflow:auto", "{json}" } }
}

#[component]
fn CategoryState(
    initial_expanded: bool,
    mounted: Signal<bool>,
    order: Signal<Vec<&'static str>>,
    wrapped_present: Signal<bool>,
    empty: Signal<bool>,
    disabled: Signal<bool>,
    one_active: Signal<bool>,
    expanded: Signal<bool>,
    events: Signal<Vec<Value>>,
) -> Element {
    let observed_events = events();
    let selected = observed_events
        .iter()
        .rev()
        .find(|event| event["event"] == "select")
        .and_then(|event| event["value"].as_str());
    let json = json!({
        "initial_expanded": initial_expanded, "mounted": mounted(), "order": order(),
        "wrapped_present": wrapped_present(), "empty": empty(), "disabled": disabled(),
        "one_active": one_active(), "expanded": expanded(), "selected": selected,
    })
    .to_string();
    rsx! { pre { id: "g04-category-state", style: "white-space:pre-wrap;overflow-wrap:anywhere;max-height:12rem;overflow:auto", "{json}" } }
}

#[component]
fn CategoryCases(initial_expanded: bool) -> Element {
    let mut mounted = use_signal(|| true);
    let mut order = use_signal(initial_order);
    let mut wrapped_present = use_signal(|| true);
    let mut empty = use_signal(|| false);
    let mut disabled = use_signal(|| true);
    let mut one_active = use_signal(|| false);
    let expanded = use_signal(|| initial_expanded);
    let events = use_signal(Vec::<Value>::new);
    rsx! {
        div { class: "demo-row",
            button { id: "g04-category-move", r#type: "button", class: "ix-button ix-button-secondary",
                onclick: move |_| {
                    if order.peek().first() == Some(&"wrapped") { order.set(initial_order()); }
                    else { let mut next = initial_order(); next.retain(|key| *key != "wrapped"); next.insert(0, "wrapped"); order.set(next); }
                }, "Move wrapped child first / restore"
            }
            button { id: "g04-category-wrapped-toggle", r#type: "button", class: "ix-button ix-button-secondary", "aria-pressed": (!wrapped_present()).to_string(),
                onclick: move |_| wrapped_present.toggle(), if wrapped_present() { "Remove wrapped child" } else { "Restore wrapped child" }
            }
            button { id: "g04-category-empty", r#type: "button", class: "ix-button ix-button-secondary", "aria-pressed": empty().to_string(),
                onclick: move |_| empty.toggle(), if empty() { "Restore category children" } else { "Empty category" }
            }
            button { id: "g04-category-disabled", r#type: "button", class: "ix-button ix-button-secondary", "aria-pressed": disabled().to_string(),
                onclick: move |_| disabled.toggle(), if disabled() { "Enable disabled child" } else { "Disable child" }
            }
            button { id: "g04-category-active", r#type: "button", class: "ix-button ix-button-secondary", "aria-pressed": one_active().to_string(),
                onclick: move |_| one_active.toggle(), if one_active() { "Clear One active state" } else { "Mark One active" }
            }
            button { id: "g04-category-mount", r#type: "button", class: "ix-button ix-button-secondary", "aria-pressed": mounted().to_string(),
                onclick: move |_| mounted.toggle(), if mounted() { "Unmount menu" } else { "Mount menu" }
            }
        }
        button { id: "g04-category-before", r#type: "button", class: "ix-button ix-button-secondary", "Before category menu" }
        div { id: "g04-category-stage", style: "height:32rem;max-width:100%;min-width:0;position:relative",
            Application { menu_expanded: expanded, start_expanded: initial_expanded, pinned_menu: true, force_breakpoint: ApplicationBreakpoint::Large,
                menu: rsx! {
                    if mounted() {
                        Menu { label: "Category navigation", start_expanded: initial_expanded, pinned: true, application_name: "Category review",
                            onexpandchange: move |value| record(events, "expand", json!(value)),
                            MenuItem { label: "Other", onclick: move |_| record(events, "select", json!("other")) }
                            MenuCategory { label: "Category",
                                for (index, entry) in order().into_iter().filter(|entry| !empty() && (*entry != "wrapped" || wrapped_present())).enumerate() {
                                    CategoryReviewEntry { key: "{entry}", entry, order: Some(index), disabled: disabled(), one_active: one_active(), events }
                                }
                            }
                            MenuCategory { label: "Second category",
                                MenuItem { label: "Solo", onclick: move |_| record(events, "select", json!("solo")) }
                            }
                        }
                    }
                },
                div { style: "min-width:0;overflow-wrap:anywhere", p { "Category review" } }
            }
        }
        button { id: "g04-category-after", r#type: "button", class: "ix-button ix-button-secondary", "After category menu" }
        h3 { "Review state" }
        CategoryState { initial_expanded, mounted, order, wrapped_present, empty, disabled, one_active, expanded, events }
        h3 { "Category events" }
        CategoryEvents { events }
    }
}

#[component]
pub fn MenuCategoryReview() -> Element {
    let mut initial_expanded = use_signal(|| true);
    let mut revision = use_signal(|| 0_usize);
    rsx! {
        section { id: "g04-category-review", class: "demo-section", style: "min-width:0;max-width:100%",
            h2 { "Category panels and flyouts" }
            p { "Compare two expanded panels with one collapsed flyout. Use the menu toggle to change modes, or start a fresh case below. Child order and active state can change while the menu is mounted." }
            p { "Pointer, Enter and Space have different source defaults. Expanded child Tab wraps and includes the disabled child; collapsed flyout navigation skips it and allows Tab to leave. The wrapped and empty controls expose additional cases for inspection." }
            div { class: "demo-row",
                button { id: "g04-category-expanded", r#type: "button", class: "ix-button ix-button-secondary",
                    onclick: move |_| { initial_expanded.set(true); revision += 1; }, "Fresh expanded case"
                }
                button { id: "g04-category-collapsed", r#type: "button", class: "ix-button ix-button-secondary",
                    onclick: move |_| { initial_expanded.set(false); revision += 1; }, "Fresh collapsed case"
                }
                button { id: "g04-category-reset", r#type: "button", class: "ix-button ix-button-secondary",
                    onclick: move |_| revision += 1, "Reset category review"
                }
            }
            for reset_revision in [revision()] {
                CategoryCases { key: "{reset_revision}", initial_expanded: initial_expanded() }
            }
        }
    }
}
