//! Native option focus and source keyboard quirks established by B10.5.
use dioxus::prelude::*;
use dioxus_ix::*;
use serde_json::{Value, json};

fn categories() -> Vec<FilterCategory> {
    vec![
        FilterCategory::new("vendor", "Vendor", vec!["Alpha".into(), "Beta".into()]),
        FilterCategory::new("product", "Product", vec!["X".into(), "Y".into()]),
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

fn record(
    mut events: Signal<Vec<Value>>,
    case_name: &'static str,
    event: &'static str,
    value: Value,
) {
    let seq = events.peek().len() + 1;
    events
        .write()
        .push(json!({"seq": seq, "case": case_name, "event": event, "value": value}));
}

/// Only observation leaves subscribe to event and accepted-value signals.
#[component]
fn KeyboardEvents(events: Signal<Vec<Value>>) -> Element {
    let json = serde_json::to_string(&events()).unwrap_or_else(|_| "[]".into());
    rsx! { pre { id: "b10-keyboard-events", style: "white-space:pre-wrap;overflow-wrap:anywhere;max-height:16rem;overflow:auto", "{json}" } }
}

#[component]
fn KeyboardValues(
    category: Signal<FilterState>,
    suggestions: Signal<FilterState>,
    top_layer: Signal<FilterState>,
) -> Element {
    let json = json!({
        "category": state_value(&category()),
        "suggestions": state_value(&suggestions()),
        "top-layer": state_value(&top_layer())
    })
    .to_string();
    rsx! { pre { id: "b10-keyboard-values", style: "white-space:pre-wrap;overflow-wrap:anywhere;max-height:12rem;overflow:auto", "{json}" } }
}

#[component]
fn NativeKeyboardCase(
    case_name: &'static str,
    title: &'static str,
    label: &'static str,
    #[props(default)] plain_suggestions: bool,
    #[props(default)] top_layer: bool,
    #[props(default)] direct_siblings: bool,
    events: Signal<Vec<Value>>,
    observed: Signal<FilterState>,
) -> Element {
    let mut observed = observed;
    let category_options = (!plain_suggestions).then(categories);
    let suggestion_options = if plain_suggestions {
        vec!["Apple".into(), "Banana".into()]
    } else {
        Vec::new()
    };
    let filter = rsx! {
        CategoryFilter { label, categories: category_options, suggestions: suggestion_options, enable_top_layer: top_layer,
            onchange: move |value: FilterState| { record(events, case_name, "onchange", state_value(&value)); observed.set(value); },
            oninput: move |value: FilterInputState| record(events, case_name, "oninput", json!({"token": value.token, "category": value.category})),
            oncategory: move |value| record(events, case_name, "oncategory", json!(value)),
            onbeforeclear: move |_| record(events, case_name, "onbeforeclear", json!("request")),
            onclear: move |_| record(events, case_name, "onclear", json!("accepted"))
        }
    };
    rsx! {
        section { id: "b10-keyboard-{case_name}", style: "min-width:0;max-width:48rem",
            h3 { "{title}" }
            if plain_suggestions { p { "Categories omitted; Apple and Banana suggestions." } }
            else { p { "Vendor: Alpha, Beta. Product: X, Y." } }
            if direct_siblings {
                p { "Before and after buttons are direct element siblings of the filter root for the source host-sibling arrow path." }
                button { id: "b10-keyboard-{case_name}-before", r#type: "button", class: "ix-button ix-button-secondary", "Before category filter" }
                {filter}
                button { id: "b10-keyboard-{case_name}-after", r#type: "button", class: "ix-button ix-button-secondary", "After category filter" }
            } else {
                button { id: "b10-keyboard-{case_name}-before", r#type: "button", class: "ix-button ix-button-secondary", "Before {title}" }
                div { id: "b10-keyboard-{case_name}-mount", style: "min-width:0", {filter} }
                button { id: "b10-keyboard-{case_name}-after", r#type: "button", class: "ix-button ix-button-secondary", "After {title}" }
            }
        }
    }
}

#[component]
fn FilterKeyboardCases() -> Element {
    let events = use_signal(Vec::<Value>::new);
    let category = use_signal(FilterState::default);
    let suggestions = use_signal(FilterState::default);
    let top_layer = use_signal(FilterState::default);
    rsx! {
        NativeKeyboardCase { case_name: "category", title: "Category options", label: "Native category filter", direct_siblings: true, events, observed: category }
        NativeKeyboardCase { case_name: "suggestions", title: "Plain suggestions", label: "Native suggestions filter", plain_suggestions: true, events, observed: suggestions }
        NativeKeyboardCase { case_name: "top-layer", title: "Top layer category options", label: "Native top-layer filter", top_layer: true, events, observed: top_layer }
        h3 { "Accepted values" }
        KeyboardValues { category, suggestions, top_layer }
        h3 { "Filter events" }
        KeyboardEvents { events }
    }
}

#[component]
pub fn FilterKeyboardReview() -> Element {
    let mut revision = use_signal(|| 0_usize);
    rsx! {
        section { id: "b10-keyboard-review", class: "demo-section", style: "min-width:0;max-width:100%",
            h2 { "Native filter keyboard and focus" }
            p { "This review follows the pinned source quirks: options receive real button focus, option Enter and Space are canceled, and input Enter commits the draft. Tab follows input, reset and operator order, then traps at the popup boundaries once focus enters its controls." }
            p { "Compare closed and open ArrowDown, literal Space, Escape, and Home/End with selection. Focusing the input alone keeps options closed. Remount restores three fresh empty filters and clears observations." }
            div { class: "demo-row",
                button { id: "b10-keyboard-reset", r#type: "button", class: "ix-button ix-button-secondary", onclick: move |_| revision += 1, "Reset keyboard review" }
            }
            for reset_revision in [revision()] {
                FilterKeyboardCases { key: "{reset_revision}" }
            }
        }
    }
}
