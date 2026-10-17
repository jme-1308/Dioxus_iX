//! Excluded diagnostic host for two genuinely separate web VirtualDoms.
//! No cross-root policy is implemented or passing oracle asserted here.
//! Each launch_cfg below constructs VirtualDom::new independently in the
//! pinned web renderer; ordinary Portal placement would not prove this scope.

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    eprintln!("menu_window_web_probe is a web-only two-VirtualDom diagnostic; run its web target");
}

#[cfg(target_arch = "wasm32")]
fn main() {
    web_probe::launch();
}

#[cfg(target_arch = "wasm32")]
mod web_probe {
    use dioxus::prelude::*;
    use dioxus_ix::*;
    use serde_json::{Value, json};
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static ROOT_INITIALIZATIONS: AtomicUsize = AtomicUsize::new(0);
    static OWNER_INITIALIZATIONS: AtomicUsize = AtomicUsize::new(0);

    #[derive(Clone)]
    struct RootIdentity {
        name: &'static str,
        token: usize,
    }

    pub fn launch() {
        let document = web_sys::window()
            .expect("native web window")
            .document()
            .expect("native web document");
        let main = document
            .get_element_by_id("main")
            .expect("default dx index must contain #main");
        let host = document.create_element("div").expect("native host");
        host.set_id("window-two-vdom-host");
        host.set_attribute("data-expected-rust-root-count", "2")
            .expect("root count marker");
        host.set_attribute("data-launch-api", "two independent launch_cfg calls")
            .expect("launch marker");
        host.set_attribute("style", "display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:12px;padding:8px;align-items:start")
            .expect("diagnostic host geometry");
        for name in ["a", "b"] {
            let container = document.create_element("div").expect("native mount");
            container.set_id(&format!("window-vdom-mount-{name}"));
            host.append_child(&container).expect("append native mount");
        }
        main.append_child(&host).expect("append two-root host");
        dioxus::web::launch::launch_cfg(
            root_a,
            dioxus::web::Config::new().rootname("window-vdom-mount-a"),
        );
        dioxus::web::launch::launch_cfg(
            root_b,
            dioxus::web::Config::new().rootname("window-vdom-mount-b"),
        );
    }

    fn root_a() -> Element {
        rsx! { ProbeRoot { name: "a" } }
    }

    fn root_b() -> Element {
        rsx! { ProbeRoot { name: "b" } }
    }

    fn record(mut rows: Signal<Vec<Value>>, kind: &'static str, details: Value) {
        let mut rows = rows.write();
        let sequence = rows.last().and_then(|row| row["seq"].as_u64()).unwrap_or(0) + 1;
        rows.push(json!({"seq": sequence, "kind": kind, "details": details}));
        if rows.len() > 40 {
            rows.remove(0);
        }
    }

