//! Outer Menu entry, current item order and roving focus review for G04.3.
use dioxus::prelude::*;
use dioxus_ix::*;
use serde_json::{Value, json};

const DEFAULT_LABEL: &str = "Application Navigation";
const DEFAULT_HINT: &str = "Use Up and Down arrow keys to navigate between menu items";

fn initial_order() -> Vec<&'static str> {
    vec![
        "alpha",
        "disabled",
        "link",
        "category",
        "second-category",
        "omega",
    ]
}

fn record(mut events: Signal<Vec<Value>>, event: &'static str, value: Value) {
    let seq = events.peek().len() + 1;
    events
        .write()
        .push(json!({"seq": seq, "event": event, "value": value}));
}

#[component]
fn MenuEvents(events: Signal<Vec<Value>>) -> Element {
    let json = serde_json::to_string(&events()).unwrap_or_else(|_| "[]".into());
    rsx! { pre { id: "g04-menu-events", style: "white-space:pre-wrap;overflow-wrap:anywhere;max-height:16rem;overflow:auto", "{json}" } }
}

#[component]
fn MenuReviewEntry(
    entry: &'static str,
    order: Option<usize>,
    disabled: bool,
    hidden: bool,
    link_hidden: bool,
    events: Signal<Vec<Value>>,
) -> Element {
    match entry {
        "alpha" => {
            rsx! { MenuItem { label: "Alpha", order, onclick: move |_| record(events, "select", json!("alpha")) } }
        }
        "disabled" => {
            rsx! { MenuItem { label: "Disabled", order, disabled, onclick: move |_| record(events, "select", json!("disabled")) } }
        }
        "link" => rsx! {
            div { id: "g04-menu-link-wrapper", style: if link_hidden { "display:none" } else { "" },
                MenuItem { label: "Link", order, hidden: link_hidden, href: "#g04-menu-stage", onclick: move |_| record(events, "select", json!("link")) }
            }
        },
        "category" => rsx! {
            MenuCategory { label: "Category", order,
                MenuItem { label: "One", onclick: move |_| record(events, "select", json!("one")) }
                MenuItem { label: "Disabled child", disabled: true, onclick: move |_| record(events, "select", json!("disabled-child")) }
                MenuItem { label: "Two", onclick: move |_| record(events, "select", json!("two")) }
            }
        },
        "second-category" => rsx! {
            MenuCategory { label: "Second category", order,
                MenuItem { label: "Solo", onclick: move |_| record(events, "select", json!("solo")) }
            }
        },
        _ => {
            rsx! { MenuItem { label: "Omega", order, hidden, onclick: move |_| record(events, "select", json!("omega")) } }
        }
    }
}

#[component]
fn MenuState(
    order: Signal<Vec<&'static str>>,
    mounted: Signal<bool>,
    generation: Signal<usize>,
    link_present: Signal<bool>,
    omega_hidden: Signal<bool>,
    item_disabled: Signal<bool>,
    link_hidden: Signal<bool>,
    alternate_labels: Signal<bool>,
    expanded: Signal<bool>,
    overlay: Signal<Option<MenuOverlayKind>>,
    events: Signal<Vec<Value>>,
) -> Element {
    let observed_events = events();
    let selected = observed_events
        .iter()
        .rev()
        .find(|event| event["event"] == "select")
        .and_then(|event| event["value"].as_str());
    let json = json!({
        "order": order(), "mounted": mounted(), "generation": generation(),
        "reversed": order().first() == Some(&"omega"), "selected": selected,
        "link_present": link_present(), "omega_hidden": omega_hidden(),
        "disabled": item_disabled(), "link_inline_hidden": link_hidden(),
        "menu_label": if alternate_labels() { "Review navigation" } else { DEFAULT_LABEL },
        "navigation_hint": if alternate_labels() { "Choose a menu item with Up and Down" } else { DEFAULT_HINT },
        "expanded": expanded(),
        "overlay": overlay().map(|kind| match kind { MenuOverlayKind::Settings => "settings", MenuOverlayKind::About => "about" })
    }).to_string();
    rsx! { pre { id: "g04-menu-state", style: "white-space:pre-wrap;overflow-wrap:anywhere;max-height:14rem;overflow:auto", "{json}" } }
}

