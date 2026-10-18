//! Persistent Dropdown primitives and trigger-linked overlay ownership.
//!
//! Copyright (c) 2023-2026 Siemens AG. SPDX-License-Identifier: MIT.
use crate::Icon;
use crate::dropdown_inventory::{
    Activation, Inventory, InventoryFacts, InventoryMode, ItemKind, ItemRef,
};
use crate::overlay_registry::{CloseBehavior, OverlayKind};
use crate::overlay_runtime::{
    self, Callbacks, Change, KeyDecision, KeyFacts, KeyRoute, NodeHandle, Reason,
};
use dioxus::prelude::*;
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    rc::Rc,
    sync::atomic::{AtomicUsize, Ordering},
};

static NEXT_ID: AtomicUsize = AtomicUsize::new(1);

static NEXT_SOURCE_ITEM: AtomicUsize = AtomicUsize::new(1);

/// Descriptive host identity is separate from inventory membership. A source
/// light-DOM descendant can be queried by more than one Dropdown.
#[derive(Clone)]
pub(crate) struct ItemHostBinding {
    pub(crate) key: String,
    pub(crate) host_id: String,
    pub(crate) generic: bool,
    pub(crate) runtime: overlay_runtime::RuntimeHandle,
    context: Option<DropdownContext>,
}
impl ItemHostBinding {
    pub(crate) fn mount(&self, target: &str, node: Rc<MountedData>) {
        self.runtime.register_item_mount(&self.key, target, node);
    }
    pub(crate) fn mount_control(&self, target: &str, focus_id: &str, node: Rc<MountedData>) {
        self.runtime
            .register_item_control_mount(&self.key, target, focus_id, node);
    }
    pub(crate) fn active(&self) -> bool {
        self.runtime.item_visual_key().as_deref() == Some(self.key.as_str())
    }
    pub(crate) fn tabindex(&self) -> Option<i32> {
        self.runtime.item_tabindex(&self.key)
    }
    pub(crate) fn navigation_key(&self, event: KeyboardEvent) -> bool {
        if !event.default_action_enabled()
            || !matches!(
                event.key(),
                Key::Home
                    | Key::End
                    | Key::ArrowUp
                    | Key::ArrowDown
                    | Key::ArrowLeft
                    | Key::ArrowRight
            )
        {
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
                || !state.external
                || state.blocked
                || state.settings.disable_focus_handling
                || state.settings.mode != DropdownNavigationMode::ActiveDescendant
                || !context.open.try_peek().is_ok_and(|open| *open)
                || !context.blocked.try_peek().is_ok_and(|blocked| !*blocked)
            {
                return false;
            }
        }
        let Some(node) = context
            .node
            .borrow()
            .clone()
            .filter(NodeHandle::is_registered)
        else {
            return false;
        };
        let Some(inventory) = node.inventory() else {
            return false;
        };
        let Some(host) = inventory.hosts.iter().find(|host| {
            host.registration_key == self.key
                && host.kind != ItemKind::NativeOptIn
                && host.eligible(InventoryMode::ActiveDescendant)
                && node.is_item_registered(host)
        }) else {
            return false;
        };
        let facts = KeyFacts {
            route: KeyRoute::Navigation,
            navigation_mode: Some("active-descendant".into()),
            key: event.key().to_string(),
            shift: event.modifiers().contains(Modifiers::SHIFT),
            alt: event.modifiers().contains(Modifiers::ALT),
            ctrl: event.modifiers().contains(Modifiers::CONTROL),
            meta: event.modifiers().contains(Modifiers::META),
            focused_target: Some(host.target.clone()),
            inside: true,
            inventory: Some(inventory),
            ..Default::default()
        };
        context.state.borrow_mut().native = true;
        let decision = context.key(&facts);
        if decision.prevent {
            event.prevent_default();
        }
        if decision.stop || event.key() == Key::ArrowLeft && !*context.open.peek() {
            event.stop_propagation();
        }
        true
    }
}
pub(crate) fn use_item_host(kind: &'static str) -> ItemHostBinding {
    let runtime = overlay_runtime::use_overlay_runtime();
    let registry = runtime.clone();
    let key = use_hook(move || {
        let key = format!(
            "ix-source-{kind}-{}",
            NEXT_SOURCE_ITEM.fetch_add(1, Ordering::Relaxed)
        );
        let kind = match kind {
            "dropdown" => ItemKind::Dropdown,
            "select" => ItemKind::Select,
            "menu" => ItemKind::Menu,
            "category" => ItemKind::Category,
            "native-opt-in" => ItemKind::NativeOptIn,
            _ => unreachable!("typed source item kind"),
        };
        registry.register_item_cue(key.clone(), kind);
        key
    });
    let cleanup = runtime.clone();
    let dropped = key.clone();
    use_drop(move || cleanup.unregister_item_cue(&dropped));
    let context = try_use_context::<DropdownContext>();
    ItemHostBinding {
        host_id: key.clone(),
        key,
        generic: context.is_some(),
        runtime,
        context,
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
    /// Apply to that element's `tabindex`. `None` preserves its natural tab order.
    pub tabindex: Option<i32>,
}

/// Bind a custom Dioxus control to Dropdown keyboard navigation.
///
/// Apply both returned fields, give the control the same stable `id`, and mark
/// it with `data-ix-roving-item`. Registered buttons and fields participate in
/// opening focus in both navigation modes; active-descendant mode preserves
/// native editing keys once they are focused. Dioxus owns focus, scrolling,
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
///             "data-ix-roving-item": "",
///             onmounted: item.onmounted,
///             tabindex: item.tabindex,
///             "Custom action"
///         }
///     }
/// }
/// ```
pub fn use_dropdown_item(id: impl Into<String>) -> DropdownItemBinding {
    let binding = use_item_host("native-opt-in");
    let tabindex = binding.tabindex();
    let target = id.into();
    let onmounted = use_callback(move |event: MountedEvent| binding.mount(&target, event.data()));
    DropdownItemBinding {
        onmounted,
        tabindex,
    }
}

/// Register opening focus and scrolling only. Use [`use_dropdown_item`] and
/// apply both fields when the Dropdown should also manage roving tab order.
pub fn use_dropdown_item_mount(id: impl Into<String>) -> EventHandler<MountedEvent> {
    use_dropdown_item(id).onmounted
}
/// Preserve own checked content attributes where the interpreter otherwise
/// writes only a native property. The marker carries Rust's literal value;
/// native plumbing copies it without selecting focus or changing selection.
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
/// Source watcher decisions are Rust state. The native bridge only resolves
/// each committed declaration and holds the actual objects it receives.
#[derive(Default)]
struct TriggerDeclaration {
    value: Option<String>,
    version: u64,
    dispose_version: u64,
}
impl TriggerDeclaration {
    fn update(&mut self, value: &Option<String>) {
        if self.value == *value {
            return;
        }
        self.version = self.version.wrapping_add(1).max(1);
        if value.as_ref().is_some_and(|value| !value.is_empty()) {
            self.dispose_version = self.dispose_version.wrapping_add(1).max(1);
        }
        self.value = value.clone();
    }
    fn refresh_mounted(&mut self) -> bool {
        if !self.value.as_ref().is_some_and(|value| !value.is_empty()) {
            return false;
        }
        // The ID may be unchanged while a keyed Dioxus control was replaced.
        // Refresh both resolution and listener ownership for that new mount.
        self.version = self.version.wrapping_add(1).max(1);
        self.dispose_version = self.dispose_version.wrapping_add(1).max(1);
        true
    }
    fn dispose_mounted(&mut self) -> bool {
        if !self.value.as_ref().is_some_and(|value| !value.is_empty()) {
            return false;
        }
        // Removal releases listeners without resolving the now-missing ID.
        // A later explicit mount publishes its own fresh resolution version.
        self.dispose_version = self.dispose_version.wrapping_add(1).max(1);
        true
    }
}
struct MenuState {
    alive: bool,
    surface: bool,
    generation: u64,
    mounted: bool,
    external: bool,
    blocked: bool,
    settings: Settings,
    active: Option<usize>,
    order: Vec<usize>,
    focus_epoch: u64,
    focus_target: String,
    focus_pending: Option<FocusRequest>,
    native: bool,
    forward: Option<DropdownHandle>,
    placement: DropdownPlacement,
    fallback_placement: Option<DropdownPlacement>,
    anchor_in_dropdown_item: bool,
    declaration: TriggerDeclaration,
}
#[derive(Clone)]
struct TriggerMount {
    node: Rc<MountedData>,
    blocked: Rc<Cell<bool>>,
}
#[derive(Clone)]
struct DropdownContext {
    id: usize,
    open: Signal<bool>,
    blocked: Signal<bool>,
    revision: Signal<u64>,
    state: Rc<RefCell<MenuState>>,
    entries: Rc<RefCell<BTreeMap<usize, MenuEntry>>>,
    node: Rc<RefCell<Option<NodeHandle>>>,
    trigger: Rc<RefCell<Option<TriggerMount>>>,
    focus_task: Rc<RefCell<Option<dioxus::core::Task>>>,
}
impl DropdownContext {
    fn native_key_route(&self) -> bool {
        self.state.borrow().native
            || self
                .node
                .borrow()
                .as_ref()
                .is_some_and(NodeHandle::native_observed)
    }
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
        if self.native_key_route() {
            if reactive {
                let _ = self.revision.try_read();
            }
            return self
                .state
                .borrow()
                .forward
                .and_then(DropdownHandle::try_context)
                .filter(|child| {
                    child.id != self.id
                        && child.state.borrow().alive
                        && child.state.borrow().surface
                })
                .unwrap_or_else(|| self.clone());
        }
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
        if let Some(node) = node
            && let Some(facts) = node.inventory()
        {
            let runtime = node.runtime()?;
            let _ = runtime.item_visual_target()?;
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
    fn trigger_click_allowed(&self) -> bool {
        let node = self.node.borrow().clone();
        node.is_none_or(|node| node.trigger_click_allowed())
    }
    fn notify(&self) {
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
    fn mount_trigger(&self, node: Rc<MountedData>, blocked: Rc<Cell<bool>>) {
        let (changed, replaced) = {
            let current = self.trigger.borrow();
            (
                current
                    .as_ref()
                    .is_none_or(|current| !Rc::ptr_eq(&current.node, &node)),
                current
                    .as_ref()
                    .is_some_and(|current| !Rc::ptr_eq(&current.node, &node)),
            )
        };
        if replaced {
            self.invalidate_focus();
        }
        *self.trigger.borrow_mut() = Some(TriggerMount { node, blocked });
        if changed {
            let refresh = self.state.borrow_mut().declaration.refresh_mounted();
            if refresh {
                self.notify();
            }
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
            self.invalidate_focus();
            let dispose = self.state.borrow_mut().declaration.dispose_mounted();
            if dispose {
                self.notify();
            }
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
                node.configure_navigation(
                    state.settings.mode.as_str(),
                    !state.settings.disable_focus_handling,
                );
            } else {
                node.clear_navigation();
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
        let current = inventory.active();
        let mut result = KeyDecision::default();
        if matches!(facts.key.as_str(), "Home" | "End" | "ArrowDown" | "ArrowUp") {
            result.prevent = true;
            if let Some(target) = inventory.navigate(&facts.key, current.as_ref()) {
                self.publish_host(raw, &target);
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
        let Some(handle) = node.first_assigned_submenu() else {
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
        let ordinary = facts.route == KeyRoute::Navigation;
        if !self.state.borrow().alive
            || !ordinary && (facts.default_prevented || self.state.borrow().blocked)
        {
            return result;
        }
        let (open, mode, no_focus, no_trap, anchor_submenu) = {
            let state = self.state.borrow();
            (
                *self.open.peek(),
                state.settings.mode,
                state.settings.disable_focus_handling,
                state.settings.disable_focus_trap,
                state.anchor_in_dropdown_item,
            )
        };
        let opening_key = matches!(
            facts.key.as_str(),
            "Home" | "End" | "ArrowDown" | "ArrowUp" | "Enter" | " "
        );
        if !ordinary && facts.on_trigger && opening_key {
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
            } else if mode == DropdownNavigationMode::RovingTabindex && !facts.alt && !no_focus {
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
            if !open || mode == DropdownNavigationMode::RovingTabindex {
                return result;
            }
        }
        // The experimental listener returns for an already-open AD menu.
        // Ordinary navigation is a separately installed, retained listener.
        if facts.route == KeyRoute::Trigger || !open || (!ordinary && no_focus) {
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
                        result.activate = Some(format!("ix-dropdown-item-{id}"));
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
                external: false,
                blocked: false,
                settings: Settings::default(),
                active: None,
                order: vec![],
                focus_epoch: 0,
                focus_target: "none".into(),
                focus_pending: None,
                native: false,
                forward: None,
                placement: DropdownPlacement::default(),
                fallback_placement: None,
                anchor_in_dropdown_item: false,
                declaration: TriggerDeclaration::default(),
            }))
        }),
        entries: use_hook(|| Rc::new(RefCell::new(BTreeMap::new()))),
        node: use_hook(|| Rc::new(RefCell::new(None))),
        trigger: use_hook(|| Rc::new(RefCell::new(None))),
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

/// Register a custom Dioxus trigger for Dropdown focus restoration.
///
/// Attach this handler to the stable element referenced by `Dropdown.trigger`.
/// Its ID still supplies click/key discovery; this hook supplies the mounted
/// focus handle and releases it with the calling component's scope. Use a new
/// component key when replacing the trigger or its controller. ID-only triggers
/// remain usable for discovery but do not receive programmatic trigger focus.
pub fn use_dropdown_trigger_mount(dropdown: DropdownHandle) -> EventHandler<MountedEvent> {
    use_trigger_mount(dropdown.try_context(), false, None)
}
fn use_trigger_mount(
    context: Option<DropdownContext>,
    disabled: bool,
    focus_id: Option<String>,
) -> EventHandler<MountedEvent> {
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
    let cleanup = owned.clone();
    use_drop(move || {
        if let Some((context, node)) = cleanup.borrow_mut().take() {
            context.unmount_trigger(&node);
        }
    });
    use_callback(move |event: MountedEvent| {
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
                    .declaration
                    .value
                    .clone()
                    .unwrap_or_else(|| context.trigger_id())
            });
            focus_binding.mount(&target, node.clone());
            context.mount_trigger(node.clone(), blocked.clone());
            *owned.borrow_mut() = Some((context.clone(), node));
        }
    })
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
    let mount = use_trigger_mount(Some(context.clone()), blocked, Some(context.trigger_id()));
    let click = context.clone();
    let key = context.clone();
    let id = context.node_id();
    let trigger_id = context.trigger_id();
    let host_id = context.host_id();
    let active_id = if context.state.borrow().settings.mode
        == DropdownNavigationMode::ActiveDescendant
        && dropdown.is_open()
    {
        context.active_dom_id()
    } else {
        None
    };
    rsx! { button {
        id:trigger_id, class:"ix-dropdown-external-trigger ix-button ix-button-{variant.as_str()} {class}",
        r#type:"button",disabled:blocked,"aria-label":aria_label,"aria-busy":loading.to_string(),
        "aria-haspopup":"menu","aria-expanded":dropdown.is_open().to_string(),"aria-controls":host_id,"aria-activedescendant":active_id,
        "data-ix-overlay-trigger":id,style:"anchor-name:--ix-dropdown-anchor-{context.id}",
        onmounted:mount,
        onclick:move |event:MouseEvent| {if !blocked&&!click.state.borrow().blocked&&event.default_action_enabled()&&click.trigger_click_allowed(){let previous=*click.open.peek();click.request_from_trigger(!previous,Reason::Trigger,click.keyboard_mode());}},
        onkeydown:move |event:KeyboardEvent| {if !key.native_key_route(){fallback_key(&key,event,true);}},
        if loading {crate::Spinner {size:crate::SpinnerSize::Sm,hide_track:true}}
        else if let Some(icon)=icon {Icon {name:icon,size:20}}
        {children}
    } }
}

/// Persistent source-default Dropdown. `controller` or `trigger` omits the
/// built-in trigger; use DropdownTrigger, a linked DropdownItem or an existing
/// element ID. Ordinary native click ancestry is preserved by the built-in trigger surrounding its
/// surface. Children keep their scopes and native nodes while closed.
#[component]
pub fn Dropdown(
    label: String,
    controller: Option<DropdownHandle>,
    /// ID of an existing native trigger, matching the source string reference.
    /// The actual host remains the trigger; no wrapper is inserted.
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
        state.blocked = blocked;
        state.placement = placement;
        state.declaration.update(&trigger);
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
                    key: Some(Rc::new(move |facts| {
                        key.state.borrow_mut().native = true;
                        key.key(facts)
                    })),
                },
            );
            *context.node.borrow_mut() = Some(node.clone());
            let query = context.clone();
            let reset = context.clone();
            let trigger_click = context.clone();
            node.set_submenu_access(
                handle,
                Rc::new(move || Some(query.navigation_owner(false).node_id())),
                Rc::new(move || {
                    reset.state.borrow_mut().forward = None;
                    reset.notify();
                }),
                Rc::new(move || {
                    if !trigger_click.state.borrow().blocked {
                        let previous = *trigger_click.open.peek();
                        trigger_click.request_from_trigger(
                            !previous,
                            Reason::Trigger,
                            trigger_click.keyboard_mode(),
                        );
                    }
                }),
            );
            let projection = context.clone();
            node.set_item_visual_reader(Rc::new(move || {
                let active = projection.active_dom_id();
                let aria = (projection.state.borrow().settings.mode
                    == DropdownNavigationMode::ActiveDescendant
                    && *projection.open.peek())
                .then(|| active.clone())
                .flatten();
                overlay_runtime::RootVisual { active, aria }
            }));
            let shape_context = context.clone();
            node.set_anchor_shape_handler(Rc::new(move |in_dropdown_item| {
                let changed = {
                    let mut state = shape_context.state.borrow_mut();
                    if !state.alive || !state.surface || state.generation != generation {
                        return;
                    }
                    let changed = state.anchor_in_dropdown_item != in_dropdown_item;
                    state.anchor_in_dropdown_item = in_dropdown_item;
                    changed
                };
                if changed {
                    shape_context.notify();
                }
            }));
            let intersection_context = context.clone();
            node.set_anchor_intersection_handler(Rc::new(move |geometry| {
                let changed = {
                    let mut state = intersection_context.state.borrow_mut();
                    if !state.alive || !state.surface || state.generation != generation {
                        return;
                    }
                    let fallback = state.placement.clipped(geometry);
                    let changed = state.fallback_placement != fallback
                        || state.anchor_in_dropdown_item != geometry.anchor_in_dropdown_item;
                    state.fallback_placement = fallback;
                    state.anchor_in_dropdown_item = geometry.anchor_in_dropdown_item;
                    changed
                };
                if changed {
                    intersection_context.notify();
                }
            }));
            node
        }
    });
    node.set_close_behavior(settings.behavior.into());
    node.set_trigger_visibility_check(!settings.suppress_trigger_visibility_check);
    node.set_host_click_receivers(enable_top_layer, true);
    use_context_provider(|| context.clone());
    let drop_context = context.clone();
    let drop_node = node.clone();
    use_drop(move || {
        {
            let mut state = drop_context.state.borrow_mut();
            state.surface = false;
            state.mounted = false;
            state.focus_epoch = state.focus_epoch.wrapping_add(1);
            state.focus_pending = None;
            state.native = false;
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
                    configure_node.configure_navigation(
                        mode.as_str(),
                        !configure.state.borrow().settings.disable_focus_handling,
                    );
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
    let state = context.state.borrow();
    let generation = state.generation;
    let trigger_version =
        (state.declaration.version > 0).then(|| state.declaration.version.to_string());
    let trigger_dispose_version = state.declaration.dispose_version.to_string();
    let effective_placement = if state.anchor_in_dropdown_item {
        DropdownPlacement::RightStart
    } else if !settings.suppress_trigger_visibility_check {
        state.fallback_placement.unwrap_or(placement)
    } else {
        placement
    };
    drop(state);
    let navigation_owner = context.navigation_owner(true);
    let items_owner = navigation_owner.node_id();
    let active_id = context.active_dom_id();
    let id = context.node_id();
    let host_id = context.host_id();
    let trigger_id = context.trigger_id();
    let aria_name = aria_label.unwrap_or_else(|| label.clone());
    let mount_context = context.clone();
    let mounted_node = node.clone();
    let mount_trigger = use_trigger_mount(
        (!external).then(|| context.clone()),
        blocked,
        Some(trigger_id.clone()),
    );
    let click_context = context.clone();
    let key_context = context.clone();
    let runtime_owner = runtime.owner_id();
    let root_role = (!external).then_some("button");
    let trigger_marker = (!external).then_some(id.clone());
    let focus_retention = use_dropdown_focus_retention(
        context.host_id(),
        settings.mode == DropdownNavigationMode::RovingTabindex,
    );
    let blur_retention = focus_retention.clone();
    rsx! { div {
        id:if external {format!("ix-dropdown-surface-owner-{}",context.id)}else{trigger_id},
        class:"ix-dropdown {class}",role:root_role,tabindex:(!external).then_some(if blocked {-1}else{0}),
        "data-ix-overlay-trigger-id":trigger,"data-ix-overlay-anchor-name":format!("--ix-dropdown-anchor-{}",context.id),
        "data-ix-overlay-trigger-version":trigger_version,"data-ix-overlay-trigger-dispose-version":trigger_dispose_version,
        "aria-label":(!external).then_some(aria_name.clone()),"aria-disabled":(!external).then_some(blocked.to_string()),"aria-busy":(!external).then_some(loading.to_string()),
        "aria-haspopup":(!external).then_some("menu"),"aria-expanded":(!external).then_some(open.to_string()),
        "aria-controls":(!external).then_some(host_id.clone()),
        "aria-activedescendant":if settings.mode==DropdownNavigationMode::ActiveDescendant&&open{active_id.clone()}else{None},
        "data-ix-overlay-root":id.clone(),"data-ix-overlay-owner":runtime_owner,"data-ix-overlay-trigger":trigger_marker,
        "data-ix-overlay-generation":generation.to_string(),"data-ix-overlay-open":open.to_string(),
        "data-ix-dropdown-focus-owned":"true",
        "data-ix-overlay-revision":node.render_revision().to_string(),
        "data-ix-overlay-top-layer":enable_top_layer.to_string(),"data-ix-overlay-mode":settings.mode.as_str(),
        "data-ix-overlay-trigger-visibility-check":(!settings.suppress_trigger_visibility_check).to_string(),
        "data-ix-overlay-focus-proxy":(settings.mode==DropdownNavigationMode::ActiveDescendant).to_string(),
        "data-ix-overlay-items-owner":items_owner,"data-ix-overlay-active-item":active_id,
        style:if external{format!("--ix-dropdown-area:{}",effective_placement.area())}else{format!("anchor-name:--ix-dropdown-anchor-{};--ix-dropdown-area:{}",context.id,effective_placement.area())},
        onmounted:move |event|{mount_context.state.borrow_mut().mounted=true;mount_trigger.call(event.clone());mounted_node.sync_open(*mount_context.open.peek());let pending=mount_context.state.borrow().focus_pending;if *mount_context.open.peek(){let state=mount_context.state.borrow();mounted_node.configure_navigation(state.settings.mode.as_str(),!state.settings.disable_focus_handling);drop(state);if let Some(last)=pending{mount_context.focus_edge_with_checked(last.last,last.allow_checked);}}},
        onclick:move |event:MouseEvent| {if !external&&!blocked&&!click_context.state.borrow().blocked&&event.default_action_enabled()&&click_context.trigger_click_allowed(){let previous=*click_context.open.peek();click_context.request_from_trigger(!previous,Reason::Trigger,click_context.keyboard_mode());}},
        onkeydown:move |event:KeyboardEvent|{if !key_context.native_key_route(){fallback_key(&key_context,event,!external);}},
        if !external {button {class:"ix-dropdown-trigger ix-button ix-button-{variant.as_str()}",r#type:"button",disabled:blocked,"aria-hidden":"true",inert:true,tabindex:-1,
            if loading {crate::Spinner {size:crate::SpinnerSize::Sm,hide_track:true}}
            else if let Some(icon)=icon {Icon {name:icon,size:20}}
            "{label}" Icon {name:"chevron-down-small",size:16}
        }}
        dialog {id:host_id,class:"ix-dropdown-menu ix-dropdown-root-menu",role:"menu","aria-label":label,
            "data-ix-overlay-host":id,"data-placement":effective_placement.as_str(),"data-configured-placement":placement.as_str(),hidden:!open,inert:(!open).then_some(""),
            popover:enable_top_layer.then_some("manual"),
            style:format!("position-anchor:--ix-dropdown-anchor-{};--ix-dropdown-area:{};{}",context.id,effective_placement.area(),if suppress_automatic_placement {"position-try-fallbacks:none"} else {""}),
            onfocusin:move |_|{if let Some(transport)=focus_retention.borrow().as_ref(){let _=transport.send("focus");}},
            onfocusout:move |_|{if let Some(transport)=blur_retention.borrow().as_ref(){let _=transport.send("blur");}},
            {children}
        }
        overlay_runtime::OverlayTransport {runtime}
    } }
}

// The fallback is for renderer-only/SSR dispatch and the interval before native
// registration. Actual native navigation uses one raw shared-runtime report.
fn fallback_key(context: &DropdownContext, event: KeyboardEvent, on_trigger: bool) {
    let owner = context.navigation_owner(false);
    let order = owner.entries.borrow().keys().copied().collect::<Vec<_>>();
    let facts = KeyFacts {
        key: event.key().to_string(),
        shift: event.modifiers().contains(Modifiers::SHIFT),
        alt: event.modifiers().contains(Modifiers::ALT),
        ctrl: event.modifiers().contains(Modifiers::CONTROL),
        meta: event.modifiers().contains(Modifiers::META),
        default_prevented: !event.default_action_enabled(),
        focused_target: None,
        focused_item: None,
        items: order,
        focus_count: 0,
        focusable_ids: Vec::new(),
        focused_index: None,
        on_trigger,
        inside: !on_trigger,
        popover_ancestor: false,
        inventory: None,
        route: KeyRoute::Legacy,
        navigation_mode: None,
    };
    if event.key() == Key::Escape {
        let node = context.node.borrow().clone();
        if let Some(node) = node {
            if let Some(runtime) = node.runtime() {
                runtime.dismiss_root(node.id(), Reason::Escape, true);
                event.stop_propagation();
            }
        }
        return;
    }
    let decision = context.key(&facts);
    if decision.prevent {
        event.prevent_default();
    }
    if decision.stop {
        event.stop_propagation();
    }
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
    #[props(default)] navigation_mode: DropdownNavigationMode,
    #[props(default)] close_behavior: DropdownCloseBehavior,
    #[props(default)] enable_top_layer: bool,
    #[props(default)] onbeforechange: EventHandler<DropdownChangeEvent>,
    #[props(default)] onchange: EventHandler<bool>,
    children: Element,
) -> Element {
    let dropdown = use_dropdown();
    rsx! {div {class:"ix-dropdown-submenu",
        DropdownItem {label:label.clone(),icon,disabled,dropdown}
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
    #[props(default)] checked: bool,
    #[props(default)] checkable: bool,
    #[props(default)] keep_open: bool,
    #[props(default)] dropdown: Option<DropdownHandle>,
    #[props(default)] onclick: Option<EventHandler<MouseEvent>>,
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
    let active = if context
        .as_ref()
        .is_some_and(|context| context.state.borrow().native)
    {
        host_binding.active()
    } else {
        context.as_ref().is_some_and(|context| {
            let _ = (context.revision)();
            context.state.borrow().active == Some(id)
        })
    };
    let trigger_marker = linked.as_ref().map(DropdownContext::node_id);
    let linked_open = linked
        .as_ref()
        .map(|linked| (linked.open)())
        .unwrap_or(false);
    let mount_context = context.clone();
    let mounted_host = host_binding.clone();
    let mounted_id = host_id.clone();
    let linked_mount = use_trigger_mount(linked.clone(), blocked, Some(host_id.clone()));
    let key_context = context.clone();
    let navigation_host = host_binding.clone();
    let click_label = label.clone();
    let key_label = label.clone();
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
    let host_style = match (consumer_style, linked.as_ref()) {
        (Some(style), Some(child)) => Some(format!(
            "{style};anchor-name:--ix-dropdown-anchor-{}",
            child.id
        )),
        (None, Some(child)) => Some(format!("anchor-name:--ix-dropdown-anchor-{}", child.id)),
        (style, None) => style,
    };
    rsx! {div {
        id:host_id,class:if compact{"ix-dropdown-item ix-dropdown-quick-action"}else{"ix-dropdown-item"},
        role:if checked||checkable{"menuitemcheckbox"}else{"menuitem"},"data-ix-dropdown-item":id.to_string(),
        "data-ix-dropdown-owner":owner,"data-ix-dropdown-label":label.clone(),
        "data-ix-item-kind":"dropdown","data-ix-item-key":host_binding.key,"data-ix-item-runtime":host_binding.runtime.owner_id(),
        "data-ix-item-focusable":(!blocked).to_string(),"checked":checked,
        "data-ix-reflect-checked":serde_json::to_string(&checked.then_some("")).unwrap_or_default(),
        style:host_style,"data-ix-overlay-trigger":trigger_marker,
        "data-active":active.to_string(),"aria-disabled":blocked.to_string(),"disabled":blocked,
        tabindex,
        "aria-label":compact.then_some(label.clone()),title:compact.then_some(label.clone()),
        "aria-haspopup":dropdown.map(|_|"menu"),"aria-expanded":dropdown.map(|_|linked_open.to_string()),
        "aria-controls":linked.as_ref().map(DropdownContext::host_id),
        "aria-checked":(checked||checkable).then_some(checked.to_string()),
        onmounted:move |event|{mounted_host.mount(&mounted_id,event.data());if let Some(context)=&mount_context{if let Some(entry)=context.entries.borrow_mut().get_mut(&id){entry.mounted=Some(event.data());}}
            linked_mount.call(event);},
        onclick:move |event:MouseEvent| {
            if disabled||dropdown.and_then(DropdownHandle::try_context).is_some_and(|child|child.state.borrow().blocked){event.prevent_default();return;}
            onitemclick.call(DropdownItemClickEvent {label:click_label.clone(),prevented:Rc::new(Cell::new(false))});
            if let Some(handler)=onclick {handler.call(event.clone());}
            if let Some(handle)=dropdown {
                if event.default_action_enabled()&&let Some(context)=handle.try_context().filter(|context|context.trigger_click_allowed()){
                    let previous=*context.open.peek();context.request_from_trigger(!previous,Reason::Trigger,context.keyboard_mode());
                }
                // Foreign-trigger protection is separate from child visibility
                // veto, and prevents the composed ancestor from also toggling.
                event.prevent_default();
            }else if keep_open{event.prevent_default();}
        },
        onkeydown:move |event:KeyboardEvent|{if !disabled&&!dropdown.and_then(DropdownHandle::try_context).is_some_and(|child|child.state.borrow().blocked) {
            if navigation_host.navigation_key(event.clone()){return;}
            if event.key()==Key::Enter||event.key()==Key::Character(" ".into()){onitemclick.call(DropdownItemClickEvent{label:key_label.clone(),prevented:Rc::new(Cell::new(false))});}
            if let Some(context)=&key_context{if !context.native_key_route(){fallback_key(context,event.clone(),false);event.stop_propagation();}}
        }},
        ..attributes,
        if let Some(name)=icon{Icon {name,size:20}}
        if !compact{span {"{label}"}}
        {children}
        if checked{Icon {name:"single-check",size:16}}
        if dropdown.is_some(){Icon {name:"chevron-right-small",size:16}}
        overlay_runtime::OverlayTransport {runtime:host_binding.runtime.clone()}
    }}
}

// Dioxus alpha.2's keyed insertBefore moves can blur an element whose
// MountedData survives. MountedData can set focus, but neither it nor the
// cross-target FocusData exposes activeElement/relatedTarget or a DOM-move
// notification (dioxus-html events/{mounted,focus}.rs). Keep this narrow native
// repair until the renderer preserves focus across moves: Dioxus owns focus
// event dispatch and scope cleanup; observation is limited to this menu.
fn use_dropdown_focus_retention(
    menu_id: String,
    enabled: bool,
) -> Rc<RefCell<Option<dioxus::document::Eval>>> {
    let transport = use_hook(|| Rc::new(RefCell::new(None::<dioxus::document::Eval>)));
    let dropped = transport.clone();
    use_drop(move || {
        if let Some(task) = dropped.borrow_mut().take() {
            let _ = task.send("finish");
        }
    });
    let effect_transport = transport.clone();
    use_effect(use_reactive(
        (&menu_id, &enabled),
        move |(menu_id, enabled)| {
            if let Some(previous) = effect_transport.borrow_mut().take() {
                let _ = previous.send("finish");
            }
            if !enabled {
                return;
            }
            let task = dioxus::document::eval(DROPDOWN_FOCUS_RETENTION);
            if task.send(menu_id).is_ok() {
                *effect_transport.borrow_mut() = Some(task);
            }
        },
    ));
    transport
}

const DROPDOWN_FOCUS_RETENTION: &str = r#"
const menu = document.getElementById(await dioxus.recv());
if (!menu) return;
const owned = target => {
  const item = target instanceof Element ? target.closest('[data-ix-dropdown-item]') : null;
  return item?.closest('[role="menu"]') === menu ? item : null;
};
let focused = owned(document.activeElement);
const clear = () => { focused = null; };
const fallback = () => document.activeElement === document.body || document.activeElement === menu;
const contains = (nodes, target) => [...nodes].some(node => node === target || node.contains(target));
const changed = records => {
  const target = focused;
  if (!menu.isConnected || !target?.isConnected || owned(target) !== target ||
      target.disabled || target.hasAttribute('disabled') || !fallback()) return;
  if (records.some(record => contains(record.removedNodes, target)) &&
      records.some(record => contains(record.addedNodes, target))) target.focus({preventScroll: true});
};
const observer = new MutationObserver(changed);
observer.observe(menu, {childList: true, subtree: true});
// Window deactivation is not exposed by Dioxus's element FocusData.
window.addEventListener('blur', clear);
try {
  for (;;) {
    const event = await dioxus.recv();
    if (event === 'finish') break;
    if (event === 'focus') focused = owned(document.activeElement);
    else if (event === 'blur' && focused) {
      // A keyed move and its blur share the renderer's mutation batch. Repair
      // that move before clearing an explicit blur, without scheduling focus.
      changed(observer.takeRecords());
      focused = fallback() ? null : owned(document.activeElement);
    }
  }
} finally {
  observer.disconnect();
  window.removeEventListener('blur', clear);
}
"#;

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
.ix-dropdown-item { display:flex;align-items:center;gap:8px;width:100%;padding:8px 16px;border:0;text-align:left;color:var(--ix-color-text);background:transparent; }
.ix-dropdown-item>span { flex:1; }
.ix-dropdown-item.ix-dropdown-quick-action { width:40px;height:40px;justify-content:center;padding:8px; }
.ix-dropdown-item:hover:not(:disabled) { background:var(--ix-color-surface-2); }
.ix-dropdown-item:disabled { opacity:.45; }
.ix-dropdown-header { padding:8px 16px;font-size:14px;color:var(--ix-color-text-secondary); }
.ix-dropdown-submenu-menu{position:fixed;inset:50% auto auto 50%;transform:translate(-50%,-50%);box-sizing:border-box;width:min(200px,calc(100vw - 16px));min-width:0;max-width:calc(100vw - 16px);max-height:min(320px,calc(100dvh - 16px));z-index:21}
.ix-dropdown-submenu-trigger[aria-expanded=true]{background:var(--ix-color-surface-2)}
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
mod tests {
    use super::*;
    struct TestMouse;
    impl dioxus::html::InteractionLocation for TestMouse {
        fn client_coordinates(&self) -> dioxus::html::geometry::ClientPoint {
            Default::default()
        }
        fn page_coordinates(&self) -> dioxus::html::geometry::PagePoint {
            Default::default()
        }
        fn screen_coordinates(&self) -> dioxus::html::geometry::ScreenPoint {
            Default::default()
        }
    }
    impl dioxus::html::InteractionElementOffset for TestMouse {
        fn element_coordinates(&self) -> dioxus::html::geometry::ElementPoint {
            Default::default()
        }
    }
    impl dioxus::html::ModifiersInteraction for TestMouse {
        fn modifiers(&self) -> Modifiers {
            Modifiers::empty()
        }
    }
    impl dioxus::html::PointerInteraction for TestMouse {
        fn trigger_button(&self) -> Option<dioxus::html::input_data::MouseButton> {
            None
        }
        fn held_buttons(&self) -> dioxus::html::input_data::MouseButtonSet {
            Default::default()
        }
    }
    impl dioxus::html::HasMouseData for TestMouse {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }
    struct TestForm(String);
    impl dioxus::html::HasFileData for TestForm {
        fn files(&self) -> Vec<dioxus::html::FileData> {
            Vec::new()
        }
    }
    impl dioxus::html::HasFormData for TestForm {
        fn value(&self) -> String {
            self.0.clone()
        }
        fn valid(&self) -> bool {
            true
        }
        fn values(&self) -> Vec<(String, dioxus::html::FormValue)> {
            Vec::new()
        }
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }
    struct TestKeyboard(Key, Modifiers);
    impl dioxus::html::ModifiersInteraction for TestKeyboard {
        fn modifiers(&self) -> Modifiers {
            self.1
        }
    }
    impl dioxus::html::HasKeyboardData for TestKeyboard {
        fn key(&self) -> Key {
            self.0.clone()
        }
        fn code(&self) -> dioxus::html::Code {
            dioxus::html::Code::Unidentified
        }
        fn location(&self) -> dioxus::html::Location {
            dioxus::html::Location::Standard
        }
        fn is_auto_repeating(&self) -> bool {
            false
        }
        fn is_composing(&self) -> bool {
            false
        }
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }
    struct TestFocus;
    impl dioxus::html::HasFocusData for TestFocus {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    struct Converter;
    macro_rules! converter {
        (enum Event { $(#[convert = $convert:ident] #[events = [$($events:tt)*]] $(#[raw = [$($raw:ident),* $(,)?]])? $group:ident($data:ident),)* }) => {
            impl dioxus::html::HtmlEventConverter for Converter {
                $(fn $convert(&self,event:&dioxus::html::PlatformEventData)->dioxus::html::$data {let _=event;converter!(@data $group,event)})*
            }
        };
        (@data Mouse,$event:ident)=>{dioxus::html::MouseData::new(TestMouse)};
        (@data Keyboard,$event:ident)=>{{let (key,modifiers)=$event.downcast::<(Key,Modifiers)>().unwrap();dioxus::html::KeyboardData::new(TestKeyboard(key.clone(),*modifiers))}};
        (@data Form,$event:ident)=>{dioxus::html::FormData::new(TestForm($event.downcast::<String>().unwrap().clone()))};
        (@data Focus,$event:ident)=>{dioxus::html::FocusData::new(TestFocus)};
        (@data Mounted,$event:ident)=>{if let Some(mock)=$event.downcast::<TriggerFocusMount>() {MountedData::new(mock.clone())}else{MountedData::new(())}};
        (@data $other:ident,$event:ident)=>{panic!("Unexpected Dropdown test event")};
    }
    dioxus::html::with_html_event_groups!(converter);
    fn dispatch(
        dom: &VirtualDom,
        name: &str,
        id: dioxus::core::ElementId,
        payload: impl std::any::Any,
    ) -> (bool, bool) {
        let data: Rc<dyn std::any::Any> =
            Rc::new(dioxus::html::PlatformEventData::new(Box::new(payload)));
        let event = Event::new(data, name != "mounted");
        dom.runtime().handle_event(name, event.clone(), id);
        (!event.default_action_enabled(), event.propagates())
    }
    #[derive(Clone)]
    struct Fixture {
        context: Rc<RefCell<Option<DropdownContext>>>,
        linked: Rc<Cell<Option<DropdownHandle>>>,
        update: Option<std::sync::Arc<dyn Fn() + Send + Sync>>,
        visible: bool,
        mode: DropdownNavigationMode,
        layer: bool,
        blocked: bool,
        loading: bool,
        linked_blocked: bool,
        focus_checked: bool,
        no_focus: bool,
        no_trap: bool,
        alpha_id: Option<String>,
        placement: DropdownPlacement,
        suppress_visibility: bool,
        suppress_automatic: bool,
        linked_veto: bool,
        deep_veto: bool,
        veto_open: bool,
        veto_close: bool,
        close_after_open: bool,
        requests: Vec<(bool, bool)>,
        changed: Vec<bool>,
        listener_revision: usize,
        deliveries: Vec<(usize, bool, bool)>,
        item_requests: usize,
        item_veto: bool,
        standalone_requests: usize,
        standalone_mouse: usize,
        pending_drop: Option<Rc<Cell<bool>>>,
        pending_started: Rc<Cell<bool>>,
        native_clicks: usize,
        mounts: usize,
        drops: usize,
    }
    impl Default for Fixture {
        fn default() -> Self {
            Self {
                context: Rc::new(RefCell::new(None)),
                linked: Rc::new(Cell::new(None)),
                update: None,
                visible: true,
                mode: DropdownNavigationMode::default(),
                layer: false,
                blocked: false,
                loading: false,
                linked_blocked: false,
                focus_checked: false,
                no_focus: false,
                no_trap: false,
                alpha_id: None,
                placement: DropdownPlacement::default(),
                suppress_visibility: false,
                suppress_automatic: false,
                linked_veto: false,
                deep_veto: false,
                veto_open: false,
                veto_close: false,
                close_after_open: false,
                requests: vec![],
                changed: vec![],
                listener_revision: 0,
                deliveries: vec![],
                item_requests: 0,
                item_veto: false,
                standalone_requests: 0,
                standalone_mouse: 0,
                pending_drop: None,
                pending_started: Rc::new(Cell::new(false)),
                native_clicks: 0,
                mounts: 0,
                drops: 0,
            }
        }
    }
    impl PartialEq for Fixture {
        fn eq(&self, other: &Self) -> bool {
            Rc::ptr_eq(&self.context, &other.context)
        }
    }
    struct PendingRead(Rc<Cell<bool>>);
    impl Drop for PendingRead {
        fn drop(&mut self) {
            self.0.set(true);
        }
    }
    #[component]
    fn Stateful(config: Rc<RefCell<Fixture>>) -> Element {
        let context = use_context::<DropdownContext>();
        *config.borrow().context.borrow_mut() = Some(context.clone());
        use_hook(|| config.borrow_mut().mounts += 1);
        use_hook(|| {
            if let Some(flag) = config.borrow().pending_drop.clone() {
                let started = config.borrow().pending_started.clone();
                let task = spawn(async move {
                    let _read = PendingRead(flag);
                    started.set(true);
                    std::future::pending::<()>().await;
                });
                *context.focus_task.borrow_mut() = Some(task);
            }
        });
        let dropped = config.clone();
        use_drop(move || dropped.borrow_mut().drops += 1);
        let mut count = use_signal(|| 0_usize);
        rsx! {div{input {value:"uncontrolled initial"}button{r#type:"button","data-count-button":true.to_string(),onclick:move |_|count+=1,"Local {count}"}}}
    }
    fn fixture(config: Rc<RefCell<Fixture>>) -> Element {
        let settings = config.borrow().clone();
        config.borrow_mut().update = Some(dioxus::core::schedule_update());
        let linked = use_dropdown();
        settings.linked.set(Some(linked));
        let before = config.clone();
        let changed = config.clone();
        let item = config.clone();
        let click = config.clone();
        let standalone_item = config.clone();
        let standalone_mouse = config.clone();
        let linked_before = config.clone();
        let deep_before = config.clone();
        rsx! {div {
            if settings.visible {
            Dropdown {label:"Menu",navigation_mode:settings.mode,enable_top_layer:settings.layer,disabled:settings.blocked,loading:settings.loading,focus_checked_item:settings.focus_checked,disable_focus_handling:settings.no_focus,disable_focus_trap:settings.no_trap,placement:settings.placement,suppress_trigger_visibility_check:settings.suppress_visibility,suppress_automatic_placement:settings.suppress_automatic,
                onbeforechange:move |event:DropdownChangeEvent|{let veto={let mut state=before.borrow_mut();state.requests.push((event.open,event.previous));if event.open{state.veto_open}else{state.veto_close}};if veto{event.prevent_default();}},
                onchange:move |open:bool|{let (close,capture)={let mut state=changed.borrow_mut();state.changed.push(open);(state.close_after_open&&open,state.context.clone())};let context=capture.borrow().clone();if let Some(context)=context{changed.borrow_mut().deliveries.push((settings.listener_revision,open,*context.open.peek()));if close{context.request(false,Reason::Trigger,false);}}},
                Stateful {config:config.clone()}
                DropdownItem {label:"Alpha",id:settings.alpha_id.clone(),onitemclick:move |event:DropdownItemClickEvent|{let veto={let mut state=item.borrow_mut();state.item_requests+=1;state.item_veto};if veto{event.prevent_default();}},onclick:move |_|click.borrow_mut().native_clicks+=1}
                DropdownItem {label:"Disabled",disabled:true}
                DropdownItem {label:"Linked",dropdown:linked,checked:true}
                DropdownSubmenu {label:"Explicit",DropdownItem {label:"Persistent nested"}}
            }
            DropdownTrigger {dropdown:linked,aria_label:"External trigger","External trigger"}
            Dropdown {label:"Detached",controller:linked,disabled:settings.linked_blocked,focus_checked_item:true,onbeforechange:move |event:DropdownChangeEvent|{if event.open&&linked_before.borrow().linked_veto{event.prevent_default();}},DropdownItem {label:"Detached leaf"} DropdownItem {label:"Checked detached",checked:true} DropdownSubmenu {label:"Deep",onbeforechange:move |event:DropdownChangeEvent|{if event.open&&deep_before.borrow().deep_veto{event.prevent_default();}},DropdownItem {label:"Deep first"} DropdownItem {label:"Deep last"}}}
            DropdownItem {label:"Standalone",onitemclick:move |event:DropdownItemClickEvent|{let veto={let mut state=standalone_item.borrow_mut();state.standalone_requests+=1;state.item_veto};if veto{event.prevent_default();}},onclick:move |_|standalone_mouse.borrow_mut().standalone_mouse+=1}
            }
        }}
    }
    struct Harness {
        dom: VirtualDom,
        config: Rc<RefCell<Fixture>>,
        attrs: BTreeMap<usize, BTreeMap<String, String>>,
        listeners: BTreeMap<String, Vec<dioxus::core::ElementId>>,
    }
    impl Harness {
        fn new() -> Self {
            Self::with_settings(Fixture::default())
        }
        fn with_settings(settings: Fixture) -> Self {
            Self::with_document(settings, None)
        }
        fn with_document(
            settings: Fixture,
            document: Option<Rc<dyn dioxus::document::Document>>,
        ) -> Self {
            dioxus::html::set_event_converter(Box::new(Converter));
            let config = Rc::new(RefCell::new(settings));
            let mut dom = VirtualDom::new_with_props(fixture, config.clone());
            if let Some(document) = document {
                dom = dom.with_root_context(document);
            }
            let edits = dom.rebuild_to_vec();
            let mut result = Self {
                dom,
                config,
                attrs: BTreeMap::new(),
                listeners: BTreeMap::new(),
            };
            result.collect(edits);
            let mounted = result.listeners.get("mounted").cloned().unwrap_or_default();
            for id in mounted {
                dispatch(&result.dom, "mounted", id, ());
            }
            result.settle();
            result
        }
        fn collect(&mut self, mutations: dioxus::core::Mutations) {
            let mut current = dioxus::core::ElementId::ROOT;
            for edit in mutations.edits {
                match edit {
                    dioxus::core::Mutation::SetId { id }
                    | dioxus::core::Mutation::PushId { id } => current = id,
                    dioxus::core::Mutation::SetAttribute { name, value, .. } => {
                        let text = match value {
                            dioxus::core::AttributeValue::Text(value) => value,
                            dioxus::core::AttributeValue::Bool(value) => value.to_string(),
                            dioxus::core::AttributeValue::Int(value) => value.to_string(),
                            dioxus::core::AttributeValue::None => {
                                if let Some(attrs) = self.attrs.get_mut(&current.raw()) {
                                    attrs.remove(&name);
                                }
                                continue;
                            }
                            _ => continue,
                        };
                        self.attrs
                            .entry(current.raw())
                            .or_default()
                            .insert(name, text);
                    }
                    dioxus::core::Mutation::NewEventListener { name } => {
                        self.listeners.entry(name).or_default().push(current)
                    }
                    _ => {}
                }
            }
        }
        fn render_now(&mut self) {
            for _ in 0..4 {
                let edits = self.dom.render_immediate_to_vec();
                self.collect(edits);
            }
        }
        /// Run the real framework effects after rendering component updates.
        fn completion_effects(&mut self) {
            self.render_now();
        }
        fn settle(&mut self) {
            for _ in 0..4 {
                self.render_now();
                self.completion_effects();
            }
        }
        fn update(&mut self) {
            let update = self.config.borrow().update.clone().unwrap();
            update();
            self.settle();
        }
        fn context(&self) -> DropdownContext {
            self.config
                .borrow()
                .context
                .borrow()
                .as_ref()
                .unwrap()
                .clone()
        }
        fn target(&self, label: &str) -> dioxus::core::ElementId {
            let (id, _) = self
                .attrs
                .iter()
                .find(|(_, attrs)| {
                    attrs.get("aria-label").is_some_and(|value| value == label)
                        && (attrs.get("data-ix-overlay-root").is_some()
                            || attrs.get("data-ix-overlay-trigger").is_some())
                        || attrs
                            .get("data-ix-dropdown-label")
                            .is_some_and(|value| value == label)
                })
                .unwrap_or_else(|| panic!("Missing {label}"));
            dioxus::core::ElementId::from_raw(*id)
        }
        fn click(&mut self, label: &str) -> (bool, bool) {
            let result = dispatch(&self.dom, "click", self.target(label), ());
            self.settle();
            result
        }
        fn key(&mut self, label: &str, key: Key) -> (bool, bool) {
            let result = dispatch(
                &self.dom,
                "keydown",
                self.target(label),
                (key, Modifiers::empty()),
            );
            self.settle();
            result
        }
        fn html(&self) -> String {
            dioxus_ssr::render(&self.dom)
        }
    }

    fn unrelated_trigger_target(h: &Harness) -> (Rc<Cell<bool>>, Rc<RefCell<Vec<Change>>>) {
        let owner = h.context().node.borrow().clone().unwrap();
        let runtime = owner.runtime().unwrap();
        let open = Rc::new(Cell::new(true));
        let seen = Rc::new(RefCell::new(Vec::new()));
        let read = open.clone();
        let apply = open.clone();
        let before = seen.clone();
        owner
            .with_owner(|| {
                runtime.register(
                    "actual-trigger-unrelated-target".into(),
                    1,
                    OverlayKind::Dropdown,
                    CloseBehavior::Both,
                    overlay_runtime::Callbacks {
                        read: Rc::new(move || Some(read.get())),
                        before: Rc::new(move |change| {
                            before.borrow_mut().push(change);
                            true
                        }),
                        apply: Rc::new(move |change| apply.set(change.open)),
                        key: None,
                    },
                );
            })
            .unwrap();
        (open, seen)
    }

    #[test]
    fn observed_native_root_does_not_run_a_second_renderer_key_fallback() {
        let _guard = crate::test_event_converter_guard();
        let mut h = Harness::new();
        h.click("Menu");
        let context = h.context();
        let node = context.node.borrow().clone().unwrap();
        let runtime = node.runtime().unwrap();
        assert!(!context.state.borrow().native);
        assert!(!node.native_observed());
        let transport = h
            .attrs
            .iter()
            .find(|(_, attrs)| attrs.get("data-ix-overlay-transport") == Some(&runtime.owner_id()))
            .map(|(id, _)| dioxus::core::ElementId::from_raw(*id))
            .unwrap();
        dispatch(
            &h.dom,
            "input",
            transport,
            serde_json::json!({
                "owner":runtime.owner_id(),"sequence":1,"kind":"observe",
                "nodes":[{"id":node.id(),"generation":node.generation(),"connected":true,
                    "discovery_ready":true,"root_capture":"actual-menu-root"}]
            })
            .to_string(),
        );
        assert!(node.native_observed());
        assert!(
            !context.state.borrow().native,
            "observation did not invoke its key callback"
        );
        assert!(!h.key("Menu", Key::End).0);
        assert!(!h.key("Alpha", Key::Home).0);
        node.unregister();
        assert!(!node.native_observed());
    }

    #[test]
    fn first_observed_submenu_callback_queries_its_direct_forward_host_after_open_veto() {
        let _guard = crate::test_event_converter_guard();
        let mut h = Harness::with_settings(Fixture {
            focus_checked: true,
            deep_veto: true,
            ..Fixture::default()
        });
        let root = h.context();
        let child = h.config.borrow().linked.get().unwrap().context();
        let grand = child
            .entries
            .borrow()
            .values()
            .find(|entry| entry.label == "Deep")
            .unwrap()
            .submenu
            .unwrap()
            .context();
        // Real component key handlers establish both forwarding intents. The
        // last child's actual cancelable request rejects its presentation.
        h.key("Menu", Key::ArrowDown);
        h.key("Menu", Key::ArrowRight);
        h.key("Menu", Key::End);
        h.key("Menu", Key::ArrowRight);
        assert!(*root.open.peek() && *child.open.peek());
        assert!(!*grand.open.peek());
        assert_eq!(root.state.borrow().forward.unwrap().context().id, child.id);
        assert_eq!(child.state.borrow().forward.unwrap().context().id, grand.id);
        assert!(
            !child.state.borrow().native,
            "no own native key callback yet"
        );
        assert_eq!(
            child.navigation_owner(false).id,
            child.id,
            "unobserved renderer fallback stays open-only"
        );

        let root_node = root.node.borrow().clone().unwrap();
        let child_node = child.node.borrow().clone().unwrap();
        let grand_node = grand.node.borrow().clone().unwrap();
        let runtime = child_node.runtime().unwrap();
        let transport = h
            .attrs
            .iter()
            .find(|(_, attrs)| attrs.get("data-ix-overlay-transport") == Some(&runtime.owner_id()))
            .map(|(id, _)| dioxus::core::ElementId::from_raw(*id))
            .unwrap();
        fn inventory(context: &DropdownContext, active_first: bool) -> serde_json::Value {
            let entries = context.entries.borrow();
            let hosts = entries
                .iter()
                .enumerate()
                .map(|(index, (_, entry))| {
                    serde_json::json!({
                        "item":format!("actual-object-{}",entry.host_id),
                        "capture":format!("actual-object-{}",entry.host_id),
                        "kind":"dropdown","target":entry.host_id,
                        "disabled_attribute":entry.disabled,"checked_attribute":entry.checked,
                        "visible_focus":active_first&&index==0
                    })
                })
                .collect::<Vec<_>>();
            let order = hosts
                .iter()
                .map(|host| host["item"].clone())
                .collect::<Vec<_>>();
            serde_json::json!({"items_host":format!("actual-host-{}",context.id),
                "hosts":hosts,"navigation_order":order,"query_order":order})
        }
        let root_inventory = inventory(&root, false);
        let child_inventory = inventory(&child, true);
        let grand_inventory = inventory(&grand, false);
        let nodes = [&root, &child, &grand]
            .into_iter()
            .zip([&root_node, &child_node, &grand_node])
            .zip([root_inventory, child_inventory, grand_inventory.clone()])
            .map(|((context, node), inventory)| serde_json::json!({
                "id":node.id(),"owner":runtime.owner_id(),"generation":node.generation(),
                "connected":true,"discovery_ready":true,"open":*context.open.peek(),
                "root_capture":format!("actual-root-{}",context.id),"mode":"active-descendant",
                "trigger_capture":format!("actual-trigger-{}",context.id),
                "trigger_ancestors":if context.id==root.id {vec![]} else if context.id==child.id {vec![root.node_id()]} else {vec![child.node_id(),root.node_id()]},
                "on_trigger":context.id==child.id,"inside":context.id==root.id,
                "inventory":inventory,"own_inventory":inventory
            }))
            .collect::<Vec<_>>();
        assert!(runtime.pending_item_focus_count() == 0);
        dispatch(
            &h.dom,
            "input",
            transport,
            serde_json::json!({
                "owner":runtime.owner_id(),"sequence":1,"kind":"observe","nodes":nodes
            })
            .to_string(),
        );
        assert!(child_node.native_observed());
        assert!(
            !child.state.borrow().native,
            "observation is not a key callback"
        );
        assert_eq!(
            child.navigation_owner(false).id,
            grand.id,
            "a native-observed receiver uses its direct assigned host even if showing was vetoed"
        );

        for (sequence, key) in [(2, "ArrowDown"), (3, "End")] {
            assert!(dispatch(&h.dom,"input",transport,serde_json::json!({
                "owner":runtime.owner_id(),"sequence":sequence,"kind":"key","phase":"bubble",
                "key":key,"hosts":[root_node.id()],"triggers":[child_node.id()],"nodes":nodes
            }).to_string()).0);
        }
        assert_eq!(
            runtime.pending_item_focus_count(),
            1,
            "the receiver retains only the latest mounted focus intent"
        );
        h.settle();
        assert!(child.state.borrow().native);
        assert!(
            !*grand.open.peek(),
            "querying the forwarded host does not retry showing it"
        );
    }

    #[test]
    fn trigger_declaration_preserves_falsy_watchers_and_roundtrip_attempts() {
        let mut declaration = TriggerDeclaration::default();
        declaration.update(&None);
        assert_eq!((declaration.version, declaration.dispose_version), (0, 0));
        for (value, expected) in [
            (Some("a"), (1, 1)),
            (Some("a"), (1, 1)),
            (Some("b"), (2, 2)),
            (None, (3, 2)),
            (Some(""), (4, 2)),
            (Some("a"), (5, 3)),
        ] {
            declaration.update(&value.map(str::to_owned));
            assert_eq!((declaration.version, declaration.dispose_version), expected);
        }
    }

    #[test]
    fn retained_ordinary_navigation_ignores_new_focus_flag_and_current_trigger() {
        let _guard = crate::test_event_converter_guard();
        let mut h = Harness::new();
        h.click("Menu");
        h.config.borrow_mut().no_focus = true;
        h.update();
        let context = h.context();
        let items = context.entries.borrow().keys().copied().collect::<Vec<_>>();
        let ordinary = KeyFacts {
            route: KeyRoute::Navigation,
            navigation_mode: Some("active-descendant".into()),
            key: "Home".into(),
            default_prevented: true,
            items: items.clone(),
            // A retained receiver need not be the current trigger or host.
            on_trigger: false,
            inside: false,
            ..KeyFacts::default()
        };
        let result = h.dom.in_runtime(|| context.key(&ordinary));
        assert!(result.prevent);
        let trigger = KeyFacts {
            route: KeyRoute::Trigger,
            key: "End".into(),
            on_trigger: true,
            items,
            ..KeyFacts::default()
        };
        let result = h.dom.in_runtime(|| context.key(&trigger));
        assert!(
            !result.prevent,
            "open AD experimental listener does not navigate"
        );
    }

    #[test]
    fn actual_mouse_triggers_dismiss_others_despite_visibility_veto_but_public_handles_do_not() {
        let _guard = crate::test_event_converter_guard();
        for label in ["Menu", "External trigger", "Linked"] {
            let mut h = Harness::with_settings(Fixture {
                veto_open: true,
                linked_veto: true,
                ..Fixture::default()
            });
            let (other, seen) = unrelated_trigger_target(&h);
            let linked = h.config.borrow().linked.get().unwrap();
            h.dom.in_runtime(|| {
                linked.open();
                linked.close(false);
                linked.toggle();
            });
            h.settle();
            assert!(
                other.get(),
                "public handle requests must not acquire the trigger phase"
            );
            assert!(seen.borrow().is_empty());
            // Actual component listener dispatch is isolated from native layout
            // and hierarchy here; the unrelated target is a registered owner.
            h.click(label);
            assert!(
                !other.get(),
                "{label}: own visibility veto does not cancel other dismissal"
            );
            assert_eq!(seen.borrow().len(), 1);
            assert_eq!(seen.borrow()[0].reason, Reason::TriggerOther);
            assert!(!*h.context().open.peek());
            assert!(!linked.is_open());
        }
    }

    #[test]
    fn closed_ordinary_trigger_keys_run_others_but_open_navigation_and_submenu_delegation_do_not() {
        let _guard = crate::test_event_converter_guard();
        let mut h = Harness::new();
        let (other, seen) = unrelated_trigger_target(&h);
        h.key("Menu", Key::ArrowDown);
        assert!(!other.get());
        assert_eq!(seen.borrow().len(), 1);
        other.set(true);
        h.key("Menu", Key::Home);
        assert!(
            other.get(),
            "already open navigation has no toggle-controller phase"
        );
        h.dom
            .in_runtime(|| h.context().request(false, Reason::Trigger, false));
        h.settle();
        h.context().state.borrow_mut().anchor_in_dropdown_item = true;
        h.key("Menu", Key::Enter);
        assert!(
            other.get(),
            "anchor-submenu delegation has no ordinary trigger phase"
        );
        assert_eq!(seen.borrow().len(), 1);
    }

    #[test]
    fn original_prevented_component_click_skips_own_toggle_and_other_dismissal() {
        let _guard = crate::test_event_converter_guard();
        for label in ["Menu", "External trigger", "Linked"] {
            let mut h = Harness::new();
            let (other, seen) = unrelated_trigger_target(&h);
            let data: Rc<dyn std::any::Any> =
                Rc::new(dioxus::html::PlatformEventData::new(Box::new(())));
            let event = Event::new(data, true);
            event.prevent_default();
            h.dom
                .runtime()
                .handle_event("click", event, h.target(label));
            h.settle();
            assert!(!*h.context().open.peek());
            assert!(!h.config.borrow().linked.get().unwrap().is_open());
            assert!(other.get());
            assert!(seen.borrow().is_empty());
        }
    }

    #[test]
    fn dropdown_partial_anchor_updates_preserve_alignment_until_fresh_geometry_and_reset() {
        let _guard = crate::test_event_converter_guard();
        let mut h = Harness::with_settings(Fixture {
            placement: DropdownPlacement::BottomEnd,
            suppress_automatic: true,
            ..Fixture::default()
        });
        h.click("Menu");
        let context = h.context();
        let node = context.node.borrow().clone().unwrap();
        let transport = h
            .attrs
            .iter()
            .find(|(_, attrs)| {
                attrs.get("data-ix-overlay-transport") == Some(&node.runtime().unwrap().owner_id())
            })
            .map(|(id, _)| dioxus::core::ElementId::from_raw(*id))
            .unwrap();
        let mut sequence = 0;
        let mut deliver = |h: &mut Harness, top, bottom, left, right, submenu| {
            sequence += 1;
            let report = serde_json::json!({
                "owner":node.runtime().unwrap().owner_id(),"sequence":sequence,"kind":"anchor-visibility",
                "nodes":[{"id":context.node_id(),"generation":node.generation(),"connected":true,"trigger_id":context.trigger_id()}],
                "anchor_visibility":{"root":context.node_id(),"generation":node.generation(),"target":context.trigger_id(),"intersecting":true,
                    "geometry":{"bounding":{"top":0,"bottom":40,"left":0,"right":100},
                        "intersection":{"top":top,"bottom":bottom,"left":left,"right":right},"anchor_in_dropdown_item":submenu}}
            });
            assert_eq!(
                dispatch(&h.dom, "input", transport, report.to_string()),
                (false, false)
            );
            h.settle();
        };
        let host = |h: &Harness| {
            let html = h.html();
            html.split(&format!("id=\"{}\"", context.host_id()))
                .nth(1)
                .unwrap()
                .split('>')
                .next()
                .unwrap()
                .to_string()
        };
        // Actual handler reports, including opposing edges and a corner, must
        // change only placement, never issue visibility requests or remount.
        for (rect, expected) in [
            ((20, 40, 0, 100), "bottom-end"),
            ((0, 20, 0, 100), "top-end"),
            ((0, 40, 50, 100), "right-end"),
            ((0, 40, 0, 50), "left-end"),
            ((20, 40, 0, 50), "left-end"),
            ((20, 20, 50, 50), "left-end"),
        ] {
            deliver(&mut h, rect.0, rect.1, rect.2, rect.3, false);
            assert!(host(&h).contains(&format!("data-placement=\"{expected}\"")));
        }
        h.config.borrow_mut().placement = DropdownPlacement::TopStart;
        h.update();
        assert!(
            host(&h).contains("data-placement=\"left-end\""),
            "source stores fallback alignment at delivery, not every render"
        );
        h.config.borrow_mut().suppress_visibility = true;
        h.update();
        assert!(host(&h).contains("data-placement=\"top-start\""));
        deliver(&mut h, 0, 40, 0, 100, false);
        h.config.borrow_mut().suppress_visibility = false;
        h.update();
        assert!(
            host(&h).contains("data-placement=\"left-end\""),
            "suppressed reports do not erase retained fallback"
        );
        deliver(&mut h, 0, 40, 0, 100, false);
        assert!(host(&h).contains("data-placement=\"top-start\""));
        assert!(host(&h).contains("position-try-fallbacks:none"));
        h.config.borrow_mut().suppress_automatic = false;
        h.update();
        assert!(!host(&h).contains("position-try-fallbacks:none"));
        deliver(&mut h, 0, 20, 0, 100, true);
        assert!(
            host(&h).contains("data-placement=\"right-start\""),
            "submenu anchor overrides clipping and configured alignment"
        );
        assert!(*context.open.peek());
        assert_eq!(h.config.borrow().requests, [(true, false)]);
        assert_eq!(h.config.borrow().changed, [true]);
        assert_eq!((h.config.borrow().mounts, h.config.borrow().drops), (1, 0));
    }

    #[test]
    fn dropdown_persistent_tree_keeps_scope_and_listeners_across_open_mode_and_layer() {
        let _guard = crate::test_event_converter_guard();
        let mut h = Harness::new();
        let alpha = h.target("Alpha");
        let context = h.context();
        assert!(!*context.open.peek());
        assert!(
            h.html().contains("Persistent nested"),
            "closed nested content remains mounted"
        );
        assert_eq!((h.config.borrow().mounts, h.config.borrow().drops), (1, 0));
        h.click("Menu");
        assert!(*context.open.peek());
        let count = *h
            .listeners
            .get("click")
            .unwrap()
            .iter()
            .find(|id| {
                h.attrs
                    .get(&id.raw())
                    .is_some_and(|a| a.get("data-count-button").is_some())
            })
            .unwrap();
        dispatch(&h.dom, "click", count, ());
        h.settle();
        assert!(h.html().contains("Local 1"));
        // Local button click can bubble to composed trigger and close; neither
        // presentation change is allowed to recreate its component scope.
        h.config.borrow_mut().mode = DropdownNavigationMode::RovingTabindex;
        h.update();
        h.config.borrow_mut().layer = true;
        h.update();
        h.click("Menu");
        h.click("Menu");
        assert_eq!(h.target("Alpha"), alpha);
        assert!(h.html().contains("Local 1"));
        assert_eq!((h.config.borrow().mounts, h.config.borrow().drops), (1, 0));
        assert!(h.html().contains("popover=\"manual\""));
        h.config.borrow_mut().visible = false;
        h.update();
        assert_eq!(h.config.borrow().drops, 1);
        assert!(
            !context.request(true, Reason::Trigger, false),
            "detached owner rejects stale controller work"
        );
    }
    #[test]
    fn dropdown_actual_visibility_requests_veto_retry_and_reentrant_change_are_ordered() {
        let _guard = crate::test_event_converter_guard();
        let mut h = Harness::new();
        let context = h.context();
        let host = *h
            .attrs
            .iter()
            .find(|(_, attrs)| attrs.get("data-ix-overlay-host") == Some(&context.node_id()))
            .unwrap()
            .0;
        assert_eq!(
            h.attrs[&host].get("inert").map(String::as_str),
            Some(""),
            "closed host uses boolean attribute presence"
        );
        h.config.borrow_mut().veto_open = true;
        let result = h.click("Menu");
        assert!(
            !result.0,
            "visibility veto is distinct from original click cancellation"
        );
        assert!(!*context.open.peek());
        assert!(h.config.borrow().changed.is_empty());
        h.config.borrow_mut().veto_open = false;
        h.click("Menu");
        assert!(*context.open.peek());
        assert!(
            !h.attrs[&host].contains_key("inert"),
            "open host removes inert instead of serializing inert=false"
        );
        h.config.borrow_mut().veto_close = true;
        h.click("Menu");
        assert!(*context.open.peek());
        h.config.borrow_mut().veto_close = false;
        h.click("Menu");
        assert!(!*context.open.peek());
        assert_eq!(
            h.attrs[&host].get("inert").map(String::as_str),
            Some(""),
            "closing restores inert presence on same persistent host"
        );
        assert_eq!(
            h.config.borrow().requests,
            [(true, false), (true, false), (false, true), (false, true)]
        );
        h.config.borrow_mut().close_after_open = true;
        h.click("Menu");
        assert!(
            !*context.open.peek(),
            "nested command's final state survives outer callback"
        );
        assert_eq!(h.config.borrow().changed, [true, false, true, false]);
    }
    #[test]
    fn dropdown_linked_item_prevents_ancestor_toggle_but_notification_veto_is_separate() {
        let _guard = crate::test_event_converter_guard();
        let mut h = Harness::new();
        let root = h.context();
        let child = h.config.borrow().linked.get().unwrap();
        // This callback-only renderer has no native observer. Supply the
        // fixture's real linked-trigger ancestry before testing its exclusion
        // from the new post-trigger dismissal pass.
        let root_node = root.node.borrow().clone().unwrap();
        let child_node = child.context().node.borrow().clone().unwrap();
        let runtime = root_node.runtime().unwrap();
        let transport = h
            .attrs
            .iter()
            .find(|(_, attrs)| attrs.get("data-ix-overlay-transport") == Some(&runtime.owner_id()))
            .map(|(id, _)| dioxus::core::ElementId::from_raw(*id))
            .unwrap();
        dispatch(&h.dom, "input", transport, serde_json::json!({
            "owner":runtime.owner_id(),"sequence":1,"kind":"observe",
            "nodes":[
                {"id":root_node.id(),"generation":root_node.generation(),"connected":true,"discovery_ready":true},
                {"id":child_node.id(),"generation":child_node.generation(),"connected":true,"discovery_ready":true,"trigger_ancestors":[root_node.id()]}
            ]
        }).to_string());
        h.settle();
        h.click("Menu");
        let result = h.click("Linked");
        assert!(result.0);
        assert!(*root.open.peek());
        assert!(child.is_open());
        h.dom.in_runtime(|| child.toggle());
        h.settle();
        assert!(
            !child.is_open(),
            "mounted controller toggle releases its read before request"
        );
        h.dom.in_runtime(|| child.toggle());
        h.settle();
        assert!(child.is_open());
        h.config.borrow_mut().item_veto = true;
        let result = h.click("Alpha");
        assert!(
            !result.0,
            "internal item notification does not cancel original click"
        );
        assert_eq!(h.config.borrow().item_requests, 1);
        assert_eq!(h.config.borrow().native_clicks, 1);
        assert!(
            !*root.open.peek(),
            "composed trigger ancestor still handles original leaf click"
        );
        let result = h.click("Disabled");
        assert!(result.0);
        assert_eq!(h.config.borrow().item_requests, 1);
        let enter = h.key("Standalone", Key::Enter);
        let space = h.key("Standalone", Key::Character(" ".into()));
        assert!(
            !enter.0 && !space.0,
            "notification veto does not prevent original standalone keys"
        );
        assert_eq!(h.config.borrow().standalone_requests, 2);
        assert_eq!(
            h.config.borrow().standalone_mouse,
            0,
            "keyboard notification never fabricates MouseEvent"
        );
    }
    fn mixed_inventory_facts() -> InventoryFacts {
        serde_json::from_value(serde_json::json!({
            "items_host":"mixed-items-host",
            "hosts":[
                {"item":"actual-category","kind":"category","target":"mixed-category","capture":"actual-category","registration_key":"category-key","source_focusable":false},
                {"item":"actual-menu","kind":"menu","target":"mixed-menu","capture":"actual-menu","registration_key":"menu-key"},
                {"item":"actual-native","kind":"native-opt-in","target":"mixed-native","capture":"actual-native","source_focusable":false,"roving_opt_in":true}
            ],
            "navigation_order":["actual-category","actual-menu","actual-native"],
            "query_order":["actual-category","actual-menu","actual-native"]
        })).unwrap()
    }
    struct InventoryMount {
        target: String,
        focuses: Rc<RefCell<Vec<String>>>,
    }
    impl dioxus::html::RenderedElementBacking for InventoryMount {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
        fn set_focus(
            &self,
            focus: bool,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = dioxus::html::MountedResult<()>>>>
        {
            assert!(focus);
            self.focuses.borrow_mut().push(self.target.clone());
            Box::pin(async { Ok(()) })
        }
    }
    fn mount_inventory(h: &Harness, facts: &InventoryFacts) -> Rc<RefCell<Vec<String>>> {
        let focuses = Rc::new(RefCell::new(Vec::new()));
        let runtime = h
            .context()
            .node
            .borrow()
            .as_ref()
            .unwrap()
            .runtime()
            .unwrap();
        for host in &facts.hosts {
            let key = if host.registration_key.is_empty() {
                &host.capture
            } else {
                &host.registration_key
            };
            runtime.register_item_cue(key.clone(), host.kind);
            runtime.register_item_mount(
                key,
                &host.target,
                Rc::new(MountedData::new(InventoryMount {
                    target: host.target.clone(),
                    focuses: focuses.clone(),
                })),
            );
        }
        focuses
    }
    fn registered_inventory_key(
        h: &Harness,
        sequence: u64,
        key: &str,
        facts: &InventoryFacts,
    ) -> bool {
        let context = h.context();
        let node = context.node.borrow().clone().unwrap();
        let runtime = node.runtime().unwrap();
        let mode = context.state.borrow().settings.mode;
        let transport = h
            .attrs
            .iter()
            .find(|(_, attrs)| attrs.get("data-ix-overlay-transport") == Some(&runtime.owner_id()))
            .map(|(id, _)| dioxus::core::ElementId::from_raw(*id))
            .unwrap();
        let legacy = context.entries.borrow().keys().copied().collect::<Vec<_>>();
        let report = serde_json::json!({"owner":runtime.owner_id(),"sequence":sequence,"kind":"key",
            "phase":if mode==DropdownNavigationMode::RovingTabindex {"capture"} else {"bubble"},"key":key,
            "hosts":[node.id()],"triggers":if mode==DropdownNavigationMode::ActiveDescendant {vec![node.id()]}else{vec![]},
            "candidate_ids":["mixed-category","mixed-menu","mixed-native"],
            "nodes":[{"id":node.id(),"owner":runtime.owner_id(),"generation":node.generation(),"connected":true,"open":true,
                "mode":mode.as_str(),"items":legacy,"focused_item":legacy.first(),"inside":true,
                "on_trigger":mode==DropdownNavigationMode::ActiveDescendant,"inventory":facts}]});
        dispatch(&h.dom, "input", transport, report.to_string()).0
    }
    #[test]
    fn registered_category_inventory_ignores_legacy_focus_and_retains_projection_seam() {
        let _guard = crate::test_event_converter_guard();
        let mut h = Harness::new();
        h.click("Menu");
        let context = h.context();
        let legacy = context.entries.borrow().keys().next().copied().unwrap();
        context.state.borrow_mut().active = Some(legacy);
        let facts = mixed_inventory_facts();
        let focuses = mount_inventory(&h, &facts);
        assert!(registered_inventory_key(&h, 1, "Home", &facts));
        let runtime = context.node.borrow().as_ref().unwrap().runtime().unwrap();
        assert_eq!(runtime.pending_item_focus_count(), 1);
        h.settle();
        assert_eq!(focuses.borrow().as_slice(), ["mixed-category"]);
        assert_eq!(
            context.active_dom_id(),
            None,
            "projected parent is not an outer AD marker/ARIA alias"
        );
        assert!(registered_inventory_key(&h, 2, "ArrowDown", &facts));
        h.settle();
        assert_eq!(
            focuses.borrow().as_slice(),
            ["mixed-category", "mixed-category"],
            "no current host marker means Down selects the first Category again"
        );
        assert!(
            registered_inventory_key(&h, 3, "Enter", &facts),
            "source activation callback prevents Enter even without any active host"
        );
        assert!(runtime.pending_item_focus_count() == 0);
    }
    #[test]
    fn registered_roving_inventory_projects_logical_tab_order_before_render() {
        let _guard = crate::test_event_converter_guard();
        let mut h = Harness::with_settings(Fixture {
            mode: DropdownNavigationMode::RovingTabindex,
            ..Fixture::default()
        });
        h.click("Menu");
        let context = h.context();
        let runtime = context.node.borrow().as_ref().unwrap().runtime().unwrap();
        let facts = mixed_inventory_facts();
        let focuses = mount_inventory(&h, &facts);
        for (index, key) in ["Home", "ArrowDown", "ArrowDown"].into_iter().enumerate() {
            assert!(registered_inventory_key(&h, index as u64 + 1, key, &facts));
        }
        assert_eq!(runtime.pending_item_focus_count(), 1);
        assert!(
            !registered_inventory_key(&h, 4, "Enter", &facts),
            "logical tabindex selects the custom control before rendering; native Enter passes through"
        );
        h.settle();
        assert_eq!(focuses.borrow().as_slice(), ["mixed-native"]);
        assert!(runtime.pending_item_focus_count() == 0);
    }
    #[test]
    fn rapid_navigation_commits_the_latest_mounted_target_without_foreign_commands() {
        let _guard = crate::test_event_converter_guard();
        for mode in [
            DropdownNavigationMode::ActiveDescendant,
            DropdownNavigationMode::RovingTabindex,
        ] {
            let mut h = Harness::with_settings(Fixture {
                mode,
                ..Fixture::default()
            });
            h.click("Menu");
            let context = h.context();
            let node = context.node.borrow().clone().unwrap();
            let runtime = node.runtime().unwrap();
            let transport = h
                .attrs
                .iter()
                .find(|(_, attrs)| {
                    attrs.get("data-ix-overlay-transport") == Some(&runtime.owner_id())
                })
                .map(|(id, _)| dioxus::core::ElementId::from_raw(*id))
                .unwrap();
            let order = context.entries.borrow().keys().copied().collect::<Vec<_>>();
            for (index, key) in ["End", "Home", "End"].into_iter().enumerate() {
                let report = serde_json::json!({"owner":runtime.owner_id(),"sequence":index+1,"kind":"key",
                    "phase":if mode==DropdownNavigationMode::ActiveDescendant {"bubble"}else{"capture"},"key":key,
                    "hosts":[node.id()],"triggers":if mode==DropdownNavigationMode::ActiveDescendant{vec![node.id()]}else{vec![]},
                    "nodes":[{"id":node.id(),"generation":node.generation(),"connected":true,"discovery_ready":true,
                        "mode":mode.as_str(),"items":order,"inside":true,"on_trigger":false}]});
                assert!(dispatch(&h.dom, "input", transport, report.to_string()).0);
            }
            assert_eq!(runtime.pending_item_focus_count(), 1);
            h.settle();
            assert_eq!(
                context.active_dom_id(),
                Some(format!("ix-dropdown-item-{}", order.last().unwrap()))
            );
            assert!(runtime.pending_item_focus_count() == 0);
            h.click("Menu");
            assert!(!*context.open.peek());
            assert!(
                runtime.pending_item_focus_count() == 0,
                "dismissal leaves no stale focus attempt"
            );
        }
    }
    #[test]
    fn dropdown_actual_keyboard_flags_distinguish_active_descendant_and_roving() {
        let _guard = crate::test_event_converter_guard();
        let mut h = Harness::new();
        let context = h.context();
        let result = h.key("Menu", Key::ArrowDown);
        assert!(result.0);
        assert!(*context.open.peek());
        let first = context.state.borrow().active.unwrap();
        assert_eq!(context.entries.borrow()[&first].label, "Alpha");
        assert_eq!(
            context.state.borrow().focus_target,
            "none",
            "source default retains trigger focus"
        );
        let result = h.key("Menu", Key::End);
        assert!(result.0);
        let last = context.state.borrow().active.unwrap();
        assert_eq!(context.entries.borrow()[&last].label, "Explicit");
        let result = h.key("Menu", Key::ArrowRight);
        assert!(!result.0, "submenu Right leaves original default enabled");
        let forward = context.state.borrow().forward.unwrap();
        h.dom.in_runtime(|| forward.close(false));
        h.settle();
        let next = h.key("Menu", Key::ArrowDown);
        assert!(next.0);
        assert_eq!(
            context.entries.borrow()[&context.state.borrow().active.unwrap()].label,
            "Alpha",
            "closed child no longer owns parent item queries"
        );
        h.config.borrow_mut().mode = DropdownNavigationMode::RovingTabindex;
        h.update();
        let result = h.key("Menu", Key::ArrowDown);
        assert!(result.0);
        assert_eq!(
            context.state.borrow().focus_target,
            "none",
            "source item focus is independently queued, not the latest attribute"
        );
        let result = h.key("Menu", Key::Character("e".into()));
        assert!(result.0);
        let active = context.state.borrow().active.unwrap();
        assert_eq!(
            context.entries.borrow()[&active].label,
            "Explicit",
            "explicit roving preserves existing typeahead adapter"
        );
        let target = h.target("Menu");
        let previous = context.state.borrow().active;
        let shortcut = dispatch(
            &h.dom,
            "keydown",
            target,
            (Key::Character("a".into()), Modifiers::CONTROL),
        );
        h.settle();
        assert!(!shortcut.0);
        assert_eq!(
            context.state.borrow().active,
            previous,
            "roving typeahead adapter leaves browser shortcuts unchanged"
        );
        assert!(
            !h.html().contains("aria-activedescendant"),
            "roving has no focus proxy aria attribute"
        );
    }

    #[test]
    fn completion_effects_keep_each_payload_and_use_current_committed_listener() {
        let _guard = crate::test_event_converter_guard();
        let mut h = Harness::new();
        let context = h.context();
        h.dom.in_runtime(|| {
            for value in [true, false, true] {
                assert!(context.request(value, Reason::Trigger, false));
            }
        });
        assert!(
            h.config.borrow().changed.is_empty(),
            "request admission is synchronous, completion is not"
        );
        h.config.borrow_mut().listener_revision = 7;
        h.config.borrow().update.clone().unwrap()();
        h.completion_effects();
        assert!(*context.open.peek());
        assert_eq!(h.config.borrow().changed, [true, false, true]);
        assert_eq!(
            h.config.borrow().deliveries,
            [(7, true, true), (7, false, true), (7, true, true)]
        );
        h.completion_effects();
        assert_eq!(h.config.borrow().changed.len(), 3, "effects deliver once");
    }

    #[test]
    fn completion_effect_allows_reentrant_state_change_without_borrow_conflict() {
        let _guard = crate::test_event_converter_guard();
        let mut h = Harness::with_settings(Fixture {
            close_after_open: true,
            ..Default::default()
        });
        let context = h.context();
        h.dom
            .in_runtime(|| assert!(context.request(true, Reason::Trigger, false)));
        assert!(h.config.borrow().changed.is_empty());
        h.settle();
        assert_eq!(h.config.borrow().changed, [true, false]);
        assert!(!*context.open.peek());
        assert_eq!(h.config.borrow().requests, [(true, false), (false, true)]);
        assert_eq!(
            h.config.borrow().deliveries,
            [(0, true, true), (0, false, false)]
        );
    }

    #[test]
    fn dropped_component_effect_cannot_call_the_replacement_listener() {
        let _guard = crate::test_event_converter_guard();
        let mut h = Harness::new();
        let old = h.context();
        let runtime = old.node.borrow().clone().unwrap().runtime().unwrap();
        h.dom
            .in_runtime(|| assert!(old.request(true, Reason::Trigger, false)));
        assert_eq!(runtime.pending_completions(), 1);
        h.config.borrow_mut().visible = false;
        h.update();
        assert_eq!(runtime.pending_completions(), 0);
        assert!(h.config.borrow().changed.is_empty());
        h.config.borrow_mut().visible = true;
        h.config.borrow_mut().listener_revision = 9;
        h.update();
        let current = h.context();
        assert_ne!(current.id, old.id, "a new component has a new owner");
        assert!(!old.request(true, Reason::Trigger, false));
        h.dom
            .in_runtime(|| assert!(current.request(true, Reason::Trigger, false)));
        assert!(h.config.borrow().changed.is_empty());
        h.settle();
        assert_eq!(h.config.borrow().deliveries, [(9, true, true)]);
    }

    #[test]
    fn native_ordinary_tab_wraps_registered_controls_and_preserves_native_paths() {
        let _guard = crate::test_event_converter_guard();
        for (inside, index, shift, registered, no_trap, expected) in [
            (true, 2, false, true, false, Some("mixed-category")),
            (true, 0, true, true, false, Some("mixed-native")),
            (true, 1, false, true, false, None),
            (false, 2, false, true, false, None),
            (true, 2, false, false, false, None),
            (true, 2, false, true, true, None),
        ] {
            let mut h = Harness::with_settings(Fixture {
                no_trap,
                ..Default::default()
            });
            h.click("Menu");
            let context = h.context();
            let node = context.node.borrow().clone().unwrap();
            let runtime = node.runtime().unwrap();
            let facts = mixed_inventory_facts();
            let focuses = Rc::new(RefCell::new(Vec::new()));
            let focusable_ids = facts
                .hosts
                .iter()
                .map(|host| format!("{}-control", host.target))
                .collect::<Vec<_>>();
            if registered {
                for (host, focus_id) in facts.hosts.iter().zip(&focusable_ids) {
                    let key = if host.registration_key.is_empty() {
                        &host.capture
                    } else {
                        &host.registration_key
                    };
                    runtime.register_item_cue(key.clone(), host.kind);
                    runtime.register_item_control_mount(
                        key,
                        &host.target,
                        focus_id,
                        Rc::new(MountedData::new(InventoryMount {
                            target: host.target.clone(),
                            focuses: focuses.clone(),
                        })),
                    );
                }
            }
            let transport = h
                .attrs
                .iter()
                .find(|(_, attrs)| {
                    attrs.get("data-ix-overlay-transport") == Some(&runtime.owner_id())
                })
                .map(|(id, _)| dioxus::core::ElementId::from_raw(*id))
                .unwrap();
            let before = h.config.borrow().requests.clone();
            let report = serde_json::json!({"owner":runtime.owner_id(),"sequence":1,"kind":"key","phase":"bubble",
            "key":"Tab","shift":shift,"hosts":if inside {vec![node.id()]} else {vec![]},
            "triggers":if inside {vec![]} else {vec![node.id()]},"nodes":[{
                "id":node.id(),"generation":node.generation(),"connected":true,"discovery_ready":true,
                "mode":"active-descendant","open":true,"on_trigger":!inside,"inside":inside,
                "focus_count":3,"focused_index":index,"focusable_ids":focusable_ids,"inventory":facts
            }]});
            let revision_before = *context.revision.peek();
            let (prevented, propagates) = dispatch(&h.dom, "input", transport, report.to_string());
            if expected.is_some() {
                assert!(
                    *context.revision.peek() > revision_before,
                    "admitted Tab focus wakes the renderer before any manual settling"
                );
            }
            h.settle();
            assert_eq!(prevented, expected.is_some(), "{report}");
            assert!(!propagates, "transport remains private");
            assert_eq!(
                focuses.borrow().last().map(String::as_str),
                expected,
                "{report}"
            );
            assert_eq!(*context.open.peek(), !no_trap);
            if !no_trap {
                assert_eq!(
                    h.config.borrow().requests,
                    before,
                    "wrapping does not request visibility changes"
                );
            }
            assert_eq!(runtime.pending_item_focus_count(), 0);
        }
    }

    #[test]
    fn roving_typeahead_focuses_authored_item_id_and_dismissal_cancels_pending_mount() {
        let _guard = crate::test_event_converter_guard();
        let mut h = Harness::with_settings(Fixture {
            mode: DropdownNavigationMode::RovingTabindex,
            alpha_id: Some("authored-alpha".into()),
            ..Default::default()
        });
        h.click("Menu");
        let mock = mount_trigger_mock(&mut h, "Alpha");
        assert!(h.key("Menu", Key::Character("a".into())).0);
        assert_eq!(*mock.calls.borrow(), ["Alpha"]);
        let context = h.context();
        let selected = context.state.borrow().active.unwrap();
        assert_eq!(
            context.entries.borrow()[&selected].host_id,
            "authored-alpha"
        );
        let html = h.html();
        assert!(!html.contains("data-ix-overlay-focus-target"));
        assert!(!html.contains("data-ix-overlay-focus-version"));
        mock.pending.set(true);
        mock.started.set(false);
        mock.dropped.set(false);
        h.key("Menu", Key::Character("a".into()));
        assert!(mock.started.get());
        assert!(!mock.dropped.get());
        h.dom
            .in_runtime(|| assert!(context.request(false, Reason::Outside, false)));
        h.settle();
        assert!(
            mock.dropped.get(),
            "dismissal releases the pending mounted focus attempt"
        );
        mock.pending.set(false);
        mock.waiter.borrow_mut().take().unwrap().wake();
        h.settle();
        assert_eq!(*mock.calls.borrow(), ["Alpha"]);
    }

    #[test]
    fn dropdown_actual_request_and_surface_drop_cancel_owned_pending_work() {
        let _guard = crate::test_event_converter_guard();
        let flag = Rc::new(Cell::new(false));
        let mut h = Harness::with_settings(Fixture {
            pending_drop: Some(flag.clone()),
            ..Default::default()
        });
        assert!(
            h.config.borrow().pending_started.get(),
            "owned pending future must actually begin"
        );
        assert!(!flag.get());
        h.click("Menu");
        assert!(*h.context().open.peek());
        assert!(
            flag.get(),
            "accepted request cancels older focus work before state notifications"
        );
        let context = h.context();
        h.config.borrow_mut().visible = false;
        h.update();
        assert!(context.node.borrow().is_none());
        assert!(!context.state.borrow().surface);
        assert!(!context.request(true, Reason::Keyboard, false));
    }

    #[test]
    fn dropdown_linked_activation_keys_present_and_forward_without_click_or_key_cancellation() {
        let _guard = crate::test_event_converter_guard();
        for key in [Key::ArrowRight, Key::Enter, Key::Character(" ".into())] {
            let mut h = Harness::with_settings(Fixture {
                focus_checked: true,
                ..Default::default()
            });
            let parent = h.context();
            let child = h.config.borrow().linked.get().unwrap();
            h.key("Menu", Key::ArrowDown);
            let active = parent.state.borrow().active.unwrap();
            assert_eq!(parent.entries.borrow()[&active].label, "Linked");
            let result = h.key("Menu", key.clone());
            assert!(
                !result.0,
                "linked activation leaves the original key unprevented"
            );
            assert!(child.is_open());
            assert!(parent.state.borrow().forward == Some(child));
            let child_context = child.context();
            // This harness dispatches mounted events with unit-backed handles,
            // while NoOpDocument cannot return a native inventory. Opening
            // therefore completes with the Dioxus trigger focus intent.
            assert_eq!(child_context.state.borrow().focus_target, "trigger");
            assert!(child_context.state.borrow().focus_pending.is_none());
            // Repeated keys must never manufacture a click/toggle on the
            // already open trigger after the opening focus request completes.
            h.key("Menu", Key::Enter);
            h.key("Menu", Key::Character(" ".into()));
            assert!(child.is_open());
            assert_eq!(h.config.borrow().native_clicks, 0);
            h.key("Menu", Key::Home);
            h.key("Menu", Key::ArrowDown);
            let selected = child_context.state.borrow().active.unwrap();
            assert_eq!(
                child_context.entries.borrow()[&selected].label,
                "Checked detached",
                "forwarded navigation uses child-owned facts, not root IDs"
            );
            assert!(
                parent
                    .state
                    .borrow()
                    .order
                    .iter()
                    .all(|id| parent.entries.borrow().contains_key(id)),
                "forwarded facts cannot overwrite root order"
            );
            h.key("Menu", Key::End);
            h.key("Menu", Key::ArrowRight);
            h.key("Menu", Key::End);
            let deepest = parent.navigation_owner(false);
            assert!(deepest.id != parent.id && deepest.id != child_context.id);
            let selected = deepest.state.borrow().active.unwrap();
            assert_eq!(deepest.entries.borrow()[&selected].label, "Deep last");
            assert_eq!(
                parent.active_dom_id(),
                Some(format!("ix-dropdown-item-{selected}")),
                "deep projection and ARIA use the same Rust owner"
            );
        }
        let mut h = Harness::with_settings(Fixture {
            focus_checked: true,
            linked_veto: true,
            ..Default::default()
        });
        let parent = h.context();
        let child = h.config.borrow().linked.get().unwrap();
        h.key("Menu", Key::ArrowDown);
        let result = h.key("Menu", Key::ArrowRight);
        assert!(!result.0);
        assert!(!child.is_open());
        assert!(
            parent.state.borrow().forward == Some(child),
            "veto does not undo source forward intent"
        );
        assert_eq!(
            child.context().state.borrow().focus_target,
            "none",
            "veto cannot publish a native focus target"
        );
    }
    #[test]
    fn dropdown_configuration_updates_live_blocked_triggers_and_open_mode_focus_intent() {
        let _guard = crate::test_event_converter_guard();
        let mut h = Harness::new();
        let context = h.context();
        h.click("Menu");
        h.config.borrow_mut().mode = DropdownNavigationMode::RovingTabindex;
        h.update();
        assert_eq!(
            context.state.borrow().focus_pending,
            Some(FocusRequest {
                last: false,
                allow_checked: true
            }),
            "open roving transition initializes current first/checked via DOM order read"
        );
        h.config.borrow_mut().mode = DropdownNavigationMode::ActiveDescendant;
        h.update();
        assert_eq!(context.state.borrow().focus_target, "trigger");
        h.click("Menu");
        h.config.borrow_mut().mode = DropdownNavigationMode::RovingTabindex;
        h.update();
        assert_eq!(context.state.borrow().focus_target, "none");
        assert_eq!(
            context.state.borrow().focus_pending,
            None,
            "closed mode change does not start focus"
        );
        h.config.borrow_mut().linked_blocked = true;
        h.update();
        let linked = h.config.borrow().linked.get().unwrap();
        let html = h.html();
        let external = html
            .split("ix-dropdown-external-trigger")
            .nth(1)
            .unwrap()
            .split('>')
            .next()
            .unwrap();
        assert!(
            external
                .split_ascii_whitespace()
                .any(|token| token == "disabled" || token.starts_with("disabled="))
                && external.contains("aria-expanded=\"false\"")
        );
        assert!(
            h.attrs[&h.target("Linked").raw()]
                .get("aria-disabled")
                .is_some_and(|value| value == "true")
        );
        h.click("Linked");
        h.click("External trigger");
        assert!(!linked.is_open());
        h.config.borrow_mut().linked_blocked = false;
        h.update();
        h.click("External trigger");
        assert!(linked.is_open(), "re-enabling reaches the same controller");
        h.config.borrow_mut().loading = true;
        h.update();
        assert!(
            h.attrs[&h.target("Menu").raw()]
                .get("aria-busy")
                .is_some_and(|value| value == "true")
        );
    }
    #[test]
    fn dropdown_checked_first_last_and_disabled_focus_key_flags_follow_source() {
        let _guard = crate::test_event_converter_guard();
        let mut h = Harness::with_settings(Fixture {
            focus_checked: true,
            ..Default::default()
        });
        let context = h.context();
        h.key("Menu", Key::End);
        let active = context.state.borrow().active.unwrap();
        assert_eq!(
            context.entries.borrow()[&active].label,
            "Explicit",
            "last ignores checked preference"
        );
        h.click("Menu");
        h.key("Menu", Key::ArrowDown);
        let active = context.state.borrow().active.unwrap();
        assert_eq!(
            context.entries.borrow()[&active].label,
            "Linked",
            "first can honor checked preference"
        );
        h.click("Menu");
        h.config.borrow_mut().no_focus = true;
        h.update();
        let result = h.key("Menu", Key::ArrowDown);
        assert!(
            result.0 && !result.1,
            "disabled focus handling prevents and stops opening key"
        );
        assert!(context.state.borrow().active.is_none());
        h.click("Menu");
        h.config.borrow_mut().no_focus = false;
        h.update();
        h.click("Menu");
        assert!(context.state.borrow().active.is_none());
        let enter = h.key("Menu", Key::Enter);
        let space = h.key("Menu", Key::Character(" ".into()));
        assert!(
            enter.0 && space.0,
            "open ordinary trigger activation cancels even without active item"
        );
        assert!(*context.open.peek());
    }
    #[test]
    fn dropdown_order_selection_uses_current_owned_candidates_and_handles_unknown_current() {
        let entries = [
            (1, "Alpha", false),
            (2, "Disabled", true),
            (3, "Gamma", false),
            (4, "Delta", false),
        ]
        .into_iter()
        .map(|(id, label, disabled)| {
            (
                id,
                MenuEntry {
                    host_id: format!("ix-dropdown-item-{id}"),
                    label: label.into(),
                    disabled,
                    checked: false,
                    mounted: None,
                    submenu: None,
                },
            )
        })
        .collect();
        assert_eq!(
            menu_target(&entries, &[4, 1, 2, 3, 99], Some(4), "ArrowDown"),
            Some(1)
        );
        assert_eq!(
            menu_target(&entries, &[4, 1, 2, 3], Some(1), "ArrowUp"),
            Some(4)
        );
        assert_eq!(
            menu_target(&entries, &[3, 2, 1], None, "ArrowDown"),
            Some(3)
        );
        assert_eq!(menu_target(&entries, &[3, 2, 1], None, "ArrowUp"), Some(1));
        assert_eq!(menu_target(&entries, &[2, 99], None, "Home"), None);
        assert_eq!(menu_target(&entries, &[4, 1, 2, 3], Some(4), "g"), Some(3));
    }
    #[derive(Clone, Default)]
    struct TriggerFocusMount {
        name: String,
        calls: Rc<RefCell<Vec<String>>>,
        pending: Rc<Cell<bool>>,
        started: Rc<Cell<bool>>,
        dropped: Rc<Cell<bool>>,
        waiter: Rc<RefCell<Option<std::task::Waker>>>,
    }
    struct TriggerFocusAttempt(Rc<Cell<bool>>);
    impl Drop for TriggerFocusAttempt {
        fn drop(&mut self) {
            self.0.set(true);
        }
    }
    impl dioxus::html::RenderedElementBacking for TriggerFocusMount {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
        fn set_focus(
            &self,
            focus: bool,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = dioxus::html::MountedResult<()>>>>
        {
            assert!(focus);
            let mock = self.clone();
            let guard = TriggerFocusAttempt(self.dropped.clone());
            Box::pin(async move {
                let _guard = guard;
                mock.started.set(true);
                std::future::poll_fn(|cx| {
                    if mock.pending.get() {
                        *mock.waiter.borrow_mut() = Some(cx.waker().clone());
                        std::task::Poll::Pending
                    } else {
                        std::task::Poll::Ready(())
                    }
                })
                .await;
                mock.calls.borrow_mut().push(mock.name.clone());
                Ok(())
            })
        }
    }
    fn mount_trigger_mock(h: &mut Harness, label: &str) -> TriggerFocusMount {
        let mock = TriggerFocusMount {
            name: label.into(),
            ..Default::default()
        };
        dispatch(&h.dom, "mounted", h.target(label), mock.clone());
        h.settle();
        mock
    }
    #[test]
    fn mounted_trigger_escape_and_explicit_close_run_as_dioxus_effects_in_both_modes() {
        let _guard = crate::test_event_converter_guard();
        for mode in [
            DropdownNavigationMode::ActiveDescendant,
            DropdownNavigationMode::RovingTabindex,
        ] {
            let mut h = Harness::with_settings(Fixture {
                mode,
                ..Default::default()
            });
            let mock = mount_trigger_mock(&mut h, "Menu");
            let context = h.context();
            h.click("Menu");
            h.key("Menu", Key::Escape);
            assert!(!*context.open.peek());
            assert_eq!(*mock.calls.borrow(), ["Menu"]);
            h.click("Menu");
            h.dom
                .in_runtime(|| assert!(context.request(false, Reason::Trigger, true)));
            assert_eq!(
                mock.calls.borrow().len(),
                1,
                "admission never focuses synchronously"
            );
            h.settle();
            assert_eq!(*mock.calls.borrow(), ["Menu", "Menu"]);
            assert!(h.html().contains(r#"data-ix-dropdown-focus-owned="true""#));
        }
    }
    #[test]
    fn native_observed_cleanup_and_open_active_descendant_mode_change_focus_the_mount() {
        let _guard = crate::test_event_converter_guard();
        let mut h = Harness::new();
        let mock = mount_trigger_mock(&mut h, "Menu");
        h.click("Menu");
        let context = h.context();
        let node = context.node.borrow().clone().unwrap();
        let runtime = node.runtime().unwrap();
        let transport = h
            .attrs
            .iter()
            .find(|(_, attrs)| attrs.get("data-ix-overlay-transport") == Some(&runtime.owner_id()))
            .map(|(id, _)| dioxus::core::ElementId::from_raw(*id))
            .unwrap();
        dispatch(&h.dom, "input", transport, serde_json::json!({"owner":runtime.owner_id(),"document":format!("trigger-focus-test-{}", runtime.owner_id()),"sequence":1,
            "kind":"modality","input_kind":"keydown","key":"Tab",
            "nodes":[{"id":node.id(),"generation":node.generation(),"connected":true,"discovery_ready":true,
                "root_capture":"mounted-trigger-root","trigger_capture":"mounted-trigger-control"}]}).to_string());
        assert!(context.native_key_route());
        assert!(
            runtime.keyboard_mode(),
            "ordinary cleanup reads document modality, so the fixture must attach its document"
        );
        h.dom
            .in_runtime(|| assert!(context.request(false, Reason::AnchorVisibility, true)));
        h.settle();
        assert_eq!(*mock.calls.borrow(), ["Menu"]);
        h.click("Menu");
        h.config.borrow_mut().mode = DropdownNavigationMode::RovingTabindex;
        h.update();
        h.config.borrow_mut().mode = DropdownNavigationMode::ActiveDescendant;
        h.update();
        assert!(*context.open.peek());
        assert_eq!(*mock.calls.borrow(), ["Menu", "Menu"]);
    }
    #[test]
    fn mounted_trigger_outside_tab_and_stale_close_requests_preserve_the_destination() {
        let _guard = crate::test_event_converter_guard();
        let mut h = Harness::with_settings(Fixture {
            mode: DropdownNavigationMode::RovingTabindex,
            ..Default::default()
        });
        let mock = mount_trigger_mock(&mut h, "Menu");
        let context = h.context();
        h.click("Menu");
        h.dom
            .in_runtime(|| assert!(context.request(false, Reason::Outside, false)));
        h.settle();
        assert!(mock.calls.borrow().is_empty());
        h.click("Menu");
        h.key("Menu", Key::Tab);
        assert!(!*context.open.peek());
        assert!(mock.calls.borrow().is_empty());
        h.dom.in_runtime(|| {
            assert!(context.request(true, Reason::Trigger, false));
            assert!(context.request(false, Reason::Trigger, true));
            assert!(context.request(true, Reason::Trigger, false));
        });
        h.settle();
        assert!(
            mock.calls.borrow().is_empty(),
            "reopen cancels the queued close focus"
        );
        h.dom
            .in_runtime(|| assert!(context.request(false, Reason::Trigger, true)));
        let replacement = mount_trigger_mock(&mut h, "Menu");
        assert!(mock.calls.borrow().is_empty());
        assert!(
            replacement.calls.borrow().is_empty(),
            "a stale request cannot focus the replacement mount"
        );
    }
    #[test]
    fn trigger_focus_cancels_pending_tasks_and_rejects_disabled_loading_and_dropped_surfaces() {
        let _guard = crate::test_event_converter_guard();
        for loading in [false, true] {
            let mut h = Harness::new();
            let mock = mount_trigger_mock(&mut h, "Menu");
            let context = h.context();
            h.click("Menu");
            h.dom
                .in_runtime(|| assert!(context.request(false, Reason::Trigger, true)));
            if loading {
                h.config.borrow_mut().loading = true;
            } else {
                h.config.borrow_mut().blocked = true;
            }
            h.update();
            assert!(mock.calls.borrow().is_empty());
        }
        let mut h = Harness::new();
        let mock = mount_trigger_mock(&mut h, "Menu");
        mock.pending.set(true);
        let context = h.context();
        h.click("Menu");
        h.dom
            .in_runtime(|| assert!(context.request(false, Reason::Trigger, true)));
        h.settle();
        assert!(mock.started.get());
        assert!(!mock.dropped.get());
        h.dom.in_runtime(|| context.invalidate_focus());
        assert!(
            mock.dropped.get(),
            "a newer focus epoch cancels the pending mounted operation"
        );
        mock.pending.set(false);
        mock.waiter.borrow_mut().take().unwrap().wake();
        h.settle();
        assert!(mock.calls.borrow().is_empty());
        let dropping = mount_trigger_mock(&mut h, "Menu");
        dropping.pending.set(true);
        h.click("Menu");
        h.dom
            .in_runtime(|| assert!(context.request(false, Reason::Trigger, true)));
        h.settle();
        assert!(dropping.started.get());
        h.config.borrow_mut().visible = false;
        h.update();
        assert!(dropping.dropped.get());
        assert!(dropping.calls.borrow().is_empty());
        assert!(!context.state.borrow().surface);
    }
    #[test]
    fn linked_item_and_external_trigger_focus_use_the_current_registered_mount() {
        let _guard = crate::test_event_converter_guard();
        let mut h = Harness::new();
        let linked = h.config.borrow().linked.get().unwrap();
        let context = linked.context();
        let item = mount_trigger_mock(&mut h, "Linked");
        h.dom.in_runtime(|| {
            assert!(context.request(true, Reason::Keyboard, false));
            assert!(context.request(false, Reason::Left, true));
        });
        h.settle();
        assert_eq!(*item.calls.borrow(), ["Linked"]);
        let external = mount_trigger_mock(&mut h, "External trigger");
        h.dom.in_runtime(|| {
            assert!(context.request(true, Reason::Trigger, false));
            assert!(context.request(false, Reason::Trigger, true));
        });
        h.settle();
        assert_eq!(*external.calls.borrow(), ["External trigger"]);
        assert_eq!(item.calls.borrow().len(), 1);
    }
    #[derive(Clone, Default)]
    struct CustomTriggerFixture(Rc<CustomTriggerState>);
    #[derive(Default)]
    struct CustomTriggerState {
        dropdown: Cell<Option<DropdownHandle>>,
        first: Cell<Option<Signal<bool>>>,
        second: Cell<Option<Signal<bool>>>,
    }
    #[component]
    fn CustomTriggerControl(dropdown: DropdownHandle, name: String) -> Element {
        let mounted = use_dropdown_trigger_mount(dropdown);
        rsx! { button { id: name.clone(), "data-ix-dropdown-label":name, onmounted:mounted, "Custom trigger" } }
    }
    fn custom_trigger_fixture(fixture: CustomTriggerFixture) -> Element {
        let dropdown = use_dropdown();
        let first = use_signal(|| true);
        let second = use_signal(|| true);
        fixture.0.dropdown.set(Some(dropdown));
        fixture.0.first.set(Some(first));
        fixture.0.second.set(Some(second));
        rsx! {
            if first() { CustomTriggerControl { dropdown, name:"first" } }
            if second() { CustomTriggerControl { dropdown, name:"second" } }
            Dropdown { controller:dropdown, trigger:Some("second".into()), label:"Custom popup", DropdownItem { label:"Custom child" } }
        }
    }
    #[test]
    fn custom_trigger_scope_cleanup_preserves_newer_registration_and_releases_removed_current_mount()
     {
        let _guard = crate::test_event_converter_guard();
        dioxus::html::set_event_converter(Box::new(Converter));
        let fixture = CustomTriggerFixture::default();
        let mut dom = VirtualDom::new_with_props(custom_trigger_fixture, fixture.clone());
        let edits = dom.rebuild_to_vec();
        let mut h = Harness {
            dom,
            config: Rc::new(RefCell::new(Fixture::default())),
            attrs: BTreeMap::new(),
            listeners: BTreeMap::new(),
        };
        h.collect(edits);
        for target in h.listeners.get("mounted").cloned().unwrap_or_default() {
            dispatch(&h.dom, "mounted", target, ());
        }
        h.settle();
        let first = mount_trigger_mock(&mut h, "first");
        let second = mount_trigger_mock(&mut h, "second");
        let context = fixture.0.dropdown.get().unwrap().context();
        let runtime = context.node.borrow().as_ref().unwrap().runtime().unwrap();
        assert!(
            runtime.mounted_focus_target("second").is_some(),
            "the custom trigger hook also registers outside-Tab focus"
        );
        let previous_binding = {
            let state = context.state.borrow();
            (state.declaration.version, state.declaration.dispose_version)
        };
        let previous_second = second;
        let second = mount_trigger_mock(&mut h, "second");
        let replacement_binding = {
            let state = context.state.borrow();
            (state.declaration.version, state.declaration.dispose_version)
        };
        assert!(
            replacement_binding.0 > previous_binding.0,
            "a new mounted handle for the same declared ID must refresh native resolution"
        );
        assert!(
            replacement_binding.1 > previous_binding.1,
            "the replacement releases the old native event binding"
        );
        assert!(previous_second.calls.borrow().is_empty());
        let current = context.trigger.borrow().clone().unwrap();
        context.mount_trigger(current.node.clone(), current.blocked.clone());
        drop(current);
        h.settle();
        assert_eq!(
            {
                let state = context.state.borrow();
                (state.declaration.version, state.declaration.dispose_version)
            },
            replacement_binding,
            "registering the same handle and rerendering must not cause an event rebind loop"
        );
        assert!(h.html().contains(&format!(
            r#"data-ix-overlay-trigger-version="{}""#,
            replacement_binding.0
        )));
        h.dom
            .in_runtime(|| fixture.0.first.get().unwrap().set(false));
        h.settle();
        assert_eq!(
            {
                let state = context.state.borrow();
                (state.declaration.version, state.declaration.dispose_version)
            },
            replacement_binding,
            "cleanup from the older trigger scope cannot dispose the newer binding"
        );
        h.dom.in_runtime(|| {
            assert!(context.request(true, Reason::Trigger, false));
            assert!(context.request(false, Reason::Trigger, true));
        });
        h.settle();
        assert!(first.calls.borrow().is_empty());
        assert_eq!(
            *second.calls.borrow(),
            ["second"],
            "unmounting the earlier binding cannot clear its replacement"
        );
        let target = Rc::downgrade(&context.trigger.borrow().as_ref().unwrap().node);
        h.dom.in_runtime(|| {
            assert!(context.request(true, Reason::Trigger, false));
            assert!(context.request(false, Reason::Trigger, true));
            fixture.0.second.get().unwrap().set(false);
        });
        h.settle();
        assert!(context.trigger.borrow().is_none());
        assert!(
            runtime.mounted_focus_target("second").is_none(),
            "removing the trigger component releases its Tab destination"
        );
        assert_eq!(
            context.state.borrow().declaration.version,
            replacement_binding.0
        );
        assert!(
            context.state.borrow().declaration.dispose_version > replacement_binding.1,
            "removal of the current scope disposes the native listener without a new missing-ID lookup"
        );
        assert!(
            target.upgrade().is_none(),
            "removed custom scope releases its current handle"
        );
        assert_eq!(second.calls.borrow().len(), 1);
        let runtime = context.node.borrow().as_ref().unwrap().runtime().unwrap();
        h.dom.in_runtime(|| {
            assert!(context.request(true, Reason::Trigger, false));
            assert!(context.request(false, Reason::Trigger, true));
        });
        assert_eq!(
            runtime.pending_completions(),
            2,
            "an ID-only trigger queues changed notifications but no trigger focus effect"
        );
        h.settle();
        assert_eq!(second.calls.borrow().len(), 1);
    }
    #[derive(Default)]
    struct OpeningQuery {
        scope: Option<ScopeId>,
        reply: Option<serde_json::Value>,
        waiter: Option<std::task::Waker>,
        finished: bool,
    }
    #[derive(Default)]
    struct OpeningDocument {
        owner: dioxus::signals::Owner,
        queries: RefCell<Vec<Rc<RefCell<OpeningQuery>>>>,
    }
    struct OpeningEvaluator(Rc<RefCell<OpeningQuery>>);
    impl dioxus::document::Document for OpeningDocument {
        fn eval(&self, script: String) -> dioxus::document::Eval {
            if !script.contains("?.inventoryReaders?.get(owner)") {
                return dioxus::document::Document::eval(&dioxus::document::NoOpDocument, script);
            }
            let query = Rc::new(RefCell::new(OpeningQuery {
                scope: Some(dioxus::core::current_scope_id()),
                ..Default::default()
            }));
            self.queries.borrow_mut().push(query.clone());
            dioxus::document::Eval::new(
                self.owner.insert(
                    Box::new(OpeningEvaluator(query)) as Box<dyn dioxus::document::Evaluator>
                ),
            )
        }
    }
    impl dioxus::document::Evaluator for OpeningEvaluator {
        fn send(&self, value: serde_json::Value) -> Result<(), dioxus::document::EvalError> {
            if value["kind"] == "finish" {
                self.0.borrow_mut().finished = true;
            }
            Ok(())
        }
        fn poll_recv(
            &mut self,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Result<serde_json::Value, dioxus::document::EvalError>> {
            let mut query = self.0.borrow_mut();
            if let Some(reply) = query.reply.take() {
                std::task::Poll::Ready(Ok(reply))
            } else {
                query.waiter = Some(cx.waker().clone());
                std::task::Poll::Pending
            }
        }
        fn poll_join(
            &mut self,
            _: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Result<serde_json::Value, dioxus::document::EvalError>> {
            panic!("the inventory query must receive its acknowledged result");
        }
    }
    fn opening_reply(h: &mut Harness, query: &Rc<RefCell<OpeningQuery>>, facts: &InventoryFacts) {
        let waiter = {
            let mut query = query.borrow_mut();
            query.reply = Some(serde_json::json!({"value":facts,"error":null}));
            query.waiter.take()
        };
        if let Some(waiter) = waiter {
            waiter.wake();
        }
        h.settle();
    }
    #[test]
    fn opening_uses_registered_first_last_checked_and_custom_controls_in_both_modes() {
        let _guard = crate::test_event_converter_guard();
        for mode in [
            DropdownNavigationMode::ActiveDescendant,
            DropdownNavigationMode::RovingTabindex,
        ] {
            let document = Rc::new(OpeningDocument::default());
            let mut h = Harness::with_document(
                Fixture {
                    mode,
                    focus_checked: true,
                    ..Default::default()
                },
                Some(document.clone()),
            );
            let trigger = mount_trigger_mock(&mut h, "Menu");
            h.click("Menu");
            let context = h.context();
            let mut facts = mixed_inventory_facts();
            facts.hosts[1].checked_attribute = true;
            let extra = |name: &str, disabled: bool, hidden: bool, negative: bool| {
                let mut host = facts.hosts[1].clone();
                host.item = ItemRef(name.into());
                host.capture = name.into();
                host.target = name.into();
                host.registration_key = name.into();
                host.checked_attribute = false;
                host.disabled_attribute = disabled;
                host.hidden_attribute = hidden;
                host.tabindex_attribute = negative.then(|| "-1".into());
                host
            };
            let mut hosts = vec![
                extra("disabled-opening", true, false, false),
                extra("hidden-opening", false, true, false),
                extra("unregistered-opening", false, false, false),
                extra("negative-opening", false, false, true),
            ];
            hosts.extend(facts.hosts.clone());
            facts.hosts = hosts;
            facts.query_order = facts.hosts.iter().map(|host| host.item.clone()).collect();
            facts.navigation_order = facts.query_order.clone();
            let mut registered = facts.clone();
            registered
                .hosts
                .retain(|host| host.target != "unregistered-opening");
            let focuses = mount_inventory(&h, &registered);
            for (last, checked, target) in [
                (
                    false,
                    false,
                    if mode == DropdownNavigationMode::ActiveDescendant {
                        "mixed-category"
                    } else {
                        "negative-opening"
                    },
                ),
                (true, true, "mixed-native"),
                (false, true, "mixed-menu"),
            ] {
                h.dom
                    .in_runtime(|| context.focus_edge_with_checked(last, checked));
                h.settle();
                let query = document.queries.borrow().last().unwrap().clone();
                assert_eq!(
                    query.borrow().scope,
                    context
                        .node
                        .borrow()
                        .as_ref()
                        .unwrap()
                        .with_owner(dioxus::core::current_scope_id),
                    "query belongs to the surface owner"
                );
                opening_reply(&mut h, &query, &facts);
                assert_eq!(focuses.borrow().last().map(String::as_str), Some(target));
            }
            assert!(trigger.calls.borrow().is_empty());
            let runtime = context.node.borrow().as_ref().unwrap().runtime().unwrap();
            for host in &registered.hosts {
                runtime.unregister_item_cue(if host.registration_key.is_empty() {
                    &host.capture
                } else {
                    &host.registration_key
                });
            }
            h.dom.in_runtime(|| context.focus_edge(false));
            h.settle();
            let query = document.queries.borrow().last().unwrap().clone();
            opening_reply(&mut h, &query, &facts);
            assert_eq!(
                trigger.calls.borrow().len(),
                usize::from(mode == DropdownNavigationMode::ActiveDescendant),
                "no registered candidate uses Dioxus trigger focus in AD mode"
            );
        }
    }
    #[test]
    fn opening_inventory_await_cancels_on_new_intent_close_mode_disabled_and_drop() {
        let _guard = crate::test_event_converter_guard();
        let document = Rc::new(OpeningDocument::default());
        let mut h = Harness::with_document(Fixture::default(), Some(document.clone()));
        h.click("Menu");
        let context = h.context();
        let facts = mixed_inventory_facts();
        let focuses = mount_inventory(&h, &facts);
        h.dom.in_runtime(|| context.focus_edge(false));
        h.settle();
        let first = document.queries.borrow().last().unwrap().clone();
        h.dom.in_runtime(|| context.focus_edge(true));
        h.settle();
        assert!(
            first.borrow().finished,
            "newer opening intent drops the old query channel"
        );
        let latest = document.queries.borrow().last().unwrap().clone();
        opening_reply(&mut h, &first, &facts);
        assert!(focuses.borrow().is_empty());
        opening_reply(&mut h, &latest, &facts);
        assert_eq!(*focuses.borrow(), ["mixed-native"]);
        h.dom.in_runtime(|| context.focus_submenu_entry());
        h.settle();
        let closing = document.queries.borrow().last().unwrap().clone();
        h.dom
            .in_runtime(|| assert!(context.request(false, Reason::Outside, false)));
        h.settle();
        assert!(closing.borrow().finished);
        opening_reply(&mut h, &closing, &facts);
        assert_eq!(focuses.borrow().len(), 1);
        h.dom.in_runtime(|| {
            assert!(context.request(true, Reason::Trigger, false));
            context.focus_edge(false);
        });
        h.settle();
        let old_mode = document.queries.borrow().last().unwrap().clone();
        h.config.borrow_mut().mode = DropdownNavigationMode::RovingTabindex;
        h.update();
        assert!(old_mode.borrow().finished);
        opening_reply(&mut h, &old_mode, &facts);
        assert_eq!(focuses.borrow().len(), 1);
        let roving = document.queries.borrow().last().unwrap().clone();
        h.config.borrow_mut().no_focus = true;
        h.update();
        opening_reply(&mut h, &roving, &facts);
        assert_eq!(
            focuses.borrow().len(),
            1,
            "disabled focus handling rejects a query admitted earlier"
        );
        h.config.borrow_mut().no_focus = false;
        h.update();
        h.dom.in_runtime(|| context.focus_edge(false));
        h.settle();
        let dropping = document.queries.borrow().last().unwrap().clone();
        h.config.borrow_mut().visible = false;
        h.update();
        assert!(dropping.borrow().finished);
        opening_reply(&mut h, &dropping, &facts);
        assert_eq!(focuses.borrow().len(), 1);
    }
    #[test]
    fn registered_native_control_keeps_ad_editing_defaults_while_escape_and_roving_still_route() {
        let _guard = crate::test_event_converter_guard();
        let mut h = Harness::new();
        h.click("Menu");
        let context = h.context();
        let facts = mixed_inventory_facts();
        let _focuses = mount_inventory(&h, &facts);
        let node = context.node.borrow().clone().unwrap();
        let runtime = node.runtime().unwrap();
        let transport = h
            .attrs
            .iter()
            .find(|(_, attrs)| attrs.get("data-ix-overlay-transport") == Some(&runtime.owner_id()))
            .map(|(id, _)| dioxus::core::ElementId::from_raw(*id))
            .unwrap();
        for (index, key) in ["Home", "End", "ArrowDown", "ArrowLeft", "Enter", " "]
            .into_iter()
            .enumerate()
        {
            let prevented = dispatch(&h.dom, "input", transport, serde_json::json!({"owner":runtime.owner_id(),"sequence":index+1,"kind":"key","phase":"bubble","key":key,
                "active_id":"mixed-native","hosts":[node.id()],"triggers":[node.id()],"nodes":[{"id":node.id(),"generation":node.generation(),"connected":true,
                    "mode":"active-descendant","inside":true,"inventory":facts}]}).to_string()).0;
            h.settle();
            assert!(!prevented, "native registered control keeps {key} default");
            assert!(*context.open.peek());
            assert_eq!(runtime.pending_item_focus_count(), 0);
        }
        h.config.borrow_mut().mode = DropdownNavigationMode::RovingTabindex;
        h.update();
        assert!(
            registered_inventory_key(&h, 7, "ArrowDown", &facts),
            "roving mode still owns item arrow navigation"
        );
        h.settle();
        h.dom
            .in_runtime(|| assert!(context.request(false, Reason::Escape, true)));
        h.settle();
        assert!(!*context.open.peek());
    }
    #[test]
    fn external_ad_item_keys_navigate_current_mounts_and_return_to_parent() {
        let _guard = crate::test_event_converter_guard();
        let mut h = Harness::with_settings(Fixture {
            focus_checked: true,
            ..Default::default()
        });
        let parent = h.context();
        let child = h.config.borrow().linked.get().unwrap().context();
        assert!(!parent.state.borrow().external && child.state.borrow().external);
        let inventory = |context: &DropdownContext| {
            let hosts = context
                .entries
                .borrow()
                .values()
                .map(|entry| {
                    let attrs = &h.attrs[&h.target(&entry.label).raw()];
                    serde_json::json!({"item":entry.host_id,"capture":entry.host_id,
                    "target":entry.host_id,"registration_key":attrs["data-ix-item-key"],
                    "kind":"dropdown","disabled_attribute":entry.disabled,
                    "checked_attribute":entry.checked,"visible_focus":entry.label=="Detached leaf"})
                })
                .collect::<Vec<_>>();
            let order = hosts
                .iter()
                .map(|host| host["item"].clone())
                .collect::<Vec<_>>();
            serde_json::from_value::<InventoryFacts>(
                serde_json::json!({"items_host":context.host_id(),
                "hosts":hosts,"query_order":order,"navigation_order":order}),
            )
            .unwrap()
        };
        let parent_facts = inventory(&parent);
        let child_facts = inventory(&child);
        let parent_focus = mount_inventory(&h, &parent_facts);
        let child_focus = mount_inventory(&h, &child_facts);
        let parent_node = parent.node.borrow().clone().unwrap();
        let child_node = child.node.borrow().clone().unwrap();
        let runtime = parent_node.runtime().unwrap();
        let transport = h
            .attrs
            .iter()
            .find(|(_, attrs)| attrs.get("data-ix-overlay-transport") == Some(&runtime.owner_id()))
            .map(|(id, _)| dioxus::core::ElementId::from_raw(*id))
            .unwrap();
        dispatch(&h.dom,"input",transport,serde_json::json!({"owner":runtime.owner_id(),"sequence":1,"kind":"observe","nodes":[
            {"id":parent_node.id(),"generation":parent_node.generation(),"connected":true,"discovery_ready":true,
                "root_capture":"parent-root","mode":"active-descendant","inventory":parent_facts,"own_inventory":parent_facts},
            {"id":child_node.id(),"generation":child_node.generation(),"connected":true,"discovery_ready":true,
                "root_capture":"child-root","mode":"active-descendant","trigger_ancestors":[parent_node.id()],"inventory":child_facts}
        ]}).to_string());
        let trigger = parent_facts
            .hosts
            .iter()
            .find(|host| {
                parent
                    .entries
                    .borrow()
                    .values()
                    .any(|entry| entry.label == "Linked" && entry.host_id == host.target)
            })
            .unwrap();
        assert!(parent_node.first_assigned_submenu() == h.config.borrow().linked.get());
        // Discovery precedes opening, as it does for mounted native surfaces.
        // Opening before the first report would retain an empty parent cache.
        h.dom.in_runtime(|| {
            assert!(parent.request(true, Reason::Trigger, false));
            parent.open_submenu(trigger);
        });
        h.settle();
        assert!(*parent.open.peek() && *child.open.peek());
        assert!(h.key("Detached leaf", Key::ArrowDown).0);
        let next = child
            .entries
            .borrow()
            .values()
            .find(|entry| entry.label == "Checked detached")
            .unwrap()
            .host_id
            .clone();
        assert_eq!(child_focus.borrow().last(), Some(&next));
        let left = h.key("Checked detached", Key::ArrowLeft);
        assert!(
            !left.1,
            "successful child return stops ancestor key handling"
        );
        assert!(!*child.open.peek() && *parent.open.peek());
        assert!(parent.state.borrow().forward.is_none());
        assert_eq!(parent_focus.borrow().last(), Some(&trigger.target));

        let host = &child_facts.hosts[0];
        let binding = ItemHostBinding {
            key: host.registration_key.clone(),
            host_id: host.target.clone(),
            generic: true,
            runtime: runtime.clone(),
            context: Some(child.clone()),
        };
        let event = || {
            KeyboardEvent::new(
                Rc::new(dioxus::html::KeyboardData::new(TestKeyboard(
                    Key::ArrowDown,
                    Modifiers::empty(),
                ))),
                true,
            )
        };
        h.dom.in_runtime(|| {
            assert!(
                !binding.navigation_key(event()),
                "closed owner does not route"
            )
        });
        h.dom.in_runtime(|| {
            child.request(true, Reason::Trigger, false);
        });
        child.state.borrow_mut().settings.mode = DropdownNavigationMode::RovingTabindex;
        h.dom.in_runtime(|| {
            assert!(
                !binding.navigation_key(event()),
                "roving retains its existing route"
            )
        });
        child.state.borrow_mut().settings.mode = DropdownNavigationMode::ActiveDescendant;
        child.state.borrow_mut().external = false;
        h.dom.in_runtime(|| {
            assert!(
                !binding.navigation_key(event()),
                "builtin surfaces retain their existing route"
            )
        });
        child.state.borrow_mut().external = true;
        let unregistered = ItemHostBinding {
            key: "unregistered-control".into(),
            ..binding.clone()
        };
        h.dom.in_runtime(|| {
            assert!(
                !unregistered.navigation_key(event()),
                "unregistered controls are not enrolled"
            )
        });
        let mut native_facts = child_facts.clone();
        native_facts.hosts[0].kind = ItemKind::NativeOptIn;
        dispatch(&h.dom,"input",transport,serde_json::json!({"owner":runtime.owner_id(),"sequence":2,"kind":"observe","nodes":[
            {"id":child_node.id(),"generation":child_node.generation(),"connected":true,"discovery_ready":true,
                "root_capture":"child-root","inventory":native_facts}
        ]}).to_string());
        h.dom.in_runtime(|| {
            assert!(
                !binding.navigation_key(event()),
                "registered native controls retain their editing route"
            )
        });
    }
}
