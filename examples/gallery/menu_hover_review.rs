//! G04.5 category hover/grace fixture. Rust owns controls and child state;
//! the diagnostic observer reads native DOM only and never changes behavior.
use dioxus::prelude::*;
use dioxus_ix::*;
use serde_json::{Value, json};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_MEMORY_SCOPE: AtomicUsize = AtomicUsize::new(1);

fn record(mut events: Signal<Vec<Value>>, event: &'static str, value: Value) {
    if let Ok(mut log) = events.try_write() {
        let seq = log.last().and_then(|row| row["seq"].as_u64()).unwrap_or(0) + 1;
        log.push(json!({"seq": seq, "event": event, "value": value}));
        if log.len() > 100 {
            log.remove(0);
        }
    }
}

#[component]
fn HoverMemory(name: &'static str, events: Signal<Vec<Value>>) -> Element {
    let scope = use_hook(|| NEXT_MEMORY_SCOPE.fetch_add(1, Ordering::Relaxed));
    let mut count = use_signal(|| 0_usize);
    use_effect(move || {
        record(
            events,
            "memory-mount",
            json!({"category": name, "scope": scope}),
        )
    });
    use_drop(move || {
        record(
            events,
            "memory-drop",
            json!({"category": name, "scope": scope}),
        )
    });
    rsx! {
        div { id: "g04-hover-memory-{name}", "data-scope-token": scope.to_string(), "data-local-count": count().to_string(),
            style: "display:flex;flex-wrap:wrap;gap:.25rem;padding:.5rem;min-width:0",
            input { id: "g04-hover-input-{name}", r#type: "text", "aria-label": "Category {name} retained value", placeholder: "Retained value", autocomplete: "off", style: "width:8rem;min-width:0;max-width:100%" }
            button { id: "g04-hover-count-{name}", r#type: "button", class: "ix-button ix-button-secondary",
                onclick: move |event| { event.prevent_default(); count += 1; record(events, "memory-count", json!({"category": name, "scope": scope, "count": *count.peek()})); },
                "Count {count}"
            }
        }
    }
}

#[component]
fn HoverEntry(
    name: &'static str,
    index: usize,
    one_active: Signal<bool>,
    events: Signal<Vec<Value>>,
) -> Element {
    let label = if name == "a" {
        "Category A"
    } else {
        "Category B"
    };
    if index == 4 {
        rsx! { div { id: "g04-hover-extra-{name}", style: "padding:.5rem", span { "Appended content " } em { "{label}" } } }
    } else {
        rsx! { MenuItem { label: "{label} item {index}", active: name == "a" && index == 1 && one_active(),
        onclick: move |_| record(events, "select", json!({"category": name, "item": index})) } }
    }
}

#[component]
fn HoverCategory(
    name: &'static str,
    one_active: Signal<bool>,
    extra: Signal<bool>,
    reversed: Signal<bool>,
    second_present: Signal<bool>,
    events: Signal<Vec<Value>>,
) -> Element {
    let label = if name == "a" {
        "Category A"
    } else {
        "Category B"
    };
    let mut order = vec![1_usize, 2, 3, 4];
    order.retain(|index| (*index != 4 || extra()) && (*index != 2 || second_present()));
    if reversed() {
        order.reverse();
    }
    rsx! {
        MenuCategory { label, icon: if name == "a" { "folder" } else { "piechart" },
            for index in order {
                HoverEntry { key: "{index}", name, index, one_active, events }
            }
            HoverMemory { name, events }
        }
    }
}

#[component]
fn HoverControls(
    mut expanded: Signal<bool>,
    mut a_present: Signal<bool>,
    mut mounted: Signal<bool>,
    mut one_active: Signal<bool>,
    mut extra: Signal<bool>,
    mut reversed: Signal<bool>,
    mut second_present: Signal<bool>,
    events: Signal<Vec<Value>>,
) -> Element {
    rsx! {
        div { class: "demo-row",
            label { r#for: "g04-hover-mode", "Menu mode " }
            select { id: "g04-hover-mode", value: if expanded() { "expanded" } else { "collapsed" },
                onchange: move |event| { let next = event.value() == "expanded"; expanded.set(next); record(events, "public-mode", json!(next)); },
                option { value: "collapsed", "Collapsed" }
                option { value: "expanded", "Expanded" }
            }
            label { input { id: "g04-hover-category-toggle", r#type: "checkbox", checked: a_present(),
                onchange: move |event| a_present.set(event.checked()) } "Category A mounted" }
            label { input { id: "g04-hover-mount", r#type: "checkbox", checked: mounted(),
                onchange: move |event| mounted.set(event.checked()) } "Menu mounted" }
            label { input { id: "g04-hover-active", r#type: "checkbox", checked: one_active(),
                onchange: move |event| one_active.set(event.checked()) } "A item 1 active" }
            label { input { id: "g04-hover-extra", r#type: "checkbox", checked: extra(),
                onchange: move |event| extra.set(event.checked()) } "Appended content" }
            label { input { id: "g04-hover-reverse", r#type: "checkbox", checked: reversed(),
                onchange: move |event| reversed.set(event.checked()) } "Reverse children" }
            label { input { id: "g04-hover-second", r#type: "checkbox", checked: second_present(),
                onchange: move |event| second_present.set(event.checked()) } "Second child present" }
        }
    }
}

const NATIVE_STATE: &str = r#"
const root=document.getElementById('g04-hover-stage');if(!root)return;
let frame=0,alive=true,previous='';const listener=new AbortController();
const describe=()=>({categories:[...root.querySelectorAll('.ix-menu-category')].map(host=>{const trigger=host.querySelector(':scope > [role="menuitem"]'),panel=host.querySelector(':scope > [data-menu-category-panel]'),popup=host.querySelector(':scope > [data-menu-category-popup]'),body=host.querySelector('[data-menu-category-body]');let nativeOpen=false;try{nativeOpen=!!popup?.matches(':popover-open');}catch{}return{label:trigger?.getAttribute('aria-label'),owner:host.getAttribute('data-menu-owner'),category:host.getAttribute('data-menu-category-root'),logical_mode:host.getAttribute('data-menu-category-mode'),popup_open:host.getAttribute('data-menu-category-popup-open'),panel_open:host.getAttribute('data-menu-category-panel-open'),hover_native:host.getAttribute('data-category-hover-native'),native_popover:nativeOpen,panel_hidden:panel?.hidden,popup_hidden:popup?.hidden,body_surface:host.getAttribute('data-menu-category-body-surface'),body_parent:body?.parentElement?.hasAttribute('data-menu-category-popup')?'popup':body?.parentElement?.hasAttribute('data-menu-category-panel')?'panel':null};}),focus:{id:document.activeElement?.id,tag:document.activeElement?.tagName}});
const schedule=()=>{if(!alive||frame)return;frame=requestAnimationFrame(()=>{frame=0;if(!alive||!root.isConnected)return;const value=describe(),text=JSON.stringify(value);if(text!==previous){previous=text;dioxus.send(value);}});};
const observer=new MutationObserver(schedule);observer.observe(root,{subtree:true,childList:true,attributes:true,attributeFilter:['data-menu-native','data-category-hover-native','data-menu-category-popup-open','data-menu-category-panel-open','data-menu-category-body-surface','data-menu-category-mode','hidden','data-open','popover']});
root.addEventListener('focusin',schedule,{passive:true,signal:listener.signal});root.addEventListener('focusout',schedule,{passive:true,signal:listener.signal});schedule();
try{await dioxus.recv();}finally{alive=false;observer.disconnect();listener.abort();if(frame)cancelAnimationFrame(frame);}
"#;

#[component]
fn HoverState(
    generation: usize,
    expanded: Signal<bool>,
    mounted: Signal<bool>,
    a_present: Signal<bool>,
    one_active: Signal<bool>,
    extra: Signal<bool>,
    reversed: Signal<bool>,
    second_present: Signal<bool>,
    events: Signal<Vec<Value>>,
) -> Element {
    let mut native = use_signal(|| json!({"waiting": true}));
    let bridge = use_hook(|| {
        Rc::new(RefCell::new(
            None::<(dioxus::document::Eval, dioxus::core::Task)>,
        ))
    });
    let cleanup = bridge.clone();
    use_drop(move || {
        if let Some((eval, task)) = cleanup.borrow_mut().take() {
            let _ = eval.send("finish");
            task.cancel();
        }
    });
    let state = json!({"generation": generation, "expanded": expanded(), "mounted": mounted(), "a_present": a_present(), "one_active": one_active(), "extra": extra(), "reversed": reversed(), "second_present": second_present(), "native": native()}).to_string();
    let log = serde_json::to_string(&events()).unwrap_or_else(|_| "[]".into());
    rsx! {
        div { onmounted: move |_| {
            let eval = dioxus::document::eval(NATIVE_STATE);
            let mut receiver = eval;
            let task = spawn(async move { while let Ok(value) = receiver.recv::<Value>().await { if let Ok(mut state) = native.try_write() { *state = value; } else { break; } } });
            *bridge.borrow_mut() = Some((eval, task));
        },
            h3 { "Observed state" }
            pre { id: "g04-hover-state", style: "white-space:pre-wrap;overflow-wrap:anywhere;max-height:12rem;overflow:auto", "{state}" }
            details { summary { "Fixture events" }
                pre { id: "g04-hover-events", style: "white-space:pre-wrap;overflow-wrap:anywhere;max-height:12rem;overflow:auto", "{log}" }
            }
        }
    }
}

#[component]
fn HoverCases(generation: usize) -> Element {
    let expanded = use_signal(|| false);
    let a_present = use_signal(|| true);
    let mounted = use_signal(|| true);
    let events = use_signal(Vec::<Value>::new);
    let one_active = use_signal(|| false);
    let extra = use_signal(|| false);
    let reversed = use_signal(|| false);
    let second_present = use_signal(|| true);
    rsx! {
        HoverControls { expanded, a_present, mounted, one_active, extra, reversed, second_present, events }
        button { id: "g04-hover-before", r#type: "button", class: "ix-button ix-button-secondary", "Before hover menu" }
        div { id: "g04-hover-stage", style: "height:28rem;max-width:100%;min-width:0;position:relative",
            Application { menu_expanded: expanded, start_expanded: false, pinned_menu: true, force_breakpoint: ApplicationBreakpoint::Large,
                menu: rsx! { if mounted() {
                    Menu { label: "Hover navigation", application_name: "Hover review", start_expanded: false, pinned: true,
                        onexpandchange: move |value| record(events, "expand", json!(value)),
                        MenuItem { label: "Unrelated item", icon: "home", onclick: move |_| record(events, "select", json!("other")) }
                        for name in ["a", "b"].into_iter().filter(|name| *name != "a" || a_present()) {
                            HoverCategory { key: "{name}", name, one_active, extra, reversed, second_present, events }
                        }
                    }
                } },
                div { id: "g04-hover-body", style: "padding:.75rem;min-width:0;overflow-wrap:anywhere", "Noninteractive destination for pointer-only leave timing." }
            }
        }
        button { id: "g04-hover-after", r#type: "button", class: "ix-button ix-button-secondary", "After hover menu" }
        HoverState { generation, expanded, mounted, a_present, one_active, extra, reversed, second_present, events }
    }
}

#[component]
pub fn MenuHoverReview() -> Element {
    let mut generation = use_signal(|| 0_usize);
    rsx! {
        section { id: "g04-hover-review", class: "demo-section", style: "min-width:0;max-width:100%",
            h2 { "Category hover and leave grace" }
            p { "Move from a collapsed category into its flyout, or leave and return before the 250 ms grace expires. The 4 px pointer strip covers the 3 px gap; staying on that strip is different from waiting outside." }
            p { "Compare logical menu expansion with actual popup and panel state in the live readout. Removal and fresh-owner controls deliberately create new lifetimes." }
            button { id: "g04-hover-fresh", r#type: "button", class: "ix-button ix-button-secondary", onclick: move |_| generation += 1, "Fresh collapsed owner" }
            for current in [generation()] { HoverCases { key: "{current}", generation: current } }
        }
    }
}
