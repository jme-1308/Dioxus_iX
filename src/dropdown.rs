//! Persistent Dropdown primitives and trigger-linked overlay ownership.
//!
//! Copyright (c) 2023-2026 Siemens AG. SPDX-License-Identifier: MIT.
use crate::Icon;
use crate::dropdown_inventory::{
    Activation, HostFacts, HostPartFacts, HostRootFact, Inventory, InventoryFacts, InventoryMode,
    ItemKind, ItemRef,
};
use crate::overlay_registry::{CloseBehavior, OverlayKind};
use crate::overlay_runtime::{self, Callbacks, Change, KeyDecision, KeyFacts, NodeHandle, Reason};
use dioxus::prelude::*;
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    rc::Rc,
    sync::atomic::{AtomicUsize, Ordering},
};

static NEXT_ID: AtomicUsize = AtomicUsize::new(1);

static NEXT_SOURCE_ITEM: AtomicUsize = AtomicUsize::new(1);

/// Explicit Dioxus composition owns item membership and declared sibling order.
/// Registered parent scopes describe nesting without inspecting rendered DOM.
#[derive(Clone)]
pub(crate) struct ItemInventoryOwner(pub(crate) NodeHandle);

/// Placement follows explicit Dioxus item composition, including controls
/// rendered by consumer components inside an item. This scope renders no DOM.
#[derive(Clone, Copy)]
struct DropdownItemContentScope;

#[component]
fn DropdownItemContent(children: Element, parent_key: String) -> Element {
    use_context_provider(|| DropdownItemContentScope);
    rsx! { ItemOrderContent { parent_key, {children} } }
}

#[derive(Clone)]
struct ItemOrderParent {
    key: String,
    owner: Option<(String, u64)>,
}
/// Rust parent-item scope; this component adds no rendered element. A nested
/// Dropdown establishes a different owner, so its items cannot inherit this key.
#[component]
pub(crate) fn ItemOrderContent(children: Element, parent_key: String) -> Element {
    let owner = try_use_context::<ItemInventoryOwner>()
        .map(|owner| (owner.0.id().to_string(), owner.0.generation()));
    use_context_provider(|| ItemOrderParent {
        key: parent_key,
        owner,
    });
    rsx! {{children}}
}

pub(crate) struct ItemDescription {
    pub(crate) disabled: bool,
    pub(crate) hidden: bool,
    pub(crate) checked: bool,
    pub(crate) order: Option<usize>,
    pub(crate) source_focusable: bool,
    pub(crate) linked_trigger: Option<String>,
    pub(crate) parts: Vec<HostPartFacts>,
}
impl Default for ItemDescription {
    fn default() -> Self {
        Self {
            disabled: false,
            hidden: false,
            checked: false,
            order: None,
            source_focusable: true,
            linked_trigger: None,
            parts: Vec::new(),
        }
    }
}
#[derive(Clone)]
pub(crate) struct ItemHostBinding {
    pub(crate) key: String,
    pub(crate) host_id: String,
    pub(crate) generic: bool,
    pub(crate) runtime: overlay_runtime::RuntimeHandle,
    context: Option<DropdownContext>,
    inventory_owner: Option<NodeHandle>,
    parent_key: Option<String>,
    header_control: Option<crate::header_control_scope::HeaderControlRegistration>,
    popover_scope: Option<overlay_runtime::PopoverFocusScope>,
    identity: usize,
    kind: ItemKind,
}
impl ItemHostBinding {
    pub(crate) fn mount(&self, target: &str, node: Rc<MountedData>) {
        self.mount_control(target, target, node);
    }
    pub(crate) fn mount_control(&self, target: &str, focus_id: &str, node: Rc<MountedData>) {
        if let Some(control) = &self.header_control {
            control.update(focus_id);
        }
        let owner = self
            .inventory_owner
            .as_ref()
            .map(|node| (node.id().to_string(), node.generation()));
        self.runtime
            .register_owned_item_control_mount(&self.key, target, focus_id, node, owner);
        self.runtime
            .set_item_popover_scope(&self.key, self.popover_scope.clone());
    }
    pub(crate) fn describe(
        &self,
        target: &str,
        attributes: &[Attribute],
        options: ItemDescription,
    ) {
        let owner = self
            .inventory_owner
            .as_ref()
            .map(|node| (node.id().to_string(), node.generation()));
        let root_path = owner
            .as_ref()
            .map(|(id, generation)| HostRootFact {
                id: id.clone(),
                generation: *generation,
                is_host: false,
            })
            .into_iter()
            .collect();
        let host = HostFacts {
            item: ItemRef(self.key.clone()),
            kind: self.kind,
            target: target.into(),
            capture: self.key.clone(),
            root_path,
            registration_key: self.key.clone(),
            source_focusable: options.source_focusable,
            disabled_attribute: options.disabled || item_attribute_present(attributes, "disabled"),
            hidden_attribute: options.hidden || item_attribute_present(attributes, "hidden"),
            tabindex_attribute: item_attribute_value(attributes, "tabindex"),
            checked_attribute: options.checked || checked_content_attribute(attributes).is_some(),
            visible_focus: false,
            roving_opt_in: self.kind == ItemKind::NativeOptIn,
            linked_trigger: options.linked_trigger,
            parts: options.parts,
        };
        self.runtime.register_owned_item_description(
            &self.key,
            owner,
            host,
            overlay_runtime::ItemOrder {
                position: options.order,
                parent: self.parent_key.clone(),
            },
        );
    }
    pub(crate) fn focus(
        &self,
        source_focusable: bool,
        part: Option<crate::dropdown_inventory::HostPartKind>,
    ) {
        self.runtime
            .focus_registered_item(&self.key, source_focusable, part);
    }
    pub(crate) fn blur(&self) {
        self.runtime.blur_registered_item(&self.key);
    }
    pub(crate) fn active(&self) -> bool {
        self.runtime.item_visual_key().as_deref() == Some(self.key.as_str())
    }
    pub(crate) fn tabindex(&self) -> Option<i32> {
        self.runtime.item_tabindex(&self.key)
    }
    pub(crate) fn set_keyboard_action(&self, target: &str, action: EventHandler<KeyboardEvent>) {
        if let Some(context) = &self.context {
            context
                .actions
                .borrow_mut()
                .insert(self.key.clone(), (target.to_string(), action));
        }
    }
    pub(crate) fn navigation_key(&self, event: KeyboardEvent) -> bool {
        if !event.default_action_enabled() {
            return false;
        }
        let Some(context) = &self.context else {
            return false;
        };
        {
            let state = context.state.borrow();
            if !state.alive
                || !state.surface
                || !state.mounted
                || state.blocked
                || state.settings.disable_focus_handling
                || !*context.open.peek()
                || *context.blocked.peek()
            {
                return false;
            }
        }
        let navigation = matches!(
            event.key(),
            Key::Home
                | Key::End
                | Key::ArrowUp
                | Key::ArrowDown
                | Key::ArrowLeft
                | Key::ArrowRight
                | Key::Tab
                | Key::Escape
                | Key::Enter
        ) || event.key() == Key::Character(" ".into())
            || context.state.borrow().settings.mode == DropdownNavigationMode::RovingTabindex
                && event.key().to_string().chars().count() == 1;
        if !navigation {
            return false;
        }
        let Some(node) = context
            .node
            .borrow()
            .clone()
            .filter(NodeHandle::is_registered)
        else {
            return false;
        };
        let inventory = node.inventory();
        let host = inventory.as_ref().and_then(|inventory| {
            inventory.hosts.iter().find(|host| {
                host.registration_key == self.key
                    && host.kind != ItemKind::NativeOptIn
                    && host.eligible(context.state.borrow().settings.mode.inventory_mode())
                    && node.is_item_registered(host)
            })
        });
        // A native field keeps editing defaults; only explicitly typed items
        // enter item navigation. No arbitrary DOM fallback is installed.
        if inventory.is_some() && host.is_none() {
            return false;
        }
        let target = host.map(|host| host.target.clone()).or_else(|| {
            context
                .actions
                .borrow()
                .get(&self.key)
                .map(|(target, _)| target.clone())
        });
        let activation =
            matches!(event.key(), Key::Enter) || event.key() == Key::Character(" ".into());
        if activation && !context.actions.borrow().contains_key(&self.key) {
            return false;
        }
        direct_key(context, event.clone(), false, target, true);
        // This item already dispatched into its owning menu. A second ancestor
        // Dioxus handler must not perform the same action again.
        event.stop_propagation();
        true
    }
}
pub(crate) fn use_item_host(kind: &'static str) -> ItemHostBinding {
    let runtime = overlay_runtime::use_overlay_runtime();
    let registry = runtime.clone();
    let kind_value = match kind {
        "dropdown" => ItemKind::Dropdown,
        "select" => ItemKind::Select,
        "menu" => ItemKind::Menu,
        "category" => ItemKind::Category,
        "native-opt-in" => ItemKind::NativeOptIn,
        _ => unreachable!("typed item kind"),
    };
    let (identity, key) = use_hook(move || {
        let identity = NEXT_SOURCE_ITEM.fetch_add(1, Ordering::Relaxed);
        let key = format!("ix-source-{kind}-{identity}");
        registry.register_item_cue(key.clone(), kind_value);
        (identity, key)
    });
    let header_control = crate::header_control_scope::use_header_control_registration(key.clone());
    // This metadata follows the mounted control's Dioxus composition. A nested
    // Popover replaces the nearest scope rather than inheriting an outer header.
    let popover_scope = use_hook(try_consume_context::<overlay_runtime::PopoverFocusScope>);
    let cleanup = runtime.clone();
    let dropped = key.clone();
    use_drop(move || cleanup.unregister_item_cue(&dropped));
    let inventory_owner = try_use_context::<ItemInventoryOwner>().map(|owner| owner.0);
    let declared_parent = use_hook(try_consume_context::<ItemOrderParent>);
    let current_owner = inventory_owner
        .as_ref()
        .map(|owner| (owner.id().to_string(), owner.generation()));
    let parent_key = declared_parent
        .filter(|parent| parent.owner == current_owner)
        .map(|parent| parent.key);
    let context = try_use_context::<DropdownContext>().filter(|context| {
        inventory_owner
            .as_ref()
            .is_some_and(|owner| owner.id() == context.node_id())
    });
    let action_owner = context.clone();
    let action_key = key.clone();
    use_drop(move || {
        if let Some(context) = action_owner {
            context.actions.borrow_mut().remove(&action_key);
        }
    });
    ItemHostBinding {
        host_id: key.clone(),
        key,
        generic: context.is_some(),
        runtime,
        context,
        inventory_owner,
        parent_key,
        header_control,
        popover_scope,
        identity,
        kind: kind_value,
    }
}

/// Register a custom Dioxus control as a programmatic focus destination.
///
/// Give the control this stable DOM `id` and attach the returned handler to its
/// `onmounted` event. Registration is released with the calling component's
/// scope, so call the hook in a component that unmounts with the control. Use a
/// new component key when replacing it. Unregistered DOM controls are not
/// resolved through JavaScript.
pub fn use_focus_mount(id: impl Into<String>) -> EventHandler<MountedEvent> {
    let binding = use_item_host("native-opt-in");
    let id = id.into();
    use_callback(move |event: MountedEvent| binding.mount(&id, event.data()))
}

/// Dioxus-owned focus registration and tab order for a custom Dropdown item.
#[derive(Clone, Copy)]
pub struct DropdownItemBinding {
    /// Attach to the same stable element that carries the item's `id`.
    pub onmounted: EventHandler<MountedEvent>,
    /// Apply to a nonediting control's `onkeydown` for roving navigation.
    /// Editable fields leave this unbound to retain their editing keys.
    pub onkeydown: EventHandler<KeyboardEvent>,
    /// Apply to that element's `tabindex`. `None` preserves its natural tab order.
    pub tabindex: Option<i32>,
    identity: usize,
}
impl DropdownItemBinding {
    /// Spread on the registered element. Stable Rust identity connects native
    /// focus observations to this explicitly registered control.
    pub fn attributes(self) -> Vec<Attribute> {
        vec![
            Attribute::new(
                "data-ix-item-key",
                format!("ix-source-native-opt-in-{}", self.identity),
                None,
                false,
            ),
            Attribute::new("data-ix-roving-item", "", None, false),
        ]
    }
}

/// Rust-owned eligibility for a custom Dropdown control. Keep these values in
/// sync with its rendered attributes; Dioxus is the sole source of item state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DropdownItemOptions {
    pub disabled: bool,
    pub hidden: bool,
    pub checked: bool,
    /// Current sibling position for a dynamic list. Omit for stable registration
    /// order. Explicit positions sort first; ties retain registration order.
    pub order: Option<usize>,
    pub tabindex: Option<i32>,
}

