//! B18.2 nested ApplicationHeader and Dropdown ownership review fixture.
//! Retains one component/DOM tree per Header slot across breakpoint and top-layer modes.
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

use dioxus::prelude::*;
use dioxus_ix::*;
use serde::Serialize;
use serde_json::{Value, json};

const MAX_EVENTS: usize = 120;
static NEXT_EVENT: AtomicU64 = AtomicU64::new(1);
static NEXT_SCOPE: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ForceBreakpointMode {
    Auto,
    Small,
    Medium,
    Large,
}

impl ForceBreakpointMode {
    const ALL: [Self; 4] = [Self::Auto, Self::Small, Self::Medium, Self::Large];
    fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Small => "sm",
            Self::Medium => "md",
            Self::Large => "lg",
        }
    }
    fn to_breakpoint(self) -> Option<ApplicationBreakpoint> {
        match self {
            Self::Auto => None,
            Self::Small => Some(ApplicationBreakpoint::Small),
            Self::Medium => Some(ApplicationBreakpoint::Medium),
            Self::Large => Some(ApplicationBreakpoint::Large),
        }
    }
}

fn nav_mode_str(mode: DropdownNavigationMode) -> &'static str {
    match mode {
        DropdownNavigationMode::ActiveDescendant => "active-descendant",
        DropdownNavigationMode::RovingTabindex => "roving-tabindex",
    }
}

fn close_behavior_str(behavior: DropdownCloseBehavior) -> &'static str {
    match behavior {
        DropdownCloseBehavior::Both => "both",
        DropdownCloseBehavior::Inside => "inside",
        DropdownCloseBehavior::Outside => "outside",
        DropdownCloseBehavior::None => "none",
    }
}

#[derive(Clone, Debug, Default, Serialize)]
struct GroupDiagnostics {
    scope_mounts: usize,
    scope_drops: usize,
    native_mounts: usize,
    active_scopes: Vec<u64>,
    last_scope_id: u64,
    local_count: usize,
    prevented_count: usize,
    #[serde(rename = "child_last_notified_open")]
    child_open: bool,
    #[serde(rename = "submenu_last_notified_open")]
    submenu_open: bool,
}

