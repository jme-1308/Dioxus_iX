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
    HeaderSmallIsolated,
    HeaderSmallChild,
    HeaderHistoryChild,
    HeaderLargeIsolated,
    HeaderSmallNoOverflow,
    HeaderSmallChildNoOverflow,
    AnchorRightStart,
    AnchorBottomStart,
    AnchorBottomEnd,
    AnchorSuppressed,
    AnchorNested,
    AnchorFocus,
    AnchorFocusPointer,
    AnchorFocusNoTrap,
    TriggerOthers,
    TriggerOthersTopology,
}

impl FocusMode {
    const ALL: [Self; 31] = [
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
        Self::HeaderSmallIsolated,
        Self::HeaderSmallChild,
        Self::HeaderHistoryChild,
        Self::HeaderLargeIsolated,
        Self::HeaderSmallNoOverflow,
        Self::HeaderSmallChildNoOverflow,
        Self::AnchorRightStart,
        Self::AnchorBottomStart,
        Self::AnchorBottomEnd,
        Self::AnchorSuppressed,
        Self::AnchorNested,
        Self::AnchorFocus,
        Self::AnchorFocusPointer,
        Self::AnchorFocusNoTrap,
        Self::TriggerOthers,
        Self::TriggerOthersTopology,
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
            Self::HeaderSmallIsolated => "header-small-isolated",
            Self::HeaderSmallChild => "header-small-child",
            Self::HeaderHistoryChild => "header-history-child",
            Self::HeaderLargeIsolated => "header-large-isolated",
            Self::HeaderSmallNoOverflow => "header-small-no-overflow",
            Self::HeaderSmallChildNoOverflow => "header-small-child-no-overflow",
            Self::AnchorRightStart => "anchor-right-start",
            Self::AnchorBottomStart => "anchor-bottom-start",
            Self::AnchorBottomEnd => "anchor-bottom-end",
            Self::AnchorSuppressed => "anchor-suppressed",
            Self::AnchorNested => "anchor-nested",
            Self::AnchorFocus => "anchor-focus",
            Self::AnchorFocusPointer => "anchor-focus-pointer",
            Self::AnchorFocusNoTrap => "anchor-focus-no-trap",
            Self::TriggerOthers => "trigger-others",
            Self::TriggerOthersTopology => "trigger-others-topology",
        }
    }
    fn from_key(value: &str) -> Self {
        Self::ALL
            .into_iter()
            .find(|mode| mode.key() == value)
            .unwrap_or_default()
    }
    fn is_anchor(self) -> bool {
        matches!(
            self,
            Self::AnchorRightStart
                | Self::AnchorBottomStart
                | Self::AnchorBottomEnd
                | Self::AnchorSuppressed
                | Self::AnchorNested
                | Self::AnchorFocus
                | Self::AnchorFocusPointer
                | Self::AnchorFocusNoTrap
        )
    }
    fn is_trigger_others(self) -> bool {
        matches!(self, Self::TriggerOthers | Self::TriggerOthersTopology)
    }
    fn is_header(self) -> bool {
        matches!(
            self,
            Self::HeaderSmallIsolated
                | Self::HeaderSmallChild
                | Self::HeaderHistoryChild
                | Self::HeaderLargeIsolated
                | Self::HeaderSmallNoOverflow
                | Self::HeaderSmallChildNoOverflow
        )
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
            futures_timer::Delay::new(std::time::Duration::from_secs(150)).await;
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
                for mode in FocusMode::ALL.into_iter().filter(|mode| !mode.is_anchor() && !mode.is_trigger_others()) { button { id: format!("popover-focus-mode-{}", mode.key()), onclick: move |_| { focus_mode.set(mode); nested_child_host.set(true); hover_parent.set(false); generation += 1; }, {format!("Fresh focus: {}", mode.key())} } }
            }
            input { id: "popover-focus-mode", r#type: "hidden", oninput: move |event| { focus_mode.set(FocusMode::from_key(&event.value())); nested_child_host.set(true); hover_parent.set(false); generation += 1; } }
            button { id: "popover-before", "Before opening focus target" }
            button { id: "popover-outside", "Outside focus and Escape target" }
            for current in [generation()] { Fixture { key: "{current}", generation: current, nested_child_host: nested_child_host(), hover_parent: hover_parent(), focus_mode: focus_mode() } }
            TabNativeJournal {}
            div { style: "display:flex;gap:8px;flex-wrap:wrap;margin:12px 0",
                for mode in FocusMode::ALL.into_iter().filter(|mode| mode.is_anchor()) { button { id: format!("popover-focus-mode-{}", mode.key()), onclick: move |_| { focus_mode.set(mode); nested_child_host.set(true); hover_parent.set(false); generation += 1; }, {format!("Fresh partial anchor: {}", mode.key())} } }
            }
            div { style: "display:flex;gap:8px;flex-wrap:wrap;margin:12px 0",
                for mode in FocusMode::ALL.into_iter().filter(|mode| mode.is_trigger_others()) { button { id: format!("popover-focus-mode-{}", mode.key()), onclick: move |_| { focus_mode.set(mode); nested_child_host.set(true); hover_parent.set(false); generation += 1; }, {format!("Fresh actual trigger: {}", mode.key())} } }
            }
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
            if focus_mode.is_header() {
                HeaderProbeOwner { initial_small: !matches!(focus_mode, FocusMode::HeaderHistoryChild | FocusMode::HeaderLargeIsolated), with_child: !matches!(focus_mode, FocusMode::HeaderSmallIsolated | FocusMode::HeaderLargeIsolated | FocusMode::HeaderSmallNoOverflow), permanent_overflow: !matches!(focus_mode, FocusMode::HeaderSmallNoOverflow | FocusMode::HeaderSmallChildNoOverflow), Panel { index: 0 } }
            } else { Panel { index: 0 } }
            if !nested_child_host || (focus_mode != FocusMode::Default && focus_mode != FocusMode::Excluded && focus_mode != FocusMode::TabNested) { Panel { index: 1 } }
            if focus_mode != FocusMode::Default { Panel { index: 4 } }
            PopoverTrigger { popover: handles[2], "Unrelated native trigger" }
            Panel { index: 2 }
            PopoverTrigger { popover: handles[3], "Sticky native trigger" }
            Panel { index: 3 }
            if focus_mode.is_anchor() { AnchorProbe { mode: focus_mode } }
            if focus_mode.is_trigger_others() { TriggerOthersProbe { topology: focus_mode == FocusMode::TriggerOthersTopology } }
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
    let hide_close = mode.is_header()
        || matches!(
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
        FocusMode::HeaderSmallIsolated
        | FocusMode::HeaderSmallChild
        | FocusMode::HeaderHistoryChild
        | FocusMode::HeaderLargeIsolated
        | FocusMode::HeaderSmallNoOverflow
        | FocusMode::HeaderSmallChildNoOverflow => rsx! {
            button { id: "popover-parent-before-header", "Parent before Header" }
            HeaderProbeContents {}
            button { id: "popover-parent-after-header", "Parent after Header" }
        },
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

#[derive(Clone, Copy)]
struct HeaderProbeContext {
    child: DropdownHandle,
    small: Signal<bool>,
    root_veto: Signal<bool>,
    root_open: Signal<bool>,
    with_child: bool,
    permanent_overflow: bool,
}

#[component]
fn HeaderProbeOwner(
    initial_small: bool,
    with_child: bool,
    permanent_overflow: bool,
    children: Element,
) -> Element {
    let ProbeContext { rows, .. } = use_context();
    let child = use_dropdown();
    let mut small = use_signal(|| initial_small);
    let mut roving = use_signal(|| false);
    let mut root_veto = use_signal(|| false);
    let mut child_veto = use_signal(|| false);
    let root_open = use_signal(|| false);
    use_context_provider(|| HeaderProbeContext {
        child,
        small,
        root_veto,
        root_open,
        with_child,
        permanent_overflow,
    });
    rsx! {
        section { id: "popover-header-probe", "data-initial-small": initial_small.to_string(), "data-small": small().to_string(), "data-roving": roving().to_string(), "data-root-veto": root_veto().to_string(), "data-child-veto": child_veto().to_string(), "data-root-open": root_open().to_string(), "data-child-open": child.is_open().to_string(), "data-with-child": with_child.to_string(), "data-permanent-overflow": permanent_overflow.to_string(),
            div { style: "display:flex;gap:8px;flex-wrap:wrap;margin:8px 0",
                button { id: "popover-header-toggle-size", tabindex: "-1", onmousedown: move |event| event.prevent_default(), onclick: move |event| {
                    event.stop_propagation();
                    #[cfg(feature = "web")]
                    {
                        let data = event.data();
                        if let Some(native) = data.downcast::<web_sys::MouseEvent>() { native.stop_propagation(); }
                    }
                    let next = !small(); small.set(next);
                }, "Toggle retained Header small / large" }
                button { id: "popover-header-toggle-roving", tabindex: "-1", onclick: move |_| { let next = !roving(); roving.set(next); }, "Toggle child navigation mode" }
                button { id: "popover-header-toggle-root-veto", tabindex: "-1", onclick: move |_| { let next = !root_veto(); root_veto.set(next); }, "Toggle Header close veto" }
                button { id: "popover-header-toggle-child-veto", tabindex: "-1", onclick: move |_| { let next = !child_veto(); child_veto.set(next); }, "Toggle Header child close veto" }
            }
            input { id: "popover-header-small", r#type: "hidden", oninput: move |event| small.set(event.value() == "true") }
            input { id: "popover-header-roving", r#type: "hidden", oninput: move |event| roving.set(event.value() == "true") }
            input { id: "popover-header-root-veto", r#type: "hidden", oninput: move |event| root_veto.set(event.value() == "true") }
            input { id: "popover-header-child-veto", r#type: "hidden", oninput: move |event| child_veto.set(event.value() == "true") }
            {children}
            if with_child {
                div { id: "popover-header-child-surface", "data-open": child.is_open().to_string(),
                    Dropdown { controller: child, label: "Header Popover child", enable_top_layer: true, navigation_mode: if roving() { DropdownNavigationMode::RovingTabindex } else { DropdownNavigationMode::ActiveDescendant },
                        onbeforechange: move |event: DropdownChangeEvent| { if !event.open && child_veto() { event.prevent_default(); } record(rows, "header-child-request", "header-child", json!({"open":event.open,"previous":event.previous,"vetoed":event.default_prevented()})); },
                        onchange: move |open| record(rows, "header-child-completed", "header-child", json!({"open":open})),
                        DropdownItem { label: "Header child first" }
                        DropdownItem { label: "Header child last" }
                    }
                }
            }
        }
    }
}

#[component]
fn HeaderProbeContents() -> Element {
    let ProbeContext { rows, .. } = use_context();
    let HeaderProbeContext {
        child,
        small,
        root_veto,
        mut root_open,
        with_child,
        permanent_overflow,
    } = use_context();
    rsx! {
        div { id: "popover-header-content", style: "width:640px;max-width:100%;height:240px",
            Application { force_breakpoint: if small() { ApplicationBreakpoint::Small } else { ApplicationBreakpoint::Large },
                header: rsx! { ApplicationHeader { name: "Header Popover fixture", aria_label_more_menu_icon_button: "Header Popover More", enable_top_layer: true,
                    onbeforeoverflowchange: move |event: HeaderOverflowEvent| { if !event.open && root_veto() { event.prevent_default(); } record(rows, "header-request", "header", json!({"open":event.open,"previous":event.previous,"vetoed":event.default_prevented()})); },
                    onoverflowchange: move |open| { root_open.set(open); record(rows, "header-completed", "header", json!({"open":open})); },
                    actions: rsx! { button { id: "popover-header-default-action", "Header default action" } if with_child { DropdownTrigger { dropdown: child, "Header child trigger" } } },
                    overflow: if permanent_overflow { Some(rsx! { button { id: "popover-header-overflow-action", "Header permanent overflow action" } }) } else { None },
                } },
                p { "Real retained ApplicationHeader nested inside the parent Popover." }
            }
        }
    }
}

const ANCHOR_POSITIONS: [(&str, i32, i32); 7] = [
    ("full", 70, 60),
    ("top", 70, -20),
    ("bottom", 70, 140),
    ("left", -50, 60),
    ("right", 190, 60),
    ("corner", 190, -20),
    ("outside", -120, 60),
];

#[derive(Clone, Copy)]
struct TriggerOthersContext {
    handles: [DropdownHandle; 5],
    veto_open: Signal<[bool; 5]>,
    veto_close: Signal<[bool; 5]>,
    topology: bool,
}

#[component]
fn TriggerOthersProbe(topology: bool) -> Element {
    let ProbeContext { rows, .. } = use_context::<ProbeContext>();
    let handles = [
        use_dropdown(),
        use_dropdown(),
        use_dropdown(),
        use_dropdown(),
        use_dropdown(),
    ];
    let mut veto_open = use_signal(|| [false; 5]);
    let mut veto_close = use_signal(|| [false; 5]);
    use_context_provider(|| TriggerOthersContext {
        handles,
        veto_open,
        veto_close,
        topology,
    });
    let roles = if topology {
        [
            "active",
            "old-parent",
            "live-parent",
            "sibling",
            "descendant",
        ]
    } else {
        ["active", "both", "outside", "inside", "none"]
    };
    let surface_order = if topology {
        [1, 2, 0, 3, 4]
    } else {
        [0, 1, 2, 3, 4]
    };
    let state = json!(
        roles
            .into_iter()
            .enumerate()
            .map(|(i, role)| (role, handles[i].is_open()))
            .collect::<std::collections::BTreeMap<_, _>>()
    )
    .to_string();
    rsx! {
        section { id: "trigger-others-probe", "data-topology": topology.to_string(), "data-state": state, "data-veto-open": json!(*veto_open.read()).to_string(), "data-veto-close": json!(*veto_close.read()).to_string(),
            input { id: "trigger-others-command", r#type: "hidden", oninput: move |event| {
                let Ok(value) = serde_json::from_str::<Value>(&event.value()) else { return; };
                let Some(index) = roles.iter().position(|role| Some(*role) == value["target"].as_str()) else { return; };
                let action = value["action"].as_str().unwrap_or("");
                record(rows, "trigger-others-public-command", roles[index], value.clone());
                match action {
                    "open" => handles[index].open(),
                    "close" => handles[index].close(false),
                    "toggle" => handles[index].toggle(),
                    "veto-open" => veto_open.write()[index] = value["value"].as_bool().unwrap_or(false),
                    "veto-close" => veto_close.write()[index] = value["value"].as_bool().unwrap_or(false),
                    _ => {},
                }
                record(rows, "trigger-others-public-return", roles[index], json!({"action":action,"open":handles[index].is_open()}));
            } }
            div { id: "trigger-others-stage", style: "display:flex;gap:12px;flex-wrap:wrap;margin:12px 0",
                for (index, role) in roles.into_iter().enumerate() {
                    if !topology || index == 1 || index == 2 {
                        span { id: "trigger-others-trigger-slot-{role}", DropdownTrigger { dropdown: handles[index], aria_label: format!("Actual {role} trigger"), "Actual {role} trigger" } }
                    }
                }
            }
            for index in surface_order { TriggerOthersSurface { index } }
            div { style: "display:flex;gap:4px;flex-wrap:wrap;margin:8px 0",
                for (index, role) in roles.into_iter().enumerate() {
                    for action in ["open", "close", "toggle", "veto-open", "veto-close"] {
                        button { id: "trigger-others-control-{role}-{action}", tabindex: "-1", onmousedown: move |event| event.prevent_default(), onclick: move |event| {
                            event.stop_propagation();
                            #[cfg(feature = "web")]
                            { let data = event.data(); if let Some(native) = data.downcast::<web_sys::MouseEvent>() { native.stop_propagation(); } }
                            record(rows, "trigger-others-public-command", role, json!({"action":action,"origin":"visible-Rust-control"}));
                            match action {
                                "open" => handles[index].open(),
                                "close" => handles[index].close(false),
                                "toggle" => handles[index].toggle(),
                                "veto-open" => { let next = !veto_open()[index]; veto_open.write()[index] = next; },
                                _ => { let next = !veto_close()[index]; veto_close.write()[index] = next; },
                            }
                            record(rows, "trigger-others-public-return", role, json!({"action":action,"open":handles[index].is_open()}));
                        }, "{role}: {action}" }
                    }
                }
            }
            TriggerOthersNativeJournal {}
        }
    }
}

#[cfg(feature = "web")]
const TRIGGER_OTHERS_NATIVE_JOURNAL: &str = r#"
const abort=new AbortController(),owner=document.getElementById('trigger-others-probe'),events=new WeakMap(),originals=new WeakMap(),objects=new WeakMap();let active=true,eventSequence=0,objectSequence=0,rowSequence=0;
const alive=()=>active&&!abort.signal.aborted&&owner?.isConnected&&document.getElementById('trigger-others-probe')===owner;
const oid=node=>{if(!node)return null;if(!objects.has(node))objects.set(node,++objectSequence);return objects.get(node);};
const identity=node=>node?{id:node.id||'',tag:node.nodeName,object:oid(node),connected:node.isConnected,label:node.getAttribute?.('aria-label')||null}:null;
const rect=node=>{if(!(node instanceof Element))return null;const r=node.getBoundingClientRect();return{x:r.x,y:r.y,width:r.width,height:r.height};};
const hit=(x,y)=>Number.isFinite(x)&&Number.isFinite(y)?{x,y,first:identity(document.elementFromPoint(x,y)),stack:document.elementsFromPoint(x,y).slice(0,5).map(identity)}:null;
const element=node=>{if(!(node instanceof HTMLElement))return identity(node);const css=getComputedStyle(node);return{...identity(node),rect:rect(node),hidden:node.hidden,inert:node.inert,display:css.display,visibility:css.visibility,tabIndex:node.tabIndex,trigger:node.getAttribute('data-ix-overlay-trigger'),surface:node.getAttribute('data-ix-overlay-host')};};
const facts=()=>{const surfaces={};for(const wrapper of owner.querySelectorAll('[id^="trigger-others-surface-"]')){const role=wrapper.id.slice('trigger-others-surface-'.length),host=wrapper.querySelector('[data-ix-overlay-host]'),root=host?.closest('[data-ix-overlay-root]'),id=root?.getAttribute('data-ix-overlay-root'),trigger=Array.from(document.querySelectorAll('[data-ix-overlay-trigger]')).find(node=>node.getAttribute('data-ix-overlay-trigger')===id),r=trigger?.getBoundingClientRect();surfaces[role]={signalIndex:wrapper.dataset.signalIndex,policy:wrapper.dataset.closeBehavior,root:root?{...identity(root),id:root.id,registration:id,owner:root.dataset.ixOverlayOwner,generation:root.dataset.ixOverlayGeneration,registryOpen:root.dataset.ixOverlayOpen}:null,host:element(host),nativeOpen:host?.hasAttribute('popover')?host.matches(':popover-open'):null,trigger:element(trigger),triggerCenterHit:r?hit(r.x+r.width/2,r.y+r.height/2):null,action:element(document.getElementById('trigger-others-action-'+role))};}let callbacks=[];try{callbacks=JSON.parse(document.getElementById('popover-controller-log')?.textContent||'[]');}catch{}return{owner:identity(owner),topology:owner.dataset.topology,state:JSON.parse(owner.dataset.state||'{}'),vetoOpen:JSON.parse(owner.dataset.vetoOpen||'[]'),vetoClose:JSON.parse(owner.dataset.vetoClose||'[]'),surfaces,active:element(document.activeElement),callbackCount:callbacks.length,lastCallback:callbacks.at(-1)||null,viewport:{width:innerWidth,height:innerHeight,scrollX,scrollY}};};
const send=(phase,event,extra={})=>{if(!alive())return;if(!events.has(event)){events.set(event,++eventSequence);originals.set(event,{target:event.target,path:event.composedPath().slice(0,10)});}const original=originals.get(event),pointer='clientX'in event;dioxus.send({row:++rowSequence,event:events.get(event),phase,time:performance.now(),eventTime:event.timeStamp,type:event.type,trusted:event.isTrusted,key:event.key||null,code:event.code||null,repeat:event.repeat||false,shift:!!event.shiftKey,alt:!!event.altKey,ctrl:!!event.ctrlKey,meta:!!event.metaKey,button:event.button??null,buttons:event.buttons??null,detail:event.detail??null,pointerId:event.pointerId??null,pointerType:event.pointerType??null,defaultPrevented:event.defaultPrevented,cancelBubble:event.cancelBubble,target:element(original.target),relatedTarget:identity(event.relatedTarget),path:original.path.map(identity),pointer:pointer?{clientX:event.clientX,clientY:event.clientY,pageX:event.pageX,pageY:event.pageY,hit:hit(event.clientX,event.clientY),hitTiming:'current observed phase at original event coordinates'}:null,facts:facts(),...extra});};
for(const type of ['keydown','keyup','pointerdown','pointerup','mousedown','mouseup','click','focusin','focusout']){
 document.addEventListener(type,event=>{if(!alive())return;send('document-capture',event);const target=event.composedPath()[0];if(target instanceof EventTarget)target.addEventListener(type,actual=>{if(actual===event)send('actual-target-bubble',event);},{once:true,passive:true,signal:abort.signal});queueMicrotask(()=>send('queued-capture-microtask',event));setTimeout(()=>send('next-native-task',event),0);requestAnimationFrame(time=>{send('next-native-frame',event,{frameTime:time});requestAnimationFrame(next=>send('second-native-frame',event,{frameTime:next}));});},{capture:true,passive:true,signal:abort.signal});
 document.addEventListener(type,event=>send('document-bubble',event),{passive:true,signal:abort.signal});
 window.addEventListener(type,event=>send('window-bubble',event),{passive:true,signal:abort.signal});
}
dioxus.send({phase:'ready',readOnly:true,time:performance.now(),owner:identity(owner),ringCap:160});try{await dioxus.recv();}finally{active=false;abort.abort();}
"#;

#[component]
fn TriggerOthersNativeJournal() -> Element {
    #[cfg(not(feature = "web"))]
    {
        rsx! {}
    }
    #[cfg(feature = "web")]
    {
        let mut rows = use_signal(Vec::<Value>::new);
        let channel = use_hook(|| Rc::new(RefCell::new(None::<dioxus::document::Eval>)));
        let cleanup = channel.clone();
        use_drop(move || {
            if let Some(eval) = cleanup.borrow_mut().take() {
                let _ = eval.send("finish");
            }
        });
        let text = serde_json::to_string(&*rows.read()).unwrap();
        let last_row = rows
            .read()
            .last()
            .and_then(|row| row["row"].as_u64())
            .unwrap_or(0);
        rsx! {
            details { id: "trigger-others-native-journal", "data-read-only": "true", "data-ring-cap": "160", "data-last-row": last_row.to_string(),
                onmounted: move |_| {
                    let mut eval = dioxus::document::eval(TRIGGER_OTHERS_NATIVE_JOURNAL);
                    *channel.borrow_mut() = Some(eval);
                    spawn(async move { while let Ok(value) = eval.recv::<Value>().await { let mut current = rows.write(); current.push(value); if current.len() > 160 { current.remove(0); } } });
                },
                summary { "Read-only native keyboard and pointer journal" }
                pre { id: "trigger-others-native-log", style: "white-space:pre-wrap;overflow-wrap:anywhere", "{text}" }
            }
        }
    }
}

#[component]
fn TriggerOthersSurface(index: usize) -> Element {
    let ProbeContext { rows, .. } = use_context::<ProbeContext>();
    let TriggerOthersContext {
        handles,
        veto_open,
        veto_close,
        topology,
    } = use_context::<TriggerOthersContext>();
    let roles = if topology {
        [
            "active",
            "old-parent",
            "live-parent",
            "sibling",
            "descendant",
        ]
    } else {
        ["active", "both", "outside", "inside", "none"]
    };
    let role = roles[index];
    let behavior = if topology {
        DropdownCloseBehavior::Both
    } else {
        match index {
            2 => DropdownCloseBehavior::Outside,
            3 => DropdownCloseBehavior::Inside,
            4 => DropdownCloseBehavior::None,
            _ => DropdownCloseBehavior::Both,
        }
    };
    rsx! {
        div { id: "trigger-others-surface-{role}", "data-signal-index": index.to_string(), "data-close-behavior": if topology || index < 2 { "both" } else if index == 2 { "outside" } else if index == 3 { "inside" } else { "none" },
            Dropdown { controller: handles[index], label: format!("Actual {role} surface"), close_behavior: behavior, enable_top_layer: true, suppress_trigger_visibility_check: true,
                onbeforechange: move |event: DropdownChangeEvent| {
                    if (event.open && veto_open()[index]) || (!event.open && veto_close()[index]) { event.prevent_default(); }
                    record(rows, "trigger-others-request", role, json!({"open":event.open,"previous":event.previous,"vetoed":event.default_prevented()}));
                },
                onchange: move |open| record(rows, "trigger-others-completed", role, json!({"open":open})),
                button { id: "trigger-others-action-{role}", "Actual {role} native action" }
                if topology && index == 1 {
                    div { id: "trigger-others-old-parent-slot",
                        DropdownTrigger { dropdown: handles[0], aria_label: "Actual active trigger", "Actual active trigger" }
                        DropdownTrigger { dropdown: handles[3], aria_label: "Actual sibling trigger", "Actual sibling trigger" }
                    }
                }
                if topology && index == 2 { div { id: "trigger-others-live-parent-slot", "Live parent relocation slot" } }
                if topology && index == 0 { div { id: "trigger-others-descendant-slot", DropdownTrigger { dropdown: handles[4], aria_label: "Actual descendant trigger", "Actual descendant trigger" } } }
            }
        }
    }
}

#[component]
fn AnchorProbe(mode: FocusMode) -> Element {
    let ProbeContext { rows, .. } = use_context();
    let root = use_dropdown();
    let child = use_dropdown();
    let mut root_point = use_signal(|| (70_i32, 60_i32));
    let mut child_point = use_signal(|| (70_i32, 60_i32));
    let mut root_veto = use_signal(|| false);
    let mut child_veto = use_signal(|| false);
    let mut child_roving = use_signal(|| mode == FocusMode::AnchorFocus);
    let mut child_disable_trap = use_signal(|| mode == FocusMode::AnchorFocusNoTrap);
    let focus_case = matches!(
        mode,
        FocusMode::AnchorFocus | FocusMode::AnchorFocusPointer | FocusMode::AnchorFocusNoTrap
    );
    let nested = mode == FocusMode::AnchorNested || focus_case;
    let suppressed = mode == FocusMode::AnchorSuppressed;
    let placement = match mode {
        FocusMode::AnchorRightStart | FocusMode::AnchorSuppressed => DropdownPlacement::RightStart,
        FocusMode::AnchorBottomEnd => DropdownPlacement::BottomEnd,
        _ => DropdownPlacement::BottomStart,
    };
    rsx! {
        section { id: "popover-anchor-probe", "data-root-open": root.is_open().to_string(), "data-child-open": child.is_open().to_string(), "data-nested": nested.to_string(), "data-suppressed": suppressed.to_string(), "data-placement": placement.as_str(), "data-root-veto": root_veto().to_string(), "data-child-veto": child_veto().to_string(), "data-child-roving": child_roving().to_string(), "data-child-disable-trap": child_disable_trap().to_string(), "data-root-x": root_point().0.to_string(), "data-root-y": root_point().1.to_string(), "data-child-x": child_point().0.to_string(), "data-child-y": child_point().1.to_string(),
            style { {r#"
#popover-anchor-root-clip,#popover-anchor-child-clip{box-sizing:border-box;width:240px;height:160px;overflow:hidden;border:0;padding:0}
#popover-anchor-root-clip{position:fixed;left:320px;top:260px}
#popover-anchor-child-clip{position:relative}
#popover-anchor-root-clip>.ix-dropdown-external-trigger,#popover-anchor-child-clip>.ix-dropdown-external-trigger{position:absolute;left:var(--ix-probe-anchor-x);top:var(--ix-probe-anchor-y);box-sizing:border-box;width:100px;height:40px;min-width:0;min-height:0;margin:0;padding:0}
"#} }
            p { "Actual native anchors in Rust-owned clipping boxes; geometry controls do not choose menu placement." }
            for target in ["root", "child"] {
                if target == "root" || nested {
                    div { style: "display:flex;gap:4px;flex-wrap:wrap;margin:4px 0",
                        for (name, x, y) in ANCHOR_POSITIONS {
                            button { id: "popover-anchor-{target}-position-{name}", tabindex: "-1", onmousedown: move |event| event.prevent_default(), onclick: move |event| {
                                event.stop_propagation();
                                #[cfg(feature = "web")]
                                { let data = event.data(); if let Some(native) = data.downcast::<web_sys::MouseEvent>() { native.stop_propagation(); } }
                                if target == "root" { root_point.set((x, y)); } else { child_point.set((x, y)); }
                            }, "{target}: {name}" }
                        }
                    }
                }
            }
            input { id: "popover-anchor-position", r#type: "hidden", oninput: move |event| {
                let Ok(value) = serde_json::from_str::<Value>(&event.value()) else { return; };
                let Some(x) = value["x"].as_i64().and_then(|value| i32::try_from(value).ok()) else { return; };
                let Some(y) = value["y"].as_i64().and_then(|value| i32::try_from(value).ok()) else { return; };
                if value["target"] == "root" { root_point.set((x, y)); } else if value["target"] == "child" { child_point.set((x, y)); }
            } }
            button { id: "popover-anchor-modality-outside", "Outside partial-anchor modality seed" }
            div { id: "popover-anchor-root-clip", style: format!("--ix-probe-anchor-x:{}px;--ix-probe-anchor-y:{}px", root_point().0, root_point().1),
                DropdownTrigger { dropdown: root, aria_label: "Partial anchor root", "Root anchor" }
            }
            div { id: "popover-anchor-root-surface",
                Dropdown { controller: root, label: "Partial anchor root menu", placement, enable_top_layer: true, suppress_automatic_placement: true, suppress_trigger_visibility_check: suppressed,
                    onbeforechange: move |event: DropdownChangeEvent| { if !event.open && root_veto() { event.prevent_default(); } record(rows, "anchor-root-request", "anchor-root", json!({"open":event.open,"previous":event.previous,"vetoed":event.default_prevented()})); },
                    onchange: move |open| record(rows, "anchor-root-completed", "anchor-root", json!({"open":open})),
                    button { id: "popover-anchor-root-action", style: "box-sizing:border-box;width:140px;height:36px", "Root native action" }
                    if nested {
                        div { id: "popover-anchor-child-clip", style: format!("--ix-probe-anchor-x:{}px;--ix-probe-anchor-y:{}px", child_point().0, child_point().1),
                            DropdownTrigger { dropdown: child, aria_label: "Partial anchor child", "Child anchor" }
                        }
                    }
                }
            }
            if nested {
                div { id: "popover-anchor-child-surface",
                    Dropdown { controller: child, label: "Partial anchor child menu", placement: DropdownPlacement::RightStart, enable_top_layer: true, suppress_automatic_placement: true, navigation_mode: if child_roving() { DropdownNavigationMode::RovingTabindex } else { DropdownNavigationMode::ActiveDescendant }, disable_focus_trap: child_disable_trap(),
                        onbeforechange: move |event: DropdownChangeEvent| { if !event.open && child_veto() { event.prevent_default(); } record(rows, "anchor-child-request", "anchor-child", json!({"open":event.open,"previous":event.previous,"vetoed":event.default_prevented()})); },
                        onchange: move |open| record(rows, "anchor-child-completed", "anchor-child", json!({"open":open})),
                        button { id: "popover-anchor-child-action", style: "box-sizing:border-box;width:140px;height:36px", "Child native action" }
                    }
                }
            }
            if focus_case {
                input { id: "popover-anchor-settings", r#type: "hidden", oninput: move |event| {
                    let Ok(value) = serde_json::from_str::<Value>(&event.value()) else { return; };
                    if let Some(value) = value["rootVeto"].as_bool() { root_veto.set(value); }
                    if let Some(value) = value["childVeto"].as_bool() { child_veto.set(value); }
                    if let Some(value) = value["childRoving"].as_bool() { child_roving.set(value); }
                    if let Some(value) = value["childDisableFocusTrap"].as_bool() { child_disable_trap.set(value); }
                    record(rows, "anchor-fixture-settings", "anchor", value);
                } }
                div { style: "display:flex;gap:4px;flex-wrap:wrap;margin:4px 0",
                    for setting in ["rootVeto", "childVeto", "childRoving", "childDisableFocusTrap"] {
                        button { id: "popover-anchor-setting-{setting}", tabindex: "-1", onmousedown: move |event| event.prevent_default(), onclick: move |event| {
                            event.stop_propagation();
                            #[cfg(feature = "web")]
                            { let data = event.data(); if let Some(native) = data.downcast::<web_sys::MouseEvent>() { native.stop_propagation(); } }
                            match setting {
                                "rootVeto" => { let next = !root_veto(); root_veto.set(next); },
                                "childVeto" => { let next = !child_veto(); child_veto.set(next); },
                                "childRoving" => { let next = !child_roving(); child_roving.set(next); },
                                _ => { let next = !child_disable_trap(); child_disable_trap.set(next); },
                            }
                            record(rows, "anchor-fixture-settings", "anchor", json!({"rootVeto":root_veto(),"childVeto":child_veto(),"childRoving":child_roving(),"childDisableFocusTrap":child_disable_trap()}));
                        }, "Toggle {setting}" }
                    }
                }
            }
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
const abort=new AbortController(),events=new WeakMap(),objects=new WeakMap(),watched=new WeakMap();let sequence=0,objectSequence=0;
const objectId=element=>{if(!element)return null;if(!objects.has(element))objects.set(element,++objectSequence);return objects.get(element);};
const node=element=>element?{id:element.id,tag:element.tagName,label:element.getAttribute?.('aria-label'),host:element.closest?.('[data-ix-overlay-host]')?.getAttribute('data-ix-overlay-host'),objectId:objectId(element),connected:element.isConnected}:null;
const dropdown=event=>{const wrapper=event.target?.closest?.('#popover-tab-dropdown'),host=wrapper?.querySelector('[data-ix-overlay-host]');return{wrapper,host};};
const capture=(phase,event,extra={})=>{if(event.key!=='Tab')return;if(!events.has(event))events.set(event,++sequence);const {wrapper,host}=dropdown(event);dioxus.send({phase,sequence:events.get(event),key:event.key,shift:event.shiftKey,trusted:event.isTrusted,defaultPrevented:event.defaultPrevented,cancelBubble:event.cancelBubble,target:node(event.target),active:node(document.activeElement),dropdown:wrapper?{open:wrapper.dataset.open,host:node(host),nativeOpen:host?.matches(':popover-open')??null,hidden:host?.hidden,inert:host?.inert}:null,...extra});};
document.addEventListener('keydown',event=>{capture('document-capture',event);if(event.key!=='Tab')return;const {host}=dropdown(event);if(!host)return;
 if(!watched.has(host))for(const type of ['beforetoggle','toggle'])host.addEventListener(type,native=>{const latest=watched.get(host);if(latest)capture('dropdown-native-'+type,latest,{oldState:native.oldState,newState:native.newState});},{passive:true,signal:abort.signal});
 watched.set(host,event);
},{capture:true,passive:true,signal:abort.signal});
document.addEventListener('keydown',event=>capture('document-bubble-observer',event),{passive:true,signal:abort.signal});
window.addEventListener('keydown',event=>{capture('window-bubble-observer',event);if(event.key!=='Tab')return;
 queueMicrotask(()=>{if(!abort.signal.aborted)capture('window-after-dispatch-microtask',event);});
 requestAnimationFrame(time=>{if(abort.signal.aborted)return;capture('window-next-native-frame',event,{frameTime:time});requestAnimationFrame(next=>{if(!abort.signal.aborted)capture('window-second-native-frame',event,{frameTime:next});});});
},{passive:true,signal:abort.signal});
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
const observations=[],driverJournal=[],assertions=[],tabCases=[],tabChecks=[],deferredChecks=[],queuedChecks=[],headerCases=[],headerOpeningChecks=[],headerChecks=[],headerModeCases=[],headerModeOpeningChecks=[],headerModeChecks=[],headerModeFocusContinuity=[],headerModeVisibilityLimits=[],anchorCases=[],anchorFocusCases=[],triggerOthersCases=[];
let readFailureHeader=()=>null,readFailureAnchor=()=>null,activeHeaderMode=null,activeAnchorCase=null,activeAnchorFocus=null,activeTriggerOthers=null,readFailureTriggerOthers=()=>null;
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
 const headerNodes=()=>{const owner=get('popover-header-probe'),content=get('popover-header-content'),header=content?.querySelector('.ix-application-header'),root=header?.querySelector('.ix-header-layout'),more=header?.querySelector('[data-header-more-button]'),small=header?.querySelector('[data-header-popup-host="small"]'),large=header?.querySelector('[data-header-popup-host="large"]'),trigger=header?.querySelector('.ix-header-slot-default [data-ix-overlay-trigger]'),child=get('popover-header-child-surface'),host=child?.querySelector('[data-ix-overlay-host]');return{owner,content,header,root,more,small,large,trigger,child,host};};
 const headerFacts=()=>{const n=headerNodes();if(!n.owner)return null;const element=node=>node?{...describe(node),connected:node.isConnected,open:node.dataset.open,overlayOpen:node.dataset.ixOverlayOpen,owner:node.dataset.ixOverlayOwner,overlayRoot:node.dataset.ixOverlayRoot,overlayHost:node.dataset.ixOverlayHost,generation:node.dataset.generation,overlayGeneration:node.dataset.ixOverlayGeneration,active:node.dataset.active,hidden:node.hidden,inert:node.inert,ariaExpanded:node.getAttribute('aria-expanded'),popoverOpen:node.hasAttribute('popover')?node.matches(':popover-open'):null,geometry:geometry(node)}:null;const observer=n.root?.querySelector('input[data-header-layout-observer]'),transport=n.root?.querySelector('input[data-ix-overlay-transport]'),value=input=>{try{return JSON.parse(input?.value??'null');}catch{return input?.value;}};return{initialSmall:n.owner.dataset.initialSmall,small:n.owner.dataset.small,roving:n.owner.dataset.roving,rootVeto:n.owner.dataset.rootVeto,childVeto:n.owner.dataset.childVeto,callbackRootOpen:n.owner.dataset.rootOpen,childOpen:n.owner.dataset.childOpen,withChild:n.owner.dataset.withChild,permanentOverflow:n.owner.dataset.permanentOverflow,headerClass:n.header?.className,bridge:{headerNative:n.root?.dataset.headerLayoutNative,rootData:n.root?Object.fromEntries(Object.entries(n.root.dataset)):null,observer:{node:describe(observer),connected:!!observer?.isConnected,owner:observer?.getAttribute('data-header-layout-observer'),lastActualValue:value(observer)},transport:{node:describe(transport),connected:!!transport?.isConnected,owner:transport?.getAttribute('data-ix-overlay-transport'),native:transport?.getAttribute('data-ix-overlay-native'),lastActualValue:value(transport)},documentRegistryOwner:document.documentElement.getAttribute('data-ix-overlay-registry-native')},root:element(n.root),more:element(n.more),smallHost:element(n.small),largeHost:element(n.large),childTrigger:element(n.trigger),childHost:element(n.host),childRoot:element(n.host?.closest('[data-ix-overlay-root]')),defaultAction:element(get('popover-header-default-action')),overflowAction:element(get('popover-header-overflow-action')),parentBefore:element(get('popover-parent-before-header')),parentAfter:element(get('popover-parent-after-header')),portContainedMenuToggle:element(n.header?.querySelector('.ix-header-menu-toggle')),physical:{parentContainsHeader:native('parent')?.contains(n.header),parentContainsChildHost:native('parent')?.contains(n.host),headerContainsChildTrigger:n.header?.contains(n.trigger),childTriggerAncestors:(()=>{const rows=[];let node=n.trigger;while(node&&rows.length<12){rows.push({node:describe(node),overlayHost:node.getAttribute('data-ix-overlay-host'),headerPopup:node.getAttribute('data-header-popup-host')});node=node.parentElement;}return rows;})()}};};
 readFailureHeader=headerFacts;
 const headerCandidateInventory=()=>{const n=headerNodes(),panel=native('parent'),selector='button,a[href],area[href],audio,input,select,textarea,iframe,object,embed,video,[tabindex],[contenteditable]';return Array.from(panel?.querySelectorAll(selector)??[]).map(element=>{const style=getComputedStyle(element),host=element.closest('[data-header-popup-host]');return{...describe(element),connected:element.isConnected,tabindexAttribute:element.getAttribute('tabindex'),tabIndex:element.tabIndex,disabled:element.matches(':disabled,[disabled]'),hidden:element.hidden,inertAncestor:describe(element.closest('[inert]')),clientRectCount:element.getClientRects().length,display:style.display,visibility:style.visibility,headerMore:element===n.more,headerPopup:host?{...describe(host),active:host.dataset.active,open:host.dataset.open}:null,overlayHost:element.closest('[data-ix-overlay-host]')?.getAttribute('data-ix-overlay-host')??null};});};
 const snapshot=()=>({topology:fixture()?.dataset.topology,focusMode:fixture()?.dataset.focusMode,dynamicBody:fixture()?.dataset.dynamicBody,state:read(),veto:Object.fromEntries(names.map(name=>[name,get('popover-'+name+'-veto')?.checked])),vetoOpen:Object.fromEntries(names.map(name=>[name,get('popover-'+name+'-veto-open')?.checked])),native:Object.fromEntries(names.map(name=>{const host=native(name);let open=false;try{open=!!host?.matches(':popover-open');}catch{}return[name,{host:describe(host),connected:!!host?.isConnected,open,generation:host?.dataset.surfaceGeneration,overlayOpen:host?.dataset.ixOverlayOpen,geometry:geometry(host)}];})),childLocal:local(),active:describe(document.activeElement),log:logs()});
 const assert=(value,label)=>{if(!value)throw Error(label+' '+JSON.stringify({scenario,snapshot:snapshot()}));assertions.push({scenario,label});};
 const expect=(expected)=>{const current=snapshot();for(const name of names){const open=expected.includes(name);assert(current.state[name]===open,name+' Rust visibility '+open);assert(current.native[name].connected&&current.native[name].open===open,name+' connected native visibility '+open);}};
 const expectFocus=name=>assert(document.activeElement?.closest('[data-ix-overlay-host]')===native(name),'native focus belongs to opening '+name+' surface');
 const expectTrigger=name=>assert(document.activeElement?.getAttribute('popovertarget')===native(name)?.id,'native focus returns configured '+name+' trigger');
 const requests=(name,open)=>logs().filter(row=>row.kind==='request'&&row.target===name&&(open===undefined||row.details.open===open));
 const completions=(name,open)=>logs().filter(row=>row.kind==='completed'&&row.target===name&&(open===undefined||row.details.open===open));
 const wait=async(fn,label)=>{let end=performance.now()+5000;while(performance.now()<end){if(fn())return;await sleep(10);}throw Error('Timeout '+label+' '+JSON.stringify({snapshot:snapshot(),header:headerFacts()}));};
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
 const tab=async(label,target,shift=false,afterDispatch=null)=>{
  if(!target)throw Error('Missing native Tab target '+label);target.focus();await settle();assert(document.activeElement===target,'actual native seeded Tab focus '+label);
  const before={state:read(),active:describe(document.activeElement),dropdownOpen:get('popover-tab-dropdown')?.dataset.open??null},nativeStart=nativeTabRows().length,rustStart=logs().length,event=new KeyboardEvent('keydown',{key:'Tab',code:'Tab',shiftKey:shift,bubbles:true,cancelable:true,composed:true});
  let reachedDocument=false,reachedWindow=false;const listeners=new AbortController(),dropdown=target.closest('#popover-tab-dropdown'),dropdownHost=dropdown?.querySelector('[data-ix-overlay-host]'),headerScope=get('popover-header-probe'),hasTimeline=!!dropdown||!!headerScope,deferredTimeline=[],secondaryEvents=[];
  const phase=(name,extra={})=>{const host=dropdownHost,row={phase:name,time:performance.now(),active:describe(document.activeElement),activeObject:objectId(document.activeElement),originalDefaultPrevented:event.defaultPrevented,targetConnected:target.isConnected,dropdown:dropdown?{connected:dropdown.isConnected,open:dropdown.dataset.open,host:describe(host),hostConnected:!!host?.isConnected,nativeOpen:host?.matches(':popover-open')??null,hidden:host?.hidden,inert:host?.inert,geometry:geometry(host)}:null,header:headerScope?headerFacts():null,rustCallbacks:logs().slice(rustStart),...extra};if(hasTimeline)deferredTimeline.push(row);return row;};
  document.addEventListener('keydown',actual=>{if(actual===event){reachedDocument=true;phase('late-document-bubble');}},{passive:true,signal:listeners.signal});
  window.addEventListener('keydown',actual=>{if(actual===event){reachedWindow=true;phase('window-bubble');}},{passive:true,signal:listeners.signal});
  if(hasTimeline){
   const nodes=headerScope?headerNodes():null,hosts=dropdown?[dropdownHost]:[nodes?.small,nodes?.large,nodes?.host];
   for(const host of hosts.filter(Boolean))for(const type of ['beforetoggle','toggle'])host.addEventListener(type,native=>phase('native-'+type,{oldState:native.oldState,newState:native.newState,nativeToggleTarget:describe(native.target)}),{passive:true,signal:listeners.signal});
   for(const type of ['focusin','focusout'])document.addEventListener(type,native=>phase('native-'+type,{focusEventTarget:describe(native.target),relatedTarget:describe(native.relatedTarget)}),{capture:true,passive:true,signal:listeners.signal});
   if(headerScope)document.addEventListener('input',actual=>{const input=actual.target;if(!(input instanceof HTMLInputElement)||!input.matches('[data-ix-overlay-transport]'))return;let report;try{report=JSON.parse(input.value);}catch{return;}if(report?.kind!=='anchor-visibility')return;const fact=report.anchor_visibility;phase('native-anchor-visibility-input',{transport:describe(input),report:{owner:report.owner,sequence:report.sequence,kind:report.kind,anchor_visibility:fact,nodes:(report.nodes??[]).filter(node=>node.id===fact?.root)}});},{capture:true,passive:true,signal:listeners.signal});
   requestAnimationFrame(time=>{if(!listeners.signal.aborted)phase('native-frame-before-release',{frameTime:time});});
  }
  const dispatchSecondary=backwards=>{
   const second=new KeyboardEvent('keydown',{key:'Tab',code:'Tab',shiftKey:backwards,bubbles:true,cancelable:true,composed:true}),value={shift:backwards,target:describe(target),beforeActive:describe(document.activeElement),lateDocument:null,window:null};
   document.addEventListener('keydown',actual=>{if(actual===second)value.lateDocument={active:describe(document.activeElement),defaultPrevented:second.defaultPrevented};},{passive:true,signal:listeners.signal});
   window.addEventListener('keydown',actual=>{if(actual===second)value.window={active:describe(document.activeElement),defaultPrevented:second.defaultPrevented};},{passive:true,signal:listeners.signal});
   driverJournal.push({scenario,kind:'synthetic-native-Tab',label:label+' same-turn second key',shift:backwards,trusted:second.isTrusted,target:describe(target),beforeActive:value.beforeActive,nativeDefaultNavigation:false,sameTurn:true});
   target.dispatchEvent(second);value.defaultPrevented=second.defaultPrevented;value.afterActive=describe(document.activeElement);secondaryEvents.push(value);phase('secondary-post-dispatch',{secondary:value});return value;
  };
  driverJournal.push({scenario,kind:'synthetic-native-Tab',label,shift,trusted:event.isTrusted,target:describe(target),beforeActive:describe(document.activeElement),nativeDefaultNavigation:false});
  target.dispatchEvent(event);const immediateDefaultPrevented=event.defaultPrevented,immediateActive=describe(document.activeElement);phase('post-dispatch');
  const actionFacts=afterDispatch?afterDispatch({phase,dispatchSecondary}):null;if(afterDispatch)phase('after-same-turn-action',{actionFacts});
  if(hasTimeline){const frame=name=>new Promise(resolve=>requestAnimationFrame(time=>resolve(phase(name,{frameTime:time}))));const firstFrame=frame('first-native-frame');await Promise.resolve();phase('post-dispatch-microtask');await firstFrame;await frame('second-native-frame');}
  await settle();phase('settled');
  const row={label,shift,trusted:event.isTrusted,target:describe(target),immediateDefaultPrevented,settledDefaultPrevented:event.defaultPrevented,reachedDocument,reachedWindow,immediateActive,afterActive:describe(document.activeElement),afterActiveObject:objectId(document.activeElement),before,nativePhases:nativeTabRows().slice(nativeStart),rustCallbacks:logs().slice(rustStart),afterState:read(),dropdownOpen:get('popover-tab-dropdown')?.dataset.open??null,deferredTimeline,secondaryEvents,actionFacts};
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
  const timing=result.deferredTimeline,oldTarget=result.target.objectId,synchronous=['late-document-bubble','window-bubble','post-dispatch'].map(name=>timing.find(row=>row.phase===name)),microtask=timing.find(row=>row.phase==='post-dispatch-microtask'),settled=timing.find(row=>row.phase==='settled');
  const requests=result.rustCallbacks.filter(row=>row.kind==='tab-dropdown-request'&&row.details.open===false),completed=result.rustCallbacks.filter(row=>row.kind==='tab-dropdown-completed'&&row.details.open===false);
  const deferred=synchronous.every(row=>row?.activeObject===oldTarget&&row.originalDefaultPrevented===true)&&!!microtask&&timing.some(row=>row.phase==='first-native-frame')&&timing.some(row=>row.phase==='second-native-frame')&&settled?.activeObject===objectId(target)&&settled?.dropdown?.hostConnected===true&&settled.dropdown.nativeOpen===retained&&requests.length===1&&requests[0].details.vetoed===retained&&completed.length===(retained?0:1);
  deferredChecks.push({label:result.label,matched:deferred,expected:{synchronousActive:result.target,settledActive:describe(target),settledNativeOpen:retained,falseRequests:1,falseCompletions:retained?0:1},actual:{synchronous,microtask,settled,falseRequests:requests,falseCompletions:completed}});
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
 const queuedSetup=async(label,veto=false)=>{
  await freshFocus('tab-dropdown-active',label);await command('show','parent');await openTabDropdown();
  if(veto){input('popover-tab-dropdown-veto-input','true');await wait(()=>get('popover-tab-dropdown')?.dataset.vetoClose==='true','actual Rust queued-exit veto');}
  return{trigger:get('popover-tab-dropdown').querySelector('[data-ix-overlay-trigger]'),before:get('popover-tab-first'),after:get('popover-tab-second'),parentTrigger:document.querySelector('[popovertarget="'+native('parent').id+'"]')};
 };
 const checkQueued=(result,target,parentOpen,dropdownOpen,extra)=>{
  const synchronous=['late-document-bubble','window-bubble','post-dispatch','after-same-turn-action','post-dispatch-microtask','native-frame-before-release'].map(name=>result.deferredTimeline.find(row=>row.phase===name)),settled=result.deferredTimeline.find(row=>row.phase==='settled');
  const originRetained=synchronous.every(row=>row?.activeObject===result.target.objectId&&row.originalDefaultPrevented===true),endpoint=result.afterActiveObject===objectId(target)&&result.afterState.parent===parentOpen&&result.dropdownOpen===String(dropdownOpen)&&native('parent').matches(':popover-open')===parentOpen&&settled?.dropdown?.nativeOpen===dropdownOpen;
  queuedChecks.push({label:result.label,matched:result.immediateDefaultPrevented&&result.reachedDocument&&result.reachedWindow&&originRetained&&endpoint&&extra.matched,expected:{originBeforeFrame:result.target,settledActive:describe(target),parentOpen,dropdownOpen,...extra.expected},actual:{synchronous,settledActive:result.afterActive,parentOpen:result.afterState.parent,dropdownOpen:result.dropdownOpen,settledNativeOpen:settled?.dropdown?.nativeOpen,...extra.actual}});
 };
 let queued=await queuedSetup('source53 accepted first Tab survives same-turn unhandled reverse Tab');
 row=await tab('accepted forward plus same-turn reverse queues only first exit',queued.trigger,false,({dispatchSecondary})=>({action:'second-reverse-Tab',second:dispatchSecondary(true)}));
 let second=row.secondaryEvents[0];checkQueued(row,queued.after,true,false,{matched:second?.defaultPrevented===false&&second.afterActive?.objectId===objectId(queued.trigger)&&second.lateDocument?.active?.objectId===objectId(queued.trigger)&&second.lateDocument.defaultPrevented===false&&second.window?.active?.objectId===objectId(queued.trigger)&&second.window.defaultPrevented===false,expected:{secondDefaultPrevented:false},actual:{second}});
 queued=await queuedSetup('source54 two vetoed same-turn Tab exits retain both captured jobs',true);
 row=await tab('vetoed forward plus same-turn reverse retains native focus order',queued.trigger,false,({dispatchSecondary})=>({action:'second-reverse-Tab',second:dispatchSecondary(true)}));
 second=row.secondaryEvents[0];
 const firstBoundary=row.deferredTimeline.findIndex(value=>value.phase==='native-frame-before-release'),lastBoundary=row.deferredTimeline.findIndex(value=>value.phase==='first-native-frame'),frameFocus=row.deferredTimeline.slice(firstBoundary+1,lastBoundary).filter(value=>value.phase==='native-focusin').map(value=>value.focusEventTarget),focusOrder=frameFocus.map(value=>value.objectId),expectedOrder=[objectId(queued.after),objectId(queued.before)],beforeFrame=row.deferredTimeline[firstBoundary],afterFrame=row.deferredTimeline[lastBoundary];
 checkQueued(row,queued.before,true,true,{matched:second?.defaultPrevented===true&&second.afterActive?.objectId===objectId(queued.trigger)&&second.lateDocument?.active?.objectId===objectId(queued.trigger)&&second.lateDocument.defaultPrevented===true&&second.window?.active?.objectId===objectId(queued.trigger)&&second.window.defaultPrevented===true&&JSON.stringify(focusOrder)===JSON.stringify(expectedOrder)&&firstBoundary>=0&&lastBoundary>firstBoundary&&beforeFrame?.frameTime===afterFrame?.frameTime,expected:{secondDefaultPrevented:true,nativeFocusOrder:[describe(queued.after),describe(queued.before)],bothJobsInOneObservedFrame:true},actual:{second,nativeFocusOrder:frameFocus,beforeFrameTime:beforeFrame?.frameTime,afterFrameTime:afterFrame?.frameTime}});
 queued=await queuedSetup('source55 actual trigger reopen does not revoke captured exit focus');
 row=await tab('captured forward exit survives same-turn actual trigger click reopen',queued.trigger,false,()=>{driverJournal.push({scenario,kind:'synthetic-actual-Dropdown-trigger-click',target:describe(queued.trigger),sameTurn:true});queued.trigger.click();return{action:'reopen-actual-trigger-click',trigger:describe(queued.trigger)};});
 checkQueued(row,queued.after,true,true,{matched:true,expected:{actualTriggerClickReopened:true},actual:{action:row.actionFacts}});
 queued=await queuedSetup('source56 same-ID replacement cannot inherit captured actual target focus');
 let replacement=null;const originalAfter=queued.after;
 row=await tab('captured actual target replacement with same ID does not retarget focus',queued.trigger,false,()=>{replacement=document.createElement('button');replacement.type='button';replacement.id=originalAfter.id;replacement.tabIndex=originalAfter.tabIndex;replacement.textContent='Diagnostic same-ID native replacement';originalAfter.replaceWith(replacement);const value={action:'diagnostic-native-candidate-replacement',captured:describe(originalAfter),capturedConnected:originalAfter.isConnected,replacement:describe(replacement),replacementConnected:replacement.isConnected};driverJournal.push({scenario,kind:value.action,...value,sameTurn:true});return value;});
 const replacementFocus=row.deferredTimeline.filter(value=>value.phase==='native-focusin'&&value.focusEventTarget?.objectId===objectId(replacement));
 checkQueued(row,queued.trigger,true,false,{matched:originalAfter.isConnected===false&&replacement?.isConnected===true&&replacement!==originalAfter&&get(originalAfter.id)===replacement&&replacementFocus.length===0,expected:{capturedConnected:false,replacementConnected:true,replacementNeverFocused:true},actual:{captured:describe(originalAfter),capturedConnected:originalAfter.isConnected,replacement:describe(replacement),replacementConnected:replacement?.isConnected,replacementFocus}});
 replacement.replaceWith(originalAfter);driverJournal.push({scenario,kind:'diagnostic-native-replacement-cleanup-after-recorded-verdict',restored:describe(originalAfter)});
 queued=await queuedSetup('source57 parent public hide wins settled focus after captured child exit');
 row=await tab('captured Dropdown exit followed by same-turn parent public hide',queued.trigger,false,()=>{const detail={action:'hide',target:'parent'};driverJournal.push({scenario,kind:'public-Rust-command-via-synthetic-input',detail,sameTurn:true});input('popover-command',JSON.stringify(detail));return{action:'parent-public-hide',detail};});
 const closingAfterFocus=row.deferredTimeline.filter(value=>value.phase==='native-focusin'&&value.focusEventTarget?.objectId===objectId(queued.after));
 checkQueued(row,queued.parentTrigger,false,false,{matched:closingAfterFocus.length===0,expected:{settledConfiguredParentTrigger:true,closingAfterNeverFocused:true},actual:{action:row.actionFacts,closingAfterFocus}});
 const headerClick=(target,label)=>{
  if(!target)throw Error('Missing actual Header click target '+label);let actual=null;target.addEventListener('click',event=>actual=event,{capture:true,once:true,passive:true});target.focus();target.click();
  driverJournal.push({scenario,kind:'synthetic-actual-native-Header-click',label,target:describe(target),trusted:actual?.isTrusted,defaultPrevented:actual?.defaultPrevented,activeAfter:describe(document.activeElement)});
 };
 const openHeader=async(label,openingChecks=headerOpeningChecks)=>{
  const n=headerNodes(),start=logs().length,clicked=n.root?.dataset.open!=='true';if(clicked)headerClick(n.more,label);
  await wait(()=>{const n=headerNodes(),host=n.owner?.dataset.small==='true'?n.small:n.large;return n.root?.dataset.open==='true'&&host?.matches(':popover-open');},'actual Header native popup opens '+label);await settle();
  if(clicked){const callbacks=logs().slice(start),requests=callbacks.filter(row=>row.kind==='header-request'&&row.details.open===true),completions=callbacks.filter(row=>row.kind==='header-completed'&&row.details.open===true),matched=requests.length===1&&requests[0].details.vetoed===false&&completions.length===1;openingChecks.push({scenario,label,matched,expected:{acceptedTrueRequests:1,trueCompletions:1},actual:{requests,completions,root:headerFacts()?.root}});assert(matched,'one accepted Header true request and completion per actual More opening click '+label);}
 };
 const headerSetup=async(label,{isolated=false,history=false,roving=false,rootVeto=false,childVeto=false}={})=>{
  await freshFocus(history?'header-history-child':isolated?'header-small-isolated':'header-small-child',label);await command('show','parent');
  await wait(()=>{const n=headerNodes();return n.root?.dataset.ixOverlayReady==='true'&&n.more&&!n.more.hidden&&n.header?.classList.contains(history?'ix-app-large':'ix-app-small');},'actual forced Header breakpoint and permanent overflow presence');await settle();
  const stages=[{phase:'fresh-header-ready',facts:headerFacts()}];
  if(roving){input('popover-header-roving','true');await wait(()=>headerNodes().owner?.dataset.roving==='true','actual Rust Header child roving mode');await settle();}
  await openHeader(history?'open actual More while Large':'open actual More while Small');stages.push({phase:'actual-More-open-before-history',facts:headerFacts()});
  if(history){
   const held=headerNodes(),start=logs().length,rootOpenMutations=[],watch=new MutationObserver(records=>{for(const record of records)rootOpenMutations.push({attribute:record.attributeName,previous:record.oldValue,current:held.root.getAttribute(record.attributeName)});});watch.observe(held.root,{attributes:true,attributeOldValue:true,attributeFilter:['data-open','data-ix-overlay-open']});driverJournal.push({scenario,kind:'public-fixture-Rust-breakpoint-change',from:'large-open',to:'small',remount:false,explicitRediscovery:false});input('popover-header-small','true');
   await wait(()=>headerNodes().owner?.dataset.small==='true'&&headerNodes().header?.classList.contains('ix-app-small'),'same Application changes Large to Small');await settle();
   watch.disconnect();const current=headerNodes(),callbacks=logs().slice(start),sameActualObjects={header:held.header===current.header,root:held.root===current.root,more:held.more===current.more,smallHost:held.small===current.small,largeHost:held.large===current.large,childTrigger:held.trigger===current.trigger,childHost:held.host===current.host},stayedOpen=current.root?.dataset.open==='true'&&current.root?.dataset.ixOverlayOpen==='true'&&current.small?.matches(':popover-open')&&!rootOpenMutations.some(row=>row.previous==='false'||row.current==='false')&&!callbacks.some(row=>(row.kind==='header-request'||row.kind==='header-completed')&&row.details.open===false);
   stages.push({phase:'retained-Large-open-to-Small-without-reopen',facts:headerFacts(),callbacks,sameActualObjects,rootOpenMutations,stayedOpen,noAdditionalMoreInput:true});assert(stayedOpen&&Object.values(sameActualObjects).every(Boolean),'source-paired Large-open Header retains root presence/cache and actual objects across Small without reopen');
  }
  if(rootVeto){input('popover-header-root-veto','true');await wait(()=>headerNodes().owner?.dataset.rootVeto==='true','actual Rust Header root close veto');}
  if(childVeto){input('popover-header-child-veto','true');await wait(()=>headerNodes().owner?.dataset.childVeto==='true','actual Rust Header child close veto');}
  return{isolated,history,roving,rootVeto,childVeto,stages,limits:['Port contained Small native menu control and source custom-control presence/candidate eligibility are measured, not assumed identical','Port native More focus is compared with source deep native More role; raw native candidate inclusion is not itself a navigation policy','Retained Large-open to Small must preserve logical root/native presence and actual objects without reopen; physical dual-host projection timing is diagnostic']};
 };
 const headerContracts=[
  {sourceCase:60,prevented:false,document:true,window:true,root:false,child:null,focus:'more',requests:['header:accepted'],completed:{header:1,child:0}},
  {sourceCase:61,prevented:false,document:true,window:true,root:false,child:null,focus:'more',requests:['header:accepted'],completed:{header:1,child:0}},
  {sourceCase:62,prevented:false,document:true,window:true,root:true,child:null,focus:'default',requests:[],completed:{header:0,child:0}},
  {sourceCase:63,prevented:false,document:true,window:true,root:true,child:null,focus:'overflow',requests:[],completed:{header:0,child:0}},
  {sourceCase:64,prevented:true,document:false,window:false,root:true,child:null,focus:'parent-before',requests:[],completed:{header:0,child:0}},
  {sourceCase:65,prevented:false,document:true,window:true,root:false,child:false,focus:'body',requests:['header:accepted','child:accepted'],completed:{header:1,child:1}},
  {sourceCase:66,prevented:true,document:true,window:true,root:false,child:false,focus:'more',requests:['header:accepted','child:accepted'],completed:{header:1,child:1}},
  {sourceCase:67,prevented:false,document:true,window:true,root:false,child:false,focus:'body',requests:['header:accepted','child:accepted'],completed:{header:1,child:1}},
  {sourceCase:68,prevented:true,document:true,window:true,root:false,child:false,focus:'more',requests:['header:accepted','child:accepted'],completed:{header:1,child:1}},
  {sourceCase:69,prevented:false,document:true,window:true,root:true,child:true,focus:'child-trigger',requests:['header:vetoed'],completed:{header:0,child:0}},
  {sourceCase:70,prevented:false,document:true,window:true,root:false,child:true,focus:'body',requests:['header:accepted','child:vetoed','child:vetoed'],completed:{header:1,child:0}},
  {sourceCase:71,prevented:false,document:true,window:true,root:true,child:false,focus:'child-trigger',requests:['child:accepted'],completed:{header:0,child:1}},
  {sourceCase:72,prevented:true,document:true,window:true,root:true,child:false,focus:'child-trigger',requests:['child:accepted'],completed:{header:0,child:1}}
 ];
 const checkHeader=(entry,expected)=>{
  const h=entry.after.header,t=entry.tab,n=headerNodes(),target=expected.focus==='more'?n.more:expected.focus==='default'?get('popover-header-default-action'):expected.focus==='overflow'?get('popover-header-overflow-action'):expected.focus==='parent-before'?get('popover-parent-before-header'):expected.focus==='child-trigger'?n.trigger:document.body;
  const requests=t.rustCallbacks.filter(row=>(row.kind==='header-request'||row.kind==='header-child-request')&&row.details.open===false),requestTokens=requests.map(row=>(row.kind==='header-request'?'header':'child')+':'+(row.details.vetoed?'vetoed':'accepted')),completed={header:t.rustCallbacks.filter(row=>row.kind==='header-completed'&&row.details.open===false).length,child:t.rustCallbacks.filter(row=>row.kind==='header-child-completed'&&row.details.open===false).length};
  const activeHost=h.small==='true'?h.smallHost:h.largeHost,bodyFocusIn=t.deferredTimeline.filter(row=>row.phase==='native-focusin'&&row.focusEventTarget?.objectId===objectId(document.body)),nativeBodyFallback=expected.focus!=='body'||(t.afterActiveObject===objectId(document.body)&&bodyFocusIn.length===0);
  const childState=expected.child===null?h.childHost===null&&h.childRoot===null:h.childOpen===String(expected.child)&&h.childRoot?.overlayOpen===String(expected.child)&&h.childRoot?.connected===true&&h.childHost?.connected===true&&h.childHost?.popoverOpen===expected.child;
  const visibilityTiming=expected.sourceCase===70?(()=>{
   const childFalse=rows=>rows.filter(row=>row.kind==='header-child-request'&&row.details.open===false),sync=t.deferredTimeline.filter(row=>['late-document-bubble','window-bubble','post-dispatch'].includes(row.phase)),rootHostIds=[h.smallHost?.objectId,h.largeHost?.objectId],nativeClose=t.deferredTimeline.filter(row=>['native-beforetoggle','native-toggle'].includes(row.phase)&&row.newState==='closed'&&rootHostIds.includes(row.nativeToggleTarget?.objectId)),falseReports=t.deferredTimeline.filter(row=>row.phase==='native-anchor-visibility-input'&&row.report?.anchor_visibility?.root===h.childRoot?.overlayRoot&&row.report.anchor_visibility.target===h.childTrigger?.id&&row.report.anchor_visibility.generation===Number(h.childRoot?.overlayGeneration??h.childRoot?.generation)&&row.report.anchor_visibility.intersecting===false),causalReports=falseReports.filter(row=>row.header?.root.open==='false'&&row.header?.smallHost.popoverOpen===false&&nativeClose.some(close=>close.time<=row.time)&&childFalse(row.rustCallbacks).length<=1&&t.deferredTimeline.some(later=>later.time>=row.time&&childFalse(later.rustCallbacks).length===2)),childIndices=t.rustCallbacks.map((row,index)=>row.kind==='header-child-request'&&row.details.open===false?index:-1).filter(index=>index>=0),rootCompleted=t.rustCallbacks.findIndex(row=>row.kind==='header-completed'&&row.details.open===false),publishedSyncCounts=sync.map(row=>({phase:row.phase,count:childFalse(row.rustCallbacks).length}));
   return{matched:publishedSyncCounts.length>0&&publishedSyncCounts.every(row=>row.count<=1)&&childIndices.length===2&&childIndices[0]<rootCompleted&&rootCompleted<childIndices[1]&&causalReports.length>0,publishedSyncCounts,nativeClose,falseReports,causalReports,childRequestIndices:childIndices,rootCompletedIndex:rootCompleted,limit:'Desktop callback DOM publication can lag Rust handlers. At-most-one synchronously published request plus false native anchor report after native close, callback ordering and later second publication complement the separate exact-one Rust key-handler check; these observations do not alone timestamp Rust callback execution.'};
  })():null;
  const matched=(visibilityTiming===null||visibilityTiming.matched)&&t.immediateDefaultPrevented===expected.prevented&&t.reachedDocument===expected.document&&t.reachedWindow===expected.window&&entry.after.popoverState.parent===true&&native('parent').matches(':popover-open')&&h.root.open===String(expected.root)&&h.root.overlayOpen===String(expected.root)&&h.callbackRootOpen===String(expected.root)&&activeHost?.popoverOpen===expected.root&&childState&&t.afterActiveObject===objectId(target)&&nativeBodyFallback&&JSON.stringify(requestTokens)===JSON.stringify(expected.requests)&&requests.every(row=>row.details.previous===true)&&completed.header===expected.completed.header&&completed.child===expected.completed.child;
  headerChecks.push({label:entry.label,sourceCase:expected.sourceCase,matched,expected:{...expected,actualTarget:describe(target),focusMechanism:expected.focus==='body'?'native-body-focus-loss':'exact-actual-semantic-target'},actual:{defaultPrevented:t.immediateDefaultPrevented,reachedDocument:t.reachedDocument,reachedWindow:t.reachedWindow,parentOpen:entry.after.popoverState.parent,headerLogical:h.root.open,headerOverlay:h.root.overlayOpen,headerCallback:h.callbackRootOpen,headerNative:activeHost?.popoverOpen,childLogical:h.childOpen,childOverlay:h.childRoot?.overlayOpen??null,childNative:h.childHost?.popoverOpen??null,focus:t.afterActive,requestTokens,requests,completed,bodyFocusIn,bodyTabindexAttribute:document.body.getAttribute('tabindex'),visibilityTiming}});
 };
 const headerRaw=async(label,options,origin,backwards=false)=>{
  const setup=await headerSetup(label,options);
  if(origin==='child'){
   headerClick(headerNodes().trigger,'actual external Header child trigger');await wait(()=>headerNodes().owner?.dataset.childOpen==='true'&&headerNodes().host?.matches(':popover-open'),'actual external Header child opens');await settle();
   if(options.roving){const trigger=headerNodes().trigger;trigger.focus();const event=new KeyboardEvent('keydown',{key:'ArrowDown',code:'ArrowDown',bubbles:true,cancelable:true,composed:true});trigger.dispatchEvent(event);driverJournal.push({scenario,kind:'synthetic-actual-native-Header-child-ArrowDown',target:describe(trigger),trusted:event.isTrusted,defaultPrevented:event.defaultPrevented});await settle();setup.stages.push({phase:'actual-ArrowDown-roving-preparation-before-explicit-Tab-seed',facts:headerFacts(),active:describe(document.activeElement),items:Array.from(headerNodes().host?.querySelectorAll('[data-ix-dropdown-item]')??[]).map(item=>({...describe(item),tabindexAttribute:item.getAttribute('tabindex'),tabIndex:item.tabIndex,active:item===document.activeElement}))});}
  }
  const n=headerNodes(),target=origin==='more'?n.more:origin==='first'?get('popover-header-default-action'):origin==='last'?get('popover-header-overflow-action'):origin==='parent-after'?get('popover-parent-after-header'):options.roving?n.host?.querySelector('[data-ix-dropdown-item]'):n.trigger,before={header:headerFacts(),parentRawCandidates:headerCandidateInventory(),popoverState:read(),active:describe(document.activeElement),logs:logs()};
  const result=await tab(label,target,backwards),entry={label,classification:'strict paired actual source72 Header contract with raw diagnostic/native identity evidence retained; synthetic Tab supplies no UA default navigation',origin,shift:backwards,setup,before,tab:result,after:{header:headerFacts(),parentRawCandidates:headerCandidateInventory(),popoverState:read(),active:describe(document.activeElement),logs:logs()}};headerCases.push(entry);checkHeader(entry,headerContracts[headerCases.length-1]);
 };
 for(const backwards of [false,true])await headerRaw('isolated Header More '+(backwards?'reverse':'forward')+' Tab',{isolated:true},'more',backwards);
 await headerRaw('isolated Header first default action reverse Tab',{isolated:true},'first',true);
 await headerRaw('isolated Header last permanent overflow action forward Tab',{isolated:true},'last');
 await headerRaw('parent boundary forward while actual Header remains open',{isolated:true},'parent-after');
 for(const backwards of [false,true])await headerRaw('Header external active-descendant child '+(backwards?'reverse':'forward')+' Tab',{},'child',backwards);
 for(const backwards of [false,true])await headerRaw('Header external roving child '+(backwards?'reverse':'forward')+' Tab',{roving:true},'child',backwards);
 await headerRaw('Header root false veto on external child forward Tab',{rootVeto:true},'child');
 await headerRaw('Header child false veto on external child forward Tab',{childVeto:true},'child');
 for(const backwards of [false,true])await headerRaw('Header open Large then retained Small external child '+(backwards?'reverse':'forward')+' Tab',{history:true},'child',backwards);
 assert(tabChecks.every(check=>check.matched),'all named document Tab policy comparisons match pinned source contract');
 assert(deferredChecks.every(check=>check.matched),'all five Dropdown exits defer their exact target focus beyond original dispatch');
 assert(queuedChecks.every(check=>check.matched),'all five source-paired adversarial queued-focus cases preserve captured jobs and target identity');
 assert(headerCases.length===13&&headerOpeningChecks.length===13&&headerOpeningChecks.every(check=>check.matched),'all13 source-paired Header cases use exactly one actual More opening without history reopens');
 assert(headerChecks.length===13&&headerChecks.every(check=>check.matched),'all13 Header event/default/logical/native/focus/callback contracts match actual source72');
 const headerModeCounts=rows=>Object.fromEntries([['header','header-request','header-completed'],['child','header-child-request','header-child-completed']].map(([kind,request,completed])=>[kind,{trueRequests:rows.filter(row=>row.kind===request&&row.details.open===true).length,falseRequests:rows.filter(row=>row.kind===request&&row.details.open===false).length,acceptedFalseRequests:rows.filter(row=>row.kind===request&&row.details.open===false&&row.details.vetoed===false).length,vetoedFalseRequests:rows.filter(row=>row.kind===request&&row.details.open===false&&row.details.vetoed===true).length,trueCompletions:rows.filter(row=>row.kind===completed&&row.details.open===true).length,falseCompletions:rows.filter(row=>row.kind===completed&&row.details.open===false).length}]));
 const headerModeRaw=async({sourceCase,initialSmall,origin,permanentOverflow=true,roving=false,rootVeto=false,childVeto=false})=>{
  const withChild=origin==='child',mode=!permanentOverflow?(withChild?'header-small-child-no-overflow':'header-small-no-overflow'):initialSmall?(withChild?'header-small-child':'header-small-isolated'):(withChild?'header-history-child':'header-large-isolated'),label='source'+sourceCase+' Header '+(initialSmall?'Small-open to Large':'Large-open to Small')+' '+(!permanentOverflow?'without permanent overflow ':'')+(withChild?(roving?'already-open roving child first item':'already-open active-descendant child trigger'):origin+' action focused')+(rootVeto?' with root false veto':childVeto?' with child false veto':'');
  await freshFocus(mode,label);await command('show','parent');await wait(()=>{const n=headerNodes();return n.root?.dataset.ixOverlayReady==='true'&&n.more&&!n.more.hidden&&n.header?.classList.contains(initialSmall?'ix-app-small':'ix-app-large');},'fresh actual mode Header ready '+label);await settle();
  const timeline=[],listeners=new AbortController(),n=headerNodes(),start=logs().length,identityNodes=()=>{const n=headerNodes(),items=Array.from(n.host?.querySelectorAll('[data-ix-dropdown-item]')??[]);return{parent:native('parent'),application:n.content?.querySelector('.ix-application')??null,header:n.header,root:n.root,more:n.more,smallHost:n.small,largeHost:n.large,childTrigger:n.trigger,childHost:n.host,childFirstItem:items[0]??null,childLastItem:items[1]??null,defaultAction:get('popover-header-default-action'),overflowAction:get('popover-header-overflow-action')};},held=identityNodes(),phase=(name,extra={})=>{const current=identityNodes(),callbacks=logs().slice(start),row={phase:name,time:performance.now(),header:headerFacts(),parentState:read(),parentNative:{host:describe(native('parent')),connected:!!native('parent')?.isConnected,open:native('parent')?.matches(':popover-open')??false},active:describe(document.activeElement),callbacks,callbackCounts:headerModeCounts(callbacks),sameActualObjects:Object.fromEntries(Object.entries(held).map(([key,value])=>[key,current[key]===value])),actualObjects:Object.fromEntries(Object.entries(current).map(([key,value])=>[key,describe(value)])),...extra};timeline.push(row);return row;};
  activeHeaderMode={sourceCase,label,timeline};
  try{
   for(const host of [n.small,n.large,n.host].filter(Boolean))for(const type of ['beforetoggle','toggle'])host.addEventListener(type,event=>phase('native-'+type,{oldState:event.oldState,newState:event.newState,nativeToggleTarget:describe(event.target)}),{passive:true,signal:listeners.signal});
   for(const type of ['focusin','focusout'])document.addEventListener(type,event=>phase('native-'+type,{focusEventTarget:describe(event.target),relatedTarget:describe(event.relatedTarget)}),{capture:true,passive:true,signal:listeners.signal});
   document.addEventListener('input',event=>{const input=event.target;if(!(input instanceof HTMLInputElement)||!input.matches('[data-ix-overlay-transport]'))return;let report;try{report=JSON.parse(input.value);}catch{return;}if(report?.kind==='anchor-visibility')phase('native-anchor-visibility-input',{transport:describe(input),report:{owner:report.owner,sequence:report.sequence,kind:report.kind,anchor_visibility:report.anchor_visibility,nodes:(report.nodes??[]).filter(node=>node.id===report.anchor_visibility?.root)}});},{capture:true,passive:true,signal:listeners.signal});
   phase('fresh-mode-before-actual-More-input');
   if(roving){input('popover-header-roving','true');await wait(()=>headerNodes().owner?.dataset.roving==='true','actual mode child roving setting');await settle();}
   await openHeader('mode transition initial actual More opening '+sourceCase,headerModeOpeningChecks);phase('actual-More-open-before-any-child-input');
   if(withChild){headerClick(headerNodes().trigger,'mode transition actual external child trigger '+sourceCase);await wait(()=>headerNodes().owner?.dataset.childOpen==='true'&&headerNodes().host?.matches(':popover-open'),'mode transition actual external child opens');await settle();phase('actual-child-open-with-natural-root-state');
    if(roving){const trigger=headerNodes().trigger;trigger.focus();const event=new KeyboardEvent('keydown',{key:'ArrowDown',code:'ArrowDown',bubbles:true,cancelable:true,composed:true});trigger.dispatchEvent(event);driverJournal.push({scenario,kind:'synthetic-actual-native-Header-child-ArrowDown',target:describe(trigger),trusted:event.isTrusted,defaultPrevented:event.defaultPrevented});await settle();phase('actual-ArrowDown-roving-preparation',{items:Array.from(headerNodes().host?.querySelectorAll('[data-ix-dropdown-item]')??[]).map(item=>({...describe(item),tabindexAttribute:item.getAttribute('tabindex'),tabIndex:item.tabIndex,active:item===document.activeElement}))});}
   }
   const current=headerNodes(),target=withChild?(roving?current.host?.querySelector('[data-ix-dropdown-item]'):current.trigger):get(origin==='default'?'popover-header-default-action':'popover-header-overflow-action');if(!target)throw Error('Missing actual mode focus target '+label);target.focus();await settle();assert(document.activeElement===target,'actual native seeded Header mode-change focus '+label);
   if(rootVeto){input('popover-header-root-veto','true');await wait(()=>headerNodes().owner?.dataset.rootVeto==='true','actual mode Header root close veto');}
   if(childVeto){input('popover-header-child-veto','true');await wait(()=>headerNodes().owner?.dataset.childVeto==='true','actual mode Header child close veto');}
   phase('before-public-breakpoint-change',{seed:describe(target),rawParentCandidates:headerCandidateInventory()});
   const transitionStart=logs().length,inputTarget=get('popover-header-small'),event=new Event('input',{bubbles:true,composed:true,cancelable:true});inputTarget.value=String(!initialSmall);driverJournal.push({scenario,kind:'public-fixture-Rust-breakpoint-change',sourceCase,from:initialSmall?'small':'large',to:initialSmall?'large':'small',remount:false,explicitRediscovery:false,reopen:false,input:describe(inputTarget),trusted:event.isTrusted});requestAnimationFrame(time=>{if(!listeners.signal.aborted)phase('prequeued-native-frame-before-mode-input',{frameTime:time});});inputTarget.dispatchEvent(event);
   phase('synchronous-public-breakpoint-input-return',{fixtureInput:{trusted:event.isTrusted,defaultPrevented:event.defaultPrevented,cancelable:event.cancelable,note:'Synthetic input carries the public Rust fixture property change; this is not a keyboard/default-navigation contract.'}});
   const frame=name=>new Promise(resolve=>requestAnimationFrame(time=>resolve(phase(name,{frameTime:time}))));const firstFrame=frame('first-native-frame-after-mode-input');await Promise.resolve();phase('microtask-after-mode-input');await firstFrame;await frame('second-native-frame-after-mode-input');
   await wait(()=>headerNodes().owner?.dataset.small===String(!initialSmall)&&headerNodes().header?.classList.contains(initialSmall?'ix-app-large':'ix-app-small'),'actual retained Header reaches requested public mode');await settle();const settled=phase('settled-mode-change',{rawParentCandidates:headerCandidateInventory()}),entry={sourceCase,label,classification:'raw public Header mode/focus facts; ten settled comparisons and two visibility-incompatible pairs are reported separately',input:{initialSmall,finalSmall:!initialSmall,origin,permanentOverflow,withChild,roving,rootVeto,childVeto,reopen:false,explicitRediscovery:false},timeline,settled,transitionCallbacks:logs().slice(transitionStart),transitionCallbackCounts:headerModeCounts(logs().slice(transitionStart)),setupAndTransitionCallbacks:logs().slice(start),setupAndTransitionCallbackCounts:headerModeCounts(logs().slice(start)),limits:['Natural root state after actual child trigger click is preserved; no fallback reopen is performed','Root and child native flags, geometry, focus loss and callback order are independently observed; outcome parity is not preasserted','Same actual object equality is diagnostic until compared with the source fixture; private caches are neither read nor changed']};headerModeCases.push(entry);save({phase:'raw-public-Header-mode-transition',headerMode:entry});activeHeaderMode=null;
  }finally{listeners.abort();}
 };
 let modeSourceCase=73;
 for(const initialSmall of [false,true])for(const origin of ['default','overflow'])await headerModeRaw({sourceCase:modeSourceCase++,initialSmall,origin});
 await headerModeRaw({sourceCase:modeSourceCase++,initialSmall:true,origin:'default',permanentOverflow:false});
 for(const initialSmall of [false,true])for(const roving of [false,true])await headerModeRaw({sourceCase:modeSourceCase++,initialSmall,origin:'child',roving});
 await headerModeRaw({sourceCase:modeSourceCase++,initialSmall:true,origin:'child',permanentOverflow:false});
 await headerModeRaw({sourceCase:modeSourceCase++,initialSmall:true,origin:'default',permanentOverflow:false,rootVeto:true});
 await headerModeRaw({sourceCase:modeSourceCase++,initialSmall:true,origin:'child',permanentOverflow:false,childVeto:true});
 assert(headerModeCases.length===12&&headerModeOpeningChecks.length===12&&headerModeOpeningChecks.every(check=>check.matched),'all twelve raw paired Header mode scenarios complete actual public setup without reopening');
 const settledModeContracts={
  73:{root:true,child:null,focus:'held',requests:[],completed:{header:0,child:0}},
  74:{root:true,child:null,focus:'held',requests:[],completed:{header:0,child:0}},
  75:{root:true,child:null,focus:'held',requests:[],completed:{header:0,child:0}},
  76:{root:true,child:null,focus:'held',requests:[],completed:{header:0,child:0}},
  77:{root:false,child:null,focus:'held',requests:['header:accepted'],completed:{header:1,child:0}},
  78:{root:false,child:false,focus:'body',initialRoot:false,requests:['child:accepted'],completed:{header:0,child:1}},
  79:{root:false,child:false,focus:'body',initialRoot:false,requests:['child:accepted'],completed:{header:0,child:1}},
  82:{root:false,child:false,focus:'held',requests:['header:accepted','child:accepted'],completed:{header:1,child:1}},
  83:{root:true,child:null,focus:'held',requests:['header:vetoed'],completed:{header:0,child:0}},
  84:{root:false,child:true,focus:'held',requests:['header:accepted','child:vetoed'],completed:{header:1,child:0}}
 };
 for(const entry of headerModeCases){
  const beforeIndex=entry.timeline.findIndex(row=>row.phase==='before-public-breakpoint-change'),before=entry.timeline[beforeIndex],afterWrite=entry.timeline.slice(beforeIndex+1),settled=entry.settled,h=settled.header,focusEvents=afterWrite.filter(row=>row.phase==='native-focusin'||row.phase==='native-focusout').map(row=>({phase:row.phase,time:row.time,target:row.focusEventTarget,relatedTarget:row.relatedTarget,active:row.active})),lostFocusPhases=afterWrite.filter(row=>row.active?.objectId!==before.seed.objectId).map(row=>({phase:row.phase,time:row.time,active:row.active})),expected=settledModeContracts[entry.sourceCase];
  if([73,74,75,76,77,82,83,84].includes(entry.sourceCase))headerModeFocusContinuity.push({sourceCase:entry.sourceCase,label:entry.label,diagnosticOnly:true,sourceContinuityPreserved:focusEvents.length===0&&lostFocusPhases.length===0,status:focusEvents.length===0&&lostFocusPhases.length===0?'preserved':'lost-or-refocused',expected:{actualHeldFocus:before.seed,nativeFocusEvents:0},actual:{settledActive:settled.active,focusEvents,lostFocusPhases},limit:'Separate diagnostic from the settled contract. Transient BODY followed by restoration, or any native focusout/focusin, never counts as source focus continuity.'});
  if(!expected){headerModeVisibilityLimits.push({sourceCase:entry.sourceCase,label:entry.label,classification:'visibility-incompatible source/port native anchor geometry; no mode parity verdict',sourceObservation:{moreIntersecting:false,moreIntersectionRatio:0,transientMoreX:700.296875,childTriggerIntersecting:true,settledRoot:false,settledChild:false,settledFocusRole:'actual More'},portObservation:{rootLogical:h.root.open,rootRegistry:h.root.overlayOpen,rootNative:(h.small==='true'?h.smallHost:h.largeHost)?.popoverOpen,childLogical:h.childOpen,childNative:h.childHost?.popoverOpen,active:settled.active,anchorReports:afterWrite.filter(row=>row.phase==='native-anchor-visibility-input').map(row=>({time:row.time,fact:row.report?.anchor_visibility}))},limit:'Actual source independent IntersectionObserver records More ratio0 during its transient layout while child trigger remains intersecting; port reports its current More intersecting. Do not impose an unconditional mode-close rule or count these two rows as settled mode parity.'});continue;}
  const requests=entry.transitionCallbacks.filter(row=>(row.kind==='header-request'||row.kind==='header-child-request')&&row.details.open===false),requestTokens=requests.map(row=>(row.kind==='header-request'?'header':'child')+':'+(row.details.vetoed?'vetoed':'accepted')),completed={header:entry.transitionCallbackCounts.header.falseCompletions,child:entry.transitionCallbackCounts.child.falseCompletions},activeHost=h.small==='true'?h.smallHost:h.largeHost,target=expected.focus==='held'?before.seed:describe(document.body),childState=expected.child===null?h.childHost===null&&h.childRoot===null:h.childOpen===String(expected.child)&&h.childRoot?.overlayOpen===String(expected.child)&&h.childRoot?.connected===true&&h.childHost?.connected===true&&h.childHost?.popoverOpen===expected.child,identities=entry.timeline.every(row=>Object.values(row.sameActualObjects).every(Boolean)),bodyFocusIn=focusEvents.filter(row=>row.phase==='native-focusin'&&row.target?.objectId===objectId(document.body)),initialRootMatches=before.header.root.open===String(expected.initialRoot??true)&&(before.header.small==='true'?before.header.smallHost:before.header.largeHost)?.popoverOpen===(expected.initialRoot??true),childInitiallyOpen=expected.child===null||before.header.childOpen==='true'&&before.header.childHost?.popoverOpen===true,matched=settled.parentState.parent===true&&settled.parentNative.connected&&settled.parentNative.open&&h.root?.connected===true&&h.root.open===String(expected.root)&&h.root.overlayOpen===String(expected.root)&&h.callbackRootOpen===String(expected.root)&&activeHost?.popoverOpen===expected.root&&childState&&settled.active?.objectId===target?.objectId&&(expected.focus!=='body'||bodyFocusIn.length===0)&&identities&&initialRootMatches&&childInitiallyOpen&&JSON.stringify(requestTokens)===JSON.stringify(expected.requests)&&requests.every(row=>row.details.previous===true)&&completed.header===expected.completed.header&&completed.child===expected.completed.child&&entry.transitionCallbackCounts.header.trueRequests===0&&entry.transitionCallbackCounts.child.trueRequests===0&&entry.transitionCallbackCounts.header.trueCompletions===0&&entry.transitionCallbackCounts.child.trueCompletions===0;
  headerModeChecks.push({sourceCase:entry.sourceCase,label:entry.label,classification:'bounded settled Header mode contract; focus continuity and full callback/native chronology are separate',matched,expected:{...expected,actualTarget:target},actual:{parentOpen:settled.parentState.parent,parentNative:settled.parentNative,rootLogical:h.root.open,rootRegistry:h.root.overlayOpen,rootCallback:h.callbackRootOpen,rootNative:activeHost?.popoverOpen,childLogical:h.childOpen,childRegistry:h.childRoot?.overlayOpen??null,childNative:h.childHost?.popoverOpen??null,active:settled.active,initialRoot:before.header.root.open,childInitiallyOpen,identities,requestTokens,requests,completed,callbackCounts:entry.transitionCallbackCounts,bodyFocusIn}});
 }
 assert(headerModeChecks.length===10&&headerModeChecks.every(check=>check.matched),'all ten approved settled Header mode contracts match source84; two visibility-incompatible rows remain diagnostic');
 const anchorPositions={full:[70,60],top:[70,-20],bottom:[70,140],left:[-50,60],right:[190,60],corner:[190,-20],outside:[-120,60]};
 const anchorRun=async({sourceCase,mode,position,target='root',nested=false,suppressed=false,reset=false,history=null,historyName=null})=>{
  const label=historyName?'source'+sourceCase+' '+historyName:'source'+sourceCase+' actual '+target+' anchor '+position+(reset?' then full reset':suppressed?' then fully outside with visibility suppressed':' with cached open child/root');await freshFocus(mode,label);
  const nodes=()=>{const owner=get('popover-anchor-probe'),rootClip=get('popover-anchor-root-clip'),childClip=get('popover-anchor-child-clip'),rootTrigger=rootClip?.querySelector('[data-ix-overlay-trigger]'),childTrigger=childClip?.querySelector('[data-ix-overlay-trigger]'),rootHost=get('popover-anchor-root-surface')?.querySelector('[data-ix-overlay-host]'),childHost=get('popover-anchor-child-surface')?.querySelector('[data-ix-overlay-host]');return{owner,rootClip,childClip,rootTrigger,childTrigger,rootHost,childHost,rootRoot:rootHost?.closest('[data-ix-overlay-root]'),childRoot:childHost?.closest('[data-ix-overlay-root]'),rootAction:get('popover-anchor-root-action'),childAction:get('popover-anchor-child-action')};};
  const bound=root=>{const input=root?.querySelector('input[data-ix-overlay-transport][data-ix-overlay-native="true"]');return root?.isConnected&&root.dataset.ixOverlayRoot&&root.dataset.ixOverlayOwner&&/^\d+$/.test(root.dataset.ixOverlayGeneration??'')&&root.dataset.ixOverlayMode&&input?.getAttribute('data-ix-overlay-transport')===root.dataset.ixOverlayOwner;};
  await wait(()=>{const n=nodes();return n.owner&&n.rootTrigger&&n.rootHost&&bound(n.rootRoot)&&(!nested||n.childTrigger&&n.childHost&&bound(n.childRoot));},'actual generic partial-anchor owner/generation/mode/native transport bindings');await settle();
  const rect=r=>r?Object.fromEntries(['x','y','width','height','top','right','bottom','left'].map(k=>[k,r[k]])):null,element=node=>{if(!node)return null;const style=getComputedStyle(node);return{...describe(node),connected:node.isConnected,attributes:Object.fromEntries(Array.from(node.attributes).filter(a=>a.name.startsWith('data-')||['popover','hidden','inert','aria-expanded'].includes(a.name)).map(a=>[a.name,a.value])),nativeOpen:node.hasAttribute('popover')?node.matches(':popover-open'):null,rect:rect(node.getBoundingClientRect()),clientRectCount:node.getClientRects().length,inlineStyle:node.getAttribute('style'),computed:{position:style.position,top:style.top,left:style.left,right:style.right,bottom:style.bottom,transform:style.transform,display:style.display,visibility:style.visibility,overflowX:style.overflowX,overflowY:style.overflowY,positionAnchor:style.getPropertyValue('position-anchor'),positionArea:style.getPropertyValue('position-area'),positionTryFallbacks:style.getPropertyValue('position-try-fallbacks')},geometry:geometry(node)};},start=logs().length,held=nodes(),timeline=[],intersectionEntries=[],latest=new Map(),listeners=new AbortController();let entrySequence=0;
  const facts=()=>{const n=nodes();return{owner:n.owner?Object.fromEntries(Object.entries(n.owner.dataset)):null,elements:Object.fromEntries(Object.entries(n).map(([k,v])=>[k,element(v)])),sameActualObjects:Object.fromEntries(Object.entries(held).map(([k,v])=>[k,n[k]===v])),active:describe(document.activeElement),callbacks:logs().slice(start),cssSupport:{anchor:CSS.supports('position-anchor:--ix-probe'),area:CSS.supports('position-area:bottom'),tryFallbacks:CSS.supports('position-try-fallbacks:none')},viewport:{width:innerWidth,height:innerHeight,scrollX,scrollY},clippingScroll:{stage:{left:n.owner?.scrollLeft,top:n.owner?.scrollTop},root:{left:n.rootClip?.scrollLeft,top:n.rootClip?.scrollTop},child:{left:n.childClip?.scrollLeft,top:n.childClip?.scrollTop},rootMenu:{left:n.rootHost?.scrollLeft,top:n.rootHost?.scrollTop},childMenu:{left:n.childHost?.scrollLeft,top:n.childHost?.scrollTop}}};};
  const phase=(name,extra={})=>{const row={phase:name,time:performance.now(),facts:facts(),...extra};timeline.push(row);return row;};readFailureAnchor=facts;if(history)activeAnchorFocus={sourceCase,label,timeline,intersectionEntries};else activeAnchorCase={sourceCase,label,timeline,intersectionEntries};
  const observer=new IntersectionObserver(entries=>{for(const entry of entries){const n=nodes(),role=entry.target===n.rootTrigger?'root':entry.target===n.childTrigger?'child':'unknown',row={sequence:++entrySequence,role,time:performance.now(),entryTime:entry.time,target:describe(entry.target),isIntersecting:entry.isIntersecting,intersectionRatio:entry.intersectionRatio,boundingClientRect:rect(entry.boundingClientRect),intersectionRect:rect(entry.intersectionRect),rootBounds:rect(entry.rootBounds),clip:element(role==='root'?n.rootClip:n.childClip),note:'Independent actual native IO entry; not a fabricated transport report or access to component private observer callback.'};latest.set(role,row);intersectionEntries.push(row);phase('independent-native-intersection',{entry:row});}},{threshold:[0,0.5,1]});
  try{
   for(const trigger of [held.rootTrigger,held.childTrigger].filter(Boolean))observer.observe(trigger);
   for(const host of [held.rootHost,held.childHost].filter(Boolean))for(const type of ['beforetoggle','toggle'])host.addEventListener(type,event=>phase('native-'+type,{target:describe(event.target),oldState:event.oldState,newState:event.newState}),{passive:true,signal:listeners.signal});
   for(const type of ['focusin','focusout'])document.addEventListener(type,event=>phase('native-'+type,{target:describe(event.target),relatedTarget:describe(event.relatedTarget)}),{capture:true,passive:true,signal:listeners.signal});
   document.addEventListener('input',event=>{const input=event.target;if(!(input instanceof HTMLInputElement)||!input.matches('[data-ix-overlay-transport]'))return;let report;try{report=JSON.parse(input.value);}catch{return;}if(report?.kind==='anchor-visibility')phase('actual-native-anchor-transport',{transport:describe(input),report:{owner:report.owner,sequence:report.sequence,kind:report.kind,anchor_visibility:report.anchor_visibility,nodes:(report.nodes??[]).filter(node=>node.id===report.anchor_visibility?.root)}});else if(history&&report?.kind==='modality')phase('actual-native-modality-transport',{transport:describe(input),report:{owner:report.owner,sequence:report.sequence,kind:report.kind,input_kind:report.input_kind,key:report.key,default_prevented:report.default_prevented,target_id:report.target_id,active_id:report.active_id}});},{capture:true,passive:true,signal:listeners.signal});
   const dimensions=entry=>entry&&Math.abs(entry.boundingClientRect.width-100)<.5&&Math.abs(entry.boundingClientRect.height-40)<.5&&Math.abs(entry.clip.rect.width-240)<.5&&Math.abs(entry.clip.rect.height-160)<.5&&(entry.role!=='root'||Math.abs(entry.clip.rect.x-320)<.5&&Math.abs(entry.clip.rect.y-260)<.5),edges=entry=>{const b=entry.boundingClientRect,i=entry.intersectionRect;return['top','bottom','left','right'].filter(side=>side==='top'?i.top>b.top+.5:side==='bottom'?i.bottom<b.bottom-.5:side==='left'?i.left>b.left+.5:i.right<b.right-.5);},condition=(role,key,since=0)=>{const entry=latest.get(role);if(!entry||entry.sequence<=since||!dimensions(entry))return false;if(key==='outside')return entry.isIntersecting===false&&entry.intersectionRect.width*entry.intersectionRect.height===0;const expected=key==='full'?[]:key==='corner'?['top','right']:[key],actual=edges(entry);return entry.isIntersecting&&entry.intersectionRect.width>0&&entry.intersectionRect.height>0&&JSON.stringify(actual)===JSON.stringify(expected)&&(key==='full'?entry.intersectionRatio>.999:entry.intersectionRatio>0&&entry.intersectionRatio<1);};
   await wait(()=>condition('root','full'),'fresh actual root fully-visible 100x40 native IO');phase('conditioned-root-full-before-open',{entry:latest.get('root')});
   if(nested){const outside=get('popover-anchor-modality-outside');outside.focus();const event=new KeyboardEvent('keydown',{key:'Tab',code:'Tab',bubbles:true,cancelable:true,composed:true});let reachedWindow=false;window.addEventListener('keydown',actual=>{if(actual===event)reachedWindow=true;},{passive:true,signal:listeners.signal});outside.dispatchEvent(event);driverJournal.push({scenario,kind:'synthetic-native-Tab-modality-prime-before-anchor-opening',trusted:event.isTrusted,defaultPrevented:event.defaultPrevented,reachedWindow,nativeDefaultNavigation:false,target:describe(outside)});await settle();phase('actual-document-Tab-modality-prime',{trusted:event.isTrusted,defaultPrevented:event.defaultPrevented,reachedWindow,nativeDefaultNavigation:false});}
   const click=role=>{const target=nodes()[role+'Trigger'];target.focus();let actual=null;target.addEventListener('click',event=>actual=event,{capture:true,once:true,passive:true});target.click();driverJournal.push({scenario,kind:'synthetic-actual-linked-anchor-trigger-click',role,target:describe(target),trusted:actual?.isTrusted,defaultPrevented:actual?.defaultPrevented});};
   click('root');await wait(()=>nodes().owner?.dataset.rootOpen==='true'&&nodes().rootHost?.matches(':popover-open'),'actual public root anchor opens');await settle();phase('actual-root-open');
   if(nested){await wait(()=>condition('child','full'),'actual child trigger fully visible within open cached root');phase('conditioned-child-full-before-open',{entry:latest.get('child')});click('child');await wait(()=>nodes().owner?.dataset.rootOpen==='true'&&nodes().rootHost?.matches(':popover-open')&&nodes().owner?.dataset.childOpen==='true'&&nodes().childHost?.matches(':popover-open'),'actual cached root and child remain open together');await settle();assert(nodes().owner?.dataset.rootOpen==='true'&&nodes().rootHost?.matches(':popover-open')&&nodes().owner?.dataset.childOpen==='true'&&nodes().childHost?.matches(':popover-open'),'both actual cached root and child are logical/native open before anchor clipping');phase('actual-child-open-with-real-linked-trigger-topology');}
   const move=async(role,key)=>{const point=anchorPositions[key],since=entrySequence,detail={target:role,x:point[0],y:point[1]};driverJournal.push({scenario,kind:'public-fixture-Rust-anchor-position-change',detail,remount:false,geometryOnly:true});input('popover-anchor-position',JSON.stringify(detail));phase('synchronous-geometry-input-return',{detail});const frame=name=>new Promise(resolve=>requestAnimationFrame(time=>resolve(phase(name,{frameTime:time}))));const first=frame('first-native-frame-after-anchor-move');await Promise.resolve();phase('microtask-after-anchor-move');await first;await frame('second-native-frame-after-anchor-move');await wait(()=>nodes().owner?.dataset[role+'X']===String(point[0])&&nodes().owner?.dataset[role+'Y']===String(point[1])&&condition(role,key,since),'fresh exact native IO '+role+' '+key);await settle();phase('conditioned-'+role+'-'+key,{condition:{role,key,point,actualEntry:latest.get(role),edges:latest.get(role)?.isIntersecting?edges(latest.get(role)):null}});};
   const transitionStart=logs().length;if(history)await history({nodes,phase,move,click,condition,latest,intersectionEntries,listeners});else{await move(target,position);if(reset)await move(target,'full');if(suppressed)await move(target,'outside');}const settled=phase(history?'settled-anchor-focus-history':'settled-partial-anchor-case'),entry={sourceCase,label,classification:history?'raw public negative-focus and vetoed-Tab history; no private suppression latch or outcome oracle injected':'raw actual partial-anchor setup/IO/native placement/focus/callback facts; no FloatingUI or root/child policy outcome parity asserted before source comparison',input:{mode,position,target,nested,suppressed,reset,...(history?{historyName}:{}),suppressAutomaticPlacement:true,sourceEquivalentCondition:'same public suppression request; port CSS adapter is not FloatingUI inline/shift equivalence'},timeline,intersectionEntries,settled,transitionCallbacks:logs().slice(transitionStart),limits:['Exact clipping edges/native IO are required setup facts, not policy verdicts','Configured data-placement, current dataset and CSS properties are separate from actual computed source FloatingUI placement','Native threshold delivery and callback DOM publication do not timestamp private source/Rust callback execution','Negative cases prime actual document Tab before actual root/child clicks; no keyboard mode flags are assigned','No fallback reopen, private discovery, fake IO, host transform rewrite or native menu navigation policy is injected']};if(history){anchorFocusCases.push(entry);save({phase:'raw-anchor-focus-history',anchorFocus:entry});activeAnchorFocus=null;}else{anchorCases.push(entry);save({phase:'raw-partial-anchor-case',anchor:entry});activeAnchorCase=null;}
  }finally{observer.disconnect();listeners.abort();}
 };
 await anchorRun({sourceCase:85,mode:'anchor-right-start',position:'top',reset:true});
 await anchorRun({sourceCase:86,mode:'anchor-right-start',position:'bottom',reset:true});
 await anchorRun({sourceCase:87,mode:'anchor-bottom-start',position:'left',reset:true});
 await anchorRun({sourceCase:88,mode:'anchor-bottom-start',position:'right',reset:true});
 await anchorRun({sourceCase:89,mode:'anchor-bottom-end',position:'corner',reset:true});
 await anchorRun({sourceCase:90,mode:'anchor-suppressed',position:'top',suppressed:true});
 await anchorRun({sourceCase:91,mode:'anchor-nested',position:'outside',nested:true});
 await anchorRun({sourceCase:92,mode:'anchor-nested',position:'outside',target:'child',nested:true});
 assert(anchorCases.length===8,'all eight public partial-anchor cases complete fresh native geometry and IO conditioning');
 const focusHistory=sourceCase=>async({nodes,phase,move,click,condition,intersectionEntries,listeners})=>{
  const childAction=nodes().childAction,childHost=nodes().childHost,rootTrigger=nodes().rootTrigger,childTrigger=nodes().childTrigger;
  const settings=async values=>{driverJournal.push({scenario,kind:'public-fixture-Rust-anchor-settings',values});input('popover-anchor-settings',JSON.stringify(values));const attrs={rootVeto:'rootVeto',childVeto:'childVeto',childRoving:'childRoving',childDisableFocusTrap:'childDisableTrap'};await wait(()=>Object.entries(values).every(([key,value])=>nodes().owner?.dataset[attrs[key]]===String(value)),'actual public anchor settings render');await settle();phase('public-anchor-settings-committed',{values});};
  const action=async label=>{await wait(()=>nodes().owner?.dataset.childOpen==='true'&&nodes().childHost?.matches(':popover-open'),'actual child open before ordinary action focus');nodes().childAction.focus();await settle();assert(document.activeElement===childAction&&nodes().childAction===childAction,'actual retained child native action focus before '+label);phase('actual-child-native-action-focus',{label,heldAction:describe(childAction)});};
  const openRoot=async label=>{await wait(()=>nodes().owner?.dataset.rootOpen==='false'&&!nodes().rootHost?.matches(':popover-open'),'actual root closed before reopen toggle');const since=intersectionEntries.length?intersectionEntries.at(-1).sequence:0;click('root');await wait(()=>nodes().owner?.dataset.rootOpen==='true'&&nodes().rootHost?.matches(':popover-open')&&condition('child','full',since),'actual root reopened with fresh fully-visible child anchor');await settle();phase('actual-root-reopened',{label,heldRootTrigger:describe(rootTrigger)});};
  const openChild=async label=>{await wait(()=>nodes().owner?.dataset.childOpen==='false'&&!nodes().childHost?.matches(':popover-open'),'actual child closed before reopen toggle');click('child');await wait(()=>nodes().owner?.dataset.childOpen==='true'&&nodes().childHost?.matches(':popover-open'),'actual child reopened');await settle();assert(nodes().childHost===childHost,'actual child host identity after '+label);phase('actual-child-reopened',{label,heldChildHost:describe(childHost)});};
  if(sourceCase===93||sourceCase===94){
   await settings(sourceCase===93?{rootVeto:true}:{childVeto:true});await wait(()=>nodes().childRoot?.dataset.ixOverlayMode==='roving-tabindex','actual child public roving mode before ShiftTab');childTrigger.focus();await settle();assert(document.activeElement===childTrigger,'actual child trigger ShiftTab focus seed');
   const event=new KeyboardEvent('keydown',{key:'Tab',code:'Tab',shiftKey:true,bubbles:true,cancelable:true,composed:true});let reachedDocument=false,reachedWindow=false;document.addEventListener('keydown',actual=>{if(actual===event){reachedDocument=true;phase('history-Tab-late-document',{defaultPrevented:event.defaultPrevented});}},{passive:true,signal:listeners.signal});window.addEventListener('keydown',actual=>{if(actual===event){reachedWindow=true;phase('history-Tab-window',{defaultPrevented:event.defaultPrevented});}},{passive:true,signal:listeners.signal});
   driverJournal.push({scenario,kind:'synthetic-actual-child-trigger-ShiftTab',target:describe(childTrigger),trusted:event.isTrusted,nativeDefaultNavigation:false});childTrigger.dispatchEvent(event);phase('history-Tab-synchronous-return',{trusted:event.isTrusted,defaultPrevented:event.defaultPrevented,reachedDocument,reachedWindow});await Promise.resolve();phase('history-Tab-microtask',{defaultPrevented:event.defaultPrevented});await new Promise(resolve=>requestAnimationFrame(time=>{phase('history-Tab-first-native-frame',{frameTime:time});resolve();}));await new Promise(resolve=>requestAnimationFrame(time=>{phase('history-Tab-second-native-frame',{frameTime:time});resolve();}));await settle();phase('history-Tab-settled-with-passive-IO-observation',{defaultPrevented:event.defaultPrevented,reachedDocument,reachedWindow});
   if(sourceCase===93){await settings({rootVeto:false});await action('first root negative after vetoed Tab');await move('root','outside');phase('first-accepted-negative-cycle-complete');await move('root','full');await openRoot('root reopen after first accepted negative cycle');await openChild('root-latch second cycle');await action('second root negative after reopen');await move('root','outside');phase('second-accepted-negative-cycle-complete');}
   else{await openRoot('root actual reopen while child veto remains held');assert(nodes().owner?.dataset.childOpen==='true'&&nodes().childHost?.matches(':popover-open')&&nodes().childHost===childHost,'same actual child remains open through veto-held root reopen');phase('same-open-child-after-veto-held-root-reopen',{heldChildHost:describe(childHost)});await settings({childVeto:false});await action('first child negative after vetoed Tab');await move('child','outside');phase('first-accepted-negative-cycle-complete');await move('child','full');await openChild('child-latch second cycle');await action('second child negative after reopen');await move('child','outside');phase('second-accepted-negative-cycle-complete');}
  }else{
   if(sourceCase===95){const outside=get('popover-anchor-modality-outside'),event=new MouseEvent('mousedown',{bubbles:true,cancelable:true,composed:true,button:0,buttons:1});outside.dispatchEvent(event);driverJournal.push({scenario,kind:'actual-synthetic-mousedown-only-modality-input',target:describe(outside),trusted:event.isTrusted,defaultPrevented:event.defaultPrevented,clickDispatched:false});await settle();phase('actual-outside-mousedown-only-after-keyboard-open-pair',{trusted:event.isTrusted,defaultPrevented:event.defaultPrevented,clickDispatched:false});}
   else assert(nodes().owner?.dataset.childDisableTrap==='true','actual child disableFocusTrap configured before opening');
   await action(sourceCase===95?'pointer-gated root negative':'disableFocusTrap-gated child negative');await move(sourceCase===95?'root':'child','outside');phase('ordinary-negative-gate-cycle-complete');
  }
 };
 await anchorRun({sourceCase:93,mode:'anchor-focus',position:'outside',nested:true,historyName:'cached root vetoed ShiftTab latch then two actual root negative cycles',history:focusHistory(93)});
 await anchorRun({sourceCase:94,mode:'anchor-focus',position:'outside',target:'child',nested:true,historyName:'cached child vetoed ShiftTab latch with same-open-child root reopen then two child negative cycles',history:focusHistory(94)});
 await anchorRun({sourceCase:95,mode:'anchor-focus-pointer',position:'outside',nested:true,historyName:'ordinary root negative after actual outside mousedown resets keyboard modality',history:focusHistory(95)});
 await anchorRun({sourceCase:96,mode:'anchor-focus-no-trap',position:'outside',target:'child',nested:true,historyName:'ordinary child negative with disableFocusTrap configured before connection',history:focusHistory(96)});
 assert(anchorFocusCases.length===4,'all four public negative-focus and vetoed-Tab histories complete actual native setup and IO conditioning');
 const triggerOthersRun=async({sourceCase,operation='open',veto=false,topology=false,publicOnly=false,preprevent=false})=>{
  const mode=topology?'trigger-others-topology':'trigger-others',label='source'+sourceCase+' '+(topology?'cached and live ancestor actual-trigger diagnostic':publicOnly?'public-handle control without own trigger':preprevent?'consumer-preprevented original trigger click':(veto?'vetoed ':'accepted ')+operation+' actual trigger with four target policies');await freshFocus(mode,label);
  const roles=topology?['active','old-parent','live-parent','sibling','descendant']:['active','both','outside','inside','none'];
  const nodes=()=>{const owner=get('trigger-others-probe'),surfaces=Object.fromEntries(roles.map(role=>{const wrapper=get('trigger-others-surface-'+role),host=wrapper?.querySelector('[data-ix-overlay-host]'),root=host?.closest('[data-ix-overlay-root]'),trigger=Array.from(document.querySelectorAll('[data-ix-overlay-trigger]')).find(node=>node.getAttribute('data-ix-overlay-trigger')===root?.dataset.ixOverlayRoot);return[role,{wrapper,host,root,trigger,action:get('trigger-others-action-'+role)}];}));return{owner,surfaces};};
  await wait(()=>{const n=nodes();return n.owner&&roles.every(role=>{const s=n.surfaces[role],input=s.root?.querySelector('input[data-ix-overlay-transport][data-ix-overlay-native="true"]');return s.host&&s.trigger&&s.root?.dataset.ixOverlayOwner&&/^\d+$/.test(s.root?.dataset.ixOverlayGeneration??'')&&input;});},'actual trigger-others stable public surfaces/native bindings');await settle();
  const held=nodes(),start=logs().length,timeline=[],listeners=new AbortController();let originalParent=null,originalNext=null,moved=null;
  const ancestry=node=>{const rows=[];for(let n=node?.parentElement;n;n=n.parentElement)if(n.hasAttribute('data-ix-overlay-host'))rows.push({element:describe(n),root:n.getAttribute('data-ix-overlay-host'),nativeOpen:n.hasAttribute('popover')?n.matches(':popover-open'):null});return rows;};
  const element=node=>node?{...describe(node),connected:node.isConnected,attributes:Object.fromEntries(Array.from(node.attributes).filter(a=>a.name.startsWith('data-')||['hidden','inert','popover','aria-expanded'].includes(a.name)).map(a=>[a.name,a.value])),nativeOpen:node.hasAttribute('popover')?node.matches(':popover-open'):null,geometry:geometry(node)}:null;
  const facts=()=>{const n=nodes();return{state:JSON.parse(n.owner?.dataset.state??'{}'),vetoOpen:JSON.parse(n.owner?.dataset.vetoOpen??'[]'),vetoClose:JSON.parse(n.owner?.dataset.vetoClose??'[]'),active:describe(document.activeElement),surfaces:Object.fromEntries(roles.map(role=>[role,{...Object.fromEntries(Object.entries(n.surfaces[role]).map(([key,node])=>[key,element(node)])),sameActualObjects:Object.fromEntries(Object.entries(held.surfaces[role]).map(([key,node])=>[key,n.surfaces[role][key]===node])),liveTriggerAncestors:ancestry(n.surfaces[role].trigger)}])),callbacks:logs().slice(start)};};
  const phase=(name,extra={})=>{const row={phase:name,time:performance.now(),facts:facts(),...extra};timeline.push(row);return row;};readFailureTriggerOthers=facts;activeTriggerOthers={sourceCase,label,timeline};
  const command=async(action,target,value)=>{const before=logs().filter(row=>row.kind==='trigger-others-public-return').length;driverJournal.push({scenario,kind:'explicit-public-Rust-handle-or-consumer-control',action,target,value});input('trigger-others-command',JSON.stringify({action,target,...(value===undefined?{}:{value})}));await wait(()=>logs().filter(row=>row.kind==='trigger-others-public-return').length>before,'actual public Rust '+action+' return '+target);if(action==='open'||action==='close')await wait(()=>{const n=nodes(),open=action==='open';return JSON.parse(n.owner?.dataset.state??'{}')[target]===open&&n.surfaces[target].host?.matches(':popover-open')===open;},'actual logical/native '+action+' '+target);await settle();phase('public-Rust-input-settled',{action,target,value,adapter:'Rust public handle open/close, source public show property; origin remains separate from actual own trigger'});};
  const modality=async()=>{const outside=get('popover-outside'),keyboard=sourceCase===100;let event;if(keyboard){outside.focus();await settle();event=new KeyboardEvent('keydown',{key:'Tab',code:'Tab',bubbles:true,cancelable:true,composed:true});}else event=new MouseEvent('mousedown',{bubbles:true,cancelable:true,composed:true,button:0});driverJournal.push({scenario,kind:keyboard?'actual-outside-Tab-modality-input':'actual-outside-mousedown-only-modality-input',target:describe(outside),trusted:event.isTrusted,clickDispatched:false,nativeDefaultNavigation:false});outside.dispatchEvent(event);await settle();phase('actual-input-modality-prime-before-operation',{keyboardInput:keyboard,inputType:event.type,trusted:event.isTrusted,defaultPrevented:event.defaultPrevented,clickDispatched:false,nativeDefaultNavigation:false});};
  const actualClick=async(role,tag,{prevent=false}={})=>{const target=nodes().surfaces[role].trigger;target.focus();await settle();assert(document.activeElement===target,'actual retained native trigger focus before '+tag);const event=new MouseEvent('click',{bubbles:true,cancelable:true,composed:true,button:0});let reachedTarget=false,reachedDocument=false,reachedWindow=false;
   if(prevent)target.addEventListener('click',actual=>{if(actual===event){actual.preventDefault();phase('native-consumer-preprevent-original-click',{tag,target:describe(target),defaultPrevented:actual.defaultPrevented});}},{capture:true,once:true,signal:listeners.signal});
   target.addEventListener('click',actual=>{if(actual===event){reachedTarget=true;phase('actual-trigger-target-listener-return',{tag,defaultPrevented:event.defaultPrevented,target:describe(target),adapter:'Source own listener precedes this late local observer; Dioxus delegated own onclick may run later at document, recorded separately'});}},{passive:true,once:true,signal:listeners.signal});
   document.addEventListener('click',actual=>{if(actual===event){reachedDocument=true;phase('actual-trigger-late-document',{tag,defaultPrevented:event.defaultPrevented});}},{passive:true,signal:listeners.signal});window.addEventListener('click',actual=>{if(actual===event){reachedWindow=true;phase('actual-trigger-late-window',{tag,defaultPrevented:event.defaultPrevented});}},{passive:true,signal:listeners.signal});
   driverJournal.push({scenario,kind:'actual-registered-native-trigger-click',role,tag,target:describe(target),trusted:event.isTrusted,preprevent:prevent});phase('before-actual-trigger-click',{tag});target.dispatchEvent(event);phase('actual-trigger-synchronous-return',{tag,defaultPrevented:event.defaultPrevented,reachedTarget,reachedDocument,reachedWindow,trusted:event.isTrusted});await Promise.resolve();phase('actual-trigger-microtask',{tag,defaultPrevented:event.defaultPrevented});await new Promise(resolve=>requestAnimationFrame(time=>{phase('actual-trigger-first-native-frame',{tag,frameTime:time});resolve();}));await new Promise(resolve=>requestAnimationFrame(time=>{phase('actual-trigger-second-native-frame',{tag,frameTime:time});resolve();}));await settle();return phase('actual-trigger-settled',{tag,defaultPrevented:event.defaultPrevented,reachedTarget,reachedDocument,reachedWindow});};
  try{
   for(const role of roles)for(const type of ['beforetoggle','toggle'])held.surfaces[role].host.addEventListener(type,event=>phase('native-'+type,{role,oldState:event.oldState,newState:event.newState}),{passive:true,signal:listeners.signal});
   for(const type of ['focusin','focusout'])document.addEventListener(type,event=>phase('native-'+type,{target:describe(event.target),relatedTarget:describe(event.relatedTarget)}),{capture:true,passive:true,signal:listeners.signal});
   document.addEventListener('input',event=>{if(!(event.target instanceof HTMLInputElement)||!event.target.matches('[data-ix-overlay-transport]'))return;let report;try{report=JSON.parse(event.target.value);}catch{return;}if(['click','anchor-visibility','modality'].includes(report?.kind))phase('actual-native-overlay-report',{report:{owner:report.owner,sequence:report.sequence,kind:report.kind,phase:report.phase,input_kind:report.input_kind,key:report.key,default_prevented:report.default_prevented,target_id:report.target_id,hosts:report.hosts,triggers:report.triggers,anchor_visibility:report.anchor_visibility,nodes:(report.nodes??[]).filter(node=>roles.some(role=>nodes().surfaces[role].root?.dataset.ixOverlayRoot===node.id))}});},{capture:true,passive:true,signal:listeners.signal});
   if(topology){
    await actualClick('old-parent','setup-real-OldParent-trigger');await wait(()=>JSON.parse(nodes().owner.dataset.state)['old-parent']&&nodes().surfaces['old-parent'].host.matches(':popover-open'),'actual OldParent open consumes natural cached children');
    await actualClick('active','setup-real-Active-trigger');await wait(()=>{const n=nodes(),state=JSON.parse(n.owner.dataset.state);return state['old-parent']&&state.active&&n.surfaces['old-parent'].host.matches(':popover-open')&&n.surfaces.active.host.matches(':popover-open');},'actual OldParent and Active retained after real setup clicks');
    await command('open','live-parent');moved=held.surfaces.active.trigger;originalParent=moved.parentNode;originalNext=moved.nextSibling;get('trigger-others-live-parent-slot').append(moved);driverJournal.push({scenario,kind:'tool-only-same-native-trigger-reparent-diagnostic',target:describe(moved),originalParent:describe(originalParent),newParent:describe(moved.parentNode),privateRegistryMutation:false,remount:false});await settle();phase('same-actual-trigger-relocated-without-rediscovery');
    await command('open','sibling');await command('open','descendant');await command('veto-close','active',true);assert(roles.every(role=>JSON.parse(nodes().owner.dataset.state)[role]&&nodes().surfaces[role].host.matches(':popover-open')),'actual five topology surfaces open before tested trigger click');assert(nodes().surfaces.active.trigger===moved&&moved.closest('#trigger-others-live-parent-slot'),'same actual Active trigger occupies real LiveParent content');
    await modality();phase('ready-before-topology-trigger-operation',{cachedHistory:'OldParent and Active populated only through actual prior registered trigger opens; no registry/cache writes',currentNativeAncestry:ancestry(moved)});const operationStart=logs().length;await actualClick('active','tested-Active-close-veto-after-native-reparent');const settled=phase('settled-trigger-only-operation');triggerOthersCases.push({sourceCase,label,input:{topology,operation:'close',veto:true,reparent:'same native trigger, tool diagnostic',suppressTriggerVisibilityCheck:true},timeline,settled,operationCallbacks:logs().slice(operationStart),classification:'raw actual cached/live ancestor operation; local target phase and later document/Window phases remain separate'});
   }else{
    if(operation==='close')await command('open','active');for(const role of roles.slice(1))await command('open',role);assert(roles.slice(1).every(role=>JSON.parse(nodes().owner.dataset.state)[role]&&nodes().surfaces[role].host.matches(':popover-open')),'actual four policy targets concurrently logical/native open before operation');
    if(veto)await command(operation==='open'?'veto-open':'veto-close','active',true);await modality();phase('ready-before-trigger-operation');const operationStart=logs().length;
    if(publicOnly){await command('open','active');phase('public-handle-open-with-targets');await command('close','active');phase('public-handle-close-with-targets');}else await actualClick('active',preprevent?'tested-native-preprevented-trigger':'tested-Active-'+operation,{prevent:preprevent});
    const settled=phase('settled-trigger-only-operation');triggerOthersCases.push({sourceCase,label,input:{topology,operation,veto,publicOnly,preprevent,suppressTriggerVisibilityCheck:true},timeline,settled,operationCallbacks:logs().slice(operationStart),classification:'raw actual trigger/controller-origin and outside-policy facts; no source outcome oracle injected'});
   }
   save({phase:'raw-trigger-only-dismiss-others',triggerOthers:triggerOthersCases.at(-1)});activeTriggerOthers=null;
  }finally{
   if(moved&&originalParent?.isConnected){originalParent.insertBefore(moved,originalNext?.parentNode===originalParent?originalNext:null);await settle();phase('same-actual-trigger-restored-after-operation-observation',{sameObject:nodes().surfaces.active.trigger===moved,restoredParent:describe(moved.parentNode)});}listeners.abort();
  }
 };
 await triggerOthersRun({sourceCase:97});
 await triggerOthersRun({sourceCase:98,veto:true});
 await triggerOthersRun({sourceCase:99,operation:'close'});
 await triggerOthersRun({sourceCase:100,operation:'close',veto:true});
 await triggerOthersRun({sourceCase:101,topology:true});
 await triggerOthersRun({sourceCase:102,publicOnly:true});
 await triggerOthersRun({sourceCase:103,preprevent:true});
 assert(triggerOthersCases.length===7,'all seven actual-trigger/public-handle policy diagnostics complete matched public setup');
 return {ok:true,observations,assertions,driverJournal,tabCases,tabChecks,deferredChecks,queuedChecks,headerCases,headerOpeningChecks,headerChecks,headerModeCases,headerModeOpeningChecks,headerModeChecks,headerModeFocusContinuity,headerModeVisibilityLimits,anchorCases,anchorFocusCases,triggerOthersCases,input:'synthetic input controls invoking real public Rust handles; synthetic KeyboardEvent, programmatic focus and synthetic child button click',limits:['strict named endpoints and callback directions compared with pinned upstream contract; not complete Popover parity','WeakMap object identity and direct object equality certify retained actual nodes only within each fresh fixture; topology controls remount deliberately', 'geometry/computed styles/ancestor facts are diagnostics; no positive visibility or geometry parity asserted','focus-mode fixtures intentionally remount on mode selection; dynamic body add/remove uses Rust Signal state without native application policy injection', 'synthetic Tab has no native focus-navigation default; browser native-emulated Tab and read-only trusted phase recorder are separate evidence', 'Rust consumer callback cancellation and native DOM event cancellation/propagation are measured separately', 'Dropdown forward/reverse and veto release checks follow the paired source contract; native propagation flags remain separately observed', 'source-paired same-turn/reopen/parent-hide endpoints retain original event and frame evidence; native hide/completion chronology is diagnostic', 'same-ID replacement is an explicit native diagnostic fault injection; the exact original is restored only after recording its disconnected result', 'no physical input, accessibility, Brand visual, multiroot or cross-document certification'],renderer:navigator.userAgent};
} catch(error) { return {ok:false,error:String(error),observations,assertions,driverJournal,tabCases,tabChecks,deferredChecks,queuedChecks,headerCases,headerOpeningChecks,headerChecks,headerModeCases,headerModeOpeningChecks,headerModeChecks,headerModeFocusContinuity,headerModeVisibilityLimits,anchorCases,anchorFocusCases,triggerOthersCases,activeTriggerOthers,failureTriggerOthers:readFailureTriggerOthers(),activeHeaderMode,activeAnchorCase,activeAnchorFocus,failureAnchor:readFailureAnchor(),failureHeader:readFailureHeader()}; }
"#;