/// Bind a custom Dioxus control to Dropdown keyboard navigation.
///
/// Apply the mounted and tabindex fields, give the control the same stable `id`,
/// and spread `item.attributes()` on it. Registered buttons and fields participate in
/// opening focus in both navigation modes; active-descendant mode preserves
/// native editing keys once they are focused. Nonediting buttons also attach
/// `onkeydown` for roving navigation; editable fields leave it unbound. Dioxus owns focus, scrolling,
/// tab order and cleanup. There is no JavaScript fallback for unregistered controls. Use a
/// different component key when replacing the element. If the control has an
/// authored tab index, use `item.tabindex.or(Some(authored_index))`.
///
/// ```rust,no_run
/// use dioxus::prelude::*;
/// use dioxus_ix::*;
///
/// #[component]
/// fn CustomAction() -> Element {
///     let item = use_dropdown_item("custom-action");
///     rsx! {
///         button {
///             id: "custom-action",
///             onmounted: item.onmounted,
///             onkeydown: item.onkeydown,
///             tabindex: item.tabindex,
///             ..item.attributes(),
///             "Custom action"
///         }
///     }
/// }
/// ```
pub fn use_dropdown_item(id: impl Into<String>) -> DropdownItemBinding {
    use_dropdown_item_with_options(id, DropdownItemOptions::default())
}

/// Bind a custom control with explicit disabled, hidden, checked and authored
/// tab-index state and current sibling order. Use the same options when rendering
/// that control; dynamic wrappers forward `order` and `hidden` explicitly.
pub fn use_dropdown_item_with_options(
    id: impl Into<String>,
    options: DropdownItemOptions,
) -> DropdownItemBinding {
    let binding = use_item_host("native-opt-in");
    let tabindex = binding.tabindex();
    let target = id.into();
    let attributes = options
        .tabindex
        .map(|value| Attribute::new("tabindex", value.to_string(), None, false))
        .into_iter()
        .collect::<Vec<_>>();
    binding.describe(
        &target,
        &attributes,
        ItemDescription {
            disabled: options.disabled,
            hidden: options.hidden,
            checked: options.checked,
            order: options.order,
            ..Default::default()
        },
    );
    let identity = binding.identity;
    let keyboard = binding.context.clone();
    let key_target = target.clone();
    let onmounted = use_callback(move |event: MountedEvent| binding.mount(&target, event.data()));
    let onkeydown = use_callback(move |event: KeyboardEvent| {
        if matches!(event.key(), Key::Enter) || event.key() == Key::Character(" ".into()) {
            return;
        }
        if let Some(context) = &keyboard {
            direct_key(context, event, false, Some(key_target.clone()), true);
        }
    });
    DropdownItemBinding {
        onmounted,
        onkeydown,
        tabindex,
        identity,
    }
}

/// Register opening focus and scrolling only. Use [`use_dropdown_item`] and
/// apply both fields when the Dropdown should also manage roving tab order.
pub fn use_dropdown_item_mount(id: impl Into<String>) -> EventHandler<MountedEvent> {
    use_dropdown_item(id).onmounted
}
/// Read the authored Rust checked declaration without querying the DOM.
pub(crate) fn checked_content_attribute(attributes: &[Attribute]) -> Option<String> {
    attributes
        .iter()
        .rev()
        .find(|attribute| attribute.name == "checked")
        .and_then(|attribute| match &attribute.value {
            dioxus::core::AttributeValue::Text(value) => Some(value.clone()),
            dioxus::core::AttributeValue::Bool(true) => Some(String::new()),
            dioxus::core::AttributeValue::Int(value) => Some(value.to_string()),
            dioxus::core::AttributeValue::Float(value) => Some(value.to_string()),
            _ => None,
        })
}
fn item_attribute_present(attributes: &[Attribute], name: &str) -> bool {
    item_attribute_value(attributes, name).is_some()
}
fn item_attribute_value(attributes: &[Attribute], name: &str) -> Option<String> {
    attributes
        .iter()
        .rev()
        .find(|attribute| attribute.name == name)
        .and_then(|attribute| match &attribute.value {
            dioxus::core::AttributeValue::Text(value) => Some(value.clone()),
            dioxus::core::AttributeValue::Bool(true) => Some(String::new()),
            dioxus::core::AttributeValue::Int(value) => Some(value.to_string()),
            dioxus::core::AttributeValue::Float(value) => Some(value.to_string()),
            _ => None,
        })
}
pub(crate) fn item_host_id(attributes: &[Attribute], default: &str) -> String {
    attributes
        .iter()
        .rev()
        .find_map(|attribute| {
            if attribute.name == "id" {
                if let dioxus::core::AttributeValue::Text(value) = &attribute.value {
                    return Some(value.clone());
                }
            }
            None
        })
        .unwrap_or_else(|| default.to_string())
}

/// Merge consumer classes before spreading attributes so one rendered class
/// value retains the component's current Rust-owned state classes.
pub(crate) fn item_host_class(attributes: &mut Vec<Attribute>, base: impl Into<String>) -> String {
    let mut class = base.into();
    attributes.retain(|attribute| {
        if attribute.name != "class" {
            return true;
        }
        if let dioxus::core::AttributeValue::Text(value) = &attribute.value {
            if !value.is_empty() {
                class.push(' ');
                class.push_str(value);
            }
        }
        false
    });
    class
}

/// Native focus policy. Default mode retains consumer tabindex values and uses
/// the source visible marker; MenuItem/Category projection can move DOM focus.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DropdownNavigationMode {
    #[default]
    ActiveDescendant,
    RovingTabindex,
}
impl DropdownNavigationMode {
    fn inventory_mode(self) -> InventoryMode {
        match self {
            Self::ActiveDescendant => InventoryMode::ActiveDescendant,
            Self::RovingTabindex => InventoryMode::RovingTabindex,
        }
    }
    fn as_str(self) -> &'static str {
        match self {
            Self::ActiveDescendant => "active-descendant",
            Self::RovingTabindex => "roving-tabindex",
        }
    }
}

/// Which ordinary click positions request dismissal. Explicit, parent and
/// Escape requests still pass through the cancelable change callback.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DropdownCloseBehavior {
    #[default]
    Both,
    Inside,
    Outside,
    None,
}
impl From<DropdownCloseBehavior> for CloseBehavior {
    fn from(value: DropdownCloseBehavior) -> Self {
        match value {
            DropdownCloseBehavior::Both => Self::Both,
            DropdownCloseBehavior::Inside => Self::Inside,
            DropdownCloseBehavior::Outside => Self::Outside,
            DropdownCloseBehavior::None => Self::None,
        }
    }
}

/// Cancelable Rust visibility request adapter. It does not emulate bubbling DOM
/// `showChange`/`showChanged` counts, nor cancel an unrelated original click.
#[derive(Clone, Debug)]
pub struct DropdownChangeEvent {
    pub open: bool,
    pub previous: bool,
    prevented: Rc<Cell<bool>>,
}
impl DropdownChangeEvent {
    fn new(open: bool, previous: bool) -> Self {
        Self {
            open,
            previous,
            prevented: Rc::new(Cell::new(false)),
        }
    }
    pub fn prevent_default(&self) {
        self.prevented.set(true);
    }
    pub fn default_prevented(&self) -> bool {
        self.prevented.get()
    }
}

