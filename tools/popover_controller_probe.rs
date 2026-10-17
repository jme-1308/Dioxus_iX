//! Excluded public-API Popover controller fixture and synthetic native replay.
//! The driver records actual Rust callbacks and native state; source comparison
//! and acceptance belong to the paired upstream oracle, not invented JS policy.
use dioxus::prelude::*;
use dioxus_ix::*;
use serde_json::{Value, json};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

static CHILD_INSTANCES: AtomicUsize = AtomicUsize::new(1);

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
#[path = "../src/dom_query.rs"]
mod dom_query;

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
fn main() {
    let config = dioxus::desktop::Config::new()
        .with_window(
            dioxus::desktop::WindowBuilder::new()
                .with_title("iX nested Popover controller probe")
                .with_inner_size(dioxus::desktop::LogicalSize::new(1200.0, 1000.0)),
        )
        .with_menu(None);
    dioxus::LaunchBuilder::desktop()
        .with_cfg(config)
        .launch(Probe);
}

#[cfg(target_arch = "wasm32")]
fn main() {
    dioxus::launch(Probe);
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "desktop")))]
fn main() {
    let _fixture = Probe;
    eprintln!("Run the desktop or web Popover controller probe target");
}

fn record(mut rows: Signal<Vec<Value>>, kind: &str, target: &str, details: Value) {
    let mut values = rows.write();
    let sequence = values
        .last()
        .and_then(|row| row["seq"].as_u64())
        .unwrap_or(0)
        + 1;
    values.push(json!({"seq": sequence, "kind": kind, "target": target, "details": details}));
}

#[derive(Clone, Copy)]
struct ProbeContext {
    handles: [PopoverHandle; 5],
    veto: Signal<[bool; 5]>,
    veto_open: Signal<[bool; 5]>,
    rows: Signal<Vec<Value>>,
    nested_child_host: bool,
    focus_mode: FocusMode,
    dynamic_body: Signal<bool>,
}

const NAMES: [&str; 5] = ["parent", "child", "unrelated", "sticky", "sibling"];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum FocusMode {
    #[default]
    Default,
    Priority,
    HeaderOnly,
    FooterOnly,
    Static,
    Positive,
    Excluded,
    NoTrigger,
    CachedAdd,
    CachedRemove,
    Tab,
    TabPositive,
    TabNested,
    TabDropdownActive,
    TabDropdownRoving,
}

impl FocusMode {
    const ALL: [Self; 15] = [
        Self::Default,
        Self::Priority,
        Self::HeaderOnly,
        Self::FooterOnly,
        Self::Static,
        Self::Positive,
        Self::Excluded,
        Self::NoTrigger,
        Self::CachedAdd,
        Self::CachedRemove,
        Self::Tab,
        Self::TabPositive,
        Self::TabNested,
        Self::TabDropdownActive,
        Self::TabDropdownRoving,
    ];
    fn key(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Priority => "priority",
            Self::HeaderOnly => "header-only",
            Self::FooterOnly => "footer-only",
            Self::Static => "static",
            Self::Positive => "positive",
            Self::Excluded => "excluded",
            Self::NoTrigger => "no-trigger",
            Self::CachedAdd => "cached-add",
            Self::CachedRemove => "cached-remove",
            Self::Tab => "tab",
            Self::TabPositive => "tab-positive",
            Self::TabNested => "tab-nested",
            Self::TabDropdownActive => "tab-dropdown-active",
            Self::TabDropdownRoving => "tab-dropdown-roving",
        }
    }
    fn from_key(value: &str) -> Self {
        Self::ALL
            .into_iter()
            .find(|mode| mode.key() == value)
            .unwrap_or_default()
    }
}