#[component]
fn MenuNavigationCases() -> Element {
    let mut order = use_signal(initial_order);
    let mut mounted = use_signal(|| true);
    let mut generation = use_signal(|| 0_usize);
    let mut link_present = use_signal(|| true);
    let mut omega_hidden = use_signal(|| false);
    let mut item_disabled = use_signal(|| true);
    let mut link_hidden = use_signal(|| false);
    let mut alternate_labels = use_signal(|| false);
    let expanded = use_signal(|| true);
    let overlay = use_signal(|| None::<MenuOverlayKind>);
    let events = use_signal(Vec::<Value>::new);
    let label = if alternate_labels() {
        "Review navigation"
    } else {
        DEFAULT_LABEL
    };
    let hint = if alternate_labels() {
        "Choose a menu item with Up and Down"
    } else {
        DEFAULT_HINT
    };
    rsx! {
        div { class: "demo-row",
            button { id: "g04-menu-move", r#type: "button", class: "ix-button ix-button-secondary",
                onclick: move |_| {
                    if order.peek().first() == Some(&"omega") { order.set(initial_order()); }
                    else { let mut next = initial_order(); next.retain(|key| *key != "omega"); next.insert(0, "omega"); order.set(next); }
                }, "Move Omega first / restore"
            }
            button { id: "g04-menu-link-toggle", r#type: "button", class: "ix-button ix-button-secondary", "aria-pressed": (!link_present()).to_string(),
                onclick: move |_| link_present.toggle(), if link_present() { "Remove Link" } else { "Restore Link" }
            }
            button { id: "g04-menu-hide", r#type: "button", class: "ix-button ix-button-secondary", "aria-pressed": omega_hidden().to_string(),
                onclick: move |_| omega_hidden.toggle(), if omega_hidden() { "Show Omega" } else { "Hide Omega" }
            }
            button { id: "g04-menu-disable", r#type: "button", class: "ix-button ix-button-secondary", "aria-pressed": item_disabled().to_string(),
                onclick: move |_| item_disabled.toggle(), if item_disabled() { "Enable Disabled item" } else { "Disable Disabled item" }
            }
            button { id: "g04-menu-inline-hide", r#type: "button", class: "ix-button ix-button-secondary", "aria-pressed": link_hidden().to_string(),
                onclick: move |_| link_hidden.toggle(), if link_hidden() { "Show Link wrapper" } else { "Hide Link wrapper inline" }
            }
            button { id: "g04-menu-labels", r#type: "button", class: "ix-button ix-button-secondary", "aria-pressed": alternate_labels().to_string(),
                onclick: move |_| alternate_labels.toggle(), "Switch navigation labels"
            }
            button { id: "g04-menu-mount", r#type: "button", class: "ix-button ix-button-secondary", "aria-pressed": mounted().to_string(),
                onclick: move |_| mounted.toggle(), if mounted() { "Unmount menu" } else { "Mount menu" }
            }
            button { id: "g04-menu-remount", r#type: "button", class: "ix-button ix-button-secondary",
                onclick: move |_| { mounted.set(true); generation += 1; }, "Remount menu"
            }
        }
        button { id: "g04-menu-before", r#type: "button", class: "ix-button ix-button-secondary", "Before menu" }
        div { id: "g04-menu-stage", style: "height:40rem;max-width:100%;min-width:0;position:relative",
            Application { menu_expanded: expanded, start_expanded: true, pinned_menu: true, force_breakpoint: ApplicationBreakpoint::Large,
                menu: rsx! {
                    for menu_generation in [generation()] {
                        if mounted() {
                            Menu { key: "{menu_generation}", label, navigation_hint: hint, start_expanded: true, pinned: true,
                                application_name: "Menu navigation review", active_overlay: overlay, enable_toggle_theme: true,
                                onexpandchange: move |value| record(events, "expand", json!(value)),
                                onbeforeopensettings: move |event: MenuOpenEvent| record(events, "overlay-request", json!({"overlay": "settings", "opening": event.opening})),
                                onbeforeopenabout: move |event: MenuOpenEvent| record(events, "overlay-request", json!({"overlay": "about", "opening": event.opening})),
                                onopensettings: move |_| record(events, "open-settings", json!("accepted")),
                                onopenabout: move |_| record(events, "open-about", json!("accepted")),
                                ontoggletheme: move |_| record(events, "theme", json!("requested")),
                                bottom: rsx! { MenuItem { label: "Footer", onclick: move |_| record(events, "select", json!("footer")) } },
                                settings: rsx! { MenuSettings { div { id: "g04-menu-settings-content", p { "Settings review content." } } } },
                                about: rsx! { MenuAbout { div { id: "g04-menu-about-content", p { "About review content." } } } },
                                for (index, entry) in order().into_iter().filter(|entry| *entry != "link" || link_present()).enumerate() {
                                    MenuReviewEntry { key: "{entry}", entry, order: Some(index), disabled: item_disabled(), hidden: omega_hidden(), link_hidden: link_hidden(), events }
                                }
                                MenuItem { label: "Disabled link", href: "#g04-menu-stage", disabled: true }
                            }
                        }
                    }
                },
                div { style: "min-width:0;overflow-wrap:anywhere", p { "Menu review" } }
            }
        }
        button { id: "g04-menu-after", r#type: "button", class: "ix-button ix-button-secondary", "After menu" }
        h3 { "Review state" }
        MenuState { order, mounted, generation, link_present, omega_hidden, item_disabled, link_hidden, alternate_labels, expanded, overlay, events }
        h3 { "Menu events" }
        MenuEvents { events }
    }
}

#[component]
pub fn MenuNavigationReview() -> Element {
    let mut revision = use_signal(|| 0_usize);
    rsx! {
        section { id: "g04-menu-review", class: "demo-section", style: "min-width:0;max-width:100%",
            h2 { "Menu entry and navigation order" }
            p { "Tab into the menu, then use Up, Down, Home and End. Navigation moves focus without activating an item. Leave and return to check remembered focus; change the current item order with the controls." }
            p { "This pinned large-screen stage reviews outer menu navigation. Category-local keyboard and flyout behavior remain the separate G04.4 boundary. Disabled controls use normal Rust markup; inline wrapper hiding forwards explicit hidden state so hidden controls are skipped." }
            div { class: "demo-row",
                button { id: "g04-menu-reset", r#type: "button", class: "ix-button ix-button-secondary", onclick: move |_| revision += 1, "Reset menu review" }
            }
            for reset_revision in [revision()] {
                MenuNavigationCases { key: "{reset_revision}" }
            }
        }
    }
}