#[derive(Clone)]
struct MenuEntry {
    host_id: String,
    label: String,
    disabled: bool,
    checked: bool,
    mounted: Option<Rc<MountedData>>,
    submenu: Option<DropdownHandle>,
}
#[derive(Clone, PartialEq, Default)]
struct Settings {
    mode: DropdownNavigationMode,
    behavior: DropdownCloseBehavior,
    disable_focus_handling: bool,
    disable_focus_trap: bool,
    suppress_trigger_visibility_check: bool,
    focus_checked_item: bool,
    before: EventHandler<DropdownChangeEvent>,
    changed: EventHandler<bool>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FocusRequest {
    last: bool,
    allow_checked: bool,
}
struct MenuState {
    alive: bool,
    surface: bool,
    generation: u64,
    mounted: bool,
    top_layer: bool,
    surface_visible: bool,
    external: bool,
    blocked: bool,
    settings: Settings,
    active: Option<usize>,
    order: Vec<usize>,
    focus_epoch: u64,
    focus_target: String,
    focus_pending: Option<FocusRequest>,
    forward: Option<DropdownHandle>,
    placement: DropdownPlacement,
    fallback_placement: Option<DropdownPlacement>,
    trigger_id: Option<String>,
}
#[derive(Clone)]
struct TriggerMount {
    focus_id: String,
    in_dropdown_item: bool,
    node: Rc<MountedData>,
    blocked: Rc<Cell<bool>>,
    visibility: Option<TriggerVisibility>,
    ancestors: Vec<NodeHandle>,
}
#[derive(Clone, Copy)]
struct TriggerVisibility {
    intersecting: bool,
    geometry: Option<overlay_runtime::AnchorIntersection>,
}
type ItemKeyboardActions = BTreeMap<String, (String, EventHandler<KeyboardEvent>)>;

#[derive(Clone)]
struct DropdownContext {
    id: usize,
    open: Signal<bool>,
    blocked: Signal<bool>,
    revision: Signal<u64>,
    state: Rc<RefCell<MenuState>>,
    entries: Rc<RefCell<BTreeMap<usize, MenuEntry>>>,
    actions: Rc<RefCell<ItemKeyboardActions>>,
    node: Rc<RefCell<Option<NodeHandle>>>,
    trigger: Rc<RefCell<Option<TriggerMount>>>,
    ancestors: Rc<RefCell<Vec<NodeHandle>>>,
    focus_task: Rc<RefCell<Option<dioxus::core::Task>>>,
}
impl DropdownContext {
    fn node_id(&self) -> String {
        format!("ix-dropdown-node-{}", self.id)
    }
    fn host_id(&self) -> String {
        format!("ix-dropdown-{}", self.id)
    }
    fn trigger_id(&self) -> String {
        format!("ix-dropdown-trigger-{}", self.id)
    }
    fn navigation_owner(&self, reactive: bool) -> Self {
        let mut owner = self.clone();
        let mut visited = Vec::new();
        loop {
            if visited.contains(&owner.id) {
                break;
            }
            visited.push(owner.id);
            if reactive {
                let _ = owner.revision.try_read();
            }
            let next = owner
                .state
                .borrow()
                .forward
                .and_then(DropdownHandle::try_context);
            let Some(next) = next.filter(|child| {
                !visited.contains(&child.id)
                    && child.state.borrow().alive
                    && if reactive {
                        child.open.try_read().is_ok_and(|open| *open)
                    } else {
                        child.open.try_peek().is_ok_and(|open| *open)
                    }
            }) else {
                break;
            };
            owner = next;
        }
        owner
    }
    fn active_dom_id(&self) -> Option<String> {
        let owner = self.navigation_owner(true);
        let node = owner.node.borrow().clone();
        if let Some(node) = node {
            let runtime = node.runtime()?;
            // Opening can render a trigger before its inventory arrives. Read
            // the cue signal now so that the later first cue updates its ARIA.
            let visual_target = runtime.item_visual_target();
            if let Some(facts) = node.inventory() {
                let target = visual_target.as_ref()?;
                if facts.hosts.iter().any(|host| &host.target == target) {
                    let inventory = Inventory::new(InventoryMode::ActiveDescendant, facts);
                    return inventory
                        .query_hosts()
                        .into_iter()
                        .find(|host| {
                            runtime.item_visual_matches(host)
                                && host.kind != ItemKind::Category
                                && host.source_focusable
                        })
                        .map(|host| host.target);
                }
            }
            // A mounted Dioxus control can focus before discovery, or before
            // an existing inventory includes it. Its explicit owner supplies
            // membership without attributing the cue to sibling Dropdowns.
            if visual_target.is_some() {
                return visual_target
                    .filter(|_| runtime.item_visual_owned_by(node.id(), node.generation()));
            }
            if runtime.item_visual_known() {
                return None;
            }
        }
        owner
            .state
            .borrow()
            .active
            .map(|id| format!("ix-dropdown-item-{id}"))
    }
    fn keyboard_mode(&self) -> bool {
        let node = self.node.borrow().clone();
        node.and_then(|node| node.runtime())
            .is_some_and(|runtime| runtime.keyboard_mode())
    }
    fn notify(&self) {
        self.sync_surface_metadata();
        if !self.state.borrow().alive || !self.state.borrow().surface {
            return;
        }
        let mut revision = self.revision;
        if let Ok(mut value) = revision.try_write() {
            *value = value.wrapping_add(1);
        }
    }
    fn invalidate_focus(&self) -> u64 {
        let task = self.focus_task.borrow_mut().take();
        if let Some(task) = task {
            task.cancel();
        }
        let mut state = self.state.borrow_mut();
        state.focus_epoch = state.focus_epoch.wrapping_add(1);
        state.focus_pending = None;
        state.focus_target = "none".into();
        state.focus_epoch
    }
    fn current(&self, epoch: u64) -> bool {
        let state = self.state.borrow();
        state.alive
            && state.surface
            && state.mounted
            && state.focus_epoch == epoch
            && self.open.try_peek().is_ok_and(|open| *open)
    }
    fn anchor_in_dropdown_item(&self) -> bool {
        self.trigger
            .borrow()
            .as_ref()
            .is_some_and(|trigger| trigger.in_dropdown_item)
    }
    fn mount_trigger(
        &self,
        focus_id: String,
        node: Rc<MountedData>,
        blocked: Rc<Cell<bool>>,
        in_dropdown_item: bool,
    ) {
        let (changed, replaced) = {
            let current = self.trigger.borrow();
            (
                current.as_ref().is_none_or(|current| {
                    !Rc::ptr_eq(&current.node, &node)
                        || current.focus_id != focus_id
                        || current.in_dropdown_item != in_dropdown_item
                }),
                current.as_ref().is_some_and(|current| {
                    !Rc::ptr_eq(&current.node, &node) || current.focus_id != focus_id
                }),
            )
        };
        if replaced {
            self.invalidate_focus();
        }
        if changed {
            self.state.borrow_mut().fallback_placement = None;
        }
        let visibility = self
            .trigger
            .borrow()
            .as_ref()
            .filter(|current| Rc::ptr_eq(&current.node, &node))
            .and_then(|current| current.visibility);
        *self.trigger.borrow_mut() = Some(TriggerMount {
            focus_id,
            in_dropdown_item,
            node,
            blocked,
            visibility,
            ancestors: self.ancestors.borrow().clone(),
        });
        self.sync_ownership();
        if changed {
            self.notify();
        }
    }
    fn unmount_trigger(&self, node: &Rc<MountedData>) {
        let owns = self
            .trigger
            .borrow()
            .as_ref()
            .is_some_and(|current| Rc::ptr_eq(&current.node, node));
        if owns {
            self.trigger.borrow_mut().take();
            self.state.borrow_mut().fallback_placement = None;
            self.sync_ownership();
            self.invalidate_focus();
            self.notify();
        }
    }
    fn sync_surface_metadata(&self) {
        if let Some(node) = self.node.borrow().clone() {
            node.set_surface_metadata(overlay_runtime::SurfaceMetadata {
                trigger_id: self
                    .trigger
                    .borrow()
                    .as_ref()
                    .map(|trigger| trigger.focus_id.clone())
                    .unwrap_or_default(),
                items_host: self.navigation_owner(false).host_id(),
                popover_host: String::new(),
            });
        }
    }
    fn sync_ownership(&self) {
        self.sync_surface_metadata();
        let trigger = self.trigger.borrow().clone();
        if let Some(node) = self.node.borrow().clone() {
            let ready = trigger.is_some();
            let ancestors = trigger
                .map(|trigger| trigger.ancestors)
                .unwrap_or_else(|| self.ancestors.borrow().clone());
            node.set_ownership(ancestors, ready);
        }
    }
    fn update_trigger_metadata(
        &self,
        element: &Rc<MountedData>,
        ancestors: Vec<NodeHandle>,
        in_dropdown_item: bool,
    ) {
        let changed = {
            let mut trigger = self.trigger.borrow_mut();
            if let Some(trigger) = trigger
                .as_mut()
                .filter(|trigger| Rc::ptr_eq(&trigger.node, element))
            {
                if trigger.ancestors == ancestors && trigger.in_dropdown_item == in_dropdown_item {
                    false
                } else {
                    trigger.ancestors = ancestors;
                    trigger.in_dropdown_item = in_dropdown_item;
                    true
                }
            } else {
                false
            }
        };
        if changed {
            self.sync_ownership();
            self.notify();
        }
    }
    fn observe_trigger_visibility(&self, node: &Rc<MountedData>, visibility: TriggerVisibility) {
        if !self.state.borrow().alive {
            return;
        }
        {
            let mut trigger = self.trigger.borrow_mut();
            let Some(current) = trigger
                .as_mut()
                .filter(|current| Rc::ptr_eq(&current.node, node))
            else {
                return;
            };
            current.visibility = Some(visibility);
        }
        self.apply_trigger_visibility();
    }
    fn apply_trigger_visibility(&self) {
        let current = {
            let state = self.state.borrow();
            state.alive && state.surface && state.mounted
        };
        if !current {
            return;
        }
        let visibility = self
            .trigger
            .borrow()
            .as_ref()
            .and_then(|trigger| trigger.visibility);
        let node = self.node.borrow().clone();
        if let (Some(visibility), Some(node)) = (visibility, node) {
            node.observe_anchor_visibility(visibility.intersecting, visibility.geometry);
        }
    }
    fn trigger_focus_current(
        &self,
        epoch: u64,
        generation: u64,
        expected_open: bool,
        target: &Rc<MountedData>,
    ) -> bool {
        let state = self.state.borrow();
        state.alive
            && state.surface
            && state.mounted
            && state.generation == generation
            && state.focus_epoch == epoch
            && !state.blocked
            && self.blocked.try_peek().is_ok_and(|blocked| !*blocked)
            && self
                .open
                .try_peek()
                .is_ok_and(|open| *open == expected_open)
            && self
                .node
                .borrow()
                .as_ref()
                .is_some_and(|node| node.generation() == generation && node.is_registered())
            && (if expected_open {
                state.settings.mode == DropdownNavigationMode::ActiveDescendant
                    && !state.settings.disable_focus_handling
            } else {
                !state.settings.disable_focus_trap
            })
            && self
                .trigger
                .borrow()
                .as_ref()
                .is_some_and(|current| Rc::ptr_eq(&current.node, target) && !current.blocked.get())
    }
    fn focus_trigger(&self, expected_open: bool) {
        let epoch = self.invalidate_focus();
        self.state.borrow_mut().focus_target = "trigger".into();
        let generation = self.state.borrow().generation;
        let target = self
            .trigger
            .borrow()
            .as_ref()
            .map(|current| Rc::downgrade(&current.node));
        let node = self.node.borrow().clone();
        if let (Some(target), Some(node)) = (target, node) {
            let context = self.clone();
            node.queue_completion(Rc::new(move || {
                let Some(target) = target.upgrade() else {
                    return;
                };
                if !context.trigger_focus_current(epoch, generation, expected_open, &target) {
                    return;
                }
                let task_context = context.clone();
                let task = spawn(async move {
                    if task_context.trigger_focus_current(epoch, generation, expected_open, &target)
                    {
                        let _ = target.set_focus(true).await;
                    }
                    // No follow-up focus is published after the await. Keep a
                    // reentrant newer intent's task intact while retiring ours.
                    if task_context.state.borrow().focus_epoch == epoch {
                        task_context.focus_task.borrow_mut().take();
                    }
                });
                *context.focus_task.borrow_mut() = Some(task);
            }));
        }
        self.notify();
    }
    fn request(&self, open: bool, reason: Reason, restore_focus: bool) -> bool {
        if {
            let state = self.state.borrow();
            !state.alive || (!state.surface && state.generation > 0) || (open && state.blocked)
        } || (open && *self.blocked.peek())
        {
            return false;
        }
        let node = self.node.borrow().clone();
        if let Some(node) = node {
            return node.request(open, reason, restore_focus);
        }
        // A controller can be prepared before its surface mounts. Native
        // requests remain registration/connected guarded by the shared runtime.
        let mut value = self.open;
        value.set(open);
        true
    }
    /// Actual trigger activation has a post-toggle controller phase even when
    /// the visibility request is vetoed. Public handles do not use this path.
    fn request_from_trigger(&self, open: bool, reason: Reason, restore_focus: bool) -> bool {
        let origin = self.node.borrow().clone();
        let accepted = self.request(open, reason, restore_focus);
        if let Some(origin) = origin
            && let Some(runtime) = origin.runtime()
        {
            runtime.dismiss_dropdown_others(origin.id(), origin.generation());
        }
        accepted
    }
    fn apply(&self, change: Change) {
        if !change.open && *self.open.peek() {
            if let Some(node) = self.node.borrow().clone() {
                node.clear_roving();
                if let Some(runtime) = node.runtime() {
                    runtime.clear_item_visual();
                }
            }
        }
        let task = self.focus_task.borrow_mut().take();
        if let Some(task) = task {
            task.cancel();
        }
        let (changed, generation, restore_trigger) = {
            let mut state = self.state.borrow_mut();
            if !state.alive || !state.surface {
                return;
            }
            let changed = *self.open.peek() != change.open;
            state.focus_epoch = state.focus_epoch.wrapping_add(1);
            if !change.open {
                state.surface_visible = false;
                state.active = None;
                state.forward = None;
                state.focus_pending = None;
                state.focus_target = if change.restore_focus && !state.settings.disable_focus_trap {
                    "trigger".into()
                } else {
                    "none".into()
                };
            }
            let restore_trigger =
                !change.open && change.restore_focus && !state.settings.disable_focus_trap;
            if change.open {
                state.focus_target = "none".into();
            }
            (changed, state.generation, restore_trigger)
        };
        let mut open = self.open;
        let Ok(mut current) = open.try_write() else {
            return;
        };
        *current = change.open;
        drop(current);
        if changed && let Some(node) = self.node.borrow().clone() {
            if change.open {
                let state = self.state.borrow();
                node.set_navigation_mode(state.settings.mode.as_str());
            }
        }
        if restore_trigger {
            self.focus_trigger(false);
        }
        self.notify();
        let completion_node = if changed {
            self.node.borrow().clone()
        } else {
            None
        };
        if let Some(node) = completion_node {
            let context = self.clone();
            // Every changed value owns a Dioxus effect notification, even when
            // a later same-owner command changes visibility again. Resolve the
            // current callback at delivery, like listeners on the source host.
            node.queue_completion(Rc::new(move || {
                let handler = {
                    let state = context.state.borrow();
                    if !state.alive || !state.surface || state.generation != generation {
                        return;
                    }
                    state.settings.changed
                };
                handler.call(change.open);
            }));
        }
    }
    fn publish_item(&self, target: usize, focus: bool, nearest: bool) {
        let Some(host_id) = self
            .entries
            .borrow()
            .get(&target)
            .map(|entry| entry.host_id.clone())
        else {
            return;
        };
        if !self.state.borrow().alive || !*self.open.peek() {
            return;
        }
        self.invalidate_focus();
        {
            let mut state = self.state.borrow_mut();
            // Roving tabindex follows the current key immediately. The active-
            // descendant marker is committed by the owned focus attempt.
            if focus || !nearest {
                state.active = Some(target);
            }
        }
        self.notify();
        if focus || nearest {
            let node = self.node.borrow().clone();
            if let Some(node) = node {
                let context = self.clone();
                node.queue_item_focus(
                    host_id,
                    focus,
                    Rc::new(move || {
                        // The runtime cancels obsolete work; this callback also
                        // checks that its component-owned entry still exists.
                        if !context.entries.borrow().contains_key(&target)
                            || !context.state.borrow().alive
                        {
                            return;
                        }
                        if context.state.borrow().settings.mode
                            == DropdownNavigationMode::ActiveDescendant
                        {
                            context.state.borrow_mut().active = Some(target);
                            context.notify();
                        }
                    }),
                );
            }
        }
    }
    fn publish_host(&self, facts: &InventoryFacts, target: &ItemRef) {
        let mode = self.state.borrow().settings.mode;
        let inventory = Inventory::new(mode.inventory_mode(), facts.clone());
        let Some(host) = inventory.host(target).cloned() else {
            return;
        };
        if !host.eligible(mode.inventory_mode()) || !self.state.borrow().alive || !*self.open.peek()
        {
            return;
        }
        let Some(node) = self.node.borrow().clone() else {
            return;
        };
        if mode == DropdownNavigationMode::RovingTabindex && !node.select_roving(facts, target) {
            return;
        }
        self.queue_host_focus(&host);
    }
    fn queue_host_focus(&self, host: &crate::dropdown_inventory::HostFacts) {
        let Some(node) = self.node.borrow().clone() else {
            return;
        };
        {
            let mut state = self.state.borrow_mut();
            state.active = None;
            state.focus_pending = None;
            state.focus_epoch = state.focus_epoch.wrapping_add(1);
            state.focus_target = "none".into();
        }
        let context = self.clone();
        node.queue_item_host_focus(host, Rc::new(move || context.notify()));
        self.notify();
    }
    fn initialize_roving(&self, facts: &InventoryFacts, last: bool, checked: bool) {
        let Some(node) = self.node.borrow().clone() else {
            return;
        };
        if let Some(host) = node.init_roving(facts, last, checked) {
            self.queue_host_focus(&host);
        }
    }
    fn inventory_key(&self, facts: &KeyFacts, raw: &InventoryFacts) -> KeyDecision {
        let mode = if facts.navigation_mode.as_deref() == Some("active-descendant") {
            DropdownNavigationMode::ActiveDescendant
        } else if facts.navigation_mode.as_deref() == Some("roving-tabindex") {
            DropdownNavigationMode::RovingTabindex
        } else {
            self.state.borrow().settings.mode
        };
        let inventory = Inventory::new(mode.inventory_mode(), raw.clone());
        // Roving keys follow their registered physical origin. AD keys follow
        // the logical selection, since an unfocusable row can keep its cue while
        // native focus stays on the preceding control. A Category parent has no
        // outer AD cue, so its explicit key origin supplies that missing position.
        let origin = facts.focused_target.as_ref().and_then(|target| {
            inventory
                .query_hosts()
                .into_iter()
                .find(|host| &host.target == target)
                .map(|host| host.item)
        });
        let current = if mode == DropdownNavigationMode::ActiveDescendant {
            inventory.active().or(origin)
        } else {
            origin.or_else(|| inventory.active())
        };
        let mut result = KeyDecision::default();
        if matches!(facts.key.as_str(), "Home" | "End" | "ArrowDown" | "ArrowUp") {
            result.prevent = true;
            if let Some(target) = inventory.navigate(&facts.key, current.as_ref()) {
                self.publish_host(raw, &target);
            }
            return result;
        }
        if mode == DropdownNavigationMode::RovingTabindex
            && facts.key.chars().count() == 1
            && facts.key != " "
            && !facts.ctrl
            && !facts.meta
            && !facts.alt
        {
            let hosts = inventory.query_hosts();
            let entries = self.entries.borrow();
            let start = current
                .as_ref()
                .and_then(|item| hosts.iter().position(|host| &host.item == item))
                .unwrap_or(hosts.len().saturating_sub(1));
            let prefix = facts.key.to_lowercase();
            let target = (1..=hosts.len())
                .map(|offset| &hosts[(start + offset) % hosts.len()])
                .find(|host| {
                    entries.values().any(|entry| {
                        entry.host_id == host.target
                            && !entry.disabled
                            && entry.label.to_lowercase().starts_with(&prefix)
                    })
                })
                .map(|host| host.item.clone());
            drop(entries);
            if let Some(target) = target {
                self.publish_host(raw, &target);
                result.prevent = true;
            }
            return result;
        }
        match inventory.activation(&facts.key, &["Enter".into(), " ".into()], current.as_ref()) {
            Activation::NativeDefault | Activation::None => {}
            Activation::SyntheticHost { item } => {
                result.prevent = true;
                result.stop = mode == DropdownNavigationMode::RovingTabindex;
                result.activate =
                    item.and_then(|item| inventory.host(&item).map(|host| host.target.clone()));
            }
            Activation::Submenu { item, trigger: _ } => {
                result.stop = mode == DropdownNavigationMode::RovingTabindex;
                if let Some(host) = inventory.host(&item)
                    && let Some(node) = self.node.borrow().clone()
                    && let Some(receiver) = node
                        .submenu_receiver(host)
                        .and_then(DropdownHandle::try_context)
                {
                    receiver.open_submenu(host);
                }
            }
        }
        result
    }
    fn open_submenu(&self, host: &crate::dropdown_inventory::HostFacts) {
        let Some(node) = self.node.borrow().clone() else {
            return;
        };
        let Some(handle) = node.linked_submenu(host) else {
            return;
        };
        let Some(child) = handle.try_context().filter(|child| child.id != self.id) else {
            return;
        };
        node.mark_submenu_trigger(host);
        child.request(true, Reason::Keyboard, false);
        self.state.borrow_mut().forward = Some(handle);
        self.notify();
        child.focus_submenu_entry();
    }
    fn focus_submenu_entry(&self) {
        self.focus_edge_with_checked(false, true);
    }
    fn item_target(&self, order: &[usize], current: Option<usize>, key: &str) -> Option<usize> {
        menu_target(&self.entries.borrow(), order, current, key)
    }
    fn edge_target(&self, order: &[usize], last: bool, allow_checked: bool) -> Option<usize> {
        let state = self.state.borrow();
        let entries = self.entries.borrow();
        if !last && allow_checked && state.settings.focus_checked_item {
            if let Some(id) = order.iter().find(|id| {
                entries
                    .get(id)
                    .is_some_and(|entry| !entry.disabled && entry.checked)
            }) {
                return Some(*id);
            }
        }
        menu_target(&entries, order, None, if last { "End" } else { "Home" })
    }
    fn focus_edge(&self, last: bool) {
        self.focus_edge_with_checked(last, true);
    }
    fn opening_focus_current(
        &self,
        epoch: u64,
        generation: u64,
        mode: DropdownNavigationMode,
    ) -> bool {
        if !self.current(epoch) {
            return false;
        }
        let state = self.state.borrow();
        state.generation == generation
            && state.settings.mode == mode
            && !state.settings.disable_focus_handling
            && !state.blocked
            && self.blocked.try_peek().is_ok_and(|blocked| !*blocked)
            && self
                .node
                .borrow()
                .as_ref()
                .is_some_and(|node| node.generation() == generation && node.is_registered())
    }
    fn complete_edge_focus(
        &self,
        epoch: u64,
        generation: u64,
        mode: DropdownNavigationMode,
        request: FocusRequest,
        facts: Option<&InventoryFacts>,
    ) {
        if self.state.borrow().focus_epoch == epoch {
            self.focus_task.borrow_mut().take();
        }
        if !self.opening_focus_current(epoch, generation, mode) {
            return;
        }
        let checked = !request.last
            && request.allow_checked
            && self.state.borrow().settings.focus_checked_item;
        let Some(node) = self.node.borrow().clone() else {
            return;
        };
        if mode == DropdownNavigationMode::RovingTabindex {
            if let Some(facts) = facts {
                self.initialize_roving(facts, request.last, checked);
            }
            return;
        }
        let candidates = facts
            .map(|facts| Inventory::new(InventoryMode::RovingTabindex, facts.clone()).query_hosts())
            .unwrap_or_default()
            .into_iter()
            .filter(|host| {
                node.is_item_registered(host)
                    && !host
                        .tabindex_attribute
                        .as_deref()
                        .is_some_and(|value| value.starts_with('-'))
            })
            .collect::<Vec<_>>();
        let edge = if request.last {
            candidates.last()
        } else {
            candidates.first()
        };
        let selected = if checked {
            candidates
                .iter()
                .find(|host| host.checked_attribute)
                .or(edge)
        } else {
            edge
        };
        if let Some(host) = selected {
            self.queue_host_focus(host);
        } else {
            self.focus_trigger(true);
        }
    }
    /// Native popovers become focusable after presentation. Framework visibility
    /// owns this boundary; keep the existing intent instead of querying inventory
    /// through JavaScript or scheduling a timer/frame retry.
    fn surface_visibility(&self, visible: bool) {
        let pending = {
            let mut state = self.state.borrow_mut();
            if !state.alive || !state.surface {
                return;
            }
            let open = self.open.try_peek().is_ok_and(|open| *open);
            state.surface_visible = visible && open;
            if !visible || !state.top_layer || !open {
                return;
            }
            state.focus_pending
        };
        if let Some(request) = pending {
            self.focus_edge_with_checked(request.last, request.allow_checked);
        }
    }
    fn focus_edge_with_checked(&self, last: bool, allow_checked: bool) {
        let epoch = self.invalidate_focus();
        let (generation, mode) = {
            let mut state = self.state.borrow_mut();
            if state.settings.disable_focus_handling || state.blocked {
                return;
            }
            state.focus_pending = Some(FocusRequest {
                last,
                allow_checked,
            });
            (state.generation, state.settings.mode)
        };
        if !self.opening_focus_current(epoch, generation, mode) {
            return;
        }
        {
            let state = self.state.borrow();
            if state.top_layer && !state.surface_visible {
                return;
            }
        }
        let Some(node) = self.node.borrow().clone() else {
            return;
        };
        let owner = node.clone();
        let context = self.clone();
        let task = owner.with_owner(move || {
            spawn(async move {
                let facts = node.query_inventory().await;
                context.complete_edge_focus(
                    epoch,
                    generation,
                    mode,
                    FocusRequest {
                        last,
                        allow_checked,
                    },
                    facts.as_ref(),
                );
            })
        });
        *self.focus_task.borrow_mut() = task;
    }
    fn key(&self, facts: &KeyFacts) -> KeyDecision {
        self.key_inner(facts, &mut Vec::new())
    }
    fn key_inner(&self, facts: &KeyFacts, visited: &mut Vec<usize>) -> KeyDecision {
        if visited.contains(&self.id) {
            return KeyDecision::default();
        }
        visited.push(self.id);
        let mut result = KeyDecision::default();
        if !self.state.borrow().alive || facts.default_prevented || self.state.borrow().blocked {
            return result;
        }
        let (open, mode, no_focus, no_trap, anchor_submenu) = {
            let state = self.state.borrow();
            (
                *self.open.peek(),
                state.settings.mode,
                state.settings.disable_focus_handling,
                state.settings.disable_focus_trap,
                self.anchor_in_dropdown_item(),
            )
        };
        let opening_key = matches!(
            facts.key.as_str(),
            "Home" | "End" | "ArrowDown" | "ArrowUp" | "Enter" | " "
        );
        if facts.on_trigger && opening_key {
            if !open {
                if anchor_submenu {
                    // Anchor-submenu keyboard delegation is a separate source
                    // route; it does not acquire a trigger-toggle others pass.
                    self.request(true, Reason::Keyboard, false);
                } else {
                    self.request_from_trigger(true, Reason::Keyboard, false);
                }
                result.prevent = !facts.alt || no_focus;
                result.stop = no_focus;
                if *self.open.peek() && !facts.alt && !no_focus {
                    let last = matches!(facts.key.as_str(), "End" | "ArrowUp");
                    if facts.inventory.is_some() {
                        self.focus_edge(last);
                    } else if let Some(target) = self.edge_target(&facts.items, last, true) {
                        self.publish_item(
                            target,
                            mode == DropdownNavigationMode::RovingTabindex,
                            mode == DropdownNavigationMode::RovingTabindex,
                        );
                    } else {
                        self.focus_edge(last);
                    }
                }
            } else if mode == DropdownNavigationMode::RovingTabindex
                && !facts.alt
                && !no_focus
                && !matches!(facts.key.as_str(), "Enter" | " ")
            {
                result.prevent = true;
                let owner = self.navigation_owner(false);
                if let Some(raw) = &facts.inventory {
                    let last = matches!(facts.key.as_str(), "End" | "ArrowUp");
                    let checked = !last && owner.state.borrow().settings.focus_checked_item;
                    owner.initialize_roving(raw, last, checked);
                } else if let Some(target) = owner.edge_target(
                    &facts.items,
                    matches!(facts.key.as_str(), "End" | "ArrowUp"),
                    true,
                ) {
                    owner.publish_item(target, true, true);
                }
            }
            if !open
                || mode == DropdownNavigationMode::RovingTabindex
                    && !matches!(facts.key.as_str(), "Enter" | " ")
            {
                return result;
            }
        }
        if !open || no_focus {
            return result;
        }
        if facts.key == "Tab" {
            if mode == DropdownNavigationMode::RovingTabindex || facts.popover_ancestor {
                result.release_focus = true;
                let node = self.node.borrow().clone();
                if let Some(node) = node {
                    if let Some(runtime) = node.runtime() {
                        // The runtime owns cached-root routing; current node's
                        // close policy is not a lexical Rust-parent shortcut.
                        runtime.close_and_release_focus(node.id());
                    }
                }
            } else if no_trap {
                self.request(false, Reason::Keyboard, false);
            } else if facts.inside
                && facts.focused_index.is_some_and(|index| {
                    index < facts.focus_count
                        && ((!facts.shift && index == facts.focus_count - 1)
                            || (facts.shift && index == 0))
                })
            {
                // The source trap listens on the content host and only wraps
                // a current candidate. An external trigger keeps native Tab.
                self.invalidate_focus();
                if let Some(node) = self.node.borrow().clone() {
                    let context = self.clone();
                    if node.queue_tab_boundary_focus(
                        &facts.focusable_ids,
                        facts.shift,
                        Rc::new(move || context.notify()),
                    ) {
                        result.prevent = true;
                        result.stop = true;
                        // Wake the renderer so the queued mounted effect can run
                        // without waiting for an unrelated component update.
                        self.notify();
                    }
                }
            }
            return result;
        }
        let effective_mode = facts.navigation_mode.as_deref().map_or(mode, |mode| {
            if mode == "roving-tabindex" {
                DropdownNavigationMode::RovingTabindex
            } else {
                DropdownNavigationMode::ActiveDescendant
            }
        });
        if effective_mode == DropdownNavigationMode::ActiveDescendant
            && let (Some(focused), Some(inventory), Some(node)) = (
                &facts.focused_target,
                &facts.inventory,
                self.node.borrow().clone(),
            )
            && inventory.hosts.iter().any(|host| {
                host.kind == ItemKind::NativeOptIn
                    && host.target == *focused
                    && node.is_item_registered(host)
            })
        {
            return result;
        }
        if facts.alt && matches!(facts.key.as_str(), "ArrowDown" | "ArrowUp") {
            return result;
        }
        let forward = self
            .state
            .borrow()
            .forward
            .and_then(DropdownHandle::try_context);
        if facts.key != "ArrowLeft" && facts.inventory.is_none() {
            if let Some(child) = forward.filter(|child| {
                child.id != self.id
                    && child.state.borrow().alive
                    && child.open.try_peek().is_ok_and(|open| *open)
            }) {
                let mut forwarded = facts.clone();
                forwarded.on_trigger = false;
                forwarded.inside = true;
                return child.key_inner(&forwarded, visited);
            }
        }
        if facts.key != "ArrowLeft" {
            if let Some(raw) = &facts.inventory {
                return self.inventory_key(facts, raw);
            }
        }
        self.state.borrow_mut().order = facts
            .items
            .iter()
            .copied()
            .filter(|id| self.entries.borrow().contains_key(id))
            .collect();
        let current = self.state.borrow().active.or(facts.focused_item);
        if matches!(facts.key.as_str(), "ArrowDown" | "ArrowUp" | "Home" | "End") {
            result.prevent = true;
            if let Some(target) = self.item_target(&facts.items, current, &facts.key) {
                self.publish_item(target, mode == DropdownNavigationMode::RovingTabindex, true);
            }
        } else if mode == DropdownNavigationMode::RovingTabindex
            && facts.key.chars().count() == 1
            && facts.key != " "
            && !facts.ctrl
            && !facts.meta
            && !facts.alt
        {
            result.prevent = true;
            if let Some(target) = self.item_target(&facts.items, current, &facts.key) {
                self.publish_item(target, true, false);
            }
        } else if facts.key == "ArrowLeft" && facts.inventory.is_some() {
            let forward = self.navigation_owner(false);
            if let Some(node) = forward.node.borrow().clone()
                && let Some(runtime) = node.runtime()
            {
                runtime.close_submenu(node.id());
            }
        } else if facts.key == "ArrowLeft" {
            let forward = self.navigation_owner(false);
            let origin = self.node.borrow().clone();
            if forward.id != self.id {
                let child = forward.node.borrow().clone();
                if let Some(child) = child {
                    if let Some(runtime) = child.runtime() {
                        runtime.close_submenu(child.id());
                    }
                }
                self.state.borrow_mut().forward = None;
                self.notify();
            }
            if let Some(node) = origin {
                if let Some(runtime) = node.runtime() {
                    runtime.close_submenu(node.id());
                }
            }
        } else if matches!(facts.key.as_str(), "ArrowRight" | "Enter" | " ") {
            result.prevent = facts.key != "ArrowRight";
            if let Some(id) = current {
                let entry = self.entries.borrow().get(&id).cloned();
                if let Some(entry) = entry.filter(|entry| !entry.disabled) {
                    if let Some(submenu) = entry.submenu {
                        result.prevent = false;
                        if let Some(child) =
                            submenu.try_context().filter(|child| child.id != self.id)
                        {
                            child.request(true, Reason::Keyboard, false);
                            // Source retains this metadata even if showing is vetoed.
                            self.state.borrow_mut().forward = Some(submenu);
                            self.notify();
                            child.focus_submenu_entry();
                        }
                        result.stop = mode == DropdownNavigationMode::RovingTabindex;
                    } else if facts.key != "ArrowRight" {
                        result.prevent = true;
                        result.stop = mode == DropdownNavigationMode::RovingTabindex;
                        result.activate = Some(entry.host_id);
                    }
                }
            }
        }
        result
    }
}

fn menu_target(
    entries: &BTreeMap<usize, MenuEntry>,
    order: &[usize],
    current: Option<usize>,
    key: &str,
) -> Option<usize> {
    let enabled: Vec<_> = order
        .iter()
        .copied()
        .filter(|id| entries.get(id).is_some_and(|entry| !entry.disabled))
        .collect();
    if key == "Home" {
        return enabled.first().copied();
    }
    if key == "End" {
        return enabled.last().copied();
    }
    if enabled.is_empty() {
        return None;
    }
    let current = enabled.iter().position(|id| Some(*id) == current);
    match key {
        "ArrowDown" => enabled
            .get(current.map_or(0, |index| (index + 1) % enabled.len()))
            .copied(),
        "ArrowUp" => enabled
            .get(current.map_or(enabled.len() - 1, |index| {
                (index + enabled.len() - 1) % enabled.len()
            }))
            .copied(),
        _ if key.chars().count() == 1 && key != " " => {
            let start = current.unwrap_or(enabled.len() - 1);
            let prefix = key.to_lowercase();
            (1..=enabled.len())
                .map(|offset| enabled[(start + offset) % enabled.len()])
                .find(|id| entries[id].label.to_lowercase().starts_with(&prefix))
        }
        _ => None,
    }
}

/// A scoped trigger/surface controller. A single live surface owns a controller;
/// the actual native trigger link, not Rust ancestry, supplies overlay parentage.
#[derive(Clone, Copy, PartialEq)]
pub struct DropdownHandle {
    context: Signal<DropdownContext>,
}
impl DropdownHandle {
    fn try_context(self) -> Option<DropdownContext> {
        self.context.try_peek().ok().map(|context| context.clone())
    }
    fn context(self) -> DropdownContext {
        self.try_context()
            .expect("Dropdown controller owner is mounted")
    }
    pub fn is_open(self) -> bool {
        self.try_context()
            .and_then(|context| context.open.try_read().ok().map(|open| *open))
            .unwrap_or(false)
    }
    /// Current active item ID for a custom trigger's `aria-activedescendant`.
    /// Bind this together with [`use_dropdown_trigger_binding`] when rendering a
    /// custom trigger. Closed menus and roving navigation have no descendant.
    pub fn active_descendant(self) -> Option<String> {
        let context = self.try_context()?;
        let _ = context.revision.try_read().ok();
        if !context.open.try_read().ok().is_some_and(|open| *open)
            || context.state.borrow().settings.mode != DropdownNavigationMode::ActiveDescendant
        {
            return None;
        }
        context.active_dom_id()
    }
    pub fn open(self) {
        self.open_at(false);
    }
    fn open_at(self, last: bool) {
        let Some(context) = self.try_context() else {
            return;
        };
        if context.request(true, Reason::Trigger, false) {
            context.focus_edge(last);
        }
    }
    pub fn close(self, return_focus: bool) {
        if let Some(context) = self.try_context() {
            context.request(false, Reason::Trigger, return_focus);
        }
    }
    pub fn toggle(self) {
        if let Some(context) = self.try_context() {
            let open = *context.open.peek();
            context.request(!open, Reason::Trigger, false);
        }
    }
}
pub fn use_dropdown() -> DropdownHandle {
    let id = use_hook(|| NEXT_ID.fetch_add(1, Ordering::Relaxed));
    let context = DropdownContext {
        id,
        open: use_signal(|| false),
        blocked: use_signal(|| false),
        revision: use_signal(|| 0),
        state: use_hook(|| {
            Rc::new(RefCell::new(MenuState {
                alive: true,
                surface: false,
                generation: 0,
                mounted: false,
                top_layer: false,
                surface_visible: false,
                external: false,
                blocked: false,
                settings: Settings::default(),
                active: None,
                order: vec![],
                focus_epoch: 0,
                focus_target: "none".into(),
                focus_pending: None,
                forward: None,
                placement: DropdownPlacement::default(),
                fallback_placement: None,
                trigger_id: None,
            }))
        }),
        entries: use_hook(|| Rc::new(RefCell::new(BTreeMap::new()))),
        actions: use_hook(|| Rc::new(RefCell::new(BTreeMap::new()))),
        node: use_hook(|| Rc::new(RefCell::new(None))),
        trigger: use_hook(|| Rc::new(RefCell::new(None))),
        ancestors: use_hook(|| Rc::new(RefCell::new(Vec::new()))),
        focus_task: use_hook(|| Rc::new(RefCell::new(None))),
    };
    let dropped = context.clone();
    use_drop(move || {
        dropped.state.borrow_mut().alive = false;
        let task = dropped.focus_task.borrow_mut().take();
        if let Some(task) = task {
            task.cancel();
        }
        let node = dropped.node.borrow_mut().take();
        if let Some(node) = node {
            node.clear_roving();
            if let Some(runtime) = node.runtime() {
                runtime.clear_item_visual();
            }
            node.unregister();
        }
    });
    DropdownHandle {
        context: use_signal(|| context),
    }
}

/// Upstream's eight side/alignment combinations for a root menu.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DropdownPlacement {
    #[default]
    BottomStart,
    BottomEnd,
    TopStart,
    TopEnd,
    LeftStart,
    LeftEnd,
    RightStart,
    RightEnd,
}
impl DropdownPlacement {
    fn clipped(self, geometry: overlay_runtime::AnchorIntersection) -> Option<Self> {
        let end = matches!(
            self,
            Self::BottomEnd | Self::TopEnd | Self::LeftEnd | Self::RightEnd
        );
        let bounds = geometry.bounding;
        let intersection = geometry.intersection;
        // Source observer assignments deliberately overwrite in this order;
        // horizontal clipping wins when two edges are hidden.
        let mut fallback = None;
        if intersection.top > bounds.top {
            fallback = Some(if end {
                Self::BottomEnd
            } else {
                Self::BottomStart
            });
        }
        if intersection.bottom < bounds.bottom {
            fallback = Some(if end { Self::TopEnd } else { Self::TopStart });
        }
        if intersection.left > bounds.left {
            fallback = Some(if end {
                Self::RightEnd
            } else {
                Self::RightStart
            });
        }
        if intersection.right < bounds.right {
            fallback = Some(if end { Self::LeftEnd } else { Self::LeftStart });
        }
        fallback
    }
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BottomStart => "bottom-start",
            Self::BottomEnd => "bottom-end",
            Self::TopStart => "top-start",
            Self::TopEnd => "top-end",
            Self::LeftStart => "left-start",
            Self::LeftEnd => "left-end",
            Self::RightStart => "right-start",
            Self::RightEnd => "right-end",
        }
    }
    const fn area(self) -> &'static str {
        match self {
            Self::BottomStart => "bottom span-right",
            Self::BottomEnd => "bottom span-left",
            Self::TopStart => "top span-right",
            Self::TopEnd => "top span-left",
            Self::LeftStart => "left span-bottom",
            Self::LeftEnd => "left span-top",
            Self::RightStart => "right span-bottom",
            Self::RightEnd => "right span-top",
        }
    }
}

