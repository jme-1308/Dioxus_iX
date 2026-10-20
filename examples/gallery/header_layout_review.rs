//! Neutral ApplicationHeader responsive slot layout, single-tree identity, overflow More dropdown,
//! top-layer opt-in, and Application/Menu context review fixture for B18.2.
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

use dioxus::prelude::*;
use dioxus_ix::*;
use serde_json::{Value, json};

use crate::svg_data_uri;

const MAX_EVENTS: usize = 200;
static NEXT_EVENT: AtomicU64 = AtomicU64::new(1);

static APP_ICON_DATA_URL: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
    svg_data_uri::svg_data_uri(include_str!("assets/header-app-icon.svg"))
});
static COMPANY_LOGO_DATA_URL: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
    svg_data_uri::svg_data_uri(include_str!("assets/header-company-logo.svg"))
});

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ContainerMode {
    Contained,
    Standalone,
}

impl ContainerMode {
    fn as_str(self) -> &'static str {
        match self {
            Self::Contained => "contained",
            Self::Standalone => "standalone",
        }
    }

    fn from_query(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "contained" | "application" | "app" => Some(Self::Contained),
            "standalone" | "header" => Some(Self::Standalone),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ForceMode {
    Auto,
    Small,
    Medium,
    Large,
}

impl ForceMode {
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

    fn from_query(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "auto" | "none" => Some(Self::Auto),
            "sm" | "small" => Some(Self::Small),
            "md" | "medium" => Some(Self::Medium),
            "lg" | "large" => Some(Self::Large),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ConfigMode {
    None,
    Loading,
    PopulatedA,
    PopulatedB,
}

impl ConfigMode {
    fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Loading => "loading",
            Self::PopulatedA => "populated-a",
            Self::PopulatedB => "populated-b",
        }
    }

    fn from_query(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "none" | "absent" | "empty" => Some(Self::None),
            "loading" => Some(Self::Loading),
            "populated" | "populated-a" | "a" => Some(Self::PopulatedA),
            "populated-b" | "replace" | "b" => Some(Self::PopulatedB),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GroupMode {
    Active,
    Absent,
    EmptyComponent,
    TextOnly,
    Hidden,
    Disabled,
}

impl GroupMode {
    fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Absent => "absent",
            Self::EmptyComponent => "empty-component",
            Self::TextOnly => "text-only",
            Self::Hidden => "hidden",
            Self::Disabled => "disabled",
        }
    }

    fn is_present(self) -> bool {
        !matches!(self, Self::Absent)
    }

    fn has_elements(self) -> bool {
        matches!(self, Self::Active | Self::Hidden | Self::Disabled)
    }

    fn from_query(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "active" | "true" | "1" | "present" => Some(Self::Active),
            "absent" | "false" | "0" | "none" => Some(Self::Absent),
            "empty" | "empty-component" => Some(Self::EmptyComponent),
            "text" | "text-only" => Some(Self::TextOnly),
            "hidden" => Some(Self::Hidden),
            "disabled" => Some(Self::Disabled),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct GroupIdentityStats {
    mount_count: usize,
    native_mount_count: usize,
    drop_count: usize,
    active_count: usize,
    last_instance_id: usize,
    local_count: usize,
}

impl GroupIdentityStats {
    fn to_json(&self) -> Value {
        json!({
            "mount_count": self.mount_count,
            "native_mount_count": self.native_mount_count,
            "drop_count": self.drop_count,
            "active_count": self.active_count,
            "last_instance_id": self.last_instance_id,
            "local_count": self.local_count,
        })
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct SlotIdentityLedger {
    next_instance_id: usize,
    default_group: GroupIdentityStats,
    secondary_group: GroupIdentityStats,
    overflow_group: GroupIdentityStats,
    avatar_group: GroupIdentityStats,
}

impl SlotIdentityLedger {
    fn stats_mut(&mut self, group: &str) -> &mut GroupIdentityStats {
        match group {
            "default" => &mut self.default_group,
            "secondary" => &mut self.secondary_group,
            "overflow" => &mut self.overflow_group,
            "avatar" => &mut self.avatar_group,
            _ => &mut self.default_group,
        }
    }

    fn stats(&self, group: &str) -> GroupIdentityStats {
        match group {
            "default" => self.default_group.clone(),
            "secondary" => self.secondary_group.clone(),
            "overflow" => self.overflow_group.clone(),
            "avatar" => self.avatar_group.clone(),
            _ => self.default_group.clone(),
        }
    }

    fn allocate_instance(&mut self) -> usize {
        self.next_instance_id += 1;
        self.next_instance_id
    }

    fn record_mount(&mut self, group: &str, instance_id: usize) -> GroupIdentityStats {
        let stats = self.stats_mut(group);
        stats.mount_count += 1;
        stats.active_count += 1;
        stats.last_instance_id = instance_id;
        stats.local_count = 0;
        stats.clone()
    }

    fn record_drop(&mut self, group: &str) -> GroupIdentityStats {
        let stats = self.stats_mut(group);
        stats.drop_count += 1;
        stats.active_count = stats.active_count.saturating_sub(1);
        stats.clone()
    }

    fn record_local_count(&mut self, group: &str, count: usize) -> GroupIdentityStats {
        let stats = self.stats_mut(group);
        stats.local_count = count;
        stats.clone()
    }

    fn to_json(&self) -> Value {
        json!({
            "default": self.default_group.to_json(),
            "secondary": self.secondary_group.to_json(),
            "overflow": self.overflow_group.to_json(),
            "avatar": self.avatar_group.to_json(),
        })
    }
}

#[derive(Clone, Debug)]
struct InitialQueryOverrides {
    container_mode: ContainerMode,
    force_mode: ForceMode,
    stage_width: String,
    enable_top_layer: bool,
    show_menu: bool,
    sidebar_present: bool,
    show_app_switch: bool,
    config_mode: ConfigMode,
    veto_switch: bool,
    default_mode: GroupMode,
    secondary_mode: GroupMode,
    overflow_mode: GroupMode,
    avatar_present: bool,
    logo_present: bool,
    leading_present: bool,
    company_logo_present: bool,
    app_image_present: bool,
    hide_bottom_border: bool,
    pinned_menu: bool,
    reverse_group_order: bool,
}

impl Default for InitialQueryOverrides {
    fn default() -> Self {
        Self {
            container_mode: ContainerMode::Contained,
            force_mode: ForceMode::Auto,
            stage_width: "100%".to_string(),
            enable_top_layer: false,
            show_menu: false,
            sidebar_present: false,
            show_app_switch: false,
            config_mode: ConfigMode::None,
            veto_switch: true,
            default_mode: GroupMode::Active,
            secondary_mode: GroupMode::Active,
            overflow_mode: GroupMode::Active,
            avatar_present: true,
            logo_present: true,
            leading_present: true,
            company_logo_present: false,
            app_image_present: true,
            hide_bottom_border: false,
            pinned_menu: false,
            reverse_group_order: false,
        }
    }
}

fn parse_bool_query(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

fn read_raw_query_string() -> String {
    #[cfg(all(feature = "web", target_arch = "wasm32"))]
    {
        if let Some(search) = web_sys::window().and_then(|win| win.location().search().ok()) {
            return search;
        }
    }
    std::env::var("DIOXUS_IX_HEADER_LAYOUT_QUERY").unwrap_or_default()
}

fn load_initial_query_overrides() -> InitialQueryOverrides {
    let mut cfg = InitialQueryOverrides::default();
    let raw = read_raw_query_string();
    let trimmed = raw.trim().trim_start_matches('?');
    if trimmed.is_empty() {
        return cfg;
    }

    for pair in trimmed.split('&') {
        if pair.is_empty() {
            continue;
        }
        let (key, value) = match pair.split_once('=') {
            Some((k, v)) => (k.trim(), v.trim()),
            None => (pair.trim(), "true"),
        };
        match key {
            "mode" | "container" | "header-layout-mode" => {
                if let Some(parsed) = ContainerMode::from_query(value) {
                    cfg.container_mode = parsed;
                }
            }
            "force" | "breakpoint" | "header-layout-force" => {
                if let Some(parsed) = ForceMode::from_query(value) {
                    cfg.force_mode = parsed;
                }
            }
            "width" | "stage_width" | "header-layout-width" => {
                if matches!(value, "100%" | "360px" | "600px" | "900px") {
                    cfg.stage_width = value.to_string();
                }
            }
            "top_layer" | "enable_top_layer" | "header-layout-top-layer" => {
                if let Some(flag) = parse_bool_query(value) {
                    cfg.enable_top_layer = flag;
                }
            }
            "show_menu" | "header-layout-show-menu" => {
                if let Some(flag) = parse_bool_query(value) {
                    cfg.show_menu = flag;
                }
            }
            "sidebar" | "header-layout-sidebar" => {
                if let Some(flag) = parse_bool_query(value) {
                    cfg.sidebar_present = flag;
                }
            }
            "show_app_switch" | "header-layout-show-app-switch" => {
                if let Some(flag) = parse_bool_query(value) {
                    cfg.show_app_switch = flag;
                }
            }
            "config" | "app_switch_config" | "header-layout-config" => {
                if let Some(parsed) = ConfigMode::from_query(value) {
                    cfg.config_mode = parsed;
                }
            }
            "veto_switch" | "header-layout-veto-switch" => {
                if let Some(flag) = parse_bool_query(value) {
                    cfg.veto_switch = flag;
                }
            }
            "default" | "actions" | "header-layout-default" => {
                if let Some(parsed) = GroupMode::from_query(value) {
                    cfg.default_mode = parsed;
                }
            }
            "secondary" | "header-layout-secondary" => {
                if let Some(parsed) = GroupMode::from_query(value) {
                    cfg.secondary_mode = parsed;
                }
            }
            "overflow" | "header-layout-overflow" => {
                if let Some(parsed) = GroupMode::from_query(value) {
                    cfg.overflow_mode = parsed;
                }
            }
            "avatar" | "header-layout-avatar" => {
                if let Some(flag) = parse_bool_query(value) {
                    cfg.avatar_present = flag;
                }
            }
            "logo" | "header-layout-logo" => {
                if let Some(flag) = parse_bool_query(value) {
                    cfg.logo_present = flag;
                }
            }
            "leading" | "children" | "header-layout-leading" => {
                if let Some(flag) = parse_bool_query(value) {
                    cfg.leading_present = flag;
                }
            }
            "company_logo" | "header-layout-company-logo" => {
                if let Some(flag) = parse_bool_query(value) {
                    cfg.company_logo_present = flag;
                }
            }
            "app_image" | "app_icon" | "header-layout-app-image" => {
                if let Some(flag) = parse_bool_query(value) {
                    cfg.app_image_present = flag;
                }
            }
            "borderless" | "hide_bottom_border" | "header-layout-borderless" => {
                if let Some(flag) = parse_bool_query(value) {
                    cfg.hide_bottom_border = flag;
                }
            }
            "pinned_menu" | "header-layout-pinned-menu" => {
                if let Some(flag) = parse_bool_query(value) {
                    cfg.pinned_menu = flag;
                }
            }
            "reverse_order" | "header-layout-reverse-group-order" => {
                if let Some(flag) = parse_bool_query(value) {
                    cfg.reverse_group_order = flag;
                }
            }
            _ => {}
        }
    }

    cfg
}

fn push_event(mut events: Signal<Vec<Value>>, mut entry: Value) {
    let Ok(mut list) = events.try_write() else {
        return;
    };
    if let Some(object) = entry.as_object_mut() {
        object.insert(
            "seq".into(),
            json!(NEXT_EVENT.fetch_add(1, Ordering::Relaxed)),
        );
    }
    list.push(entry);
    if list.len() > MAX_EVENTS {
        let excess = list.len() - MAX_EVENTS;
        drop(list.drain(0..excess));
    }
}

fn build_app_switch_config(mode: ConfigMode) -> Option<AppSwitchConfig> {
    match mode {
        ConfigMode::None => None,
        ConfigMode::Loading => Some(AppSwitchConfig {
            current_app_id: "ops-console".to_string(),
            label: "Switch to application".to_string(),
            loading_label: "Loading available applications...".to_string(),
            apps: Vec::new(),
        }),
        ConfigMode::PopulatedA => Some(AppSwitchConfig {
            current_app_id: "ops-console".to_string(),
            label: "Switch to application".to_string(),
            loading_label: "Loading available applications...".to_string(),
            apps: vec![
                AppSwitchEntry {
                    id: "ops-console".to_string(),
                    name: "Operations Console".to_string(),
                    description: "Live telemetry and diagnostics".to_string(),
                    url: "#ops-console".to_string(),
                    target: "_self".to_string(),
                    icon_src: APP_ICON_DATA_URL.to_string(),
                },
                AppSwitchEntry {
                    id: "grid-planner".to_string(),
                    name: "Grid Planner".to_string(),
                    description: "Substation topology and dispatch".to_string(),
                    url: "#grid-planner".to_string(),
                    target: "_self".to_string(),
                    icon_src: APP_ICON_DATA_URL.to_string(),
                },
            ],
        }),
        ConfigMode::PopulatedB => Some(AppSwitchConfig {
            current_app_id: "asset-ledger".to_string(),
            label: "Switch to application (Updated)".to_string(),
            loading_label: "Refreshing catalog...".to_string(),
            apps: vec![
                AppSwitchEntry {
                    id: "asset-ledger".to_string(),
                    name: "Asset Ledger".to_string(),
                    description: "Substation inventory and maintenance".to_string(),
                    url: "#asset-ledger".to_string(),
                    target: "_self".to_string(),
                    icon_src: APP_ICON_DATA_URL.to_string(),
                },
                AppSwitchEntry {
                    id: "alarm-matrix".to_string(),
                    name: "Alarm Matrix".to_string(),
                    description: "Priority incident routing".to_string(),
                    url: "#alarm-matrix".to_string(),
                    target: "_self".to_string(),
                    icon_src: APP_ICON_DATA_URL.to_string(),
                },
            ],
        }),
    }
}

#[derive(Clone, PartialEq)]
struct HeaderSlotControlState {
    instance_id: usize,
    stats: GroupIdentityStats,
    count: usize,
    hidden: bool,
    disabled: bool,
}

// Each native control owns its registration lifetime. Empty/text group output
// and optional input removal therefore release exactly the removed handle.
#[component]
fn HeaderSlotControl(
    group: &'static str,
    label: &'static str,
    part: &'static str,
    state: HeaderSlotControlState,
    onclick: EventHandler<MouseEvent>,
) -> Element {
    let id = format!("header-layout-{group}-{part}");
    let onmounted = use_focus_mount(id.clone());
    let stats = &state.stats;
    if part == "input" {
        rsx! {
            input {
                key: "slot-input-{group}",
                id,
                class: "header-layout-slot-input",
                r#type: "text",
                placeholder: "{label} draft...",
                aria_label: "{label} uncontrolled input",
                disabled: state.disabled,
                hidden: state.hidden,
                "data-group": "{group}",
                "data-instance-id": "{state.instance_id}",
                "data-mount-count": "{stats.mount_count}",
                "data-drop-count": "{stats.drop_count}",
                "data-active-count": "{stats.active_count}",
                "data-local-count": "{state.count}",
                onmounted,
            }
        }
    } else {
        let action_aria = format!("{label} ({})", state.count);
        let action_text = if group == "avatar" {
            format!("A{}", state.count)
        } else {
            action_aria.clone()
        };
        rsx! {
            button {
                key: "slot-action-{group}",
                id,
                class: "header-layout-slot-action ix-button ix-button-subtle-tertiary",
                r#type: "button",
                aria_label: action_aria,
                disabled: state.disabled,
                hidden: state.hidden,
                "data-group": "{group}",
                "data-instance-id": "{state.instance_id}",
                "data-mount-count": "{stats.mount_count}",
                "data-drop-count": "{stats.drop_count}",
                "data-active-count": "{stats.active_count}",
                "data-local-count": "{state.count}",
                onmounted,
                onclick,
                "{action_text}"
            }
        }
    }
}

#[component]
fn HeaderSlotGroup(
    group: &'static str,
    label: &'static str,
    mode: GroupMode,
    reverse_order: bool,
    inputs_present: bool,
    ledger: Rc<RefCell<SlotIdentityLedger>>,
    mut diag_revision: Signal<u64>,
    events: Signal<Vec<Value>>,
) -> Element {
    let mut local_count = use_signal(|| 0_usize);
    let instance_id = use_hook({
        let ledger = ledger.clone();
        move || ledger.borrow_mut().allocate_instance()
    });
    let committed = use_hook(|| Rc::new(Cell::new(false)));
    let mut local_revision = use_signal(|| 0_u64);
    let _ = local_revision();
    use_effect({
        let ledger = ledger.clone();
        let committed = committed.clone();
        move || {
            if committed.replace(true) {
                return;
            }
            let stats = ledger.borrow_mut().record_mount(group, instance_id);
            local_revision += 1;
            if let Ok(mut revision) = diag_revision.try_write() {
                *revision += 1;
            }
            push_event(
                events,
                json!({"kind": "slot-scope-mounted", "group": group,
                "instance_id": instance_id, "mount_count": stats.mount_count}),
            );
        }
    });

    use_drop({
        let ledger = ledger.clone();
        move || {
            if !committed.get() {
                return;
            }
            let stats = ledger.borrow_mut().record_drop(group);
            if let Ok(mut revision) = diag_revision.try_write() {
                *revision += 1;
            }
            push_event(
                events,
                json!({"kind": "slot-scope-dropped", "group": group,
                "instance_id": instance_id, "drop_count": stats.drop_count}),
            );
        }
    });

    let ledger_for_click = ledger.clone();
    let onaction = use_callback(move |_: MouseEvent| {
        let next = local_count() + 1;
        local_count.set(next);
        let updated = ledger_for_click
            .borrow_mut()
            .record_local_count(group, next);
        diag_revision += 1;
        push_event(
            events,
            json!({
                "kind": "slot-action",
                "group": group,
                "instance_id": instance_id,
                "local_count": next,
                "mount_count": updated.mount_count,
                "drop_count": updated.drop_count,
                "active_count": updated.active_count,
            }),
        );
    });

    if matches!(mode, GroupMode::EmptyComponent | GroupMode::Absent) {
        return rsx! {};
    }
    if mode == GroupMode::TextOnly {
        return rsx! { "{label} static text" };
    }

    let stats = ledger.borrow().stats(group);
    let count_val = local_count();
    let is_hidden = matches!(mode, GroupMode::Hidden);
    let is_disabled = matches!(mode, GroupMode::Disabled);

    let group_id = format!("header-layout-group-{group}");
    let status_id = format!("header-layout-{group}-status");
    let control_state = HeaderSlotControlState {
        instance_id,
        stats: stats.clone(),
        count: count_val,
        hidden: is_hidden,
        disabled: is_disabled,
    };
    let mut parts = vec!["input", "action"];
    if !inputs_present {
        parts.retain(|part| *part != "input");
    }
    if reverse_order {
        parts.reverse();
    }
    let mounted_ledger = ledger.clone();

    let group_class = if is_hidden {
        "header-layout-slot-group header-layout-slot-group--hidden"
    } else {
        "header-layout-slot-group"
    };
    rsx! {
        div {
            id: "{group_id}",
            class: group_class,
            hidden: is_hidden,
            "data-group": "{group}",
            "data-mode": "{mode.as_str()}",
            "data-instance-id": "{instance_id}",
            "data-mount-count": "{stats.mount_count}",
            "data-native-mount-count": "{stats.native_mount_count}",
            "data-drop-count": "{stats.drop_count}",
            "data-active-count": "{stats.active_count}",
            "data-local-count": "{count_val}",
            "data-order-reversed": "{reverse_order}",
            onmounted: move |_| {
                mounted_ledger.borrow_mut().stats_mut(group).native_mount_count += 1;
                local_revision += 1;
                if let Ok(mut revision) = diag_revision.try_write() { *revision += 1; }
                push_event(events, json!({"kind": "slot-native-mounted", "group": group, "instance_id": instance_id}));
            },
            for part in parts {
                HeaderSlotControl {
                    key: "{part}",
                    group,
                    label,
                    part,
                    state: control_state.clone(),
                    onclick: onaction,
                }
            }
            span {
                id: "{status_id}",
                class: "header-layout-slot-badge",
                "aria-hidden": "true",
                "m:{stats.mount_count}/d:{stats.drop_count}/c:{count_val}"
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
#[component]
fn HeaderLayoutStateLeaf(
    diag_revision: Signal<u64>,
    ledger: Rc<RefCell<SlotIdentityLedger>>,
    owner_epoch: Signal<usize>,
    container_mode: Signal<ContainerMode>,
    owner_mounted: Signal<bool>,
    header_mounted: Signal<bool>,
    menu_mounted: Signal<bool>,
    force_mode: Signal<ForceMode>,
    stage_width: Signal<String>,
    pinned_menu: Signal<bool>,
    show_menu: Signal<bool>,
    menu_expanded: Signal<bool>,
    sidebar_present: Signal<bool>,
    show_app_switch: Signal<bool>,
    config_mode: Signal<ConfigMode>,
    veto_switch: Signal<bool>,
    veto_overflow: Signal<bool>,
    veto_expand: Signal<bool>,
    enable_top_layer: Signal<bool>,
    hide_bottom_border: Signal<bool>,
    company_logo_present: Signal<bool>,
    app_image_present: Signal<bool>,
    logo_present: Signal<bool>,
    leading_present: Signal<bool>,
    avatar_present: Signal<bool>,
    default_mode: Signal<GroupMode>,
    secondary_mode: Signal<GroupMode>,
    overflow_mode: Signal<GroupMode>,
    reverse_group_order: Signal<bool>,
    inputs_present: Signal<bool>,
    outside_clicks: Signal<usize>,
    leading_clicks: Signal<usize>,
    menu_toggle_count: Signal<usize>,
    switch_request_count: Signal<usize>,
) -> Element {
    let _ = diag_revision();
    let ledger_snapshot = ledger.borrow().to_json();
    let state_snapshot = json!({
        "owner_epoch": owner_epoch(),
        "container_mode": container_mode().as_str(),
        "owner_mounted": owner_mounted(),
        "header_mounted": header_mounted(),
        "menu_mounted": menu_mounted(),
        "force_breakpoint": force_mode().as_str(),
        "stage_width": stage_width(),
        "pinned_menu": pinned_menu(),
        "show_menu": show_menu(),
        "menu_expanded": menu_expanded(),
        "sidebar_present": sidebar_present(),
        "show_app_switch": show_app_switch(),
        "config_mode": config_mode().as_str(),
        "veto_switch": veto_switch(),
        "veto_overflow": veto_overflow(),
        "veto_expand": veto_expand(),
        "enable_top_layer": enable_top_layer(),
        "hide_bottom_border": hide_bottom_border(),
        "company_logo_present": company_logo_present(),
        "app_image_present": app_image_present(),
        "logo_present": logo_present(),
        "leading_present": leading_present(),
        "avatar_present": avatar_present(),
        "default_mode": default_mode().as_str(),
        "secondary_mode": secondary_mode().as_str(),
        "overflow_mode": overflow_mode().as_str(),
        "reverse_group_order": reverse_group_order(),
        "inputs_present": inputs_present(),
        "slot_identity": ledger_snapshot,
        "outside_clicks": outside_clicks(),
        "avatar_clicks": ledger.borrow().stats("avatar").local_count,
        "leading_clicks": leading_clicks(),
        "menu_toggle_count": menu_toggle_count(),
        "switch_request_count": switch_request_count(),
    });
    let state_json =
        serde_json::to_string_pretty(&state_snapshot).unwrap_or_else(|_| "{}".to_string());

    rsx! {
        pre {
            id: "header-layout-state",
            class: "header-layout-json",
            "{state_json}"
        }
    }
}

#[component]
fn HeaderLayoutEventLeaf(events: Signal<Vec<Value>>) -> Element {
    let serialized = serde_json::to_string_pretty(&events()).unwrap_or_else(|_| "[]".to_string());
    rsx! {
        pre {
            id: "header-layout-events",
            class: "header-layout-json",
            "{serialized}"
        }
    }
}

#[component]
pub fn HeaderLayoutReview() -> Element {
    let initial = use_hook(load_initial_query_overrides);
    let identity_ledger = use_hook(|| Rc::new(RefCell::new(SlotIdentityLedger::default())));
    let diag_revision = use_signal(|| 0_u64);

    let mut owner_epoch = use_signal(|| 1_usize);
    let mut container_mode = use_signal(|| initial.container_mode);
    let mut owner_mounted = use_signal(|| true);
    let mut header_mounted = use_signal(|| true);
    let mut menu_mounted = use_signal(|| true);

    let mut force_mode = use_signal(|| initial.force_mode);
    let mut stage_width = use_signal(|| initial.stage_width.clone());
    let mut pinned_menu = use_signal(|| initial.pinned_menu);

    let mut show_menu = use_signal(|| initial.show_menu);
    let mut menu_expanded = use_signal(|| false);
    let mut sidebar_present = use_signal(|| initial.sidebar_present);
    let mut show_app_switch = use_signal(|| initial.show_app_switch);
    let mut config_mode = use_signal(|| initial.config_mode);
    let mut config_signal = use_signal(|| build_app_switch_config(initial.config_mode));
    let mut veto_switch = use_signal(|| initial.veto_switch);
    let mut veto_overflow = use_signal(|| false);
    let mut veto_expand = use_signal(|| false);

    let mut enable_top_layer = use_signal(|| initial.enable_top_layer);
    let mut hide_bottom_border = use_signal(|| initial.hide_bottom_border);
    let mut company_logo_present = use_signal(|| initial.company_logo_present);
    let mut app_image_present = use_signal(|| initial.app_image_present);
    let mut header_name = use_signal(|| "Operations Console".to_string());
    let mut header_suffix = use_signal(|| Some("B18.2".to_string()));
    let mut more_aria_label = use_signal(|| Some("More header actions".to_string()));
    let switch_aria_label = use_signal(|| Some("Open application switcher".to_string()));

    let mut default_mode = use_signal(|| initial.default_mode);
    let mut secondary_mode = use_signal(|| initial.secondary_mode);
    let mut overflow_mode = use_signal(|| initial.overflow_mode);
    let mut reverse_group_order = use_signal(|| initial.reverse_group_order);
    let mut inputs_present = use_signal(|| true);

    let mut avatar_present = use_signal(|| initial.avatar_present);
    let mut logo_present = use_signal(|| initial.logo_present);
    let mut leading_present = use_signal(|| initial.leading_present);

    let mut outside_clicks = use_signal(|| 0_usize);
    let mut leading_clicks = use_signal(|| 0_usize);
    let mut menu_toggle_count = use_signal(|| 0_usize);
    let mut switch_request_count = use_signal(|| 0_usize);

    let mut events = use_signal(Vec::<Value>::new);

    let mut apply_config_mode = move |next: ConfigMode| {
        config_mode.set(next);
        config_signal.set(build_app_switch_config(next));
        push_event(
            events,
            json!({
                "kind": "config-mode",
                "mode": next.as_str(),
            }),
        );
    };

    let render_slot_group =
        |group: &'static str, label: &'static str, mode: GroupMode| -> Option<Element> {
            match mode {
                GroupMode::Absent => None,
                _ => {
                    let ledger = identity_ledger.clone();
                    let reversed = reverse_group_order();
                    Some(rsx! {
                        HeaderSlotGroup {
                            key: "slot-group-{group}",
                            group,
                            label,
                            mode,
                            reverse_order: reversed,
                            inputs_present: inputs_present(),
                            ledger,
                            diag_revision,
                            events,
                        }
                    })
                }
            }
        };

    let default_slot = render_slot_group("default", "Primary", default_mode());
    let secondary_slot = render_slot_group("secondary", "Secondary", secondary_mode());
    let overflow_slot = render_slot_group("overflow", "Overflow", overflow_mode());

    let logo_slot = logo_present().then(|| {
        rsx! {
            div {
                id: "header-layout-slot-logo",
                class: "header-layout-custom-logo",
                "NORTHGRID"
            }
        }
    });

    let avatar_slot = if avatar_present() {
        render_slot_group("avatar", "Avatar", GroupMode::Active)
    } else {
        None
    };

    let leading_slot = leading_present().then(|| {
        rsx! {
            div {
                id: "header-layout-leading-content",
                class: "header-layout-leading",
                span {
                    id: "header-layout-leading-badge",
                    class: "header-layout-leading-badge",
                    "Sector 4"
                }
                button {
                    id: "header-layout-leading-action",
                    class: "header-layout-leading-button",
                    r#type: "button",
                    onclick: move |_| {
                        let next = *leading_clicks.peek() + 1;
                        leading_clicks.set(next);
                        push_event(
                            events,
                            json!({
                                "kind": "leading-click",
                                "count": next,
                            }),
                        );
                    },
                    "Context"
                }
            }
        }
    });

    let build_header_element = || -> Element {
        let company_logo = company_logo_present().then(|| COMPANY_LOGO_DATA_URL.to_string());
        let company_logo_alt =
            company_logo_present().then(|| "Northgrid platform mark".to_string());
        let app_icon_src = app_image_present().then(|| APP_ICON_DATA_URL.to_string());
        let app_icon_alt = app_image_present().then(|| "Operations Console icon".to_string());

        rsx! {
            ApplicationHeader {
                name: header_name(),
                name_suffix: header_suffix(),
                company_logo,
                company_logo_alt,
                app_icon_src,
                app_icon_alt,
                app_icon_outline: false,
                hide_bottom_border: hide_bottom_border(),
                show_menu,
                show_app_switch: show_app_switch(),
                enable_top_layer: enable_top_layer(),
                aria_label_app_switch_icon_button: switch_aria_label(),
                aria_label_more_menu_icon_button: more_aria_label(),
                logo: logo_slot.clone(),
                actions: default_slot.clone(),
                actions_present: default_mode().has_elements(),
                secondary: secondary_slot.clone(),
                secondary_present: secondary_mode().has_elements(),
                overflow: overflow_slot.clone(),
                overflow_present: overflow_mode().has_elements(),
                avatar: avatar_slot.clone(),
                avatar_present: avatar_present(),
                onbeforeoverflowchange: move |event: HeaderOverflowEvent| {
                    let vetoed = *veto_overflow.peek();
                    if vetoed { event.prevent_default(); }
                    push_event(events, json!({"kind": "overflow-request", "open": event.open,
                        "previous": event.previous, "vetoed": vetoed}));
                },
                onoverflowchange: move |open| push_event(events, json!({"kind": "overflow-accepted-state", "open": open})),
                onmenutoggle: move |next: bool| {
                    let next_count = *menu_toggle_count.peek() + 1;
                    menu_toggle_count.set(next_count);
                    push_event(
                        events,
                        json!({
                            "kind": "header-menu-toggle",
                            "open": next,
                            "count": next_count,
                        }),
                    );
                },
                onopenappswitch: move |evt: MouseEvent| {
                    let next_count = *switch_request_count.peek() + 1;
                    switch_request_count.set(next_count);
                    let vetoed = *veto_switch.peek();
                    if vetoed {
                        evt.prevent_default();
                    }
                    push_event(
                        events,
                        json!({
                            "kind": "header-open-app-switch",
                            "vetoed": vetoed,
                            "count": next_count,
                        }),
                    );
                },
                if let Some(leading) = leading_slot.clone() {
                    {leading}
                }
            }
        }
    };

    let current_epoch = owner_epoch();

    rsx! {
        section {
            id: "header-layout-review",
            class: "demo-section header-layout-review",
            style { {REVIEW_STYLES} }

            header {
                class: "header-layout-banner",
                div {
                    class: "header-layout-banner-copy",
                    h2 { "B18.2 ApplicationHeader Responsive Slots & Single-Tree Identity" }
                    p {
                        "Inspect native inputs, component-local counts and committed lifetimes across layout and popup changes. Presentation transitions should preserve one instance; explicit removal is intentional teardown."
                    }
                }
                div {
                    class: "header-layout-banner-actions",
                    button {
                        id: "header-layout-outside-action",
                        class: "header-layout-btn",
                        r#type: "button",
                        onclick: move |_| {
                            let next = *outside_clicks.peek() + 1;
                            outside_clicks.set(next);
                            push_event(
                                events,
                                json!({
                                    "kind": "outside-click",
                                    "count": next,
                                }),
                            );
                        },
                        "Outside target ({outside_clicks()})"
                    }
                    button {
                        id: "header-layout-clear-events",
                        class: "header-layout-btn",
                        r#type: "button",
                        onclick: move |_| {
                            events.set(Vec::new());
                        },
                        "Clear events"
                    }
                    button {
                        id: "header-layout-reset",
                        class: "header-layout-btn header-layout-btn--accent",
                        r#type: "button",
                        onclick: {
                            move |_| {
                                owner_epoch += 1;
                                container_mode.set(ContainerMode::Contained);
                                owner_mounted.set(true);
                                header_mounted.set(true);
                                menu_mounted.set(true);
                                force_mode.set(ForceMode::Auto);
                                stage_width.set("100%".to_string());
                                pinned_menu.set(false);
                                show_menu.set(false);
                                menu_expanded.set(false);
                                sidebar_present.set(false);
                                show_app_switch.set(false);
                                config_mode.set(ConfigMode::None);
                                config_signal.set(None);
                                veto_switch.set(true);
                                veto_overflow.set(false);
                                veto_expand.set(false);
                                enable_top_layer.set(false);
                                hide_bottom_border.set(false);
                                company_logo_present.set(false);
                                app_image_present.set(true);
                                default_mode.set(GroupMode::Active);
                                secondary_mode.set(GroupMode::Active);
                                overflow_mode.set(GroupMode::Active);
                                reverse_group_order.set(false);
                                inputs_present.set(true);
                                avatar_present.set(true);
                                logo_present.set(true);
                                leading_present.set(true);
                                outside_clicks.set(0);
                                leading_clicks.set(0);
                                menu_toggle_count.set(0);
                                switch_request_count.set(0);
                                events.set(Vec::new());
                            }
                        },
                        "Fresh owner reset"
                    }
                }
            }

            div {
                id: "header-layout-primary-controls",
                class: "header-layout-control-grid",

                div {
                    class: "header-layout-control-card",
                    span { class: "header-layout-card-label", "Breakpoint & Width" }
                    div {
                        class: "header-layout-btn-row",
                        button {
                            id: "header-layout-force-auto",
                            class: if force_mode() == ForceMode::Auto { "header-layout-btn header-layout-btn--active" } else { "header-layout-btn" },
                            r#type: "button",
                            onclick: move |_| force_mode.set(ForceMode::Auto),
                            "Auto"
                        }
                        button {
                            id: "header-layout-force-sm",
                            class: if force_mode() == ForceMode::Small { "header-layout-btn header-layout-btn--active" } else { "header-layout-btn" },
                            r#type: "button",
                            onclick: move |_| force_mode.set(ForceMode::Small),
                            "SM"
                        }
                        button {
                            id: "header-layout-force-md",
                            class: if force_mode() == ForceMode::Medium { "header-layout-btn header-layout-btn--active" } else { "header-layout-btn" },
                            r#type: "button",
                            onclick: move |_| force_mode.set(ForceMode::Medium),
                            "MD"
                        }
                        button {
                            id: "header-layout-force-lg",
                            class: if force_mode() == ForceMode::Large { "header-layout-btn header-layout-btn--active" } else { "header-layout-btn" },
                            r#type: "button",
                            onclick: move |_| force_mode.set(ForceMode::Large),
                            "LG"
                        }
                        select {
                            id: "header-layout-force",
                            class: "header-layout-select",
                            aria_label: "Force breakpoint",
                            value: "{force_mode().as_str()}",
                            onchange: move |evt| {
                                if let Some(parsed) = ForceMode::from_query(&evt.value()) {
                                    force_mode.set(parsed);
                                }
                            },
                            option { value: "auto", "auto (responsive)" }
                            option { value: "sm", "sm (small)" }
                            option { value: "md", "md (medium)" }
                            option { value: "lg", "lg (large)" }
                        }
                    }
                    div {
                        class: "header-layout-btn-row",
                        button {
                            id: "header-layout-width-full",
                            class: if stage_width() == "100%" { "header-layout-btn header-layout-btn--active" } else { "header-layout-btn" },
                            r#type: "button",
                            onclick: move |_| stage_width.set("100%".to_string()),
                            "100%"
                        }
                        button {
                            id: "header-layout-width-360",
                            class: if stage_width() == "360px" { "header-layout-btn header-layout-btn--active" } else { "header-layout-btn" },
                            r#type: "button",
                            onclick: move |_| stage_width.set("360px".to_string()),
                            "360px"
                        }
                        button {
                            id: "header-layout-width-600",
                            class: if stage_width() == "600px" { "header-layout-btn header-layout-btn--active" } else { "header-layout-btn" },
                            r#type: "button",
                            onclick: move |_| stage_width.set("600px".to_string()),
                            "600px"
                        }
                        button {
                            id: "header-layout-width-900",
                            class: if stage_width() == "900px" { "header-layout-btn header-layout-btn--active" } else { "header-layout-btn" },
                            r#type: "button",
                            onclick: move |_| stage_width.set("900px".to_string()),
                            "900px"
                        }
                        select {
                            id: "header-layout-width",
                            class: "header-layout-select",
                            aria_label: "Stage width",
                            value: "{stage_width()}",
                            onchange: move |evt| stage_width.set(evt.value()),
                            option { value: "100%", "Stage: 100%" }
                            option { value: "360px", "Stage: 360px" }
                            option { value: "600px", "Stage: 600px" }
                            option { value: "900px", "Stage: 900px" }
                        }
                    }
                }

                div {
                    class: "header-layout-control-card",
                    span { class: "header-layout-card-label", "Slot Presence & Top Layer" }
                    div {
                        class: "header-layout-btn-row",
                        button {
                            id: "header-layout-toggle-default",
                            class: if default_mode().is_present() { "header-layout-btn header-layout-btn--active" } else { "header-layout-btn" },
                            r#type: "button",
                            onclick: move |_| {
                                let next = if default_mode().is_present() { GroupMode::Absent } else { GroupMode::Active };
                                default_mode.set(next);
                            },
                            "Default: {default_mode().as_str()}"
                        }
                        button {
                            id: "header-layout-toggle-secondary",
                            class: if secondary_mode().is_present() { "header-layout-btn header-layout-btn--active" } else { "header-layout-btn" },
                            r#type: "button",
                            onclick: move |_| {
                                let next = if secondary_mode().is_present() { GroupMode::Absent } else { GroupMode::Active };
                                secondary_mode.set(next);
                            },
                            "Secondary: {secondary_mode().as_str()}"
                        }
                        button {
                            id: "header-layout-toggle-overflow",
                            class: if overflow_mode().is_present() { "header-layout-btn header-layout-btn--active" } else { "header-layout-btn" },
                            r#type: "button",
                            onclick: move |_| {
                                let next = if overflow_mode().is_present() { GroupMode::Absent } else { GroupMode::Active };
                                overflow_mode.set(next);
                            },
                            "Overflow: {overflow_mode().as_str()}"
                        }
                        button {
                            id: "header-layout-top-layer",
                            class: if enable_top_layer() { "header-layout-btn header-layout-btn--active" } else { "header-layout-btn" },
                            r#type: "button",
                            onclick: move |_| enable_top_layer.set(!enable_top_layer()),
                            "Top layer: {enable_top_layer()}"
                        }
                    }
                    div {
                        class: "header-layout-btn-row",
                        button {
                            id: "header-layout-toggle-avatar",
                            class: if avatar_present() { "header-layout-btn header-layout-btn--active" } else { "header-layout-btn" },
                            r#type: "button",
                            onclick: move |_| avatar_present.set(!avatar_present()),
                            "Avatar: {avatar_present()}"
                        }
                        button {
                            id: "header-layout-toggle-logo",
                            class: if logo_present() { "header-layout-btn header-layout-btn--active" } else { "header-layout-btn" },
                            r#type: "button",
                            onclick: move |_| logo_present.set(!logo_present()),
                            "Logo: {logo_present()}"
                        }
                        button {
                            id: "header-layout-toggle-leading",
                            class: if leading_present() { "header-layout-btn header-layout-btn--active" } else { "header-layout-btn" },
                            r#type: "button",
                            onclick: move |_| leading_present.set(!leading_present()),
                            "Leading: {leading_present()}"
                        }
                        button {
                            id: "header-layout-reverse-group-order",
                            class: if reverse_group_order() { "header-layout-btn header-layout-btn--active" } else { "header-layout-btn" },
                            r#type: "button",
                            onclick: move |_| reverse_group_order.set(!reverse_group_order()),
                            "Reverse group order: {reverse_group_order()}"
                        }
                    }
                }

                div {
                    class: "header-layout-control-card",
                    span { class: "header-layout-card-label", "Context, Mount & Menu" }
                    div {
                        class: "header-layout-btn-row",
                        button {
                            id: "header-layout-container-mode",
                            class: "header-layout-btn header-layout-btn--active",
                            r#type: "button",
                            onclick: move |_| {
                                let next = match container_mode() {
                                    ContainerMode::Contained => ContainerMode::Standalone,
                                    ContainerMode::Standalone => ContainerMode::Contained,
                                };
                                container_mode.set(next);
                            },
                            "Mode: {container_mode().as_str()}"
                        }
                        button {
                            id: "header-layout-mode-contained",
                            class: if container_mode() == ContainerMode::Contained { "header-layout-btn header-layout-btn--active" } else { "header-layout-btn" },
                            r#type: "button",
                            onclick: move |_| container_mode.set(ContainerMode::Contained),
                            "Contained"
                        }
                        button {
                            id: "header-layout-mode-standalone",
                            class: if container_mode() == ContainerMode::Standalone { "header-layout-btn header-layout-btn--active" } else { "header-layout-btn" },
                            r#type: "button",
                            onclick: move |_| container_mode.set(ContainerMode::Standalone),
                            "Standalone"
                        }
                        button {
                            id: "header-layout-owner-mount",
                            class: if owner_mounted() { "header-layout-btn header-layout-btn--active" } else { "header-layout-btn" },
                            r#type: "button",
                            onclick: move |_| owner_mounted.set(!owner_mounted()),
                            "Owner: {owner_mounted()}"
                        }
                        button {
                            id: "header-layout-header-mount",
                            class: if header_mounted() { "header-layout-btn header-layout-btn--active" } else { "header-layout-btn" },
                            r#type: "button",
                            onclick: move |_| header_mounted.set(!header_mounted()),
                            "Header: {header_mounted()}"
                        }
                        button {
                            id: "header-layout-menu-mount",
                            class: if menu_mounted() { "header-layout-btn header-layout-btn--active" } else { "header-layout-btn" },
                            r#type: "button",
                            onclick: move |_| menu_mounted.set(!menu_mounted()),
                            "Menu: {menu_mounted()}"
                        }
                    }
                    div {
                        class: "header-layout-btn-row",
                        button {
                            id: "header-layout-show-menu",
                            class: if show_menu() { "header-layout-btn header-layout-btn--active" } else { "header-layout-btn" },
                            r#type: "button",
                            onclick: move |_| show_menu.set(!show_menu()),
                            "show_menu: {show_menu()}"
                        }
                        button {
                            id: "header-layout-sidebar",
                            class: if sidebar_present() { "header-layout-btn header-layout-btn--active" } else { "header-layout-btn" },
                            r#type: "button",
                            onclick: move |_| sidebar_present.set(!sidebar_present()),
                            "Sidebar: {sidebar_present()}"
                        }
                        button {
                            id: "header-layout-show-app-switch",
                            class: if show_app_switch() { "header-layout-btn header-layout-btn--active" } else { "header-layout-btn" },
                            r#type: "button",
                            onclick: move |_| show_app_switch.set(!show_app_switch()),
                            "show_app_switch: {show_app_switch()}"
                        }
                        button {
                            id: "header-layout-config-none",
                            class: if config_mode() == ConfigMode::None { "header-layout-btn header-layout-btn--active" } else { "header-layout-btn" },
                            r#type: "button",
                            onclick: move |_| apply_config_mode(ConfigMode::None),
                            "Cfg: None"
                        }
                        button {
                            id: "header-layout-config-loading",
                            class: if config_mode() == ConfigMode::Loading { "header-layout-btn header-layout-btn--active" } else { "header-layout-btn" },
                            r#type: "button",
                            onclick: move |_| apply_config_mode(ConfigMode::Loading),
                            "Cfg: Loading"
                        }
                        button {
                            id: "header-layout-config-populated",
                            class: if config_mode() == ConfigMode::PopulatedA { "header-layout-btn header-layout-btn--active" } else { "header-layout-btn" },
                            r#type: "button",
                            onclick: move |_| apply_config_mode(ConfigMode::PopulatedA),
                            "Cfg: Populated"
                        }
                        button {
                            id: "header-layout-config-replace",
                            class: if config_mode() == ConfigMode::PopulatedB { "header-layout-btn header-layout-btn--active" } else { "header-layout-btn" },
                            r#type: "button",
                            onclick: move |_| apply_config_mode(ConfigMode::PopulatedB),
                            "Cfg: Replace"
                        }
                    }
                }
            }

            details {
                id: "header-layout-advanced-details",
                class: "header-layout-details",
                open: true,
                summary {
                    id: "header-layout-advanced-summary",
                    class: "header-layout-summary",
                    "Advanced Slot Modes, Chrome Precedence & Context Controls"
                }
                div {
                    class: "header-layout-advanced-body",
                    div {
                        class: "header-layout-btn-row",
                        label {
                            class: "header-layout-field",
                            span { "Default group mode" }
                            select {
                                id: "header-layout-default-mode",
                                class: "header-layout-select",
                                value: "{default_mode().as_str()}",
                                onchange: move |evt| {
                                    if let Some(parsed) = GroupMode::from_query(&evt.value()) {
                                        default_mode.set(parsed);
                                    }
                                },
                                option { value: "active", "active (stateful)" }
                                option { value: "absent", "absent (None)" }
                                option { value: "empty-component", "empty-component" }
                                option { value: "text-only", "text-only" }
                                option { value: "hidden", "hidden" }
                                option { value: "disabled", "disabled" }
                            }
                        }
                        label {
                            class: "header-layout-field",
                            span { "Secondary group mode" }
                            select {
                                id: "header-layout-secondary-mode",
                                class: "header-layout-select",
                                value: "{secondary_mode().as_str()}",
                                onchange: move |evt| {
                                    if let Some(parsed) = GroupMode::from_query(&evt.value()) {
                                        secondary_mode.set(parsed);
                                    }
                                },
                                option { value: "active", "active (stateful)" }
                                option { value: "absent", "absent (None)" }
                                option { value: "empty-component", "empty-component" }
                                option { value: "text-only", "text-only" }
                                option { value: "hidden", "hidden" }
                                option { value: "disabled", "disabled" }
                            }
                        }
                        label {
                            class: "header-layout-field",
                            span { "Overflow group mode" }
                            select {
                                id: "header-layout-overflow-mode",
                                class: "header-layout-select",
                                value: "{overflow_mode().as_str()}",
                                onchange: move |evt| {
                                    if let Some(parsed) = GroupMode::from_query(&evt.value()) {
                                        overflow_mode.set(parsed);
                                    }
                                },
                                option { value: "active", "active (stateful)" }
                                option { value: "absent", "absent (None)" }
                                option { value: "empty-component", "empty-component" }
                                option { value: "text-only", "text-only" }
                                option { value: "hidden", "hidden" }
                                option { value: "disabled", "disabled" }
                            }
                        }
                    }
                    div {
                        class: "header-layout-btn-row",
                        button {
                            id: "header-layout-keyed-inputs",
                            class: "header-layout-btn",
                            r#type: "button",
                            onclick: move |_| inputs_present.toggle(),
                            if inputs_present() { "Remove keyed inputs" } else { "Restore keyed inputs" }
                        }
                        button {
                            id: "header-layout-company-logo",
                            class: if company_logo_present() { "header-layout-btn header-layout-btn--active" } else { "header-layout-btn" },
                            r#type: "button",
                            onclick: move |_| company_logo_present.set(!company_logo_present()),
                            "company_logo: {company_logo_present()}"
                        }
                        button {
                            id: "header-layout-app-image",
                            class: if app_image_present() { "header-layout-btn header-layout-btn--active" } else { "header-layout-btn" },
                            r#type: "button",
                            onclick: move |_| app_image_present.set(!app_image_present()),
                            "app_icon_src: {app_image_present()}"
                        }
                        button {
                            id: "header-layout-borderless",
                            class: if hide_bottom_border() { "header-layout-btn header-layout-btn--active" } else { "header-layout-btn" },
                            r#type: "button",
                            onclick: move |_| hide_bottom_border.set(!hide_bottom_border()),
                            "hide_bottom_border: {hide_bottom_border()}"
                        }
                        button {
                            id: "header-layout-pinned-menu",
                            class: if pinned_menu() { "header-layout-btn header-layout-btn--active" } else { "header-layout-btn" },
                            r#type: "button",
                            onclick: move |_| pinned_menu.set(!pinned_menu()),
                            "pinned_menu: {pinned_menu()}"
                        }
                        button {
                            id: "header-layout-veto-overflow",
                            class: if veto_overflow() { "header-layout-btn header-layout-btn--active" } else { "header-layout-btn" },
                            r#type: "button", onclick: move |_| veto_overflow.toggle(),
                            "Veto overflow requests: {veto_overflow()}"
                        }
                        button {
                            id: "header-layout-veto-expand",
                            class: if veto_expand() { "header-layout-btn header-layout-btn--active" } else { "header-layout-btn" },
                            r#type: "button", onclick: move |_| veto_expand.toggle(),
                            "Veto menu expansion: {veto_expand()}"
                        }
                        button {
                            id: "header-layout-veto-switch",
                            class: if veto_switch() { "header-layout-btn header-layout-btn--active" } else { "header-layout-btn" },
                            r#type: "button",
                            onclick: move |_| veto_switch.set(!veto_switch()),
                            "Veto switch modal: {veto_switch()}"
                        }
                        button {
                            id: "header-layout-suffix-toggle",
                            class: if header_suffix().is_some() { "header-layout-btn header-layout-btn--active" } else { "header-layout-btn" },
                            r#type: "button",
                            onclick: move |_| {
                                if header_suffix().is_some() {
                                    header_suffix.set(None);
                                } else {
                                    header_suffix.set(Some("B18.2".to_string()));
                                }
                            },
                            "Suffix: {header_suffix().is_some()}"
                        }
                        button {
                            id: "header-layout-more-aria-toggle",
                            class: "header-layout-btn",
                            r#type: "button",
                            onclick: move |_| {
                                let next = match more_aria_label().as_deref() {
                                    Some("More header actions") => Some("Additional header tools".to_string()),
                                    _ => Some("More header actions".to_string()),
                                };
                                more_aria_label.set(next);
                            },
                            "Cycle More ARIA"
                        }
                        button {
                            id: "header-layout-name-cycle",
                            class: "header-layout-btn",
                            r#type: "button",
                            onclick: move |_| {
                                let next = if header_name() == "Operations Console" {
                                    "Substation Telemetry Supervision Matrix".to_string()
                                } else {
                                    "Operations Console".to_string()
                                };
                                header_name.set(next);
                            },
                            "Cycle header name"
                        }
                    }
                }
            }

            div {
                id: "header-layout-stage-shell",
                class: "header-layout-stage-shell",
                style: "width: {stage_width()}; max-width: 100%;",
                "data-container-mode": "{container_mode().as_str()}",
                "data-force-breakpoint": "{force_mode().as_str()}",
                "data-stage-width": "{stage_width()}",

                if owner_mounted() {
                    div {
                        key: "header-layout-owner-{current_epoch}",
                        id: "header-layout-owner-host",
                        class: "header-layout-owner-host",
                        match container_mode() {
                            ContainerMode::Contained => rsx! {
                                Application {
                                    class: "header-layout-application".to_string(),
                                    force_breakpoint: force_mode().to_breakpoint(),
                                    app_switch_config: config_signal,
                                    menu_expanded: Some(menu_expanded),
                                    start_expanded: false,
                                    pinned_menu: pinned_menu(),
                                    sidebar: sidebar_present().then(|| rsx! {
                                        aside {
                                            id: "header-layout-sidebar-panel",
                                            class: "header-layout-sidebar-panel",
                                            "Sidebar context active"
                                        }
                                    }),
                                    header: header_mounted().then(build_header_element),
                                    menu: menu_mounted().then(|| rsx! {
                                        Menu {
                                            application_name: Some("Operations Console".to_string()),
                                            label: "Operations navigation".to_string(),
                                            pinned: pinned_menu(),
                                            start_expanded: false,
                                            onbeforeexpandchange: move |event: MenuExpandEvent| {
                                                let vetoed = *veto_expand.peek();
                                                if vetoed { event.prevent_default(); }
                                                push_event(events, json!({"kind": "menu-expand-request", "expanded": event.expanded, "previous": event.previous, "vetoed": vetoed}));
                                            },
                                            onexpandchange: move |next: bool| {
                                                push_event(
                                                    events,
                                                    json!({
                                                        "kind": "menu-expand-change",
                                                        "expanded": next,
                                                    }),
                                                );
                                            },
                                            MenuItem {
                                                label: "Overview".to_string(),
                                                icon: "home",
                                                active: true,
                                            }
                                            MenuItem {
                                                label: "Diagnostics".to_string(),
                                                icon: "info",
                                            }
                                        }
                                    }),
                                    div {
                                        id: "header-layout-workspace",
                                        class: "header-layout-workspace",
                                        p {
                                            id: "header-layout-workspace-copy",
                                            "Contained Application workspace body (menu_expanded={menu_expanded()}, force={force_mode().as_str()})."
                                        }
                                    }
                                }
                            },
                            ContainerMode::Standalone => rsx! {
                                div {
                                    id: "header-layout-standalone-host",
                                    class: "header-layout-standalone-host",
                                    if header_mounted() {
                                        {build_header_element()}
                                    }
                                    div {
                                        id: "header-layout-workspace",
                                        class: "header-layout-workspace",
                                        p {
                                            id: "header-layout-workspace-copy",
                                            "Standalone ApplicationHeader host (viewport media breakpoint authority)."
                                        }
                                    }
                                }
                            },
                        }
                    }
                } else {
                    div {
                        id: "header-layout-unmounted-placeholder",
                        class: "header-layout-unmounted",
                        "Owner unmounted"
                    }
                }
            }

            div {
                class: "header-layout-diagnostics-grid",
                div {
                    class: "header-layout-diag-panel",
                    h3 { "State Snapshot (#header-layout-state)" }
                    HeaderLayoutStateLeaf {
                        diag_revision,
                        ledger: identity_ledger.clone(),
                        owner_epoch,
                        container_mode,
                        owner_mounted,
                        header_mounted,
                        menu_mounted,
                        force_mode,
                        stage_width,
                        pinned_menu,
                        show_menu,
                        menu_expanded,
                        sidebar_present,
                        show_app_switch,
                        config_mode,
                        veto_switch,
                        veto_overflow,
                        veto_expand,
                        enable_top_layer,
                        hide_bottom_border,
                        company_logo_present,
                        app_image_present,
                        logo_present,
                        leading_present,
                        avatar_present,
                        default_mode,
                        secondary_mode,
                        overflow_mode,
                        reverse_group_order,
                        inputs_present,
                        outside_clicks,
                        leading_clicks,
                        menu_toggle_count,
                        switch_request_count,
                    }
                }
                div {
                    class: "header-layout-diag-panel",
                    h3 { "Event Leaf (#header-layout-events)" }
                    HeaderLayoutEventLeaf { events }
                }
            }
        }
    }
}

const REVIEW_STYLES: &str = include_str!("styles/header_layout_review.css");