#[derive(Clone, Default)]
struct NestedLedger(Rc<RefCell<BTreeMap<&'static str, GroupDiagnostics>>>);

impl NestedLedger {
    fn update<R>(&self, group: &'static str, f: impl FnOnce(&mut GroupDiagnostics) -> R) -> R {
        f(self.0.borrow_mut().entry(group).or_default())
    }
}

impl PartialEq for NestedLedger {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

fn record_event(mut events: Signal<Vec<Value>>, mut payload: Value) {
    let Ok(mut list) = events.try_write() else {
        return;
    };
    if let Some(obj) = payload.as_object_mut() {
        obj.insert(
            "seq".into(),
            json!(NEXT_EVENT.fetch_add(1, Ordering::Relaxed)),
        );
    }
    list.push(payload);
    if list.len() > MAX_EVENTS {
        let overflow = list.len() - MAX_EVENTS;
        drop(list.drain(0..overflow));
    }
}

fn btn_cls(active: bool) -> &'static str {
    if active {
        "header-nested-btn header-nested-btn--active"
    } else {
        "header-nested-btn"
    }
}

#[allow(clippy::too_many_arguments)]
#[component]
fn NestedSlotGroup(
    group: &'static str,
    ledger: NestedLedger,
    external_submenu: DropdownHandle,
    enable_top_layer: bool,
    navigation_mode: DropdownNavigationMode,
    close_behavior: DropdownCloseBehavior,
    reversed: bool,
    input_present: bool,
    veto_child_open: Signal<bool>,
    veto_child_close: Signal<bool>,
    mut diag_revision: Signal<u64>,
    events: Signal<Vec<Value>>,
) -> Element {
    let (mut local_count, mut prevented_count, mut local_revision) = (
        use_signal(|| 0_usize),
        use_signal(|| 0_usize),
        use_signal(|| 0_u64),
    );
    let _ = local_revision();
    let scope_id = use_hook(|| NEXT_SCOPE.fetch_add(1, Ordering::Relaxed));
    let committed = use_hook(|| Rc::new(Cell::new(false)));

    use_effect({
        let (ledger, committed) = (ledger.clone(), committed.clone());
        move || {
            if !committed.replace(true) {
                let mounts = ledger.update(group, |e| {
                    e.scope_mounts += 1;
                    e.local_count = 0;
                    e.prevented_count = 0;
                    e.last_scope_id = scope_id;
                    e.active_scopes.push(scope_id);
                    e.scope_mounts
                });
                local_revision += 1;
                if let Ok(mut rev) = diag_revision.try_write() {
                    *rev += 1;
                }
                record_event(
                    events,
                    json!({"kind": "slot-scope-mounted", "group": group, "scope_id": scope_id, "scope_mounts": mounts}),
                );
            }
        }
    });

    use_drop({
        let ledger = ledger.clone();
        move || {
            if committed.get() {
                let drops = ledger.update(group, |e| {
                    e.scope_drops += 1;
                    e.active_scopes.retain(|id| *id != scope_id);
                    e.scope_drops
                });
                if let Ok(mut rev) = diag_revision.try_write() {
                    *rev += 1;
                }
                record_event(
                    events,
                    json!({"kind": "slot-scope-dropped", "group": group, "scope_id": scope_id, "scope_drops": drops}),
                );
            }
        }
    });

    let stats = ledger.0.borrow().get(group).cloned().unwrap_or_default();
    let (count_val, prev_val) = (local_count(), prevented_count());
    let group_mark = match group {
        "secondary" => "S",
        "overflow" => "O",
        _ => "D",
    };
    let order = if reversed {
        ["menu", "prevent", "action", "input"]
    } else {
        ["input", "action", "prevent", "menu"]
    };
    let item_order = if reversed {
        ["submenu", "external", "disabled", "plain"]
    } else {
        ["plain", "disabled", "submenu", "external"]
    };
    let (mount_ledger, action_ledger, prevent_ledger, change_ledger) =
        (ledger.clone(), ledger.clone(), ledger.clone(), ledger);

    rsx! {
        div {
            id: "header-nested-group-{group}", class: "header-nested-slot-group",
            "data-group": group, "data-scope-token": scope_id.to_string(),
            "data-scope-mounts": stats.scope_mounts.to_string(), "data-scope-drops": stats.scope_drops.to_string(),
            "data-native-mounts": stats.native_mounts.to_string(), "data-local-count": count_val.to_string(),
            "data-prevented-count": prev_val.to_string(), "data-reversed": reversed.to_string(),
            onmounted: move |_| {
                let native_mounts = mount_ledger.update(group, |e| { e.native_mounts += 1; e.native_mounts });
                local_revision += 1;
                if let Ok(mut rev) = diag_revision.try_write() { *rev += 1; }
                record_event(events, json!({"kind": "slot-native-mounted", "group": group, "scope_id": scope_id, "native_mounts": native_mounts}));
            },
            for part in order.into_iter().filter(|part| *part != "input" || input_present) {
                Fragment { key: "{part}", {match part {
                    "input" => rsx! {
                        input {
                            id: "header-nested-{group}-input", class: "header-nested-input",
                            r#type: "text", placeholder: "{group} value", "aria-label": "{group} uncontrolled value", autocomplete: "off",
                        }
                    },
                    "action" => {
                        let action_ledger = action_ledger.clone();
                        rsx! {
                            button {
                                id: "header-nested-{group}-action", class: "header-nested-btn", r#type: "button",
                                aria_label: "{group} action ({count_val})",
                                onclick: move |_| {
                                    let next = local_count() + 1;
                                    local_count.set(next);
                                    action_ledger.update(group, |e| e.local_count = next);
                                    diag_revision += 1;
                                    record_event(events, json!({"kind": "slot-action", "group": group, "prevented": false, "count": next}));
                                },
                                "{group_mark}{count_val}"
                            }
                        }
                    },
                    "prevent" => {
                        let prevent_ledger = prevent_ledger.clone();
                        rsx! {
                            button {
                                id: "header-nested-{group}-prevent", class: "header-nested-btn", r#type: "button",
                                aria_label: "Prevent click ({prev_val})",
                                onclick: move |event: MouseEvent| {
                                    event.prevent_default();
                                    let next = prevented_count() + 1;
                                    prevented_count.set(next);
                                    prevent_ledger.update(group, |e| e.prevented_count = next);
                                    diag_revision += 1;
                                    record_event(events, json!({"kind": "slot-action", "group": group, "prevented": true, "count": next}));
                                },
                                "P{prev_val}"
                            }
                        }
                    },
                    _ => {
                        let change_ledger = change_ledger.clone();
                        rsx! {
                            DropdownButton {
                                label: "Menu", aria_label: "{group} menu",
                                variant: ButtonVariant::Secondary, enable_top_layer, navigation_mode, close_behavior,
                                onbeforechange: move |event: DropdownChangeEvent| {
                                    let veto = if event.open { *veto_child_open.peek() } else { *veto_child_close.peek() };
                                    if veto { event.prevent_default(); }
                                    diag_revision += 1;
                                    record_event(events, json!({"kind": "child-before-change", "group": group, "open": event.open, "previous": event.previous, "veto": veto}));
                                },
                                onchange: move |open: bool| {
                                    change_ledger.update(group, |e| e.child_open = open);
                                    diag_revision += 1;
                                    record_event(events, json!({"kind": "child-change", "group": group, "open": open}));
                                },
                                for item_key in item_order {
                                    Fragment { key: "{item_key}", {match item_key {
                                        "plain" => rsx! { DropdownItem { label: "{group} plain item", onitemclick: move |_| record_event(events, json!({"kind": "item-click", "group": group, "item": "plain"})) } },
                                        "disabled" => rsx! { DropdownItem { label: "Disabled item", disabled: true } },
                                        "submenu" => rsx! {
                                            DropdownSubmenu {
                                                label: "{group} inline submenu",
                                                navigation_mode, enable_top_layer, close_behavior,
                                                DropdownItem { label: "{group} inline leaf 1", onitemclick: move |_| record_event(events, json!({"kind": "item-click", "group": group, "item": "inline-leaf-1"})) }
                                                DropdownItem { label: "{group} inline disabled", disabled: true }
                                                DropdownItem { label: "{group} inline leaf 2", onitemclick: move |_| record_event(events, json!({"kind": "item-click", "group": group, "item": "inline-leaf-2"})) }
                                            }
                                        },
                                        _ => rsx! {
                                            DropdownItem {
                                                label: "{group} submenu", dropdown: external_submenu,
                                            }
                                        },
                                    }} }
                                }
                            }
                        }
                    }
                }} }
            }
            span { class: "header-nested-badge", "aria-hidden": "true", "m{stats.scope_mounts}/n{stats.native_mounts}/d{stats.scope_drops}" }
        }
    }
}

#[allow(clippy::too_many_arguments)]
#[component]
fn HeaderNestedStage(
    ledger: NestedLedger,
    force_mode: ForceBreakpointMode,
    root_top_layer: bool,
    child_top_layer: bool,
    nav_mode: DropdownNavigationMode,
    close_behavior: DropdownCloseBehavior,
    reversed: bool,
    input_present: bool,
    default_present: bool,
    secondary_present: bool,
    overflow_present: bool,
    submenus_present: bool,
    header_present: bool,
    app_present: bool,
    veto_root_open: Signal<bool>,
    veto_root_close: Signal<bool>,
    veto_child_open: Signal<bool>,
    veto_child_close: Signal<bool>,
    mut header_open: Signal<bool>,
    mut diag_revision: Signal<u64>,
    events: Signal<Vec<Value>>,
) -> Element {
    let (default_submenu, secondary_submenu, overflow_submenu) =
        (use_dropdown(), use_dropdown(), use_dropdown());
    let slot = |group: &'static str, present: bool, external_submenu: DropdownHandle| {
        let ledger = ledger.clone();
        present.then(|| {
            rsx! {
                NestedSlotGroup {
                    group, ledger, external_submenu, enable_top_layer: child_top_layer,
                    navigation_mode: nav_mode, close_behavior, reversed, input_present,
                    veto_child_open, veto_child_close, diag_revision, events,
                }
            }
        })
    };

    let header = header_present.then(|| rsx! {
        ApplicationHeader {
            name: "Nested review", aria_label_more_menu_icon_button: "More nested review actions", enable_top_layer: root_top_layer,
            onbeforeoverflowchange: move |event: HeaderOverflowEvent| {
                let veto = if event.open { *veto_root_open.peek() } else { *veto_root_close.peek() };
                if veto { event.prevent_default(); }
                diag_revision += 1;
                record_event(events, json!({"kind": "header-overflow-request", "open": event.open, "previous": event.previous, "veto": veto}));
            },
            onoverflowchange: move |open: bool| {
                header_open.set(open);
                diag_revision += 1;
                record_event(events, json!({"kind": "header-overflow-change", "open": open}));
            },
            actions: slot("default", default_present, default_submenu),
            secondary: slot("secondary", secondary_present, secondary_submenu),
            overflow: slot("overflow", overflow_present, overflow_submenu),
        }
    });

    rsx! {
        div { id: "header-nested-stage", class: "header-nested-stage",
            if app_present {
                Application {
                    force_breakpoint: force_mode.to_breakpoint(), header,
                    menu: rsx! { Menu { pinned: false, MenuItem { label: "Review navigation" } } },
                    p { "Application body. External submenu surfaces mount outside ApplicationHeader." }
                }
            }
            if submenus_present {
                for (group, controller) in [("default", default_submenu), ("secondary", secondary_submenu), ("overflow", overflow_submenu)] {
                    {
                        let ledger = ledger.clone();
                        rsx! {
                            Dropdown {
                                key: "ext-{group}", controller, label: "{group} external submenu",
                                enable_top_layer: child_top_layer, navigation_mode: nav_mode, close_behavior,
                                onbeforechange: move |event: DropdownChangeEvent| {
                                    let veto = if event.open { *veto_child_open.peek() } else { *veto_child_close.peek() };
                                    if veto { event.prevent_default(); }
                                    diag_revision += 1;
                                    record_event(events, json!({"kind": "submenu-before-change", "group": group, "open": event.open, "previous": event.previous, "veto": veto}));
                                },
                                onchange: move |open: bool| {
                                    ledger.update(group, |e| e.submenu_open = open);
                                    diag_revision += 1;
                                    record_event(events, json!({"kind": "submenu-change", "group": group, "open": open}));
                                },
                                DropdownItem { label: "{group} child one", onitemclick: move |_| record_event(events, json!({"kind": "item-click", "group": group, "item": "external-one"})) }
                                DropdownItem { label: "{group} child two", onitemclick: move |_| record_event(events, json!({"kind": "item-click", "group": group, "item": "external-two"})) }
                                input {
                                    id: "header-nested-{group}-submenu-input", class: "header-nested-input",
                                    r#type: "text", placeholder: "{group} submenu value", "aria-label": "{group} submenu value", autocomplete: "off",
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
#[component]
fn HeaderNestedDiagnostics(
    diag_revision: Signal<u64>,
    ledger: NestedLedger,
    owner_epoch: Signal<usize>,
    force_mode: Signal<ForceBreakpointMode>,
    root_top_layer: Signal<bool>,
    child_top_layer: Signal<bool>,
    nav_mode: Signal<DropdownNavigationMode>,
    close_behavior: Signal<DropdownCloseBehavior>,
    veto_root_open: Signal<bool>,
    veto_root_close: Signal<bool>,
    veto_child_open: Signal<bool>,
    veto_child_close: Signal<bool>,
    app_present: Signal<bool>,
    header_present: Signal<bool>,
    default_present: Signal<bool>,
    secondary_present: Signal<bool>,
    overflow_present: Signal<bool>,
    submenus_present: Signal<bool>,
    input_present: Signal<bool>,
    reversed: Signal<bool>,
    header_open: Signal<bool>,
    outside_count: Signal<usize>,
    outside_prevent_count: Signal<usize>,
    events: Signal<Vec<Value>>,
) -> Element {
    let _ = diag_revision();
    let state_json = serde_json::to_string_pretty(&json!({
        "owner_epoch": owner_epoch(), "force_breakpoint": force_mode().as_str(),
        "root_top_layer": root_top_layer(), "child_top_layer": child_top_layer(),
        "navigation_mode": nav_mode_str(nav_mode()), "close_behavior": close_behavior_str(close_behavior()),
        "veto": {"root_open": veto_root_open(), "root_close": veto_root_close(), "child_open": veto_child_open(), "child_close": veto_child_close()},
        "presence": {"app": app_present(), "header": header_present(), "default": default_present(), "secondary": secondary_present(), "overflow": overflow_present(), "submenus": submenus_present(), "inputs": input_present(), "reversed": reversed()},
        "header_last_notified_open": header_open(), "outside_count": outside_count(), "outside_prevent_count": outside_prevent_count(),
        "groups": *ledger.0.borrow(),
    })).unwrap_or_else(|_| "{}".to_string());
    let events_json = serde_json::to_string_pretty(&events()).unwrap_or_else(|_| "[]".to_string());

    rsx! {
        div { class: "header-nested-diagnostics",
            pre { id: "header-nested-state", class: "header-nested-json", tabindex: "0", "{state_json}" }
            details { class: "header-nested-details",
                summary { "Bounded event log (#header-nested-events)" }
                pre { id: "header-nested-events", class: "header-nested-json", tabindex: "0", "{events_json}" }
            }
        }
    }
}

#[component]
pub fn HeaderNestedReview() -> Element {
    let (ledger, diag_revision, events) = (
        use_hook(NestedLedger::default),
        use_signal(|| 0_u64),
        use_signal(Vec::<Value>::new),
    );
    let (mut owner_epoch, mut force_mode) = (
        use_signal(|| 1_usize),
        use_signal(|| ForceBreakpointMode::Auto),
    );
    let (mut root_top_layer, mut child_top_layer) = (use_signal(|| false), use_signal(|| false));
    let (mut nav_mode, mut close_behavior) = (
        use_signal(|| DropdownNavigationMode::ActiveDescendant),
        use_signal(|| DropdownCloseBehavior::Both),
    );
    let (mut veto_root_open, mut veto_root_close) = (use_signal(|| false), use_signal(|| false));
    let (mut veto_child_open, mut veto_child_close) = (use_signal(|| false), use_signal(|| false));
    let (mut reversed, mut input_present) = (use_signal(|| false), use_signal(|| true));
    let (mut default_present, mut secondary_present, mut overflow_present) = (
        use_signal(|| true),
        use_signal(|| true),
        use_signal(|| true),
    );
    let (mut submenus_present, mut header_present, mut app_present) = (
        use_signal(|| true),
        use_signal(|| true),
        use_signal(|| true),
    );
    let (header_open, mut outside_count, mut outside_prevent_count) = (
        use_signal(|| false),
        use_signal(|| 0_usize),
        use_signal(|| 0_usize),
    );

    rsx! {
        style { {STYLES} }
        section {
            id: "header-nested-review", class: "demo-section header-nested-review",
            "data-force": force_mode().as_str(), "data-root-top-layer": root_top_layer().to_string(),
            "data-child-top-layer": child_top_layer().to_string(),
            "data-navigation-mode": nav_mode_str(nav_mode()), "data-close-behavior": close_behavior_str(close_behavior()),
            h2 { "Header nested dropdown ownership" }
            p {
                "Inspect component scope and uncontrolled input identity across responsive and top-layer changes. "
                "Nested menus include inline submenus and linked DropdownItems whose external surfaces mount outside Header. The log records Rust requests and accepted-state notifications; native presentation and focus require observation."
            }
            div { class: "header-nested-controls",
                for candidate in ForceBreakpointMode::ALL {
                    button { id: "header-nested-force-{candidate.as_str()}", class: btn_cls(force_mode() == candidate), r#type: "button", onclick: move |_| force_mode.set(candidate), "Force {candidate.as_str()}" }
                }
                button { id: "header-nested-root-layer", class: btn_cls(root_top_layer()), r#type: "button", onclick: move |_| root_top_layer.toggle(), "Header top layer: {root_top_layer}" }
                button { id: "header-nested-child-layer", class: btn_cls(child_top_layer()), r#type: "button", onclick: move |_| child_top_layer.toggle(), "Child top layer: {child_top_layer}" }
                button {
                    id: "header-nested-nav-mode", class: "header-nested-btn", r#type: "button",
                    onclick: move |_| nav_mode.set(match nav_mode() {
                        DropdownNavigationMode::ActiveDescendant => DropdownNavigationMode::RovingTabindex,
                        DropdownNavigationMode::RovingTabindex => DropdownNavigationMode::ActiveDescendant,
                    }),
                    "Mode: {nav_mode_str(nav_mode())}"
                }
                for (label, candidate) in [("both", DropdownCloseBehavior::Both), ("outside", DropdownCloseBehavior::Outside), ("inside", DropdownCloseBehavior::Inside), ("none", DropdownCloseBehavior::None)] {
                    button { id: "header-nested-close-{label}", class: btn_cls(close_behavior() == candidate), r#type: "button", onclick: move |_| close_behavior.set(candidate), "Close: {label}" }
                }
            }
            details { class: "header-nested-details",
                summary { "Request vetoes and dynamic teardown" }
                div { class: "header-nested-controls",
                    button { id: "header-nested-veto-root-open", class: btn_cls(veto_root_open()), r#type: "button", onclick: move |_| veto_root_open.toggle(), "Veto Header open: {veto_root_open}" }
                    button { id: "header-nested-veto-root-close", class: btn_cls(veto_root_close()), r#type: "button", onclick: move |_| veto_root_close.toggle(), "Veto Header close: {veto_root_close}" }
                    button { id: "header-nested-veto-child-open", class: btn_cls(veto_child_open()), r#type: "button", onclick: move |_| veto_child_open.toggle(), "Veto child open: {veto_child_open}" }
                    button { id: "header-nested-veto-child-close", class: btn_cls(veto_child_close()), r#type: "button", onclick: move |_| veto_child_close.toggle(), "Veto child close: {veto_child_close}" }
                    button { id: "header-nested-reorder", class: "header-nested-btn", r#type: "button", onclick: move |_| reversed.toggle(), "Reorder keyed children" }
                    button { id: "header-nested-toggle-inputs", class: "header-nested-btn", r#type: "button", onclick: move |_| input_present.toggle(), "Toggle inputs" }
                    button { id: "header-nested-toggle-default", class: "header-nested-btn", r#type: "button", onclick: move |_| default_present.toggle(), "Toggle default group" }
                    button { id: "header-nested-toggle-secondary", class: "header-nested-btn", r#type: "button", onclick: move |_| secondary_present.toggle(), "Toggle secondary group" }
                    button { id: "header-nested-toggle-overflow", class: "header-nested-btn", r#type: "button", onclick: move |_| overflow_present.toggle(), "Toggle overflow group" }
                    button { id: "header-nested-toggle-submenus", class: "header-nested-btn", r#type: "button", onclick: move |_| submenus_present.toggle(), "Toggle external submenus" }
                    button { id: "header-nested-toggle-header", class: "header-nested-btn", r#type: "button", onclick: move |_| header_present.toggle(), "Toggle Header" }
                    button { id: "header-nested-toggle-app", class: "header-nested-btn", r#type: "button", onclick: move |_| app_present.toggle(), "Toggle Application" }
                    button { id: "header-nested-fresh-owner", class: "header-nested-btn", r#type: "button", onclick: move |_| owner_epoch += 1, "Fresh owner ({owner_epoch})" }
                }
            }
            div { class: "header-nested-controls",
                button { id: "header-nested-outside", class: "header-nested-btn", r#type: "button", onclick: move |_| { outside_count += 1; record_event(events, json!({"kind": "outside-action", "prevented": false, "count": outside_count()})); }, "Outside action ({outside_count})" }
                button { id: "header-nested-outside-prevent", class: "header-nested-btn", r#type: "button", onclick: move |event: MouseEvent| { event.prevent_default(); outside_prevent_count += 1; record_event(events, json!({"kind": "outside-action", "prevented": true, "count": outside_prevent_count()})); }, "Outside prevent click ({outside_prevent_count})" }
            }
            for epoch in [owner_epoch()] {
                HeaderNestedStage {
                    key: "owner-{epoch}", ledger: ledger.clone(), force_mode: force_mode(),
                    root_top_layer: root_top_layer(), child_top_layer: child_top_layer(),
                    nav_mode: nav_mode(), close_behavior: close_behavior(), reversed: reversed(),
                    input_present: input_present(), default_present: default_present(),
                    secondary_present: secondary_present(), overflow_present: overflow_present(),
                    submenus_present: submenus_present(), header_present: header_present(),
                    app_present: app_present(), veto_root_open, veto_root_close,
                    veto_child_open, veto_child_close, header_open, diag_revision, events,
                }
            }
            button { id: "header-nested-after", class: "header-nested-btn", r#type: "button", "After stage focus target" }
            HeaderNestedDiagnostics {
                diag_revision, ledger, owner_epoch, force_mode, root_top_layer, child_top_layer,
                nav_mode, close_behavior, veto_root_open, veto_root_close, veto_child_open,
                veto_child_close, app_present, header_present, default_present, secondary_present,
                overflow_present, submenus_present, input_present, reversed, header_open,
                outside_count, outside_prevent_count, events,
            }
        }
    }
}

const STYLES: &str = include_str!("styles/header_nested_review.css");