// Component implementation follows the source trigger/host relationship.

impl PartialEq for DropdownContext {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.state, &other.state)
    }
}

/// Dioxus handlers and attributes for a custom Dropdown trigger.
///
/// Attach all four handlers and spread [`Self::attributes`] on the same stable
/// Dioxus element. The framework owns
/// its visibility observer and releases it when that element is removed.
#[derive(Clone, Copy)]
pub struct DropdownTriggerBinding {
    pub onmounted: EventHandler<MountedEvent>,
    pub onvisible: EventHandler<VisibleEvent>,
    pub onkeydown: EventHandler<KeyboardEvent>,
    pub onclick: EventHandler<MouseEvent>,
    identity: Option<usize>,
}
impl DropdownTriggerBinding {
    /// Spread on the bound trigger after its other attributes. Dioxus owns the
    /// overlay relationship and CSS anchor property throughout this element's
    /// lifetime. The style namespace preserves the control's other CSS styles.
    pub fn attributes(self) -> Vec<Attribute> {
        self.identity.map_or_else(Vec::new, |id| {
            vec![
                Attribute::new(
                    "data-ix-overlay-trigger",
                    format!("ix-dropdown-node-{id}"),
                    None,
                    false,
                ),
                Attribute::new(
                    "anchor-name",
                    format!("--ix-dropdown-anchor-{id}"),
                    Some("style"),
                    false,
                ),
            ]
        })
    }
}

