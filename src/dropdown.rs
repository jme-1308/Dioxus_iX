//! Persistent Dropdown primitives and trigger-linked overlay ownership.
//!
//! Copyright (c) 2023-2026 Siemens AG. SPDX-License-Identifier: MIT.
use crate::Icon;
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

/// Native focus policy. The default retains trigger focus and a Rust active item.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DropdownNavigationMode {
    #[default]
    ActiveDescendant,
    RovingTabindex,
}
impl DropdownNavigationMode {
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
    trigger: Rc<RefCell<Option<Rc<MountedData>>>>,
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
        self.navigation_owner(true)
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
        let task = self.focus_task.borrow_mut().take();
        if let Some(task) = task {
            task.cancel();
        }
        let (changed, generation, trigger_cleanup) = {
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
            let trigger_cleanup = (!change.open
                && change.restore_focus
                && !state.settings.disable_focus_trap
                && matches!(
                    change.reason,
                    Reason::AnchorVisibility
                        | Reason::AnchorVisibilityParent
                        | Reason::TriggerOther
                ))
            .then_some(state.focus_epoch);
            (changed, state.generation, trigger_cleanup)
        };
        let mut open = self.open;
        let Ok(mut current) = open.try_write() else {
            return;
        };
        *current = change.open;
        drop(current);
        if let Some(version) = trigger_cleanup
            && let Some(node) = self.node.borrow().clone()
            && node.queue_trigger_restore(version)
        {
            // Native cleanup is an independently queued source task, not the
            // latest rendered focus intent. A same-owner reopen cannot erase it.
            self.state.borrow_mut().focus_target = "none".into();
        }
        self.notify();
        let completion_node = if changed {
            self.node.borrow().clone()
        } else {
            None
        };
        if let Some(node) = completion_node {
            let context = self.clone();
            // Every changed value owns a native-frame notification, even when
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
        {
            let mut state = self.state.borrow_mut();
            if !state.alive || !*self.open.peek() {
                return;
            }
            // Roving tabindex changes synchronously in the source. The default
            // active-descendant marker changes when its first RAF runs.
            if focus || !nearest {
                state.active = Some(target);
            }
            state.focus_pending = None;
            state.focus_epoch = state.focus_epoch.wrapping_add(1);
            state.focus_target = if focus && !nearest {
                format!("ix-dropdown-item-{target}")
            } else {
                "none".into()
            };
        }
        self.notify();
        if nearest {
            let node = self.node.borrow().clone();
            if let Some(node) = node {
                let context = self.clone();
                node.queue_item_focus(
                    format!("ix-dropdown-item-{target}"),
                    focus,
                    Rc::new(move || {
                        // An admitted source focusItem survives later open,
                        // mode and key changes, but cannot adopt a new item.
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
    fn focus_edge_with_checked(&self, last: bool, allow_checked: bool) {
        let task = self.focus_task.borrow_mut().take();
        if let Some(task) = task {
            task.cancel();
        }
        if self.state.borrow().settings.disable_focus_handling {
            return;
        }
        let epoch = {
            let mut state = self.state.borrow_mut();
            state.focus_epoch = state.focus_epoch.wrapping_add(1);
            state.focus_pending = Some(FocusRequest {
                last,
                allow_checked,
            });
            state.focus_epoch
        };
        let context = self.clone();
        let task = spawn(async move {
            let query = crate::dom_query::eval(
                r#"const id=await dioxus.recv();const host=document.getElementById(id);if(!host?.isConnected)return null;return [...host.querySelectorAll('[data-ix-dropdown-item]')].filter(n=>n.closest('[data-ix-overlay-host]')===host&&!n.disabled&&!n.hasAttribute('hidden')).map(n=>Number(n.getAttribute('data-ix-dropdown-item')));"#,
            );
            if query.send(context.host_id()).is_err() {
                return;
            }
            let Ok(Some(order)) = query.join::<Option<Vec<usize>>>().await else {
                return;
            };
            if !context.current(epoch) {
                return;
            }
            let mode = context.state.borrow().settings.mode;
            if let Some(target) = context.edge_target(&order, last, allow_checked) {
                context.publish_item(
                    target,
                    mode == DropdownNavigationMode::RovingTabindex,
                    mode == DropdownNavigationMode::RovingTabindex,
                );
            }
        });
        *self.focus_task.borrow_mut() = Some(task);
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
        if facts.default_prevented || !self.state.borrow().alive || self.state.borrow().blocked {
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
                    if let Some(target) = self.edge_target(&facts.items, last, true) {
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
                if let Some(target) = owner.edge_target(
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
                result.prevent = true;
                result.stop = true;
                let mut state = self.state.borrow_mut();
                state.focus_epoch = state.focus_epoch.wrapping_add(1);
                state.focus_target = if facts.shift {
                    "last-native".into()
                } else {
                    "first-native".into()
                };
                drop(state);
                self.notify();
            }
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
        if facts.key != "ArrowLeft" {
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
                            let checked = child.state.borrow().settings.mode
                                == DropdownNavigationMode::RovingTabindex;
                            child.focus_edge_with_checked(false, checked);
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
    let mount = context.trigger.clone();
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
        onmounted:move |event| *mount.borrow_mut()=Some(event.data()),
        onclick:move |event:MouseEvent| {if !blocked&&!click.state.borrow().blocked&&event.default_action_enabled()&&click.trigger_click_allowed(){let previous=*click.open.peek();click.request_from_trigger(!previous,Reason::Trigger,click.keyboard_mode());}},
        onkeydown:move |event:KeyboardEvent| {if !key.state.borrow().native{fallback_key(&key,event,true);}},
        if loading {crate::Spinner {size:crate::SpinnerSize::Sm,hide_track:true}}
        else if let Some(icon)=icon {Icon {name:icon,size:20}}
        {children}
    } }
}

/// Persistent source-default Dropdown. `controller` omits the built-in composed
/// trigger; use DropdownTrigger or a linked DropdownItem elsewhere. Ordinary
/// native click ancestry is preserved by the built-in trigger surrounding its
/// surface. Children keep their scopes and native nodes while closed.
#[component]
pub fn Dropdown(
    label: String,
    controller: Option<DropdownHandle>,
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
    let context = controller.unwrap_or(internal).context();
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
    rsx! { DropdownSurface {key:"{key}",context,label,external:controller.is_some(),placement,blocked:disabled||loading,loading,variant,icon,aria_label,class,settings,enable_top_layer,suppress_automatic_placement,{children}} }
}

#[component]
fn DropdownSurface(
    context: DropdownContext,
    label: String,
    external: bool,
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
        state.blocked = blocked;
        state.placement = placement;
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
                configure_node.close_children(Reason::Configuration);
                configure.invalidate_focus();
                {
                    let mut state = configure.state.borrow_mut();
                    state.active = None;
                    state.forward = None;
                }
                if *configure.open.peek() {
                    if mode == DropdownNavigationMode::RovingTabindex {
                        configure.focus_edge(false);
                    } else if !configure.state.borrow().settings.disable_focus_handling {
                        let mut state = configure.state.borrow_mut();
                        state.focus_target = "trigger".into();
                        state.focus_epoch = state.focus_epoch.wrapping_add(1);
                    }
                }
                configure.notify();
            }
        },
    ));
    let state = context.state.borrow();
    let generation = state.generation;
    let focus_target = state.focus_target.clone();
    let focus_version = state.focus_epoch;
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
    let mount_trigger = context.trigger.clone();
    let click_context = context.clone();
    let key_context = context.clone();
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
        "data-ix-overlay-top-layer":enable_top_layer.to_string(),"data-ix-overlay-mode":settings.mode.as_str(),
        "data-ix-overlay-trigger-visibility-check":(!settings.suppress_trigger_visibility_check).to_string(),
        "data-ix-overlay-focus-proxy":(settings.mode==DropdownNavigationMode::ActiveDescendant).to_string(),
        "data-ix-overlay-items-owner":items_owner,"data-ix-overlay-active-item":active_id,"data-ix-overlay-focus-target":focus_target,"data-ix-overlay-focus-version":focus_version.to_string(),
        style:if external{format!("--ix-dropdown-area:{}",effective_placement.area())}else{format!("anchor-name:--ix-dropdown-anchor-{};--ix-dropdown-area:{}",context.id,effective_placement.area())},
        onmounted:move |event|{mount_context.state.borrow_mut().mounted=true;if !external{*mount_trigger.borrow_mut()=Some(event.data());}mounted_node.sync_open(*mount_context.open.peek());let pending=mount_context.state.borrow().focus_pending;if *mount_context.open.peek(){if let Some(last)=pending{mount_context.focus_edge_with_checked(last.last,last.allow_checked);}}},
        onclick:move |event:MouseEvent| {if !external&&!blocked&&!click_context.state.borrow().blocked&&event.default_action_enabled()&&click_context.trigger_click_allowed(){let previous=*click_context.open.peek();click_context.request_from_trigger(!previous,Reason::Trigger,click_context.keyboard_mode());}},
        onkeydown:move |event:KeyboardEvent|{if !key_context.state.borrow().native{fallback_key(&key_context,event,!external);}},
        if !external {button {class:"ix-dropdown-trigger ix-button ix-button-{variant.as_str()}",r#type:"button",disabled:blocked,"aria-hidden":"true",inert:true,tabindex:-1,
            if loading {crate::Spinner {size:crate::SpinnerSize::Sm,hide_track:true}}
            else if let Some(icon)=icon {Icon {name:icon,size:20}}
            "{label}" Icon {name:"chevron-down-small",size:16}
        }}
        dialog {id:host_id,class:"ix-dropdown-menu ix-dropdown-root-menu",role:"menu","aria-label":label,
            "data-ix-overlay-host":id,"data-placement":effective_placement.as_str(),"data-configured-placement":placement.as_str(),hidden:!open,inert:(!open).then_some(""),
            popover:enable_top_layer.then_some("manual"),
            style:format!("position-anchor:--ix-dropdown-anchor-{};--ix-dropdown-area:{};{}",context.id,effective_placement.area(),if suppress_automatic_placement {"position-try-fallbacks:none"} else {""}),
            DropdownFocusRetention {menu_id:context.host_id(),enabled:settings.mode==DropdownNavigationMode::RovingTabindex}
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
        focused_item: None,
        items: order,
        focus_count: 0,
        focused_index: None,
        on_trigger,
        inside: !on_trigger,
        popover_ancestor: false,
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
) -> Element {
    let id = use_hook(|| NEXT_ID.fetch_add(1, Ordering::Relaxed));
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
                entry.label = label.clone();
                entry.disabled = blocked;
                entry.checked = checked;
                entry.submenu = dropdown;
            })
            .or_insert(MenuEntry {
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
    let (active, mode, owner) = context
        .as_ref()
        .map(|context| {
            let _ = (context.revision)();
            let state = context.state.borrow();
            (
                state.active == Some(id),
                state.settings.mode,
                Some(context.node_id()),
            )
        })
        .unwrap_or((false, DropdownNavigationMode::ActiveDescendant, None));
    let trigger_marker = linked.as_ref().map(DropdownContext::node_id);
    let linked_open = linked
        .as_ref()
        .map(|linked| (linked.open)())
        .unwrap_or(false);
    let mount_context = context.clone();
    let linked_mount = linked.clone();
    let key_context = context.clone();
    let click_label = label.clone();
    let key_label = label.clone();
    let tabindex = if blocked {
        -1
    } else if mode == DropdownNavigationMode::RovingTabindex {
        if active { 0 } else { -1 }
    } else {
        0
    };
    rsx! {div {
        id:"ix-dropdown-item-{id}",class:if compact{"ix-dropdown-item ix-dropdown-quick-action"}else{"ix-dropdown-item"},
        role:if checked||checkable{"menuitemcheckbox"}else{"menuitem"},"data-ix-dropdown-item":id.to_string(),
        "data-ix-dropdown-owner":owner,"data-ix-dropdown-label":label.clone(),
        style:linked.as_ref().map(|child|format!("anchor-name:--ix-dropdown-anchor-{}",child.id)),"data-ix-overlay-trigger":trigger_marker,
        "data-active":active.to_string(),"aria-disabled":blocked.to_string(),"disabled":blocked,
        "aria-label":compact.then_some(label.clone()),title:compact.then_some(label.clone()),
        "aria-haspopup":dropdown.map(|_|"menu"),"aria-expanded":dropdown.map(|_|linked_open.to_string()),
        "aria-controls":linked.as_ref().map(DropdownContext::host_id),
        "aria-checked":(checked||checkable).then_some(checked.to_string()),tabindex,
        onmounted:move |event|{if let Some(context)=&mount_context{if let Some(entry)=context.entries.borrow_mut().get_mut(&id){entry.mounted=Some(event.data());}}
            if let Some(linked)=&linked_mount{*linked.trigger.borrow_mut()=Some(event.data());}},
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
            if event.key()==Key::Enter||event.key()==Key::Character(" ".into()){onitemclick.call(DropdownItemClickEvent{label:key_label.clone(),prevented:Rc::new(Cell::new(false))});}
            if let Some(context)=&key_context{if !context.state.borrow().native{fallback_key(context,event.clone(),false);event.stop_propagation();}}
        }},
        if let Some(name)=icon{Icon {name,size:20}}
        if !compact{span {"{label}"}}
        {children}
        if checked{Icon {name:"single-check",size:16}}
        if dropdown.is_some(){Icon {name:"chevron-right-small",size:16}}
    }}
}

// Dioxus moves keyed DOM nodes with insertBefore, which can reset browser focus
// even though the Rust component and its element both survive. This bridge
// preserves only existing native focus; selection and navigation stay in Rust.
#[component]
fn DropdownFocusRetention(menu_id: String, enabled: bool) -> Element {
    let transport = use_hook(|| Rc::new(RefCell::new(None::<dioxus::document::Eval>)));
    let dropped = transport.clone();
    use_drop(move || {
        if let Some(task) = dropped.borrow_mut().take() {
            let _ = task.send("finish");
        }
    });
    use_effect(use_reactive(
        (&menu_id, &enabled),
        move |(menu_id, enabled)| {
            if let Some(previous) = transport.borrow_mut().take() {
                let _ = previous.send("finish");
            }
            if !enabled {
                return;
            }
            let task = dioxus::document::eval(DROPDOWN_FOCUS_RETENTION);
            if task.send(menu_id.clone()).is_ok() {
                if let Some(previous) = transport.borrow_mut().replace(task) {
                    let _ = previous.send("finish");
                }
            }
        },
    ));
    VNode::empty()
}

const DROPDOWN_FOCUS_RETENTION: &str = r#"
const id = await dioxus.recv(), menu = document.getElementById(id);
if (!menu) return;
const owned = target => {
  const item = target instanceof Element ? target.closest('[data-ix-dropdown-item]') : null;
  return item?.closest('[role="menu"]') === menu ? item : null;
};
let focused = owned(document.activeElement), disposed = false, removed;
const removal = new Promise(resolve => { removed = resolve; });
const fallback = () => document.activeElement === document.body || document.activeElement === menu;
const contains = (nodes, target) => [...nodes].some(node => node === target || node.contains(target));
const changed = records => {
  if (disposed) return;
  if (!menu.isConnected) { removed(); return; }
  const target = focused;
  if (!target || !target.isConnected || owned(target) !== target || (target.disabled||target.hasAttribute('disabled')) || !fallback()) return;
  if (records.some(record => contains(record.removedNodes, target)) &&
      records.some(record => contains(record.addedNodes, target))) {
    target.focus({preventScroll: true});
  }
};
const observer = new MutationObserver(changed);
const onFocus = event => { focused = owned(event.target); };
const clear = () => { focused = null; };
const onPointer = event => { if (!menu.contains(event.target)) clear(); };
const onFocusOut = event => {
  if (event.target !== focused) return;
  if (event.relatedTarget) { focused = owned(event.relatedTarget); return; }
  const target = focused;
  queueMicrotask(() => {
    if (focused !== target || disposed) return;
    // A move and its focusout belong to the same DOM mutation batch. An
    // explicit blur without that move must invalidate the remembered node.
    changed(observer.takeRecords());
    if (focused === target && fallback()) clear();
  });
};
document.addEventListener('focusin', onFocus, true);
document.addEventListener('focusout', onFocusOut, true);
document.addEventListener('pointerdown', onPointer, true);
window.addEventListener('blur', clear);
observer.observe(document.body, {childList: true, subtree: true});
try { await Promise.race([dioxus.recv(), removal]); }
finally {
  disposed = true; observer.disconnect();
  document.removeEventListener('focusin', onFocus, true);
  document.removeEventListener('focusout', onFocusOut, true);
  document.removeEventListener('pointerdown', onPointer, true);
  window.removeEventListener('blur', clear);
}
"#;

pub(crate) const STYLES: &str = r#"
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
        (@data Mounted,$event:ident)=>{MountedData::new(())};
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
        placement: DropdownPlacement,
        suppress_visibility: bool,
        suppress_automatic: bool,
        linked_veto: bool,
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
                placement: DropdownPlacement::default(),
                suppress_visibility: false,
                suppress_automatic: false,
                linked_veto: false,
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
        rsx! {div {
            if settings.visible {
            Dropdown {label:"Menu",navigation_mode:settings.mode,enable_top_layer:settings.layer,disabled:settings.blocked,loading:settings.loading,focus_checked_item:settings.focus_checked,disable_focus_handling:settings.no_focus,placement:settings.placement,suppress_trigger_visibility_check:settings.suppress_visibility,suppress_automatic_placement:settings.suppress_automatic,
                onbeforechange:move |event:DropdownChangeEvent|{let veto={let mut state=before.borrow_mut();state.requests.push((event.open,event.previous));if event.open{state.veto_open}else{state.veto_close}};if veto{event.prevent_default();}},
                onchange:move |open:bool|{let (close,capture)={let mut state=changed.borrow_mut();state.changed.push(open);(state.close_after_open&&open,state.context.clone())};let context=capture.borrow().clone();if let Some(context)=context{changed.borrow_mut().deliveries.push((settings.listener_revision,open,*context.open.peek()));if close{context.request(false,Reason::Trigger,false);}}},
                Stateful {config:config.clone()}
                DropdownItem {label:"Alpha",onitemclick:move |event:DropdownItemClickEvent|{let veto={let mut state=item.borrow_mut();state.item_requests+=1;state.item_veto};if veto{event.prevent_default();}},onclick:move |_|click.borrow_mut().native_clicks+=1}
                DropdownItem {label:"Disabled",disabled:true}
                DropdownItem {label:"Linked",dropdown:linked,checked:true}
                DropdownSubmenu {label:"Explicit",DropdownItem {label:"Persistent nested"}}
            }
            DropdownTrigger {dropdown:linked,aria_label:"External trigger","External trigger"}
            Dropdown {label:"Detached",controller:linked,disabled:settings.linked_blocked,focus_checked_item:true,onbeforechange:move |event:DropdownChangeEvent|{if event.open&&linked_before.borrow().linked_veto{event.prevent_default();}},DropdownItem {label:"Detached leaf"} DropdownItem {label:"Checked detached",checked:true} DropdownSubmenu {label:"Deep",DropdownItem {label:"Deep first"} DropdownItem {label:"Deep last"}}}
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
            dioxus::html::set_event_converter(Box::new(Converter));
            let config = Rc::new(RefCell::new(settings));
            let mut dom = VirtualDom::new_with_props(fixture, config.clone());
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
        /// A synthetic frame ACK exercises the real registered transport. The
        /// snapshot excludes work admitted by a callback in this frame.
        fn completion_frame(&mut self) -> usize {
            let node = self.context().node.borrow().clone();
            let Some(runtime) = node.and_then(|node| node.runtime()) else {
                return 0;
            };
            let frames = runtime.pending_completion_frames();
            let transport = self
                .attrs
                .iter()
                .find(|(_, attrs)| {
                    attrs.get("data-ix-overlay-transport") == Some(&runtime.owner_id())
                })
                .map(|(id, _)| dioxus::core::ElementId::from_raw(*id))
                .unwrap();
            let count = frames.len();
            for (sequence, root, generation) in frames {
                dispatch(
                    &self.dom,
                    "change",
                    transport,
                    serde_json::json!({
                        "owner":runtime.owner_id(),"sequence":0,"completion_sequence":sequence,
                        "completion_root":root,"completion_generation":generation
                    })
                    .to_string(),
                );
            }
            self.render_now();
            count
        }
        fn item_frames(&mut self) -> usize {
            let Some(runtime) = self
                .context()
                .node
                .borrow()
                .as_ref()
                .and_then(NodeHandle::runtime)
            else {
                return 0;
            };
            let jobs = runtime.pending_item_focus_jobs();
            let Some(transport) = self
                .attrs
                .iter()
                .find(|(_, attrs)| {
                    attrs.get("data-ix-overlay-transport") == Some(&runtime.owner_id())
                })
                .map(|(id, _)| dioxus::core::ElementId::from_raw(*id))
            else {
                return 0;
            };
            // Explicit synthetic protocol phases exercise real registered Rust
            // handlers. This does not establish native frame/scroll geometry.
            for phase in ["arm", "focus", "scroll"] {
                for job in &jobs {
                    dispatch(&self.dom, "change", transport, serde_json::json!({
                        "owner":runtime.owner_id(),"sequence":0,
                        "item_focus_sequence":job.sequence,"item_focus_root":job.root,
                        "item_focus_generation":job.generation,"item_focus_target":job.target,
                        "item_focus_capture":job.capture.clone().unwrap_or_else(|| format!("test-object-{}",job.target)),
                        "item_focus_phase":phase
                    }).to_string());
                }
            }
            self.render_now();
            jobs.len()
        }
        fn settle(&mut self) {
            for _ in 0..4 {
                self.render_now();
                self.completion_frame();
                self.item_frames();
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
    #[test]
    fn registered_rapid_navigation_keeps_each_job_and_source_active_timing() {
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
            let initial = context.state.borrow().active;
            for (index, key) in ["End", "Home", "End"].into_iter().enumerate() {
                let report = serde_json::json!({"owner":runtime.owner_id(),"sequence":index+1,"kind":"key",
                    "phase":if mode==DropdownNavigationMode::ActiveDescendant {"bubble"} else {"capture"},"key":key,
                    "hosts":[node.id()],"triggers":if mode==DropdownNavigationMode::ActiveDescendant {vec![node.id()]} else {vec![]},
                    "nodes":[{"id":node.id(),"generation":node.generation(),"connected":true,"discovery_ready":true,
                        "mode":mode.as_str(),"items":order,"inside":true,"on_trigger":false}]});
                assert!(dispatch(&h.dom, "input", transport, report.to_string()).0);
                if mode == DropdownNavigationMode::ActiveDescendant {
                    assert_eq!(
                        context.state.borrow().active,
                        initial,
                        "AD marker awaits first frame"
                    );
                }
            }
            let jobs = runtime.pending_item_focus_jobs();
            assert_eq!(jobs.len(), 3, "rapid calls must not coalesce");
            assert_eq!(jobs[0].target, jobs[2].target);
            assert_ne!(jobs[0].target, jobs[1].target);
            assert!(
                jobs.windows(2)
                    .all(|pair| pair[0].sequence < pair[1].sequence)
            );
            assert!(jobs.iter().all(|job|job.focus_native==(mode==DropdownNavigationMode::RovingTabindex)));
            // A first RAF for an earlier roving job must not undo the
            // tabindex selected synchronously by the last native key.
            let latest = context.state.borrow().active;
            let first = &jobs[0];
            for phase in ["arm", "focus"] {
                dispatch(&h.dom,"change",transport,serde_json::json!({
                    "owner":runtime.owner_id(),"sequence":0,"item_focus_sequence":first.sequence,
                    "item_focus_root":first.root,"item_focus_generation":first.generation,
                    "item_focus_target":first.target,"item_focus_capture":format!("test-object-{}",first.target),
                    "item_focus_phase":phase
                }).to_string());
            }
            if mode == DropdownNavigationMode::RovingTabindex {
                assert_eq!(context.state.borrow().active, latest);
            }
            h.item_frames();
            assert_eq!(
                context.active_dom_id().as_deref(),
                Some(jobs[2].target.as_str())
            );
            assert!(runtime.pending_item_focus_jobs().is_empty());
            for (index, key) in ["Home", "ArrowDown", "ArrowDown"].into_iter().enumerate() {
                let report = serde_json::json!({"owner":runtime.owner_id(),"sequence":index+4,"kind":"key",
                    "phase":if mode==DropdownNavigationMode::ActiveDescendant {"bubble"} else {"capture"},"key":key,
                    "hosts":[node.id()],"triggers":if mode==DropdownNavigationMode::ActiveDescendant {vec![node.id()]} else {vec![]},
                    "nodes":[{"id":node.id(),"generation":node.generation(),"connected":true,"discovery_ready":true,
                        "mode":mode.as_str(),"items":order,"inside":true,"on_trigger":false}]});
                assert!(dispatch(&h.dom, "input", transport, report.to_string()).0);
                if index == 0 {
                    h.item_frames();
                }
            }
            let down = runtime.pending_item_focus_jobs();
            assert_eq!(down.len(), 2);
            if mode == DropdownNavigationMode::ActiveDescendant {
                assert_eq!(
                    down[0].target, down[1].target,
                    "AD uses unchanged marker before first frame"
                );
            } else {
                assert_ne!(
                    down[0].target, down[1].target,
                    "roving uses synchronous tabindex before physical focus"
                );
            }
            h.item_frames();
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
    fn completion_payloads_wait_for_frame_keep_each_write_and_use_current_listener() {
        let _guard = crate::test_event_converter_guard();
        let mut h = Harness::new();
        let context = h.context();
        h.dom.in_runtime(|| {
            for value in [true, false, true] {
                assert!(context.request(value, Reason::Trigger, false));
            }
        });
        h.render_now();
        assert!(*context.open.peek());
        assert!(
            h.config.borrow().changed.is_empty(),
            "rendering is not a native frame"
        );
        h.config.borrow_mut().listener_revision = 7;
        h.config.borrow().update.clone().unwrap()();
        h.render_now();
        assert!(h.config.borrow().changed.is_empty());
        assert_eq!(h.completion_frame(), 3);
        assert_eq!(h.config.borrow().changed, [true, false, true]);
        assert_eq!(
            h.config.borrow().deliveries,
            [(7, true, true), (7, false, true), (7, true, true)]
        );
        assert_eq!(h.completion_frame(), 0);
    }

    #[test]
    fn completion_callback_reentrant_write_waits_for_another_frame_ack() {
        let _guard = crate::test_event_converter_guard();
        let mut h = Harness::with_settings(Fixture {
            close_after_open: true,
            ..Default::default()
        });
        let context = h.context();
        h.dom
            .in_runtime(|| assert!(context.request(true, Reason::Trigger, false)));
        h.render_now();
        assert!(h.config.borrow().changed.is_empty());
        assert_eq!(h.completion_frame(), 1);
        assert_eq!(h.config.borrow().changed, [true]);
        assert!(
            !*context.open.peek(),
            "the callback's close applies immediately"
        );
        assert_eq!(h.config.borrow().requests, [(true, false), (false, true)]);
        assert_eq!(h.completion_frame(), 1);
        assert_eq!(h.config.borrow().changed, [true, false]);
        assert_eq!(
            h.config.borrow().deliveries,
            [(0, true, true), (0, false, false)]
        );
        assert_eq!(h.completion_frame(), 0);
    }

    #[test]
    fn dropped_component_completion_cannot_call_the_replacement_listener() {
        let _guard = crate::test_event_converter_guard();
        let mut h = Harness::new();
        let old = h.context();
        let runtime = old.node.borrow().clone().unwrap().runtime().unwrap();
        h.dom
            .in_runtime(|| assert!(old.request(true, Reason::Trigger, false)));
        h.render_now();
        let pending = runtime.pending_completion_frames();
        assert_eq!(pending.len(), 1);
        h.config.borrow_mut().visible = false;
        h.update();
        assert!(runtime.pending_completion_frames().is_empty());
        assert!(h.config.borrow().changed.is_empty());
        h.config.borrow_mut().visible = true;
        h.config.borrow_mut().listener_revision = 9;
        h.update();
        let current = h.context();
        assert_ne!(current.id, old.id, "a new component has a new owner");
        let transport = h
            .attrs
            .iter()
            .find(|(_, attrs)| attrs.get("data-ix-overlay-transport") == Some(&runtime.owner_id()))
            .map(|(id, _)| dioxus::core::ElementId::from_raw(*id))
            .unwrap();
        let (sequence, root, generation) = &pending[0];
        let result = dispatch(
            &h.dom,
            "change",
            transport,
            serde_json::json!({
                "owner":runtime.owner_id(),"sequence":0,"completion_sequence":sequence,
                "completion_root":root,"completion_generation":generation
            })
            .to_string(),
        );
        assert!(!result.0, "an old frame has no permission to notify");
        assert!(h.config.borrow().changed.is_empty());
        h.dom
            .in_runtime(|| assert!(current.request(true, Reason::Trigger, false)));
        h.render_now();
        assert_eq!(h.completion_frame(), 1);
        assert_eq!(h.config.borrow().deliveries, [(9, true, true)]);
    }

    #[test]
    fn native_ordinary_tab_wraps_only_current_content_candidates() {
        let _guard = crate::test_event_converter_guard();
        for (inside, on_trigger, count, index, shift, expected) in [
            (false, true, 1, None, false, "none"),
            (false, true, 1, None, true, "none"),
            (false, true, 1, Some(0), false, "none"),
            (true, false, 1, None, false, "none"),
            (true, false, 1, Some(1), false, "none"),
            (true, false, 0, None, false, "none"),
            (true, false, 3, Some(1), false, "none"),
            (true, false, 3, Some(0), false, "none"),
            (true, false, 3, Some(2), false, "first-native"),
            (true, false, 3, Some(0), true, "last-native"),
            // A custom trigger within the content can itself be a candidate.
            (true, true, 1, Some(0), false, "first-native"),
        ] {
            let mut h = Harness::new();
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
            let before = h.config.borrow().requests.clone();
            let report = serde_json::json!({
                "owner":runtime.owner_id(),"sequence":1,"kind":"key","phase":"bubble",
                "key":"Tab","shift":shift,
                "hosts":if inside {vec![node.id()]} else {vec![]},
                "triggers":if on_trigger {vec![node.id()]} else {vec![]},
                "nodes":[{
                    "id":node.id(),"generation":node.generation(),"connected":true,
                    "discovery_ready":true,"mode":"active-descendant","open":true,
                    "on_trigger":on_trigger,"inside":inside,
                    "focus_count":count,"focused_index":index
                }]
            });
            let (prevented, propagates) = dispatch(&h.dom, "input", transport, report.to_string());
            h.settle();
            assert_eq!(prevented, expected != "none", "{report}");
            assert!(
                !propagates,
                "transport stays private; native stop uses the acknowledgement channel"
            );
            assert_eq!(context.state.borrow().focus_target, expected, "{report}");
            assert!(
                *context.open.peek(),
                "ordinary Tab leaves visibility unchanged"
            );
            assert_eq!(h.config.borrow().requests, before, "no visibility request");
        }
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
            assert_eq!(
                child_context.state.borrow().focus_pending,
                Some(FocusRequest {
                    last: false,
                    allow_checked: false
                }),
                "AD submenu's first focus excludes checked preference"
            );
            // The fallback has not delivered native focus; repeated keys must
            // never manufacture a click/toggle on the already open trigger.
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
                "fallback navigates child-owned facts, not root IDs"
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
}