#[component]
fn Probe() -> Element {
    let mut generation = use_signal(|| 0_usize);
    let mut nested_child_host = use_signal(|| true);
    let mut hover_parent = use_signal(|| false);
    let mut focus_mode = use_signal(FocusMode::default);
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    use_effect(move || {
        spawn(async move {
            futures_timer::Delay::new(std::time::Duration::from_secs(90)).await;
            eprintln!("Popover controller native global timeout");
            std::process::exit(1);
        });
    });
    rsx! { IxTheme { scheme: ColorScheme::Dark,
        main { id: "popover-controller-probe", "data-fixture-generation": generation().to_string(), style: "padding:24px;min-height:90vh",
            onmounted: move |_| {
                #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
                {
                    let window = window.clone();
                    spawn(async move {
                        let query = dom_query::eval(VERIFY);
                        let _ = query.send(());
                        match query.join::<Value>().await {
                            Ok(result) if result["ok"] == true => { println!("Popover controller native observations: {result}"); window.close(); }
                            Ok(result) => { eprintln!("Popover controller native failure JSON: {result}"); std::process::exit(1); }
                            Err(error) => { eprintln!("Popover controller native transport failure: {error:?}"); std::process::exit(1); }
                        }
                    });
                }
            },
            h1 { "Nested Popover controller fixture" }
            p { "Public Rust show/hide calls and callback vetoes. Desktop replay uses synthetic input only; web controls enable real browser inspection." }
            button { id: "popover-fresh-button", onclick: move |_| generation += 1, "Fresh fixture" }
            input { id: "popover-fresh", r#type: "hidden", oninput: move |_| generation += 1 }
            button { id: "popover-topology-nested", onclick: move |_| { nested_child_host.set(true); hover_parent.set(false); focus_mode.set(FocusMode::Default); generation += 1; }, "Fresh nested child host" }
            button { id: "popover-topology-sibling", onclick: move |_| { nested_child_host.set(false); hover_parent.set(false); focus_mode.set(FocusMode::Default); generation += 1; }, "Fresh sibling child host; nested trigger" }
            button { id: "popover-topology-hover", onclick: move |_| { nested_child_host.set(true); hover_parent.set(true); focus_mode.set(FocusMode::Default); generation += 1; }, "Fresh hover parent trigger" }
            input { id: "popover-topology", r#type: "hidden", oninput: move |event| { let value = event.value(); nested_child_host.set(value != "sibling"); hover_parent.set(value == "hover"); focus_mode.set(FocusMode::Default); generation += 1; } }
            div { style: "display:flex;gap:8px;flex-wrap:wrap;margin:12px 0",
                for mode in FocusMode::ALL { button { id: format!("popover-focus-mode-{}", mode.key()), onclick: move |_| { focus_mode.set(mode); nested_child_host.set(true); hover_parent.set(false); generation += 1; }, {format!("Fresh focus: {}", mode.key())} } }
            }
            input { id: "popover-focus-mode", r#type: "hidden", oninput: move |event| { focus_mode.set(FocusMode::from_key(&event.value())); nested_child_host.set(true); hover_parent.set(false); generation += 1; } }
            button { id: "popover-before", "Before opening focus target" }
            button { id: "popover-outside", "Outside focus and Escape target" }
            for current in [generation()] { Fixture { key: "{current}", generation: current, nested_child_host: nested_child_host(), hover_parent: hover_parent(), focus_mode: focus_mode() } }
            TabNativeJournal {}
        }
    } }
}

#[component]
fn Fixture(
    generation: usize,
    nested_child_host: bool,
    hover_parent: bool,
    focus_mode: FocusMode,
) -> Element {
    let handles = [
        use_popover(),
        use_popover(),
        use_popover(),
        use_popover(),
        use_popover(),
    ];
    let veto = use_signal(|| [false; 5]);
    let veto_open = use_signal(|| [false; 5]);
    let rows = use_signal(Vec::<Value>::new);
    let mut dynamic_body = use_signal(|| focus_mode == FocusMode::CachedRemove);
    let context = ProbeContext {
        handles,
        veto,
        veto_open,
        rows,
        nested_child_host,
        focus_mode,
        dynamic_body,
    };
    use_context_provider(|| context);
    let mut completed = use_signal(|| 0_usize);
    let state = serde_json::to_string(&json!({
        "parent": handles[0].is_open(), "child": handles[1].is_open(),
        "unrelated": handles[2].is_open(), "sticky": handles[3].is_open(), "sibling": handles[4].is_open(),
    }))
    .unwrap();
    let logs = serde_json::to_string(&*rows.read()).unwrap();
    rsx! {
        section { id: "popover-fixture", "data-generation": generation.to_string(), "data-topology": if hover_parent { "hover" } else if nested_child_host { "nested" } else { "sibling" }, "data-focus-mode": focus_mode.key(), "data-dynamic-body": dynamic_body().to_string(), "data-state": state, "data-completed": completed().to_string(),
            input { id: "popover-command", r#type: "hidden", oninput: move |event| {
                let command: Value = match serde_json::from_str(&event.value()) { Ok(value) => value, Err(_) => return };
                let action = command["action"].as_str().unwrap_or("").to_owned();
                let target = command["target"].as_str().unwrap_or("").to_owned();
                let Some(index) = NAMES.iter().position(|name| *name == target) else { return; };
                record(rows, "command", &target, command.clone());
                if action == "focus-body" {
                    dynamic_body.set(command["value"].as_bool().unwrap_or(false));
                    completed += 1;
                } else if action == "veto" {
                    let mut veto = veto;
                    veto.write()[index] = command["value"].as_bool().unwrap_or(false);
                    completed += 1;
                } else if action == "veto-open" {
                    let mut veto_open = veto_open;
                    veto_open.write()[index] = command["value"].as_bool().unwrap_or(false);
                    completed += 1;
                } else if action == "hide" {
                    handles[index].hide();
                    record(rows, "hide-return", &target, json!({"open": handles[index].is_open()}));
                    completed += 1;
                } else if action == "show" {
                    spawn(async move {
                        let result = handles[index].show().await;
                        record(rows, "show-return", &target, json!({"result": format!("{result:?}"), "open": handles[index].is_open()}));
                        completed += 1;
                    });
                }
            } }
            div { style: "display:flex;gap:12px;flex-wrap:wrap;margin:16px 0",
                for (index, name) in NAMES.into_iter().enumerate() {
                    button { id: "popover-{name}-show", onclick: move |_| { spawn(async move { let result = handles[index].show().await; record(rows, "show-return", name, json!({"result":format!("{result:?}"),"open":handles[index].is_open()})); }); }, "Show {name}" }
                    button { id: "popover-{name}-hide", onclick: move |_| { handles[index].hide(); record(rows, "hide-return", name, json!({"open":handles[index].is_open()})); }, "Hide {name}" }
                    label { input { id: "popover-{name}-veto", r#type: "checkbox", checked: veto()[index], oninput: move |event| { let mut veto = veto; veto.write()[index] = event.checked(); } } "Veto {name} close" }
                    label { input { id: "popover-{name}-veto-open", r#type: "checkbox", checked: veto_open()[index], oninput: move |event| { let mut veto_open = veto_open; veto_open.write()[index] = event.checked(); } } "Veto {name} opening" }
                }
            }
            if focus_mode != FocusMode::Default {
                button { id: "popover-focus-add-body", onclick: move |_| dynamic_body.set(true), "Add parent body control" }
                button { id: "popover-focus-remove-body", onclick: move |_| dynamic_body.set(false), "Remove parent body control" }
            }
            if focus_mode != FocusMode::NoTrigger { PopoverTrigger { popover: handles[0], trigger_mode: if hover_parent { PopoverTriggerMode::Hover } else { PopoverTriggerMode::Click }, "Parent native trigger" } }
            Panel { index: 0 }
            if !nested_child_host || (focus_mode != FocusMode::Default && focus_mode != FocusMode::Excluded && focus_mode != FocusMode::TabNested) { Panel { index: 1 } }
            if focus_mode != FocusMode::Default { Panel { index: 4 } }
            PopoverTrigger { popover: handles[2], "Unrelated native trigger" }
            Panel { index: 2 }
            PopoverTrigger { popover: handles[3], "Sticky native trigger" }
            Panel { index: 3 }
            pre { id: "popover-controller-log", style: "white-space:pre-wrap;overflow-wrap:anywhere", "{logs}" }
        }
    }
}

#[component]
fn Panel(index: usize) -> Element {
    let ProbeContext {
        handles,
        veto,
        veto_open,
        rows,
        nested_child_host,
        focus_mode,
        dynamic_body,
    } = use_context();
    let popover = handles[index];
    let name = NAMES[index];
    rsx! {
        div { id: "popover-{name}-wrapper", "data-native-id": popover.id(), "data-open": popover.is_open().to_string(),
            Popover { popover, label: "Controller {name}", close_on_click_outside: index != 3 && (index != 0 || focus_mode == FocusMode::Default),
                onbeforechange: move |event: PopoverChangeEvent| {
                    if (event.open && veto_open()[index]) || (!event.open && veto()[index]) { event.prevent_default(); }
                    record(rows, "request", name, json!({"previous":event.previous,"open":event.open,"vetoed":event.default_prevented(),"boundary":"typed public Rust PopoverChangeEvent"}));
                },
                onshowchanged: move |value| record(rows, "completed", name, json!({"open":value})),
                if index == 0 && focus_mode != FocusMode::Default { FocusContents { mode: focus_mode, dynamic_body } }
                else {
                    PopoverHeader { "Controller {name}" }
                    PopoverContent {
                        input { id: "popover-{name}-focus", r#type: "text", "aria-label": "Controller {name} focus", placeholder: "Native {name} focus" }
                        if index == 0 {
                            PopoverTrigger { popover: handles[1], "Child native trigger" }
                            if nested_child_host { Panel { index: 1 } }
                            PopoverTrigger { popover: handles[4], "Sibling native trigger" }
                            Panel { index: 4 }
                        }
                        if index == 1 { ChildState {} }
                    }
                }
            }
        }
    }
}

#[component]
fn FocusContents(mode: FocusMode, dynamic_body: Signal<bool>) -> Element {
    let ProbeContext { handles, rows, .. } = use_context();
    let hide_close = matches!(
        mode,
        FocusMode::Static
            | FocusMode::FooterOnly
            | FocusMode::NoTrigger
            | FocusMode::CachedAdd
            | FocusMode::CachedRemove
    );
    let body = match mode {
        FocusMode::Priority => {
            rsx! { button { id: "popover-focus-body", "First body action" } button { id: "popover-focus-body-later", "Later body action" } }
        }
        FocusMode::Positive => {
            rsx! { button { id: "popover-focus-body-four", tabindex: "4", "Body positive four" } button { id: "popover-focus-body-two", tabindex: "2", "Body positive two" } button { id: "popover-focus-body", "Body zero" } }
        }
        FocusMode::Excluded => rsx! {
            button { id: "popover-focus-hidden", hidden: true, "Hidden control" }
            button { id: "popover-focus-disabled", disabled: true, "Disabled control" }
            button { id: "popover-focus-negative", tabindex: "-1", "Negative tab order" }
            div { "inert": "", button { id: "popover-focus-inert", "Inert child control" } }
            Panel { index: 1 }
            button { id: "popover-focus-body", "Eligible body after exclusions" }
        },
        FocusMode::Tab
        | FocusMode::TabPositive
        | FocusMode::TabNested
        | FocusMode::TabDropdownActive
        | FocusMode::TabDropdownRoving => rsx! {
            button { id: "popover-tab-first", tabindex: if mode == FocusMode::TabPositive { "2" } else { "0" }, "Tab body first" }
            if mode == FocusMode::TabNested {
                PopoverTrigger { popover: handles[1], "Tab child trigger" }
                Panel { index: 1 }
            }
            if mode == FocusMode::TabDropdownActive || mode == FocusMode::TabDropdownRoving { TabDropdown { roving: mode == FocusMode::TabDropdownRoving } }
            button { id: "popover-tab-second", tabindex: if mode == FocusMode::TabPositive { "4" } else { "0" }, "Tab body second" }
            if mode != FocusMode::TabPositive {
                button { id: "popover-tab-prevent", onkeydown: move |event| if event.key() == Key::Tab { event.prevent_default(); record(rows, "tab-rust-consumer", "parent", json!({"action":"prevent-default","defaultEnabled":event.default_action_enabled()})); }, "Rust prevent-default consumer" }
                button { id: "popover-tab-stop", onkeydown: move |event| if event.key() == Key::Tab { event.stop_propagation(); record(rows, "tab-rust-consumer", "parent", json!({"action":"virtual-stop-propagation","defaultEnabled":event.default_action_enabled()})); }, "Rust virtual-stop consumer" }
            }
        },
        FocusMode::NoTrigger => {
            rsx! { button { id: "popover-focus-body", "Unanchored body action" } }
        }
        FocusMode::CachedAdd | FocusMode::CachedRemove => {
            rsx! { if dynamic_body() { button { id: "popover-focus-body", "Dynamic body action" } } p { "Dynamic content changes through Rust fixture state." } }
        }
        _ => rsx! { p { "Static body copy without an interactive control." } },
    };
    rsx! {
        PopoverHeader { hide_close,
            additional_items: if mode == FocusMode::Positive || mode == FocusMode::TabPositive { Some(rsx! { button { id: "popover-focus-header-positive", tabindex: "1", "Header positive one" } }) } else { None },
            {format!("Focus contract {}", mode.key())}
        }
        PopoverContent {
            {body}
        }
        PopoverFooter {
            if mode == FocusMode::Priority || mode == FocusMode::FooterOnly { button { id: "popover-focus-footer", "Footer action" } }
            else if mode == FocusMode::Positive { button { id: "popover-focus-footer-three", tabindex: "3", "Footer positive three" } }
            else if matches!(mode, FocusMode::Tab | FocusMode::TabPositive | FocusMode::TabNested | FocusMode::TabDropdownActive | FocusMode::TabDropdownRoving) { button { id: "popover-tab-last", tabindex: if mode == FocusMode::TabPositive { "3" } else { "0" }, "Tab footer last" } }
            else { span { "Static footer copy" } }
        }
    }
}

#[component]
fn TabDropdown(roving: bool) -> Element {
    let ProbeContext { rows, .. } = use_context();
    let dropdown = use_dropdown();
    let mut veto_close = use_signal(|| false);
    rsx! {
        div { id: "popover-tab-dropdown", "data-open": dropdown.is_open().to_string(), "data-veto-close": veto_close().to_string(), "data-mode": if roving { "roving" } else { "active-descendant" },
            DropdownTrigger { dropdown, "Nested Tab Dropdown" }
            Dropdown { controller: dropdown, label: "Nested Tab Dropdown", enable_top_layer: true, navigation_mode: if roving { DropdownNavigationMode::RovingTabindex } else { DropdownNavigationMode::ActiveDescendant },
                onbeforechange: move |event: DropdownChangeEvent| { if !event.open && veto_close() { event.prevent_default(); } record(rows, "tab-dropdown-request", "parent", json!({"open":event.open,"previous":event.previous,"vetoed":event.default_prevented()})); },
                onchange: move |open| record(rows, "tab-dropdown-completed", "parent", json!({"open":open})),
                DropdownItem { label: "Tab Dropdown first" }
                DropdownItem { label: "Tab Dropdown last" }
            }
            input { id: "popover-tab-dropdown-open", r#type: "hidden", oninput: move |_| dropdown.open() }
            input { id: "popover-tab-dropdown-close", r#type: "hidden", oninput: move |_| dropdown.close(false) }
            button { id: "popover-tab-dropdown-veto", tabindex: "-1", onclick: move |_| { let next = !veto_close(); veto_close.set(next); }, "Toggle nested Dropdown close veto" }
            input { id: "popover-tab-dropdown-veto-input", r#type: "hidden", oninput: move |event| veto_close.set(event.value() == "true") }
        }
    }
}

const TAB_NATIVE_JOURNAL: &str = r#"
const abort=new AbortController(),events=new WeakMap();let sequence=0;
const node=element=>element?{id:element.id,tag:element.tagName,label:element.getAttribute?.('aria-label'),host:element.closest?.('[data-ix-overlay-host]')?.getAttribute('data-ix-overlay-host')}:null;
const capture=(phase,event)=>{if(event.key!=='Tab')return;if(!events.has(event))events.set(event,++sequence);dioxus.send({phase,sequence:events.get(event),key:event.key,shift:event.shiftKey,trusted:event.isTrusted,defaultPrevented:event.defaultPrevented,cancelBubble:event.cancelBubble,target:node(event.target),active:node(document.activeElement)});};
document.addEventListener('keydown',event=>capture('document-capture',event),{capture:true,passive:true,signal:abort.signal});
document.addEventListener('keydown',event=>capture('document-bubble-observer',event),{passive:true,signal:abort.signal});
window.addEventListener('keydown',event=>capture('window-bubble-observer',event),{passive:true,signal:abort.signal});
dioxus.send({phase:'ready',readOnly:true});try{await dioxus.recv();}finally{abort.abort();}
"#;

#[component]
fn TabNativeJournal() -> Element {
    let mut rows = use_signal(Vec::<Value>::new);
    let channel = use_hook(|| Rc::new(RefCell::new(None::<dioxus::document::Eval>)));
    let cleanup = channel.clone();
    use_drop(move || {
        if let Some(eval) = cleanup.borrow_mut().take() {
            let _ = eval.send("finish");
        }
    });
    let text = serde_json::to_string(&*rows.read()).unwrap();
    rsx! {
        details {
            onmounted: move |_| {
                let mut eval = dioxus::document::eval(TAB_NATIVE_JOURNAL);
                *channel.borrow_mut() = Some(eval);
                spawn(async move { while let Ok(value) = eval.recv::<Value>().await { let mut current = rows.write(); current.push(value); if current.len() > 400 { current.remove(0); } } });
            },
            summary { "Read-only native Tab phase journal (trusted/browser vs synthetic)" }
            pre { id: "popover-tab-native-log", style: "white-space:pre-wrap;overflow-wrap:anywhere", "{text}" }
        }
    }
}

#[component]
fn ChildState() -> Element {
    let instance = use_hook(|| CHILD_INSTANCES.fetch_add(1, Ordering::Relaxed));
    let mut draft = use_signal(String::new);
    let mut count = use_signal(|| 0_usize);
    rsx! {
        section { id: "popover-child-local", "data-instance": instance.to_string(), "data-draft": draft(), "data-count": count().to_string(),
            label { "Retained child draft" input { id: "popover-child-draft", value: draft(), oninput: move |event| draft.set(event.value()) } }
            button { id: "popover-child-increment", onclick: move |_| count += 1, "Increment retained child: {count}" }
        }
    }
}

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const VERIFY: &str = r#"
const observations=[],driverJournal=[],assertions=[],tabCases=[],tabChecks=[];
try {
 await dioxus.recv();
 const sleep=ms=>new Promise(r=>setTimeout(r,ms)),get=id=>document.getElementById(id),names=['parent','child','unrelated','sticky','sibling'];
 const objects=new WeakMap();let scenario='initial',objectSequence=0;
 const objectId=node=>{if(!node)return null;if(!objects.has(node))objects.set(node,++objectSequence);return objects.get(node);};
 const fixture=()=>get('popover-fixture'),read=()=>JSON.parse(fixture()?.dataset.state??'{}'),logs=()=>JSON.parse(get('popover-controller-log')?.textContent??'[]');
 const native=name=>get('popover-'+name+'-wrapper')?.dataset.nativeId?get(get('popover-'+name+'-wrapper').dataset.nativeId):null;
 const describe=n=>n?{id:n.id,tag:n.tagName,label:n.getAttribute?.('aria-label'),objectId:objectId(n)}:null;
 const local=()=>{const node=get('popover-child-local');return{node:describe(node),instance:node?.dataset.instance,draft:node?.dataset.draft,count:node?.dataset.count,input:describe(get('popover-child-draft')),inputValue:get('popover-child-draft')?.value};};
 const geometry=host=>{if(!host)return null;const box=host.getBoundingClientRect(),style=getComputedStyle(host),ancestor=host.parentElement?.closest('[popover]');return{box:{x:box.x,y:box.y,width:box.width,height:box.height},clientRectCount:host.getClientRects().length,display:style.display,visibility:style.visibility,opacity:style.opacity,inert:host.inert,hidden:host.hidden,nativeParent:describe(host.parentElement),popoverAncestor:describe(ancestor),ancestorPopoverOpen:ancestor?.matches(':popover-open')??null,ancestors:(()=>{const rows=[];let node=host.parentElement;while(node&&rows.length<10){const value=getComputedStyle(node);rows.push({...describe(node),display:value.display,visibility:value.visibility,inert:node.inert,hidden:node.hidden});node=node.parentElement;}return rows;})()};};
 const snapshot=()=>({topology:fixture()?.dataset.topology,focusMode:fixture()?.dataset.focusMode,dynamicBody:fixture()?.dataset.dynamicBody,state:read(),veto:Object.fromEntries(names.map(name=>[name,get('popover-'+name+'-veto')?.checked])),vetoOpen:Object.fromEntries(names.map(name=>[name,get('popover-'+name+'-veto-open')?.checked])),native:Object.fromEntries(names.map(name=>{const host=native(name);let open=false;try{open=!!host?.matches(':popover-open');}catch{}return[name,{host:describe(host),connected:!!host?.isConnected,open,generation:host?.dataset.surfaceGeneration,overlayOpen:host?.dataset.ixOverlayOpen,geometry:geometry(host)}];})),childLocal:local(),active:describe(document.activeElement),log:logs()});
 const assert=(value,label)=>{if(!value)throw Error(label+' '+JSON.stringify({scenario,snapshot:snapshot()}));assertions.push({scenario,label});};
 const expect=(expected)=>{const current=snapshot();for(const name of names){const open=expected.includes(name);assert(current.state[name]===open,name+' Rust visibility '+open);assert(current.native[name].connected&&current.native[name].open===open,name+' connected native visibility '+open);}};
 const expectFocus=name=>assert(document.activeElement?.closest('[data-ix-overlay-host]')===native(name),'native focus belongs to opening '+name+' surface');
 const expectTrigger=name=>assert(document.activeElement?.getAttribute('popovertarget')===native(name)?.id,'native focus returns configured '+name+' trigger');
 const requests=(name,open)=>logs().filter(row=>row.kind==='request'&&row.target===name&&(open===undefined||row.details.open===open));
 const completions=(name,open)=>logs().filter(row=>row.kind==='completed'&&row.target===name&&(open===undefined||row.details.open===open));
 const wait=async(fn,label)=>{let end=performance.now()+5000;while(performance.now()<end){if(fn())return;await sleep(10);}throw Error('Timeout '+label+' '+JSON.stringify(snapshot()));};
 const settle=async()=>{for(let i=0;i<5;i++)await new Promise(requestAnimationFrame);await sleep(40);};
 const input=(id,value)=>{const el=get(id);if(!el)throw Error('Missing input '+id);el.value=value;el.dispatchEvent(new Event('input',{bubbles:true,composed:true}));};
 const command=async(action,target,value)=>{const previous=Number(fixture().dataset.completed),detail={action,target,value};driverJournal.push({scenario,kind:'public-Rust-command-via-synthetic-input',detail});input('popover-command',JSON.stringify(detail));await wait(()=>Number(fixture()?.dataset.completed)>previous,'command '+action+' '+target);await settle();};
 const fresh=async(label)=>{scenario=label;let old=fixture();input('popover-fresh','');await wait(()=>fixture()&&fixture()!==old&&names.every(name=>native(name)),'fresh fixture native mounts');await settle();};
 const freshTopology=async(topology,label)=>{scenario=label;const old=fixture();input('popover-topology',topology);await wait(()=>fixture()&&fixture()!==old&&fixture().dataset.topology===topology&&names.every(name=>native(name)),'fresh actual '+topology+' topology');await settle();};
 const freshFocus=async(mode,label)=>{scenario=label;const old=fixture();input('popover-focus-mode',mode);await wait(()=>fixture()&&fixture()!==old&&fixture().dataset.focusMode===mode&&names.every(name=>native(name)),'fresh actual focus mode '+mode);await settle();};
 const save=(extra={})=>observations.push({label:scenario,...extra,snapshot:snapshot()});
 const pair=async()=>{await command('show','parent');await command('show','child');};
 await wait(()=>fixture()&&names.every(name=>native(name)),'initial native mounts');await settle();expect([]);save();
 await fresh('public parent show and repeated show');await command('show','parent');const repeatBaseline=logs().length;await command('show','parent');expect(['parent']);assert(requests('parent',true).length===1,'one true parent request across repeated show');assert(logs().slice(repeatBaseline).every(row=>row.kind!=='request'&&row.kind!=='completed'),'repeat show emits no request or completion');expectFocus('parent');save();
 await fresh('public parent and child show then parent hide');await pair();expect(['parent','child']);expectFocus('child');save();const pairStart=logs().length;await command('hide','parent');expect([]);assert(logs().slice(pairStart).filter(row=>row.kind==='request'&&['parent','child'].includes(row.target)).map(row=>row.target+':'+row.details.open).join(',')==='parent:false,child:false','parent accepted false request precedes child false request');assert(completions('parent',false).length===1&&completions('child',false).length===1,'both accepted closures complete once');expectTrigger('parent');save();
 await fresh('parent close veto stops child close requests');await pair();await command('veto','parent',true);const parentVetoChildCount=requests('child').length;await command('hide','parent');expect(['parent','child']);assert(requests('child').length===parentVetoChildCount,'parent veto leaves child request count unchanged');assert(requests('parent',false).at(-1)?.details.vetoed===true,'parent false request records actual veto');assert(completions('parent',false).length===0&&completions('child',false).length===0,'parent veto produces no false completion');save();
 await fresh('child close veto while parent closes');await pair();await command('veto','child',true);await command('hide','parent');expect(['child']);assert(requests('child',false).at(-1)?.details.vetoed===true,'child false request records actual veto');assert(completions('parent',false).length===1&&completions('child',false).length===0,'child veto does not roll back parent completion');save();
 await fresh('closed child receives dismissal attempt');await command('show','parent');const closedChildBaseline=requests('child',false).length;await command('hide','parent');expect([]);assert(requests('child',false).length===closedChildBaseline+1,'parent dismissal requests false from already closed cached child');assert(requests('child',false).at(-1)?.details.previous===false,'closed child false request has exact previous=false');assert(completions('child',false).length===0,'closed child request has no false completion');save();
 await fresh('closed parent explicit hide request');await command('hide','parent');expect([]);assert(requests('parent',false).length===1&&requests('parent',false)[0].details.previous===false,'already closed parent still receives exact false request');assert(completions('parent').length===0,'closed parent dismissal has no completion');save();
 await fresh('nested child direct hide leaves parent');await pair();const directParentBaseline=requests('parent').length;await command('hide','child');expect(['parent']);assert(requests('parent').length===directParentBaseline,'direct child close does not request parent');assert(completions('child',false).length===1,'direct child close completes once');expectTrigger('child');save();
 await fresh('unrelated outside-enabled Popover opening');await pair();const unrelatedStart=logs().length;await command('show','unrelated');expect(['unrelated']);assert(logs().slice(unrelatedStart).filter(row=>row.kind==='request')[0]?.target==='unrelated'&&logs().slice(unrelatedStart).filter(row=>row.kind==='request')[0]?.details.open===true,'new unrelated true request precedes dismissal requests');expectFocus('unrelated');save();
 await fresh('unrelated sticky Popover remains during opening');await command('show','sticky');await command('show','unrelated');expect(['sticky','unrelated']);assert(requests('sticky',false).length===0,'sticky outside policy blocks unrelated dismissal request');expectFocus('unrelated');save();
 await fresh('unrelated close veto does not veto new opening');await pair();await command('veto','parent',true);await command('show','unrelated');expect(['parent','unrelated']);assert(requests('parent',false).at(-1)?.details.vetoed===true,'unrelated opening observes existing parent close veto');assert(completions('unrelated',true).length===1,'new unrelated opening completes despite existing veto');assert(requests('child',false).at(-1)?.details.previous===true&&completions('child',false).length===1,'unrelated opening separately dismisses child after parent veto');expectFocus('unrelated');save();
 await fresh('vetoed unrelated opening leaves actual nested pair untouched');await pair();await command('veto-open','unrelated',true);const openingVetoBaseline=logs().length;await command('show','unrelated');expect(['parent','child']);assert(logs().slice(openingVetoBaseline).filter(row=>row.kind==='request').map(row=>row.target+':'+row.details.open+':'+row.details.vetoed).join(',')==='unrelated:true:true','opening veto emits only its true vetoed request without false requests');assert(completions('unrelated',true).length===0,'opening veto has no opening completion');expectFocus('child');save();
 await fresh('sibling opening closes child while retaining cached parent');await pair();const siblingParentBaseline=requests('parent').length;await command('show','sibling');expect(['parent','sibling']);assert(requests('parent').length===siblingParentBaseline,'sibling opening does not request cached parent dismissal');assert(completions('child',false).length===1&&completions('sibling',true).length===1,'sibling opens and old child closes once');expectFocus('sibling');save();
 await fresh('Escape targets selected nested child only');await pair();get('popover-child-focus').focus();const escapeParentBaseline=requests('parent').length,escape=new KeyboardEvent('keydown',{key:'Escape',code:'Escape',bubbles:true,composed:true,cancelable:true});get('popover-child-focus').dispatchEvent(escape);driverJournal.push({scenario,kind:'synthetic-key',key:'Escape',trusted:escape.isTrusted,defaultPrevented:escape.defaultPrevented});await settle();expect(['parent']);assert(requests('parent').length===escapeParentBaseline,'child Escape does not request parent');assert(completions('child',false).length===1,'child Escape completes selected child dismissal');expectTrigger('child');save();
 await fresh('retained native child identity and Rust local state across parent hide/reopen');await pair();
 const retained={parent:native('parent'),child:native('child'),local:get('popover-child-local'),input:get('popover-child-draft'),instance:local().instance};
 input('popover-child-draft','retained draft via synthetic native input');get('popover-child-increment').click();driverJournal.push({scenario,kind:'synthetic-input-and-Rust-button-click',draft:'retained draft via synthetic native input',button:'popover-child-increment'});await settle();assert(local().draft==='retained draft via synthetic native input'&&local().count==='1','actual Rust child local draft and counter updated');
 await command('hide','parent');expect([]);assert(native('parent')===retained.parent&&native('child')===retained.child&&get('popover-child-local')===retained.local&&get('popover-child-draft')===retained.input,'native parent, child, local section and input HTMLElement identity retained while closed');assert(local().instance===retained.instance&&local().draft==='retained draft via synthetic native input'&&local().count==='1','Rust component token, draft and local counter retained while closed');expectTrigger('parent');save();
 await command('show','parent');await command('show','child');expect(['parent','child']);assert(native('parent')===retained.parent&&native('child')===retained.child&&get('popover-child-local')===retained.local&&get('popover-child-draft')===retained.input,'same actual native objects reused after reopening');assert(local().instance===retained.instance&&local().draft==='retained draft via synthetic native input'&&local().inputValue==='retained draft via synthetic native input'&&local().count==='1','same Rust component and local state survive reopen');expectFocus('child');save();
 for(const topology of ['nested','sibling']){
  await freshTopology(topology,'actual '+topology+' child host: vetoed child survives parent hide/reopen');await pair();
  input('popover-child-draft','retained '+topology+' topology draft');get('popover-child-increment').click();await settle();
  const held={parent:native('parent'),child:native('child'),local:get('popover-child-local'),input:get('popover-child-draft'),instance:local().instance};
  assert((native('parent').contains(native('child'))) === (topology==='nested'),'actual native child host DOM nesting matches '+topology);
  const trigger=document.querySelector('[popovertarget="'+native('child').id+'"]');assert(native('parent').contains(trigger),'actual child trigger remains nested inside parent in '+topology);
  await command('veto','child',true);await command('hide','parent');expect(['child']);
  assert(native('parent')===held.parent&&native('child')===held.child&&get('popover-child-local')===held.local&&get('popover-child-draft')===held.input,'actual native node identities retained after '+topology+' parent closes');
  assert(local().instance===held.instance&&local().draft==='retained '+topology+' topology draft'&&local().count==='1','Rust local instance and draft/count retained in closed '+topology+' topology');save();
  await command('show','parent');expect(['parent','child']);
  assert(native('parent')===held.parent&&native('child')===held.child&&get('popover-child-local')===held.local&&get('popover-child-draft')===held.input,'actual native node identities retained after '+topology+' parent reopens');
  assert(local().instance===held.instance&&local().draft==='retained '+topology+' topology draft'&&local().count==='1','Rust local instance and draft/count retained after '+topology+' reopen');save();
 }
 await freshTopology('hover','hover already-focused restore permits later fresh focus activation');
 const hoverTrigger=document.querySelector('[popovertarget="'+native('parent').id+'"]'),hoverFocusRows=[];
 const focusListener=event=>{const row={scenario,kind:'actual-native-focus-on-hover-trigger',trusted:event.isTrusted,activeIsTrigger:document.activeElement===hoverTrigger};hoverFocusRows.push(row);driverJournal.push(row);};
 hoverTrigger.addEventListener('focus',focusListener);
 driverJournal.push({scenario,kind:'programmatic-native-focus',target:describe(hoverTrigger),phase:'initial-hover-focus'});hoverTrigger.focus();
 await wait(()=>read().parent===true&&native('parent').matches(':popover-open'),'hover trigger focus actually opens parent');await settle();expect(['parent']);assert(document.activeElement===hoverTrigger,'hover opening preserves actual trigger focus');
 const hoverRepeatBaseline=logs().length;await command('show','parent');expect(['parent']);assert(logs().slice(hoverRepeatBaseline).every(row=>row.kind!=='request'&&row.kind!=='completed'),'public show of focus-opened hover parent is request-free');assert(document.activeElement===hoverTrigger,'repeated public show keeps hover trigger focused');save({phase:'hover-open-trigger-still-focused'});
 const closeFocusBaseline=hoverFocusRows.length;await command('hide','parent');expect([]);assert(document.activeElement===hoverTrigger,'already-focused hover trigger remains focused after public hide and following RAFs');assert(hoverFocusRows.length===closeFocusBaseline,'restoring already-focused trigger dispatches no new native focus event');save({phase:'closed-after-restore-and-following-frames'});
 driverJournal.push({scenario,kind:'programmatic-native-focus',target:describe(get('popover-outside')),phase:'outside-before-refocus'});get('popover-outside').focus();await settle();expect([]);assert(document.activeElement===get('popover-outside'),'actual outside focus occurs before fresh trigger focus');save({phase:'outside-focus-before-refocus'});
 driverJournal.push({scenario,kind:'programmatic-native-focus',target:describe(hoverTrigger),phase:'fresh-focus-after-restore'});hoverTrigger.focus();
 await wait(()=>read().parent===true&&native('parent').matches(':popover-open'),'fresh focus reopens hover parent after already-focused restoration');await settle();expect(['parent']);assert(hoverFocusRows.length===closeFocusBaseline+1,'fresh refocus dispatches one actual native focus event');assert(document.activeElement===hoverTrigger,'fresh hover focus opening keeps focus on trigger');save({phase:'fresh-focus-reopened'});hoverTrigger.removeEventListener('focus',focusListener);
 const focusOutside=async(id,phase)=>{const target=get(id);driverJournal.push({scenario,kind:'programmatic-native-focus',target:describe(target),phase});target.focus();await settle();assert(document.activeElement===target,'actual '+phase+' native outside focus');};
 const activeId=id=>assert(document.activeElement===get(id),'source-directed focus selects '+id);
 for(const [mode,target] of [['priority','popover-focus-body'],['header-only',null],['footer-only','popover-focus-footer'],['positive','popover-focus-body-two'],['excluded','popover-focus-body']]){
  await freshFocus(mode,'opening focus policy '+mode);await focusOutside('popover-before','before-show');await command('show','parent');expect(['parent']);save({phase:'open-before-focus-verdict'});
  if(target)activeId(target);else assert(document.activeElement===native('parent').querySelector('.ix-popover-header button'),'header close fallback only when no eligible content action');
  if(mode==='excluded')assert(document.activeElement.closest('[data-ix-overlay-host]')===native('parent'),'closed nested overlay controls excluded from parent initial focus');
  await focusOutside('popover-outside','before-hide');await command('hide','parent');expect([]);expectTrigger('parent');save({phase:'interactive-close-configured-trigger'});
 }
 await freshFocus('static','static configured Popover preserves outside focus on close');await focusOutside('popover-before','before-static-show');await command('show','parent');expect(['parent']);activeId('popover-before');save({phase:'static-open-no-focus-move'});
 await focusOutside('popover-outside','after-static-show');await command('hide','parent');expect([]);activeId('popover-outside');assert(requests('parent',false).length===1&&completions('parent',false).length===1,'static close retains separate request and completion');save({phase:'static-close-outside-retained'});
 await freshFocus('no-trigger','unconfigured interactive Popover closes without fallback-focus steal');assert(!document.querySelector('[popovertarget="'+native('parent').id+'"]'),'actual parent has no configured trigger');await focusOutside('popover-before','before-unconfigured-show');await command('show','parent');expect(['parent']);activeId('popover-focus-body');save({phase:'unconfigured-open-body-focused'});
 await focusOutside('popover-outside','after-unconfigured-show');await command('hide','parent');expect([]);activeId('popover-outside');save({phase:'unconfigured-close-outside-retained'});
 await freshFocus('cached-add','opening-static focus eligibility remains cached after Rust body control addition');await focusOutside('popover-before','before-static-add-show');await command('show','parent');expect(['parent']);activeId('popover-before');save({phase:'first-open-static'});
 await command('focus-body','parent',true);assert(get('popover-focus-body')&&fixture().dataset.dynamicBody==='true','actual Rust body control added while open');await focusOutside('popover-outside','after-body-added');await command('hide','parent');expect([]);activeId('popover-outside');save({phase:'first-close-keeps-static-opening-eligibility'});
 await command('show','parent');expect(['parent']);activeId('popover-focus-body');save({phase:'second-open-refreshes-interactive-eligibility'});await focusOutside('popover-outside','before-second-interactive-hide');await command('hide','parent');expect([]);expectTrigger('parent');save({phase:'second-close-restores-configured-trigger'});
 await freshFocus('cached-remove','opening-interactive focus eligibility remains cached after Rust body control removal');await focusOutside('popover-before','before-interactive-remove-show');await command('show','parent');expect(['parent']);activeId('popover-focus-body');save({phase:'first-open-interactive'});
 await command('focus-body','parent',false);assert(!get('popover-focus-body')&&fixture().dataset.dynamicBody==='false','actual Rust body control removed while open');await focusOutside('popover-outside','after-body-removed');await command('hide','parent');expect([]);expectTrigger('parent');save({phase:'first-close-keeps-interactive-opening-eligibility'});
 await focusOutside('popover-before','before-second-static-show');await command('show','parent');expect(['parent']);activeId('popover-before');save({phase:'second-open-refreshes-static-eligibility'});await focusOutside('popover-outside','before-second-static-hide');await command('hide','parent');expect([]);activeId('popover-outside');save({phase:'second-close-retains-outside-focus'});
 const nativeTabRows=()=>JSON.parse(get('popover-tab-native-log')?.textContent??'[]');
 const checkTab=(row,expected)=>{
  const matched=row.afterActiveObject===objectId(expected.target)&&row.immediateDefaultPrevented===expected.prevented&&row.reachedWindow===expected.window;
  tabChecks.push({label:row.label,matched,expected:{active:describe(expected.target),defaultPrevented:expected.prevented,reachedWindow:expected.window},actual:{active:row.afterActive,defaultPrevented:row.immediateDefaultPrevented,reachedWindow:row.reachedWindow}});
 };
 const tab=async(label,target,shift=false)=>{
  if(!target)throw Error('Missing native Tab target '+label);target.focus();await settle();assert(document.activeElement===target,'actual native seeded Tab focus '+label);
  const before={state:read(),active:describe(document.activeElement),dropdownOpen:get('popover-tab-dropdown')?.dataset.open??null},nativeStart=nativeTabRows().length,rustStart=logs().length,event=new KeyboardEvent('keydown',{key:'Tab',code:'Tab',shiftKey:shift,bubbles:true,cancelable:true,composed:true});
  let reachedDocument=false,reachedWindow=false;const listeners=new AbortController();
  document.addEventListener('keydown',actual=>{if(actual===event)reachedDocument=true;},{passive:true,signal:listeners.signal});
  window.addEventListener('keydown',actual=>{if(actual===event)reachedWindow=true;},{passive:true,signal:listeners.signal});
  driverJournal.push({scenario,kind:'synthetic-native-Tab',label,shift,trusted:event.isTrusted,target:describe(target),beforeActive:describe(document.activeElement),nativeDefaultNavigation:false});
  target.dispatchEvent(event);const immediateDefaultPrevented=event.defaultPrevented,immediateActive=describe(document.activeElement);await settle();
  const row={label,shift,trusted:event.isTrusted,target:describe(target),immediateDefaultPrevented,settledDefaultPrevented:event.defaultPrevented,reachedDocument,reachedWindow,immediateActive,afterActive:describe(document.activeElement),afterActiveObject:objectId(document.activeElement),before,nativePhases:nativeTabRows().slice(nativeStart),rustCallbacks:logs().slice(rustStart),afterState:read(),dropdownOpen:get('popover-tab-dropdown')?.dataset.open??null};
  tabCases.push(row);save({phase:'synthetic-Tab',tab:row});listeners.abort();return row;
 };
 await wait(()=>nativeTabRows().some(row=>row.phase==='ready'),'read-only document/window Tab recorder ready');
 await freshFocus('tab','document Tab boundary policy and native propagation');await command('show','parent');
 const tabClose=()=>native('parent').querySelector('.ix-dialog-close');
 let row=await tab('body interior forward',get('popover-tab-first'));checkTab(row,{target:get('popover-tab-first'),prevented:false,window:true});
 row=await tab('body interior backward',get('popover-tab-second'),true);checkTab(row,{target:get('popover-tab-second'),prevented:false,window:true});
 row=await tab('footer forward wraps header',get('popover-tab-last'));checkTab(row,{target:tabClose(),prevented:true,window:false});
 row=await tab('header reverse wraps footer',tabClose(),true);checkTab(row,{target:get('popover-tab-last'),prevented:true,window:false});
 await tab('Rust target prevent-default consumer (native cancellation observed)',get('popover-tab-prevent'));
 await tab('Rust target virtual-stop consumer (native propagation observed)',get('popover-tab-stop'));
 row=await tab('unknown panel active target is untouched',native('parent'));checkTab(row,{target:native('parent'),prevented:false,window:true});
 row=await tab('outside active target is untouched',get('popover-outside'));checkTab(row,{target:get('popover-outside'),prevented:false,window:true});
 await freshFocus('static','static document Tab does not invent panel focus');await command('show','parent');row=await tab('static panel has no trap',native('parent'));checkTab(row,{target:native('parent'),prevented:false,window:true});
 await freshFocus('cached-add','static opening does not acquire a document trap after body control addition');await command('show','parent');await command('focus-body','parent',true);
 row=await tab('cached-static added body control has no trap',get('popover-focus-body'));checkTab(row,{target:get('popover-focus-body'),prevented:false,window:true});
 await freshFocus('cached-remove','interactive opening with a now-empty live candidate list');await command('show','parent');await command('focus-body','parent',false);
 row=await tab('live empty candidate list returns without focus command',native('parent'));checkTab(row,{target:native('parent'),prevented:false,window:true});
 await freshFocus('tab-positive','positive tabindex full Tab order differs from opening role priority');await command('show','parent');
 row=await tab('positive body interior remains native default',get('popover-tab-first'));checkTab(row,{target:get('popover-tab-first'),prevented:false,window:true});
 row=await tab('positive header first reverse wraps zero close',get('popover-focus-header-positive'),true);checkTab(row,{target:tabClose(),prevented:true,window:false});
 row=await tab('zero header close last forward wraps positive header',tabClose());checkTab(row,{target:get('popover-focus-header-positive'),prevented:true,window:false});
 await freshFocus('tab-nested','parent defers to actual active nested Popover owner');await command('show','parent');await command('show','child');
 const childClose=native('child').querySelector('.ix-dialog-close');
 row=await tab('nested child interior defers parent',childClose);checkTab(row,{target:childClose,prevented:false,window:true});
 row=await tab('nested child forward boundary owns wrap',get('popover-child-increment'));checkTab(row,{target:childClose,prevented:true,window:false});
 row=await tab('parent focus wraps while child remains topmost',get('popover-tab-last'));checkTab(row,{target:tabClose(),prevented:true,window:false});
 await freshFocus('tab','parent defers to unrelated actual topmost Popover');await command('show','parent');await command('veto','parent',true);await command('show','unrelated');expect(['parent','unrelated']);
 const unrelatedClose=native('unrelated').querySelector('.ix-dialog-close');row=await tab('unrelated topmost interior leaves parent untouched',unrelatedClose);checkTab(row,{target:unrelatedClose,prevented:false,window:true});
 const openTabDropdown=async()=>{input('popover-tab-dropdown-open','');await wait(()=>get('popover-tab-dropdown')?.dataset.open==='true','actual nested Dropdown opens');await settle();};
 const checkDropdown=(result,target,retained)=>{
  const matched=result.afterActiveObject===objectId(target)&&result.immediateDefaultPrevented===true&&result.dropdownOpen===String(retained)&&result.afterState.parent===true;
  tabChecks.push({label:result.label,matched,expected:{active:describe(target),defaultPrevented:true,dropdownOpen:String(retained),parentOpen:true},actual:{active:result.afterActive,defaultPrevented:result.immediateDefaultPrevented,dropdownOpen:result.dropdownOpen,parentOpen:result.afterState.parent}});
 };
 for(const mode of ['tab-dropdown-active','tab-dropdown-roving']){
  await freshFocus(mode,'actual nested Dropdown Tab ownership '+mode);await command('show','parent');
  for(const backwards of [false,true]){
   await openTabDropdown();
   const dropdown=get('popover-tab-dropdown'),trigger=dropdown.querySelector('[data-ix-overlay-trigger]'),item=dropdown.querySelector('[data-ix-dropdown-item]');
   const target=backwards?(mode==='tab-dropdown-roving'?trigger:get('popover-tab-first')):get('popover-tab-second');
   const result=await tab(mode+' '+(backwards?'reverse':'forward')+' parent release',mode==='tab-dropdown-active'?trigger:item,backwards);checkDropdown(result,target,false);
  }
 }
 await freshFocus('tab-dropdown-active','vetoed Dropdown Tab still queues parent-adjacent focus');await command('show','parent');await openTabDropdown();input('popover-tab-dropdown-veto-input','true');await wait(()=>get('popover-tab-dropdown')?.dataset.vetoClose==='true','actual Rust Dropdown close veto');
 const vetoTrigger=get('popover-tab-dropdown').querySelector('[data-ix-overlay-trigger]');row=await tab('vetoed active-descendant forward release retains child but focuses parent',vetoTrigger);checkDropdown(row,get('popover-tab-second'),true);
 assert(row.rustCallbacks.some(value=>value.kind==='tab-dropdown-request'&&value.details.open===false&&value.details.vetoed===true),'actual Dropdown false request was vetoed before retained parent-adjacent focus');
 assert(tabChecks.every(check=>check.matched),'all named document Tab policy comparisons match pinned source contract');
 return {ok:true,observations,assertions,driverJournal,tabCases,tabChecks,input:'synthetic input controls invoking real public Rust handles; synthetic KeyboardEvent, programmatic focus and synthetic child button click',limits:['strict named endpoints and callback directions compared with pinned upstream contract; not complete Popover parity','WeakMap object identity and direct object equality certify retained actual nodes only within each fresh fixture; topology controls remount deliberately', 'geometry/computed styles/ancestor facts are diagnostics; no positive visibility or geometry parity asserted','focus-mode fixtures intentionally remount on mode selection; dynamic body add/remove uses Rust Signal state without native application policy injection', 'synthetic Tab has no native focus-navigation default; browser native-emulated Tab and read-only trusted phase recorder are separate evidence', 'Rust consumer callback cancellation and native DOM event cancellation/propagation are measured separately', 'Dropdown forward/reverse and veto release checks follow the paired source contract; native propagation flags remain separately observed', 'no physical input, accessibility, Brand visual, multiroot or cross-document certification'],renderer:navigator.userAgent};
} catch(error) { return {ok:false,error:String(error),observations,assertions,driverJournal,tabCases,tabChecks}; }
"#;