/// Register a custom Dioxus trigger for focus restoration, automatic placement
/// and dismissal when its anchor leaves the viewport.
///
/// Attach `onmounted`, `onvisible`, `onkeydown` and `onclick`, and spread
/// `..binding.attributes()` on the element referenced by
/// `Dropdown.trigger`. Use a new component key when replacing that control or
/// its controller. Use one live bound trigger per controller. An ID alone does
/// not bind a control: activation, anchor styling,
/// visibility, keyboard and programmatic focus require the Dioxus binding. No
/// JavaScript fallback is used. Controls composed inside [`DropdownItem`]
/// content inherit submenu placement through their Dioxus scope.
pub fn use_dropdown_trigger_binding(dropdown: DropdownHandle) -> DropdownTriggerBinding {
    use_trigger_binding(dropdown.try_context(), false, None, false)
}

/// Register a custom Dioxus trigger for Dropdown focus restoration only.
///
/// Attach this handler to the stable element referenced by `Dropdown.trigger`.
/// This hook supplies the mounted focus handle and releases it with the calling
/// component's scope. Use a new
/// component key when replacing the trigger or its controller. This mount-only
/// hook does not supply click/keyboard events, anchor metadata or visibility; use
/// [`use_dropdown_trigger_binding`] for a complete custom trigger.
pub fn use_dropdown_trigger_mount(dropdown: DropdownHandle) -> EventHandler<MountedEvent> {
    use_trigger_binding(dropdown.try_context(), false, None, false).onmounted
}
fn use_trigger_binding(
    context: Option<DropdownContext>,
    disabled: bool,
    focus_id: Option<String>,
    in_dropdown_item: bool,
) -> DropdownTriggerBinding {
    // Read the current scope on every render: a surviving binding must not
    // retain placement metadata from a previous Dioxus parent. Missing explicit
    // item composition is a standalone anchor, never a native-query fallback.
    let in_dropdown_item =
        in_dropdown_item || try_consume_context::<DropdownItemContentScope>().is_some();
    let identity = context.as_ref().map(|context| context.id);
    let ancestors = overlay_runtime::use_overlay_ancestors()
        .into_iter()
        .filter(|node| {
            context
                .as_ref()
                .is_none_or(|context| node.id() != context.node_id())
        })
        .collect::<Vec<_>>();
    let focus_binding = use_item_host("native-opt-in");
    let blocked = use_hook(|| Rc::new(Cell::new(disabled)));
    blocked.set(disabled);
    let owned = use_hook(|| Rc::new(RefCell::new(None::<(DropdownContext, Rc<MountedData>)>)));
    let cancel_disabled = owned
        .borrow()
        .as_ref()
        .filter(|(context, node)| {
            disabled
                && context
                    .trigger
                    .borrow()
                    .as_ref()
                    .is_some_and(|current| Rc::ptr_eq(&current.node, node))
                && context.state.borrow().focus_target == "trigger"
        })
        .map(|(context, _)| context.clone());
    if let Some(context) = cancel_disabled {
        context.invalidate_focus();
    }
    if let Some((current, node)) = owned.borrow().as_ref() {
        current.update_trigger_metadata(node, ancestors.clone(), in_dropdown_item);
    }
    let cleanup = owned.clone();
    use_drop(move || {
        if let Some((context, node)) = cleanup.borrow_mut().take() {
            context.unmount_trigger(&node);
        }
    });
    let visible_owner = owned.clone();
    let key_owner = owned.clone();
    let click_owner = owned.clone();
    let onmounted = use_callback(move |event: MountedEvent| {
        let node = event.data();
        if let Some((previous, mounted)) = owned.borrow_mut().take() {
            let same = context.as_ref().is_some_and(|context| context == &previous)
                && Rc::ptr_eq(&mounted, &node);
            if !same {
                previous.unmount_trigger(&mounted);
            }
        }
        if let Some(context) = &context {
            let target = focus_id.clone().unwrap_or_else(|| {
                context
                    .state
                    .borrow()
                    .trigger_id
                    .clone()
                    .unwrap_or_else(|| context.trigger_id())
            });
            focus_binding.mount(&target, node.clone());
            context.mount_trigger(target, node.clone(), blocked.clone(), in_dropdown_item);
            context.update_trigger_metadata(&node, ancestors.clone(), in_dropdown_item);
            *owned.borrow_mut() = Some((context.clone(), node));
        }
    });
    let onvisible = use_callback(move |event: VisibleEvent| {
        let owner = visible_owner.borrow().clone();
        let Some((context, node)) = owner else {
            return;
        };
        let Ok(intersecting) = event.is_intersecting() else {
            return;
        };
        let geometry = event
            .get_bounding_client_rect()
            .ok()
            .zip(event.get_intersection_rect().ok())
            .map(
                |(bounding, intersection)| overlay_runtime::AnchorIntersection {
                    bounding: overlay_runtime::AnchorRect {
                        top: bounding.min_y(),
                        bottom: bounding.max_y(),
                        left: bounding.min_x(),
                        right: bounding.max_x(),
                    },
                    intersection: overlay_runtime::AnchorRect {
                        top: intersection.min_y(),
                        bottom: intersection.max_y(),
                        left: intersection.min_x(),
                        right: intersection.max_x(),
                    },
                },
            );
        context.observe_trigger_visibility(
            &node,
            TriggerVisibility {
                intersecting,
                geometry,
            },
        );
    });
    let onkeydown = use_callback(move |event: KeyboardEvent| {
        let owner = key_owner.borrow().clone();
        let Some((context, node)) = owner else {
            return;
        };
        if context
            .trigger
            .borrow()
            .as_ref()
            .is_some_and(|current| Rc::ptr_eq(&current.node, &node) && !current.blocked.get())
        {
            direct_key(&context, event, true, None, false);
        }
    });
    let onclick = use_callback(move |event: MouseEvent| {
        if !event.default_action_enabled() {
            return;
        }
        let Some((context, node)) = click_owner.borrow().clone() else {
            return;
        };
        let current = context
            .trigger
            .borrow()
            .as_ref()
            .is_some_and(|current| Rc::ptr_eq(&current.node, &node) && !current.blocked.get());
        if !current || context.state.borrow().blocked {
            return;
        }
        let previous = *context.open.peek();
        context.request_from_trigger(!previous, Reason::Trigger, context.keyboard_mode());
        // Only this Dioxus trigger handles the click. Preserve native defaults
        // and document-wide outside observation; do not toggle an ancestor.
        event.stop_propagation();
    });
    DropdownTriggerBinding {
        onmounted,
        onvisible,
        onkeydown,
        onclick,
        identity,
    }
}