    #[component]
    fn ProbeRoot(name: &'static str) -> Element {
        let identity = use_root_context(|| RootIdentity {
            name,
            token: ROOT_INITIALIZATIONS.fetch_add(1, Ordering::Relaxed) + 1,
        });
        let mut count = use_signal(|| 0_usize);
        let mut mounted = use_signal(|| true);
        let mut generation = use_signal(|| 0_usize);
        let events = use_signal(Vec::<Value>::new);
        let event_text = serde_json::to_string(&*events.read()).unwrap_or_default();
        let token = identity.token;
        let context_name = identity.name;
        rsx! {
            IxTheme { scheme: ColorScheme::Dark,
                section {
                    id: "window-rust-root-{name}",
                    "data-window-rust-root": name,
                    "data-vdom-scope-token": token.to_string(),
                    "data-context-root-name": context_name,
                    "data-root-init-sequence": token.to_string(),
                    "data-local-count": count().to_string(),
                    "data-owner-mounted": mounted().to_string(),
                    "data-owner-generation": generation().to_string(),
                    style: "min-width:0;border:1px solid;padding:8px",
                    h2 { "Independent Rust root {name}" }
                    p { "Root context {context_name}, token {token}; local count {count}; owner generation {generation}." }
                    button { id: "window-{name}-increment", r#type: "button", onclick: move |_| count += 1, "Increment this root" }
                    button { id: "window-{name}-remove", r#type: "button", onclick: move |_| mounted.set(false), "Remove this root's owners" }
                    button { id: "window-{name}-restore", r#type: "button", onclick: move |_| { generation += 1; mounted.set(true); }, "Fresh owners" }
                    input { id: "window-{name}-remove-input", r#type: "hidden", oninput: move |_| mounted.set(false) }
                    input { id: "window-{name}-restore-input", r#type: "hidden", oninput: move |_| { generation += 1; mounted.set(true); } }
                    button { id: "window-{name}-outside", r#type: "button", "Ordinary outside target {name}" }
                    button { id: "window-{name}-prevented-outside", r#type: "button",
                        onkeydown: move |event| event.prevent_default(),
                        "Default-prevented outside target {name}"
                    }
                    p { "Increment A and inspect B's unchanged count. Hover category A then B; compare both popup states. Focus a category child, open the other root's dropdown, and compare Escape/topmost and delayed return. Outcomes are observations, not asserted parity." }
                    p { "Visible owner buttons inject normal click/down input. For a modality gap, dispatch only input on the retained hidden remove/restore inputs. Root contexts survive owner removal." }
                    if mounted() {
                        for current in [generation()] {
                            Owner { key: "{current}", name, generation: current, token, events }
                        }
                    }
                    details { summary { "This root's Rust Dropdown request/accept log" } pre { style: "white-space:pre-wrap;overflow-wrap:anywhere", "{event_text}" } }
                    if name == "a" { NativeInventory {} }
                }
            }
        }
    }

    #[component]
    fn Owner(
        name: &'static str,
        generation: usize,
        token: usize,
        events: Signal<Vec<Value>>,
    ) -> Element {
        let owner_token = use_hook(|| OWNER_INITIALIZATIONS.fetch_add(1, Ordering::Relaxed) + 1);
        let inherited = use_root_context(|| RootIdentity {
            name: "UNEXPECTED NEW CONTEXT",
            token: usize::MAX,
        });
        rsx! {
            div {
                id: "window-{name}-owner",
                "data-owner-init-sequence": owner_token.to_string(),
                "data-owner-generation": generation.to_string(),
                "data-vdom-scope-token": inherited.token.to_string(),
                "data-context-matches-root": (inherited.token == token && inherited.name == name).to_string(),
                style: "position:relative;height:23rem;min-width:0",
                Application {
                    start_expanded: false, pinned_menu: true, force_breakpoint: ApplicationBreakpoint::Large,
                    menu: rsx! {
                        Menu { label: "Root {name} navigation", application_name: "Root {name}", start_expanded: false, pinned: true,
                            MenuCategory { label: "Root {name} category", icon: "folder",
                                MenuItem { label: "Root {name} child" }
                                input { id: "window-{name}-category-input", r#type: "text", "aria-label": "Root {name} category focus target", style: "width:10rem", placeholder: "Native child focus" }
                                DiagnosticDropdown { name, kind: "nested", events }
                            }
                        }
                    },
                    div { style: "padding:8px;min-width:0",
                        p { "Owner {owner_token}; context {inherited.token}." }
                        DiagnosticDropdown { name, kind: "outside", events }
                        DiagnosticPopover { name, events }
                    }
                }
            }
        }
    }

    #[component]
    fn DiagnosticDropdown(
        name: &'static str,
        kind: &'static str,
        events: Signal<Vec<Value>>,
    ) -> Element {
        let dropdown = use_dropdown();
        let mut veto_close = use_signal(|| false);
        let mut requests = use_signal(|| 0_usize);
        let mut accepted = use_signal(|| 0_usize);
        let open = dropdown.is_open();
        rsx! {
            div {
                id: "window-{name}-{kind}-dropdown",
                "data-open": open.to_string(),
                "data-request-count": requests().to_string(),
                "data-accepted-count": accepted().to_string(),
                "data-veto-close": veto_close().to_string(),
                label {
                    input { id: "window-{name}-{kind}-veto", r#type: "checkbox", checked: veto_close(), oninput: move |event| veto_close.set(event.checked()) }
                    "Veto this Dropdown's close"
                }
                div {
                    DropdownTrigger { dropdown, "Root {name} {kind} dropdown" }
                    Dropdown { controller: dropdown, label: "Root {name} {kind} dropdown", enable_top_layer: true,
                        navigation_mode: DropdownNavigationMode::RovingTabindex,
                        onbeforechange: move |event: DropdownChangeEvent| {
                            requests += 1;
                            if !event.open && veto_close() { event.prevent_default(); }
                            record(events, "dropdown-request", json!({"root": name, "kind": kind, "open": event.open, "previous": event.previous, "vetoed": event.default_prevented()}));
                        },
                        onchange: move |value| { accepted += 1; record(events, "dropdown-accepted", json!({"root": name, "kind": kind, "open": value})); },
                        DropdownItem { label: "Root {name} {kind} action" }
                    }
                }
                input { id: "window-{name}-{kind}-open-input", r#type: "hidden", oninput: move |_| dropdown.open() }
                input { id: "window-{name}-{kind}-close-input", r#type: "hidden", oninput: move |_| dropdown.close(false) }
            }
        }
    }

    #[component]
    fn DiagnosticPopover(name: &'static str, events: Signal<Vec<Value>>) -> Element {
        let popover = use_popover();
        let mut veto_close = use_signal(|| false);
        let mut requests = use_signal(|| 0_usize);
        let mut accepted = use_signal(|| 0_usize);
        let open = popover.is_open();
        rsx! {
            div {
                id: "window-{name}-popover",
                "data-open": open.to_string(),
                "data-request-count": requests().to_string(),
                "data-accepted-count": accepted().to_string(),
                "data-veto-close": veto_close().to_string(),
                label {
                    input { id: "window-{name}-popover-veto", r#type: "checkbox", checked: veto_close(), oninput: move |event| veto_close.set(event.checked()) }
                    "Veto this Popover's close"
                }
                PopoverTrigger { popover, "Root {name} native popover" }
                Popover { popover, label: "Root {name} native popover", close_on_click_outside: true,
                    onshowchange: move |event: ToggleEvent| {
                        requests += 1;
                        let previous = popover.is_open();
                        if previous && veto_close() { event.prevent_default(); }
                        record(events, "popover-request", json!({"root": name, "open": !previous, "previous": previous, "vetoed": !event.default_action_enabled()}));
                    },
                    onshowchanged: move |value| { accepted += 1; record(events, "popover-accepted", json!({"root": name, "open": value})); },
                    input { id: "window-{name}-popover-focus", r#type: "text", "aria-label": "Root {name} popover focus target", placeholder: "Native popover focus" }
                }
            }
        }
    }

    const INVENTORY: &str = r#"
let previous='',frame=0,alive=true;const listeners=new AbortController();
const describe=n=>n?{id:n.id,tag:n.tagName,label:n.getAttribute?.('aria-label')}:null;
function read(){
 const roots=[...document.querySelectorAll('[data-window-rust-root]')];
 const value={
  nativeRootHostCount:roots.length,uniqueRustRootTokens:[...new Set(roots.map(n=>n.dataset.vdomScopeToken))],
  roots:roots.map(root=>({name:root.dataset.windowRustRoot,token:root.dataset.vdomScopeToken,contextName:root.dataset.contextRootName,localCount:root.dataset.localCount,generation:root.dataset.ownerGeneration,mounted:root.dataset.ownerMounted,
   owner:root.querySelector('[data-owner-init-sequence]')?.dataset.ownerInitSequence??null,
   contextMatchesRoot:root.querySelector('[data-owner-init-sequence]')?.dataset.contextMatchesRoot??null,
   menus:[...root.querySelectorAll('nav.ix-menu')].map(n=>({id:n.id,native:n.dataset.menuNative})),
   documentOwners:[...root.querySelectorAll('input[data-menu-document-owner]')].map(n=>({owner:n.dataset.menuDocumentOwner,ready:n.dataset.menuDocumentReady})),
   overlayOwners:[...root.querySelectorAll('input[data-ix-overlay-transport]')].map(n=>({owner:n.dataset.ixOverlayTransport,native:n.dataset.ixOverlayNative})),
   categories:[...root.querySelectorAll('[data-menu-category-root]')].map(n=>({id:n.dataset.menuCategoryRoot,mode:n.dataset.menuCategoryKeyboardMode,open:n.dataset.menuCategoryPopupOpen,native:n.dataset.categoryNative,hoverNative:n.dataset.categoryHoverNative})),
   overlays:[...root.querySelectorAll('[data-ix-overlay-root]')].map(n=>({id:n.dataset.ixOverlayRoot,owner:n.dataset.ixOverlayOwner,open:n.getAttribute('data-ix-overlay-open')}))})),
  active:describe(document.activeElement),
  limits:['DOM host/token inventory corroborates two independent Rust launch_cfg calls; DOM hosts alone do not establish VirtualDom identity','read-only native facts; no window policy, oracle verdict, IPC ordering, physical input, AT, Brand or Desktop proof','generic Dropdown veto controls affect that actual Dropdown, not an invented public MenuCategory inner-flyout veto API']
 };
 const next=JSON.stringify(value);if(next!==previous){previous=next;dioxus.send(value);}
}
function schedule(){if(!alive||frame)return;frame=requestAnimationFrame(()=>{frame=0;if(alive)read();});}
const observer=new MutationObserver(schedule);observer.observe(document.body,{subtree:true,childList:true,attributes:true,attributeFilter:['data-local-count','data-owner-generation','data-owner-mounted','data-menu-native','data-menu-document-ready','data-menu-category-keyboard-mode','data-menu-category-popup-open','data-ix-overlay-native','data-open','data-category-native','data-category-hover-native']});
document.addEventListener('focusin',schedule,{capture:true,passive:true,signal:listeners.signal});
window.addEventListener('keydown',schedule,{passive:true,signal:listeners.signal});
schedule();try{await dioxus.recv();}finally{alive=false;observer.disconnect();listeners.abort();if(frame)cancelAnimationFrame(frame);}
"#;

    #[component]
    fn NativeInventory() -> Element {
        let mut facts = use_signal(|| json!({"waiting": true}));
        let channel = use_hook(|| Rc::new(RefCell::new(None::<dioxus::document::Eval>)));
        let cleanup = channel.clone();
        use_drop(move || {
            if let Some(eval) = cleanup.borrow_mut().take() {
                let _ = eval.send("finish");
            }
        });
        let text = serde_json::to_string_pretty(&*facts.read()).unwrap_or_default();
        rsx! {
            details { open: true,
                onmounted: move |_| {
                    let mut eval = dioxus::document::eval(INVENTORY);
                    *channel.borrow_mut() = Some(eval);
                    spawn(async move {
                        while let Ok(mut value) = eval.recv::<Value>().await {
                            value["rustRootInitializationCount"] = json!(ROOT_INITIALIZATIONS.load(Ordering::Relaxed));
                            facts.set(value);
                        }
                    });
                },
                summary { "Read-only native document inventory for both Rust roots" }
                pre { id: "window-two-vdom-native-inventory", style: "white-space:pre-wrap;overflow-wrap:anywhere;font-size:11px", "{text}" }
            }
        }
    }
}