/// External native trigger for a separately rendered persistent surface.
#[component]
pub fn DropdownTrigger(
    dropdown: DropdownHandle,
    children: Element,
    #[props(default = crate::ButtonVariant::Primary)] variant: crate::ButtonVariant,
    #[props(default)] disabled: bool,
    #[props(default)] loading: bool,
    icon: Option<String>,
    aria_label: Option<String>,
    #[props(default)] class: String,
) -> Element {
    let context = dropdown.context();
    let _ = (context.revision)();
    let blocked = disabled || loading || (context.blocked)();
    let binding = use_trigger_binding(
        Some(context.clone()),
        blocked,
        Some(context.trigger_id()),
        false,
    );
    let id = context.node_id();
    let trigger_id = context.trigger_id();
    let host_id = context.host_id();
    let active_id = dropdown.active_descendant();
    rsx! { button {
        id:trigger_id, class:"ix-dropdown-external-trigger ix-button ix-button-{variant.as_str()} {class}",
        r#type:"button",disabled:blocked,"aria-label":aria_label,"aria-busy":loading.to_string(),
        "aria-haspopup":"menu","aria-expanded":dropdown.is_open().to_string(),"aria-controls":host_id,"aria-activedescendant":active_id,
        "data-ix-overlay-trigger":id,style:"anchor-name:--ix-dropdown-anchor-{context.id}",
        onmounted:binding.onmounted,onvisible:binding.onvisible,
        onclick:binding.onclick,
        onkeydown:binding.onkeydown,
        if loading {crate::Spinner {size:crate::SpinnerSize::Sm,hide_track:true}}
        else if let Some(icon)=icon {Icon {name:icon,size:20}}
        {children}
    } }
}

/// Persistent source-default Dropdown. `controller` or `trigger` omits the
/// built-in trigger; use DropdownTrigger, a linked DropdownItem or an existing
/// explicitly bound Dioxus control ID. Ordinary native click ancestry is preserved by the built-in trigger surrounding its
/// surface. Children keep their scopes and native nodes while closed.
#[component]
pub fn Dropdown(
    label: String,
    controller: Option<DropdownHandle>,
    /// ID of an explicitly bound Dioxus trigger. An ID alone installs no listeners.
    /// The actual host remains the trigger; no wrapper is inserted. Custom
    /// controls attach [`use_dropdown_trigger_binding`] for mounted focus and
    /// visibility and click support, and spread its returned attributes.
    trigger: Option<String>,
    #[props(default)] placement: DropdownPlacement,
    #[props(default)] disabled: bool,
    #[props(default)] loading: bool,
    #[props(default=crate::ButtonVariant::Secondary)] variant: crate::ButtonVariant,
    icon: Option<String>,
    aria_label: Option<String>,
    #[props(default)] class: String,
    #[props(default)] navigation_mode: DropdownNavigationMode,
    #[props(default)] close_behavior: DropdownCloseBehavior,
    #[props(default)] enable_top_layer: bool,
    /// Disable automatic side flips. Inline/shift positioning remains a
    /// platform adapter; full FloatingUI geometry is not implemented.
    #[props(default)]
    suppress_automatic_placement: bool,
    #[props(default)] disable_focus_handling: bool,
    #[props(default)] disable_focus_trap: bool,
    /// Keep the menu open when its native anchor leaves the viewport or hides.
    #[props(default)]
    suppress_trigger_visibility_check: bool,
    #[props(default)] focus_checked_item: bool,
    #[props(default)] onbeforechange: EventHandler<DropdownChangeEvent>,
    #[props(default)] onchange: EventHandler<bool>,
    children: Element,
) -> Element {
    let internal = use_dropdown();
    let handle = controller.unwrap_or(internal);
    let context = handle.context();
    let key = context.node_id();
    let settings = Settings {
        mode: navigation_mode,
        behavior: close_behavior,
        disable_focus_handling,
        disable_focus_trap,
        suppress_trigger_visibility_check,
        focus_checked_item,
        before: onbeforechange,
        changed: onchange,
    };
    rsx! { DropdownSurface {key:"{key}",context,handle,label,external:controller.is_some()||trigger.is_some(),trigger,placement,blocked:disabled||loading,loading,variant,icon,aria_label,class,settings,enable_top_layer,suppress_automatic_placement,{children}} }
}

#[component]
fn DropdownSurface(
    context: DropdownContext,
    handle: DropdownHandle,
    label: String,
    external: bool,
    trigger: Option<String>,
    placement: DropdownPlacement,
    blocked: bool,
    loading: bool,
    variant: crate::ButtonVariant,
    icon: Option<String>,
    aria_label: Option<String>,
    class: String,
    settings: Settings,
    enable_top_layer: bool,
    suppress_automatic_placement: bool,
    children: Element,
) -> Element {
    let runtime = overlay_runtime::use_overlay_runtime();
    let _ = (context.revision)();
    let open = (context.open)();
    {
        let mut state = context.state.borrow_mut();
        state.settings = settings.clone();
        state.external = external;
        state.top_layer = enable_top_layer;
        state.blocked = blocked;
        state.placement = placement;
        state.trigger_id = trigger.clone();
    }
    let node = use_hook({
        let context = context.clone();
        let runtime = runtime.clone();
        move || {
            let generation = {
                let mut state = context.state.borrow_mut();
                state.surface = true;
                state.generation = state.generation.wrapping_add(1);
                state.generation
            };
            let read = context.clone();
            let before = context.clone();
            let apply = context.clone();
            let key = context.clone();
            let node = runtime.register(
                context.node_id(),
                generation,
                OverlayKind::Dropdown,
                settings.behavior.into(),
                Callbacks {
                    read: Rc::new(move || {
                        let state = read.state.borrow();
                        if state.alive && state.surface && state.generation == generation {
                            read.open.try_peek().ok().map(|open| *open)
                        } else {
                            None
                        }
                    }),
                    before: Rc::new(move |change| {
                        let (previous, handler, alive) = {
                            let state = before.state.borrow();
                            (
                                *before.open.peek(),
                                state.settings.before,
                                state.alive && state.surface && state.generation == generation,
                            )
                        };
                        if !alive || change.open && before.state.borrow().blocked {
                            return false;
                        }
                        let event = DropdownChangeEvent::new(change.open, previous);
                        handler.call(event.clone());
                        !event.default_prevented()
                    }),
                    apply: Rc::new(move |change| apply.apply(change)),
                    key: Some(Rc::new(move |facts| key.key(facts))),
                },
            );
            *context.node.borrow_mut() = Some(node.clone());
            let query = context.clone();
            let reset = context.clone();
            node.set_submenu_access(
                handle,
                Rc::new(move || Some(query.navigation_owner(false).node_id())),
                Rc::new(move || {
                    reset.state.borrow_mut().forward = None;
                    reset.notify();
                }),
            );
            let intersection_context = context.clone();
            node.set_anchor_intersection_handler(Rc::new(move |geometry| {
                let changed = {
                    let mut state = intersection_context.state.borrow_mut();
                    if !state.alive || !state.surface || state.generation != generation {
                        return;
                    }
                    let fallback = state.placement.clipped(geometry);
                    let changed = state.fallback_placement != fallback;
                    state.fallback_placement = fallback;
                    changed
                };
                if changed {
                    intersection_context.notify();
                }
            }));
            node
        }
    });
    let ancestors = overlay_runtime::use_overlay_ancestors();
    *context.ancestors.borrow_mut() = ancestors.clone();
    let body_ancestors = if external {
        context
            .trigger
            .borrow()
            .as_ref()
            .map(|trigger| trigger.ancestors.clone())
            .unwrap_or(ancestors)
    } else {
        ancestors
    };
    overlay_runtime::use_overlay_scope_with_ancestors(Some(node.clone()), body_ancestors);
    context.sync_ownership();
    node.set_close_behavior(settings.behavior.into());
    node.set_trigger_visibility_check(!settings.suppress_trigger_visibility_check);
    use_context_provider(|| context.clone());
    use_context_provider(|| ItemInventoryOwner(node.clone()));
    let drop_context = context.clone();
    let drop_node = node.clone();
    use_drop(move || {
        {
            let mut state = drop_context.state.borrow_mut();
            state.surface = false;
            state.mounted = false;
            state.focus_epoch = state.focus_epoch.wrapping_add(1);
            state.focus_pending = None;
        }
        let task = drop_context.focus_task.borrow_mut().take();
        if let Some(task) = task {
            task.cancel();
        }
        drop_context.node.borrow_mut().take();
        drop_node.clear_roving();
        if let Some(runtime) = drop_node.runtime() {
            runtime.clear_item_visual();
        }
        drop_node.unregister();
    });
    let configure = context.clone();
    let configure_node = node.clone();
    let mode_sync = use_hook(|| Rc::new(Cell::new(settings.mode)));
    use_effect(use_reactive(
        (&blocked, &settings.mode),
        move |(blocked, mode)| {
            let mut flag = configure.blocked;
            flag.set(blocked);
            if blocked {
                configure_node.request(false, Reason::Configuration, false);
            }
            if mode_sync.replace(mode) != mode {
                configure_node.clear_roving();
                if let Some(runtime) = configure_node.runtime() {
                    runtime.clear_item_visual();
                }
                configure_node.close_children(Reason::Configuration);
                configure.invalidate_focus();
                {
                    let mut state = configure.state.borrow_mut();
                    state.active = None;
                    state.forward = None;
                }
                if *configure.open.peek() {
                    configure_node.set_navigation_mode(mode.as_str());
                    if mode == DropdownNavigationMode::RovingTabindex {
                        configure.focus_edge(false);
                    } else if !configure.state.borrow().settings.disable_focus_handling {
                        configure.focus_trigger(true);
                    }
                }
                configure.notify();
            }
        },
    ));
    let visible_context = context.clone();
    use_effect(use_reactive(
        (
            &open,
            &settings.suppress_trigger_visibility_check,
            &placement,
        ),
        move |(open, _, _)| {
            if open {
                // Framework visibility events also arrive while closed. Apply
                // the current mounted trigger's last fact on ordinary opening.
                visible_context.apply_trigger_visibility();
            }
        },
    ));
    let state = context.state.borrow();
    let generation = state.generation;
    let effective_placement = if context.anchor_in_dropdown_item() {
        DropdownPlacement::RightStart
    } else if !settings.suppress_trigger_visibility_check {
        state.fallback_placement.unwrap_or(placement)
    } else {
        placement
    };
    drop(state);
    let navigation_owner = context.navigation_owner(true);
    let items_owner = navigation_owner.node_id();
    node.set_surface_metadata(overlay_runtime::SurfaceMetadata {
        trigger_id: context
            .trigger
            .borrow()
            .as_ref()
            .map(|trigger| trigger.focus_id.clone())
            .unwrap_or_default(),
        items_host: navigation_owner.host_id(),
        popover_host: String::new(),
    });
    let active_id = context.active_dom_id();
    let id = context.node_id();
    let host_id = context.host_id();
    let trigger_id = context.trigger_id();
    let native_popup =
        crate::native_surface::use_native_popover(host_id.clone(), open && enable_top_layer);
    let aria_name = aria_label.unwrap_or_else(|| label.clone());
    let mount_context = context.clone();
    let mounted_node = node.clone();
    let trigger_binding = use_trigger_binding(
        (!external).then(|| context.clone()),
        blocked,
        Some(trigger_id.clone()),
        false,
    );
    let trigger_visibility = if external {
        Vec::new()
    } else {
        dioxus::html::events::EventsExtension::onvisible(Vec::new(), trigger_binding.onvisible)
    };
    let click_context = context.clone();
    let key_context = context.clone();
    let trigger_focused = use_hook(|| Rc::new(Cell::new(false)));
    let focused_in = trigger_focused.clone();
    let focused_out = trigger_focused.clone();
    let content_key = context.clone();
    let content_click = node.clone();
    let surface_visibility = context.clone();
    let panel_focused = use_hook(|| Rc::new(Cell::new(false)));
    let panel_focus = panel_focused.clone();
    let panel_blur = panel_focused.clone();
    let runtime_owner = runtime.owner_id();
    let root_role = (!external).then_some("button");
    let trigger_marker = (!external).then_some(id.clone());
    rsx! { div {
        id:if external {format!("ix-dropdown-surface-owner-{}",context.id)}else{trigger_id},
        class:"ix-dropdown {class}",role:root_role,tabindex:(!external).then_some(if blocked {-1}else{0}),
        "aria-label":(!external).then_some(aria_name.clone()),"aria-disabled":(!external).then_some(blocked.to_string()),"aria-busy":(!external).then_some(loading.to_string()),
        "aria-haspopup":(!external).then_some("menu"),"aria-expanded":(!external).then_some(open.to_string()),
        "aria-controls":(!external).then_some(host_id.clone()),
        "aria-activedescendant":if settings.mode==DropdownNavigationMode::ActiveDescendant&&open{active_id.clone()}else{None},
        "data-ix-overlay-root":id.clone(),"data-ix-overlay-owner":runtime_owner,"data-ix-overlay-trigger":trigger_marker,
        "data-ix-overlay-generation":generation.to_string(),"data-ix-overlay-open":open.to_string(),
        "data-ix-overlay-revision":node.render_revision().to_string(),
        "data-ix-overlay-top-layer":enable_top_layer.to_string(),"data-ix-overlay-mode":settings.mode.as_str(),
        "data-ix-overlay-focus-proxy":(settings.mode==DropdownNavigationMode::ActiveDescendant).to_string(),
        "data-ix-overlay-items-owner":items_owner,"data-ix-overlay-active-item":active_id,
        style:if external{format!("--ix-dropdown-area:{}",effective_placement.area())}else{format!("anchor-name:--ix-dropdown-anchor-{};--ix-dropdown-area:{}",context.id,effective_placement.area())},
        onmounted:move |event|{mounted_node.set_mounted();mount_context.state.borrow_mut().mounted=true;trigger_binding.onmounted.call(event.clone());mounted_node.sync_open(*mount_context.open.peek());let pending=mount_context.state.borrow().focus_pending;if *mount_context.open.peek(){let state=mount_context.state.borrow();mounted_node.set_navigation_mode(state.settings.mode.as_str());drop(state);if let Some(last)=pending{mount_context.focus_edge_with_checked(last.last,last.allow_checked);}}mount_context.apply_trigger_visibility();},
        onclick:move |event:MouseEvent| {if !external {event.stop_propagation();if !blocked&&!click_context.state.borrow().blocked&&event.default_action_enabled(){let previous=*click_context.open.peek();click_context.request_from_trigger(!previous,Reason::Trigger,click_context.keyboard_mode());}}},
        onfocus:move |_|focused_in.set(true), onblur:move |_|focused_out.set(false),
        onkeydown:move |event:KeyboardEvent|{if !external&&trigger_focused.get(){direct_key(&key_context,event,true,None,false);}},
        ..trigger_visibility,
        if !external {button {class:"ix-dropdown-trigger ix-button ix-button-{variant.as_str()}",r#type:"button",disabled:blocked,"aria-hidden":"true",inert:true,tabindex:-1,
            if loading {crate::Spinner {size:crate::SpinnerSize::Sm,hide_track:true}}
            else if let Some(icon)=icon {Icon {name:icon,size:20}}
            "{label}" Icon {name:"chevron-down-small",size:16}
        }}
        dialog {id:host_id,class:"ix-dropdown-menu ix-dropdown-root-menu",role:"menu","aria-label":label,
            "data-ix-overlay-host":id,"data-placement":effective_placement.as_str(),"data-configured-placement":placement.as_str(),hidden:!open,inert:(!open).then_some(""),
            popover:enable_top_layer.then_some("manual"),
            "data-ix-native-popover":native_popup.marker,
            onmounted:native_popup.onmounted,
            onvisible:move |event:VisibleEvent|{if let Ok(visible)=event.is_intersecting(){surface_visibility.surface_visibility(visible);}},
            style:format!("position-anchor:--ix-dropdown-anchor-{};--ix-dropdown-area:{};{}",context.id,effective_placement.area(),if suppress_automatic_placement {"position-try-fallbacks:none"} else {""}),
            onfocus:move |_|panel_focus.set(true),onblur:move |_|panel_blur.set(false),
            onclick:move |event:MouseEvent|{event.stop_propagation();content_click.dioxus_inside_click(&event);},
            onkeydown:move |event:KeyboardEvent|{if panel_focused.get()||matches!(event.key(),Key::Escape|Key::Tab){direct_key(&content_key,event.clone(),false,None,true);event.stop_propagation();}},
            {children}
        }
        overlay_runtime::OverlayTransport {runtime}
    } }
}

// One Dioxus path on both targets, independent of native observation readiness.
fn direct_key(
    context: &DropdownContext,
    event: KeyboardEvent,
    on_trigger: bool,
    focused_target: Option<String>,
    inside: bool,
) {
    let Some(node) = context
        .node
        .borrow()
        .clone()
        .filter(NodeHandle::is_registered)
    else {
        return;
    };
    let Some(mut facts) = node.dioxus_key_facts(&event, on_trigger, focused_target, inside) else {
        return;
    };
    if facts.inventory.is_none() {
        let owner = context.navigation_owner(false);
        facts.items = owner.entries.borrow().keys().copied().collect();
        facts.focused_item = facts.focused_target.as_ref().and_then(|target| {
            owner
                .entries
                .borrow()
                .iter()
                .find(|(_, entry)| &entry.host_id == target)
                .map(|(id, _)| *id)
        });
    }
    let decision = node.dioxus_key(facts);
    if let Some(target) = decision.activate.as_ref() {
        let owner = context.navigation_owner(false);
        let action = owner
            .actions
            .borrow()
            .values()
            .find(|(id, _)| id == target)
            .map(|(_, action)| *action);
        if let Some(action) = action {
            action.call(event.clone());
            // Item activation has a semantic Rust callback; no native click is
            // fabricated. Existing close policy still honors keep-open/veto.
            let close = matches!(
                owner.state.borrow().settings.behavior,
                DropdownCloseBehavior::Both | DropdownCloseBehavior::Inside
            );
            if close && event.default_action_enabled() {
                owner.request(false, Reason::Keyboard, true);
            }
        }
    }
    if decision.prevent {
        event.prevent_default();
    }
    if decision.stop {
        event.stop_propagation();
    }
    context.notify();
}

/// Heading content inside a Dropdown surface.
#[component]
pub fn DropdownHeader(label: String) -> Element {
    rsx! {div {class:"ix-dropdown-header",role:"presentation","{label}"}}
}

/// Persistent explicit submenu convenience API. Native trigger ancestry is
/// registered exactly as for an externally linked DropdownItem/controller.
#[component]
pub fn DropdownSubmenu(
    label: String,
    icon: Option<String>,
    #[props(default)] disabled: bool,
    /// Current sibling position of the submenu trigger in a dynamic list.
    #[props(default)]
    order: Option<usize>,
    #[props(default)] navigation_mode: DropdownNavigationMode,
    #[props(default)] close_behavior: DropdownCloseBehavior,
    #[props(default)] enable_top_layer: bool,
    #[props(default)] onbeforechange: EventHandler<DropdownChangeEvent>,
    #[props(default)] onchange: EventHandler<bool>,
    children: Element,
) -> Element {
    let dropdown = use_dropdown();
    rsx! {div {class:"ix-dropdown-submenu",
        DropdownItem {label:label.clone(),icon,disabled,order,dropdown}
        Dropdown {label,controller:dropdown,disabled,navigation_mode,close_behavior,enable_top_layer,onbeforechange,onchange,placement:DropdownPlacement::RightStart,class:"ix-dropdown-submenu-surface",{children}}
    }}
}

/// Separate cancelable item notification adapter, including standalone
/// Enter/Space. Preventing it never cancels the original native event. The
/// compatibility MouseEvent callback is native-click-only; no MouseEvent is
/// fabricated from a keyboard notification.
#[derive(Clone, Debug)]
pub struct DropdownItemClickEvent {
    pub label: String,
    prevented: Rc<Cell<bool>>,
}
impl DropdownItemClickEvent {
    pub fn prevent_default(&self) {
        self.prevented.set(true);
    }
    pub fn default_prevented(&self) -> bool {
        self.prevented.get()
    }
}

/// A persistent role-menuitem surface. `dropdown` links this actual native node
/// to a separate controller/surface. Checkability and `keep_open` are retained
/// Rust adapters; the latter cancels the original click to suppress dismissal.
#[component]
pub fn DropdownItem(
    label: String,
    #[props(default)] compact: bool,
    #[props(default)] icon: Option<String>,
    #[props(default)] disabled: bool,
    /// Current sibling position for dynamic lists; static items may omit it.
    #[props(default)]
    order: Option<usize>,
    #[props(default)] checked: bool,
    #[props(default)] checkable: bool,
    #[props(default)] keep_open: bool,
    #[props(default)] dropdown: Option<DropdownHandle>,
    #[props(default)] onclick: Option<EventHandler<MouseEvent>>,
    #[props(default)] onfocus: EventHandler<FocusEvent>,
    #[props(default)] onblur: EventHandler<FocusEvent>,
    #[props(default)] onitemclick: EventHandler<DropdownItemClickEvent>,
    #[props(default)] children: Option<Element>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let id = use_hook(|| NEXT_ID.fetch_add(1, Ordering::Relaxed));
    let host_binding = use_item_host("dropdown");
    let host_id = item_host_id(&attributes, &format!("ix-dropdown-item-{id}"));
    let context = try_use_context::<DropdownContext>();
    let linked = dropdown.and_then(DropdownHandle::try_context);
    let linked_blocked = linked.as_ref().is_some_and(|linked| (linked.blocked)());
    let blocked = disabled || linked_blocked;
    if let Some(context) = &context {
        context
            .entries
            .borrow_mut()
            .entry(id)
            .and_modify(|entry| {
                entry.host_id = host_id.clone();
                entry.label = label.clone();
                entry.disabled = blocked;
                entry.checked = checked;
                entry.submenu = dropdown;
            })
            .or_insert(MenuEntry {
                host_id: host_id.clone(),
                label: label.clone(),
                disabled: blocked,
                checked,
                mounted: None,
                submenu: dropdown,
            });
    }
    let drop_context = context.clone();
    use_drop(move || {
        if let Some(context) = drop_context {
            context.entries.borrow_mut().remove(&id);
            if context.state.borrow().active == Some(id) {
                context.state.borrow_mut().active = None;
                context.invalidate_focus();
                context.notify();
            }
        }
    });
    let owner = context.as_ref().map(|context| context.node_id());
    let source_active = host_binding.active();
    let active = source_active
        || !host_binding.runtime.item_visual_known()
            && context.as_ref().is_some_and(|context| {
                let _ = (context.revision)();
                let state = context.state.borrow();
                state.active == Some(id)
            });
    let trigger_marker = linked.as_ref().map(DropdownContext::node_id);
    host_binding.describe(
        &host_id,
        &attributes,
        ItemDescription {
            disabled: blocked,
            checked,
            order,
            source_focusable: !blocked,
            linked_trigger: trigger_marker.clone(),
            ..Default::default()
        },
    );
    let linked_open = linked
        .as_ref()
        .map(|linked| (linked.open)())
        .unwrap_or(false);
    let mount_context = context.clone();
    let mounted_host = host_binding.clone();
    let mounted_id = host_id.clone();
    let linked_binding = use_trigger_binding(linked.clone(), blocked, Some(host_id.clone()), true);
    let navigation_host = host_binding.clone();
    let focus_host = host_binding.clone();
    let blur_host = host_binding.clone();
    let focus_context = context.clone();
    let click_label = label.clone();
    let generic = host_binding.generic;
    let action_label = label.clone();
    let action = use_callback(move |event: KeyboardEvent| {
        if disabled
            || !event.default_action_enabled()
            || dropdown
                .and_then(DropdownHandle::try_context)
                .is_some_and(|child| child.state.borrow().blocked)
        {
            return;
        }
        let notification = DropdownItemClickEvent {
            label: action_label.clone(),
            prevented: Rc::new(Cell::new(false)),
        };
        onitemclick.call(notification.clone());
        if generic && (keep_open || notification.default_prevented()) {
            event.prevent_default();
        }
    });
    host_binding.set_keyboard_action(&host_id, action);
    let tabindex = host_binding.tabindex();
    let mut attributes = attributes;
    let consumer_style = attributes.iter().rev().find_map(|attribute| {
        if attribute.name == "style" {
            if let dioxus::core::AttributeValue::Text(value) = &attribute.value {
                return Some(value.clone());
            }
        }
        None
    });
    attributes.retain(|attribute| {
        attribute.name != "style" && (tabindex.is_none() || attribute.name != "tabindex")
    });
    if linked.is_some() {
        attributes =
            dioxus::html::events::EventsExtension::onvisible(attributes, linked_binding.onvisible);
    }
    let host_style = match (consumer_style, linked.as_ref()) {
        (Some(style), Some(child)) => Some(format!(
            "{style};anchor-name:--ix-dropdown-anchor-{}",
            child.id
        )),
        (None, Some(child)) => Some(format!("anchor-name:--ix-dropdown-anchor-{}", child.id)),
        (style, None) => style,
    };
    let class = item_host_class(
        &mut attributes,
        format!(
            "ix-dropdown-item{}{}{}",
            if compact {
                " ix-dropdown-quick-action"
            } else {
                ""
            },
            if active { " ix-focused" } else { "" },
            if linked_open {
                " ix-dropdown-submenu-trigger-active"
            } else {
                ""
            }
        ),
    );
    rsx! {div {
        id:host_id,class,
        role:if checked||checkable{"menuitemcheckbox"}else{"menuitem"},"data-ix-dropdown-item":id.to_string(),
        "data-ix-dropdown-owner":owner,"data-ix-dropdown-label":label.clone(),
        "data-ix-item-kind":"dropdown","data-ix-item-key":host_binding.key.clone(),"data-ix-item-runtime":host_binding.runtime.owner_id(),
        "data-ix-item-focusable":(!blocked).to_string(),"checked":checked,
        style:host_style,"data-ix-overlay-trigger":trigger_marker,
        "data-active":active.to_string(),"aria-disabled":blocked.to_string(),"disabled":blocked,
        tabindex,
        "aria-label":compact.then_some(label.clone()),title:compact.then_some(label.clone()),
        "aria-haspopup":dropdown.map(|_|"menu"),"aria-expanded":dropdown.map(|_|linked_open.to_string()),
        "aria-controls":linked.as_ref().map(DropdownContext::host_id),
        "aria-activedescendant":dropdown.and_then(DropdownHandle::active_descendant),
        "aria-checked":(checked||checkable).then_some(checked.to_string()),
        onmounted:move |event|{mounted_host.mount(&mounted_id,event.data());if let Some(context)=&mount_context{if let Some(entry)=context.entries.borrow_mut().get_mut(&id){entry.mounted=Some(event.data());}}
            linked_binding.onmounted.call(event);},
        // FocusData has no target/relatedTarget. Direct focus/blur avoid
        // a nested item's event repainting its ancestor host.
        onfocus:move |event|{
            if let Some(context)=&focus_context {
                let changed=context.state.borrow().active!=Some(id);
                context.state.borrow_mut().active=Some(id);
                if changed {context.notify();}
            }
            focus_host.focus(!blocked,None);onfocus.call(event);
        },
        onblur:move |event|{blur_host.blur();onblur.call(event);},
        onclick:move |event:MouseEvent| {
            if disabled||dropdown.and_then(DropdownHandle::try_context).is_some_and(|child|child.state.borrow().blocked){event.prevent_default();return;}
            onitemclick.call(DropdownItemClickEvent {label:click_label.clone(),prevented:Rc::new(Cell::new(false))});
            if let Some(handler)=onclick {handler.call(event.clone());}
            if let Some(handle)=dropdown {
                if event.default_action_enabled()&&let Some(context)=handle.try_context(){
                    let previous=*context.open.peek();context.request_from_trigger(!previous,Reason::Trigger,context.keyboard_mode());
                }
                // This explicit child controller owns activation. Keep native
                // defaults while excluding ancestor Dioxus trigger handlers.
                event.stop_propagation();
            }else if keep_open{event.prevent_default();}
        },
        onkeydown:move |event:KeyboardEvent|{
            if blocked || !event.default_action_enabled(){return;}
            if navigation_host.navigation_key(event.clone()){return;}
            if dropdown.is_some(){linked_binding.onkeydown.call(event);}
            else if matches!(event.key(),Key::Enter)||event.key()==Key::Character(" ".into()){
                action.call(event);
            }
        },
        ..attributes,
        if let Some(name)=icon{Icon {name,size:20}}
        if !compact{span {"{label}"}}
        DropdownItemContent {parent_key:host_binding.key.clone(),{children}}
        if checked{Icon {name:"single-check",size:16}}
        if dropdown.is_some(){Icon {name:"chevron-right-small",size:16}}
        overlay_runtime::OverlayTransport {runtime:host_binding.runtime.clone()}
    }}
}

pub(crate) const STYLES: &str = r#"
.ix-theme .ix-select-item{display:block;position:relative}
.ix-theme .ix-select-item>.ix-select-item-option{width:100%;color:var(--ix-select-item--color,var(--ix-color-text))}
.ix-theme .ix-select-item .select-item-checked{background-color:var(--ix-select-item--background--selected,var(--ix-color-surface-2))}
.ix-theme .ix-select-item[disabled]{pointer-events:none;cursor:default}
.ix-dropdown { display:inline-block;position:relative; }

.ix-dropdown-trigger { position:relative;z-index:19; }
.ix-dropdown-trigger { display:flex;align-items:center;gap:12px;padding:6px 12px;min-height:var(--ix-control-height);border:1px solid var(--ix-color-primary);border-radius:var(--ix-radius);color:var(--ix-color-primary);background:transparent; }
.ix-dropdown-menu { color:var(--ix-color-text);font:inherit;margin:0;position:absolute;top:calc(100% + 4px);left:0;z-index:20;min-width:200px;max-height:320px;overflow:auto;padding:4px 0;border:1px solid var(--ix-color-border);border-radius:var(--ix-radius);background:var(--ix-color-surface-1);box-shadow:0 6px 20px #0004; }
.ix-dropdown-external-trigger{position:relative;z-index:19}
.ix-dropdown-root-menu{position:fixed;inset:50% auto auto 50%;transform:translate(-50%,-50%);box-sizing:border-box;width:max-content;min-width:min(200px,calc(100vw - 16px));max-width:calc(100vw - 16px);max-height:min(320px,calc(100dvh - 16px))}
@supports(position-anchor:--example) and (position-area:bottom){
 .ix-dropdown-root-menu{inset:auto;transform:none;position-area:var(--ix-dropdown-area);justify-self:start;align-self:start;margin:4px;position-try-fallbacks:flip-block,flip-inline,flip-block flip-inline,--ix-dropdown-contained}
 .ix-dropdown-root-menu[data-placement^=top]{align-self:end}
 .ix-dropdown-root-menu[data-placement=bottom-end],.ix-dropdown-root-menu[data-placement=top-end],.ix-dropdown-root-menu[data-placement^=left]{justify-self:end}
 .ix-dropdown-root-menu[data-placement=left-end],.ix-dropdown-root-menu[data-placement=right-end]{align-self:end}
 @position-try --ix-dropdown-contained{position-area:none;inset:8px;margin:auto;height:max-content;max-width:calc(100% - 16px);max-height:min(320px,calc(100dvh - 16px))}
}
[data-ix-item-key][hidden]{display:none!important}
.ix-dropdown-item { display:flex;align-items:center;gap:8px;width:100%;padding:8px 16px;border:0;text-align:left;color:var(--ix-color-text);background:transparent; }
.ix-dropdown-item>span { flex:1; }
.ix-dropdown-item.ix-dropdown-quick-action { width:40px;height:40px;justify-content:center;padding:8px; }
.ix-dropdown-item:hover:not(:disabled) { background:var(--ix-color-surface-2); }
.ix-dropdown-item:disabled { opacity:.45; }
.ix-dropdown-header { padding:8px 16px;font-size:14px;color:var(--ix-color-text-secondary); }
.ix-dropdown-submenu-menu{position:fixed;inset:50% auto auto 50%;transform:translate(-50%,-50%);box-sizing:border-box;width:min(200px,calc(100vw - 16px));min-width:0;max-width:calc(100vw - 16px);max-height:min(320px,calc(100dvh - 16px));z-index:21}
.ix-dropdown-submenu-trigger[aria-expanded=true],.ix-dropdown-item.ix-dropdown-submenu-trigger-active{background:var(--ix-color-surface-2)}
@supports (position-anchor:--example) and (position-area:right){
 .ix-dropdown-submenu-menu{inset:auto;transform:none;position-area:right span-bottom;align-self:start;margin:0 4px;position-try-fallbacks:flip-inline,flip-block,flip-inline flip-block,--ix-submenu-contained,--ix-submenu-contained-above}
 @position-try --ix-submenu-contained{position-area:bottom span-right;inset:auto;justify-self:start;align-self:start;margin:4px 0;width:min(200px,calc(100vw - 16px));height:max-content;max-width:calc(100vw - 16px);max-height:min(320px,calc(100dvh - 16px))}
 @position-try --ix-submenu-contained-above{position-area:top span-right;inset:auto;justify-self:start;align-self:end;margin:4px 0;width:min(200px,calc(100vw - 16px));height:max-content;max-width:calc(100vw - 16px);max-height:min(320px,calc(100dvh - 16px))}
}
/* Closed content remains mounted but absent from layout/focus and hit testing. */
.ix-dropdown-menu[hidden]{display:none!important}
.ix-dropdown-menu:not([hidden]):not([popover]){display:block}
.ix-dropdown-menu[popover]:not(:popover-open){display:none}
.ix-dropdown-menu:popover-open{display:block}
.ix-dropdown>.ix-dropdown-trigger[inert]{pointer-events:none}
.ix-dropdown[role=button]:focus-visible,.ix-dropdown-item:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:2px}
.ix-dropdown-item[data-active=true]{background:var(--ix-color-surface-2);outline:2px solid var(--ix-color-focus);outline-offset:-2px}
.ix-dropdown-item[aria-disabled=true]{opacity:.45;pointer-events:none;cursor:default}
.ix-dropdown[aria-disabled=true]{cursor:default}

"#;

#[cfg(test)]
mod tests;
