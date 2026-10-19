//! Owner-scoped dispatch with document-scoped Rust overlay coordination.
//! Native adapters supply read-only DOM observations and minimal global input.
//! Dioxus handlers own event defaults, propagation, activation and mounted focus.
use crate::dropdown_inventory::{
    HostFacts, HostPartFacts, HostPartKind, InventoryFacts, ItemKind, ItemRef,
};
use crate::overlay_completion::CompletionQueue;
use crate::overlay_domain::{Domain, DomainCallbacks};
use crate::overlay_registry::{CloseBehavior, NativeFact, OverlayKind, Registry};
use dioxus::prelude::*;
use serde::Deserialize;
use std::{
    cell::RefCell,
    collections::BTreeMap,
    rc::{Rc, Weak},
    sync::atomic::{AtomicUsize, Ordering},
};

static NEXT_RUNTIME: AtomicUsize = AtomicUsize::new(1);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Reason {
    Trigger,
    Keyboard,
    Escape,
    Left,
    Inside,
    Outside,
    TriggerOther,
    AnchorVisibility,
    AnchorVisibilityParent,
    Parent,
    Configuration,
    Drop,
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct Change {
    pub(crate) open: bool,
    pub(crate) reason: Reason,
    pub(crate) restore_focus: bool,
}
#[derive(Clone, Debug, Default)]
pub(crate) struct KeyFacts {
    pub(crate) navigation_mode: Option<String>,
    pub(crate) key: String,
    pub(crate) shift: bool,
    pub(crate) alt: bool,
    pub(crate) ctrl: bool,
    pub(crate) meta: bool,
    pub(crate) default_prevented: bool,
    pub(crate) focused_item: Option<usize>,
    pub(crate) focused_target: Option<String>,
    pub(crate) items: Vec<usize>,
    pub(crate) focus_count: usize,
    pub(crate) focusable_ids: Vec<String>,
    pub(crate) focused_index: Option<usize>,
    pub(crate) on_trigger: bool,
    pub(crate) inside: bool,
    pub(crate) popover_ancestor: bool,
    pub(crate) inventory: Option<InventoryFacts>,
}
#[derive(Clone, Debug, Default)]
pub(crate) struct KeyDecision {
    pub(crate) prevent: bool,
    pub(crate) stop: bool,
    pub(crate) release_focus: bool,
    pub(crate) activate: Option<String>,
}
type KeyCallback = Rc<dyn Fn(&KeyFacts) -> KeyDecision>;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
pub(crate) struct AnchorRect {
    pub(crate) top: f64,
    pub(crate) bottom: f64,
    pub(crate) left: f64,
    pub(crate) right: f64,
}
impl AnchorRect {
    fn valid(self) -> bool {
        [self.top, self.bottom, self.left, self.right]
            .into_iter()
            .all(f64::is_finite)
            && self.top <= self.bottom
            && self.left <= self.right
    }
}
#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
pub(crate) struct AnchorIntersection {
    pub(crate) bounding: AnchorRect,
    pub(crate) intersection: AnchorRect,
    #[serde(default)]
    pub(crate) anchor_in_dropdown_item: bool,
}
impl AnchorIntersection {
    fn valid(self) -> bool {
        self.bounding.valid() && self.intersection.valid()
    }
}
type AnchorIntersectionCallback = Rc<dyn Fn(AnchorIntersection)>;

#[derive(Clone)]
pub(crate) struct Callbacks {
    pub(crate) read: Rc<dyn Fn() -> Option<bool>>,
    pub(crate) before: Rc<dyn Fn(Change) -> bool>,
    pub(crate) apply: Rc<dyn Fn(Change)>,
    pub(crate) key: Option<KeyCallback>,
}
#[derive(Clone)]
struct Entry {
    generation: u64,
    revision: u64,
    mounted: bool,
    ownership_ready: bool,
    ancestors: Vec<NodeHandle>,
    before_operation: Option<u64>,
    kind: OverlayKind,
    behavior: CloseBehavior,
    trigger_visibility_check: bool,
    anchor_intersection: Option<AnchorIntersectionCallback>,
    anchor_shape: Option<Rc<dyn Fn(bool)>>,
    submenu: Option<SubmenuAccess>,
    navigation_mode: Option<String>,
    suppress_trigger_restore: bool,
    callbacks: Callbacks,
    scope: Option<(Weak<dioxus::core::Runtime>, ScopeId)>,
}
struct Runtime {
    owner: String,
    registry: Registry,
    nodes: BTreeMap<String, Entry>,
    order: Vec<String>,
    sequence: u64,
    observation: Option<Report>,
    keyboard: bool,
    completions: CompletionQueue,
    item_mounts: BTreeMap<String, ItemMount>,
    owned_item_focus: BTreeMap<String, OwnedItemFocus>,
    item_focus_sequence: u64,
    submenu_marks: BTreeMap<String, (String, u64)>,
    item_descriptions: BTreeMap<String, OwnedItemDescription>,
    item_description_sequence: u64,
    item_tabindices: BTreeMap<(String, u64), BTreeMap<String, ItemTabindex>>,
    item_tabindex_revision: u64,
    item_tabindex_signal: Option<Signal<u64>>,
    item_visual: Option<String>,
    item_visual_target: Option<String>,
    item_visual_known: bool,
    item_visual_host: Option<HostFacts>,
    item_kinds: BTreeMap<String, ItemKind>,
    item_visual_signal: Option<Signal<Option<String>>>,
    bridge: Option<dioxus::document::Eval>,
    domain: Option<Domain>,
    document: String,
    categories: BTreeMap<usize, CategoryCallback>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct OwnedItemDescription {
    owner: Option<(String, u64)>,
    host: HostFacts,
    order: u64,
}

#[derive(Clone)]
struct ItemMount {
    target: String,
    focus_id: String,
    owner: Option<(String, u64)>,
    node: Rc<MountedData>,
    renderer: Option<Weak<dioxus::core::Runtime>>,
}
/// A captured Dioxus destination retains no element or renderer ownership.
/// Replacing/unmounting its registration or changing documents invalidates it.
#[derive(Clone)]
pub(crate) struct MountedFocusTarget {
    runtime: Weak<RefCell<Runtime>>,
    key: String,
    logical_target: String,
    focus_id: String,
    node: Weak<MountedData>,
    document: String,
    renderer: Option<Weak<dioxus::core::Runtime>>,
}
impl MountedFocusTarget {
    /// Cleanup compares the captured element even after its registration was
    /// removed. Identity alone does not authorize a new focus operation.
    pub(crate) fn matches_node(&self, node: &Rc<MountedData>) -> bool {
        Weak::ptr_eq(&self.node, &Rc::downgrade(node))
    }
    pub(crate) fn live_node(&self) -> Option<Rc<MountedData>> {
        // Component cleanup unregisters its handle. Also reject a retained
        // registry after the renderer VirtualDom itself has been dropped.
        let _renderer = match &self.renderer {
            Some(owner) => Some(owner.upgrade()?),
            None => None,
        };
        let runtime = self.runtime.upgrade()?;
        let node = self.node.upgrade()?;
        let state = runtime.borrow();
        (state.document == self.document
            && state.item_mounts.get(&self.key).is_some_and(|mount| {
                mount.target == self.logical_target
                    && mount.focus_id == self.focus_id
                    && Rc::ptr_eq(&mount.node, &node)
            }))
        .then_some(node)
    }
}
struct OwnedItemFocus {
    sequence: u64,
    generation: u64,
    key: String,
    callback: Option<Rc<dyn Fn()>>,
    visual: Option<HostFacts>,
    focus_native: bool,
    exit_target: Option<MountedFocusTarget>,
    expected_open: Option<bool>,
    destination_popovers: Vec<String>,
    task: Option<(Weak<dioxus::core::Runtime>, dioxus::core::Task)>,
}

#[derive(Clone)]
struct SubmenuAccess {
    handle: crate::dropdown::DropdownHandle,
    query_root: Rc<dyn Fn() -> Option<String>>,
    reset: Rc<dyn Fn()>,
}

#[derive(Clone, Copy)]
struct ItemTabindex {
    revision: u64,
    value: i32,
}
type CategoryCallback = Rc<dyn Fn(usize, &[usize])>;

impl Runtime {
    /// Only explicitly described Dioxus members belong to an item inventory.
    /// Native observations may reorder these keys, never add or change a member.
    fn owned_inventory(&self, root: &str, generation: u64) -> Option<InventoryFacts> {
        if !self
            .nodes
            .get(root)
            .is_some_and(|entry| entry.generation == generation)
        {
            return None;
        }
        let owner = (root.to_string(), generation);
        let mut members = self
            .item_descriptions
            .values()
            .filter(|item| item.owner.as_ref() == Some(&owner))
            .collect::<Vec<_>>();
        members.sort_by_key(|item| item.order);
        let observed = self.observation.as_ref().and_then(|report| {
            report
                .nodes
                .iter()
                .find(|node| node.id == root && node.generation == generation)
        });
        let mut keys = Vec::with_capacity(members.len());
        if let Some(observed) = observed {
            for key in &observed.item_order {
                if !keys.contains(key)
                    && members
                        .iter()
                        .any(|item| item.host.registration_key == *key)
                {
                    keys.push(key.clone());
                }
            }
        }
        for item in &members {
            if !keys.contains(&item.host.registration_key) {
                keys.push(item.host.registration_key.clone());
            }
        }
        let hosts = keys
            .iter()
            .filter_map(|key| {
                self.item_descriptions
                    .get(key)
                    .map(|item| item.host.clone())
            })
            .collect::<Vec<_>>();
        let order = hosts
            .iter()
            .map(|host| host.item.clone())
            .collect::<Vec<_>>();
        Some(InventoryFacts {
            items_host: ItemRef(root.into()),
            hosts,
            navigation_order: order.clone(),
            query_order: order,
            slots: None,
        })
    }
}

impl Drop for Runtime {
    fn drop(&mut self) {
        self.completions.clear();

        self.item_descriptions.clear();
        if let Some(domain) = &self.domain {
            domain.remove_owner(&self.owner);
        }
        if let Some(bridge) = self.bridge.take() {
            let _ = bridge.send(serde_json::json!({"finish": true}));
        }
    }
}

#[derive(Clone)]
pub(crate) struct RuntimeHandle(Rc<RefCell<Runtime>>);
impl PartialEq for RuntimeHandle {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
#[derive(Clone)]
pub(crate) struct NodeHandle {
    runtime: Weak<RefCell<Runtime>>,
    id: String,
    generation: u64,
}
impl PartialEq for NodeHandle {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.generation == other.generation
            && Weak::ptr_eq(&self.runtime, &other.runtime)
    }
}
impl Eq for NodeHandle {}

#[derive(Clone, Copy)]
struct OverlayScope(Signal<Vec<NodeHandle>>);

/// Consume the parent once: a provider installed by this component must never
/// become its own parent on subsequent renders.
pub(crate) fn use_overlay_ancestors() -> Vec<NodeHandle> {
    let parent = try_use_context::<OverlayScope>();
    parent
        .map(|scope| scope.0.read().clone())
        .unwrap_or_default()
}

pub(crate) fn use_overlay_scope(node: Option<NodeHandle>) -> Vec<NodeHandle> {
    let ancestors = use_overlay_ancestors();
    use_overlay_scope_with_ancestors(node, ancestors.clone());
    ancestors
}

/// Provide the effective authored lineage for a surface rendered elsewhere
/// than its trigger. This hook deliberately consumes no lexical parent scope.
pub(crate) fn use_overlay_scope_with_ancestors(
    node: Option<NodeHandle>,
    ancestors: Vec<NodeHandle>,
) {
    let mut scope = use_context_provider(|| OverlayScope(Signal::new(Vec::new())));
    let mut lineage = Vec::new();
    for current in node.into_iter().chain(ancestors) {
        if current.is_registered() && !lineage.contains(&current) {
            lineage.push(current);
        }
    }
    let changed = *scope.0.peek() != lineage;
    if changed {
        scope.0.set(lineage);
    }
}

impl RuntimeHandle {
    fn new() -> Self {
        Self(Rc::new(RefCell::new(Runtime {
            owner: format!(
                "ix-overlay-runtime-{}",
                NEXT_RUNTIME.fetch_add(1, Ordering::Relaxed)
            ),
            registry: Registry::new(),
            nodes: BTreeMap::new(),
            order: Vec::new(),
            sequence: 0,
            observation: None,
            keyboard: false,
            completions: CompletionQueue::default(),
            item_mounts: BTreeMap::new(),
            owned_item_focus: BTreeMap::new(),
            item_focus_sequence: 0,
            submenu_marks: BTreeMap::new(),
            item_descriptions: BTreeMap::new(),
            item_description_sequence: 0,
            item_tabindices: BTreeMap::new(),
            item_tabindex_revision: 0,
            item_tabindex_signal: dioxus::core::Runtime::try_current()
                .map(|_| Signal::new_in_scope(0, ScopeId::ROOT)),
            item_visual: None,
            item_visual_target: None,
            item_visual_known: false,
            item_visual_host: None,
            item_kinds: BTreeMap::new(),
            item_visual_signal: dioxus::core::Runtime::try_current()
                .map(|_| Signal::new_in_scope(None, ScopeId::ROOT)),
            bridge: None,
            domain: None,
            document: String::new(),
            categories: BTreeMap::new(),
        })))
    }
    pub(crate) fn owner_id(&self) -> String {
        self.0.borrow().owner.clone()
    }
    pub(crate) fn item_visual_key(&self) -> Option<String> {
        let (signal, value) = {
            let state = self.0.borrow();
            (state.item_visual_signal, state.item_visual.clone())
        };
        signal
            .and_then(|signal| signal.try_read().ok().map(|value| value.clone()))
            .unwrap_or(value)
    }
    /// Reads the same reactive selection as the outer host. Select's option
    /// inherits its host cue; Category marks only its eligible parent control.
    pub(crate) fn part_active(&self, key: &str, kind: HostPartKind) -> bool {
        let _ = self.item_visual_key();
        let state = self.0.borrow();
        let Some(host) = state
            .item_visual_host
            .as_ref()
            .filter(|host| host.registration_key == key)
        else {
            return false;
        };
        match (host.kind, kind) {
            (ItemKind::Select, HostPartKind::SelectOption) => host.source_focusable,
            (ItemKind::Category, HostPartKind::CategoryParent) => host
                .parts
                .iter()
                .any(|part| part.kind == kind && part.source_focusable),
            _ => false,
        }
    }
    pub(crate) fn item_visual_target(&self) -> Option<String> {
        let _ = self.item_visual_key();
        self.0.borrow().item_visual_target.clone()
    }
    pub(crate) fn item_visual_known(&self) -> bool {
        let _ = self.item_visual_key();
        self.0.borrow().item_visual_known
    }
    /// Explicit Rust composition supplies membership before native inventory.
    /// A sibling or replacement owner cannot claim the selected control.
    pub(crate) fn item_visual_owned_by(&self, root: &str, generation: u64) -> bool {
        let _ = self.item_visual_key();
        let state = self.0.borrow();
        let Some(host) = state.item_visual_host.as_ref() else {
            return false;
        };
        state.item_visual_target.as_ref() == Some(&host.target)
            && Self::registered_item_live(&state, host)
            && state
                .nodes
                .get(root)
                .is_some_and(|entry| entry.generation == generation)
            && state
                .item_mounts
                .get(&host.registration_key)
                .is_some_and(|mount| {
                    mount
                        .owner
                        .as_ref()
                        .is_some_and(|(id, version)| id == root && *version == generation)
                })
    }
    pub(crate) fn register_item_cue(&self, key: String, kind: ItemKind) {
        if !key.is_empty() {
            self.0.borrow_mut().item_kinds.entry(key).or_insert(kind);
        }
    }
    /// Publish current component props and its explicit overlay owner.
    /// A description grants membership, while only a mounted handle grants focus.
    pub(crate) fn register_owned_item_description(
        &self,
        key: &str,
        owner: Option<(String, u64)>,
        mut host: HostFacts,
    ) {
        if key.is_empty() {
            return;
        }
        host.item = ItemRef(key.into());
        host.capture = key.into();
        host.registration_key = key.into();
        let identity_changed = self
            .0
            .borrow()
            .item_descriptions
            .get(key)
            .is_some_and(|old| {
                old.owner != owner || old.host.target != host.target || old.host.kind != host.kind
            });
        if identity_changed {
            self.cancel_item_key(key);
        } else if host.disabled_attribute || host.hidden_attribute {
            // Closing an inline owner may hide a still-focused registered item.
            // Preserve its completed cue, but never execute newly ineligible work.
            let roots = self
                .0
                .borrow()
                .owned_item_focus
                .iter()
                .filter(|(_, job)| job.key == key && job.exit_target.is_none())
                .map(|(root, _)| root.clone())
                .collect::<Vec<_>>();
            for root in roots {
                self.cancel_owned_item_focus(&root);
            }
        }
        let mut state = self.0.borrow_mut();
        let order = state
            .item_descriptions
            .get(key)
            .map(|old| old.order)
            .unwrap_or_else(|| {
                state.item_description_sequence =
                    state.item_description_sequence.wrapping_add(1).max(1);
                state.item_description_sequence
            });
        if let Some(selected) = state
            .item_visual_host
            .as_mut()
            .filter(|selected| selected.registration_key == key)
        {
            *selected = host.clone();
        }
        for job in state
            .owned_item_focus
            .values_mut()
            .filter(|job| job.key == key)
        {
            if let Some(visual) = job.visual.as_mut() {
                *visual = host.clone();
            }
        }
        state
            .item_descriptions
            .insert(key.into(), OwnedItemDescription { owner, host, order });
    }
    #[cfg(test)]
    pub(crate) fn register_item_mount(&self, key: &str, target: &str, node: Rc<MountedData>) {
        self.register_item_control_mount(key, target, target, node);
    }
    pub(crate) fn mounted_focus_target(&self, id: &str) -> Option<MountedFocusTarget> {
        if id.is_empty() {
            return None;
        }
        let state = self.0.borrow();
        let (key, mount) = state
            .item_mounts
            .iter()
            .find(|(_, mount)| mount.focus_id == id)?;
        Some(MountedFocusTarget {
            runtime: Rc::downgrade(&self.0),
            key: key.clone(),
            logical_target: mount.target.clone(),
            focus_id: id.into(),
            node: Rc::downgrade(&mount.node),
            document: state.document.clone(),
            renderer: mount.renderer.clone(),
        })
    }
    pub(crate) fn capture_registered_focus(&self, id: &str) -> Option<MountedFocusTarget> {
        self.mounted_focus_target(id)
            .or_else(|| self.domain()?.focus_target(id))
            .filter(|target| target.live_node().is_some())
    }
    #[cfg(test)]
    pub(crate) fn register_item_control_mount(
        &self,
        key: &str,
        target: &str,
        focus_id: &str,
        node: Rc<MountedData>,
    ) {
        self.register_owned_item_control_mount(key, target, focus_id, node, None);
    }
    pub(crate) fn register_owned_item_control_mount(
        &self,
        key: &str,
        target: &str,
        focus_id: &str,
        node: Rc<MountedData>,
        owner: Option<(String, u64)>,
    ) {
        let renderer = dioxus::core::Runtime::try_current().map(|owner| Rc::downgrade(&owner));
        let changed = self.0.borrow().item_mounts.get(key).is_some_and(|old| {
            old.target != target
                || old.focus_id != focus_id
                || old.owner != owner
                || !Rc::ptr_eq(&old.node, &node)
        });
        if changed {
            self.cancel_item_key(key);
        }
        self.0.borrow_mut().item_mounts.insert(
            key.into(),
            ItemMount {
                target: target.into(),
                focus_id: focus_id.into(),
                owner,
                node,
                renderer,
            },
        );
    }
    pub(crate) fn unregister_item_cue(&self, key: &str) {
        self.cancel_item_key(key);
        let mut state = self.0.borrow_mut();
        state.item_mounts.remove(key);
        state.item_kinds.remove(key);
        state.item_descriptions.remove(key);
        state.submenu_marks.remove(key);
    }
    fn cancel_item_key(&self, key: &str) {
        let selected = self
            .0
            .borrow()
            .item_visual_host
            .as_ref()
            .is_some_and(|host| host.registration_key == key);
        if selected {
            self.clear_item_visual();
        }
        let roots: Vec<_> = self
            .0
            .borrow()
            .owned_item_focus
            .iter()
            .filter(|(_, job)| job.key == key)
            .map(|(root, _)| root.clone())
            .collect();
        for root in roots {
            self.cancel_owned_item_focus(&root);
        }
        let changed = {
            let mut state = self.0.borrow_mut();
            let mut changed = false;
            state.item_tabindices.retain(|_, values| {
                changed |= values.remove(key).is_some();
                !values.is_empty()
            });
            changed
        };
        if changed {
            self.notify_item_tabindex();
        }
    }
    fn cancel_owned_item_focus(&self, root: &str) {
        let job = self.0.borrow_mut().owned_item_focus.remove(root);
        if let Some((owner, task)) = job.and_then(|job| job.task)
            && let Some(owner) = owner.upgrade()
        {
            owner.in_scope(ScopeId::ROOT, || task.cancel());
        }
    }
    fn retire_owned_item_focus(&self, root: &str, sequence: u64) {
        let mut state = self.0.borrow_mut();
        if state
            .owned_item_focus
            .get(root)
            .is_some_and(|job| job.sequence == sequence)
        {
            state.owned_item_focus.remove(root);
        }
    }
    fn item_owner_open(&self, root: &str, generation: u64) -> bool {
        self.with_owner(root, generation, || {
            let entry = self.entry(root)?;
            (self.read_current(root, &entry) == Some(true)
                && self
                    .entry(root)
                    .is_some_and(|entry| entry.generation == generation))
            .then_some(())
        })
        .flatten()
        .is_some()
    }
    fn focus_owner_registered(&self, root: &str, generation: u64) -> bool {
        self.with_owner(root, generation, || {
            let entry = self.entry(root)?;
            (self.read_current(root, &entry).is_some()
                && self
                    .entry(root)
                    .is_some_and(|entry| entry.generation == generation))
            .then_some(())
        })
        .flatten()
        .is_some()
    }
    fn destination_popovers_visible(&self, popovers: &[String]) -> bool {
        self.domain().map_or_else(
            || {
                popovers.iter().all(|id| {
                    self.entry(id).is_some_and(|entry| {
                        entry.kind == OverlayKind::Popover
                            && self.read_current(id, &entry) == Some(true)
                    })
                })
            },
            |domain| domain.focus_destinations_visible(popovers),
        )
    }
    fn queue_exit_focus(
        &self,
        root: &str,
        generation: u64,
        target: MountedFocusTarget,
        destination_popovers: Vec<String>,
    ) -> bool {
        self.queue_registered_focus(root, generation, target, None, destination_popovers)
    }
    fn queue_registered_focus(
        &self,
        root: &str,
        generation: u64,
        target: MountedFocusTarget,
        expected_open: Option<bool>,
        destination_popovers: Vec<String>,
    ) -> bool {
        self.with_owner(root, generation, || {
            if !self.focus_owner_registered(root, generation)
                || target.live_node().is_none()
                || !self.destination_popovers_visible(&destination_popovers)
                || expected_open.is_some_and(|expected| {
                    self.entry(root)
                        .and_then(|entry| self.read_current(root, &entry))
                        != Some(expected)
                })
            {
                return false;
            }
            if self
                .entry(root)
                .is_none_or(|entry| entry.generation != generation || entry.scope.is_none())
                || target.live_node().is_none()
            {
                return false;
            }
            self.cancel_owned_item_focus(root);
            let sequence = {
                let mut state = self.0.borrow_mut();
                state.item_focus_sequence = state.item_focus_sequence.wrapping_add(1).max(1);
                let sequence = state.item_focus_sequence;
                state.owned_item_focus.insert(
                    root.into(),
                    OwnedItemFocus {
                        sequence,
                        generation,
                        key: target.key.clone(),
                        callback: None,
                        visual: None,
                        focus_native: true,
                        exit_target: Some(target),
                        expected_open,
                        destination_popovers,
                        task: None,
                    },
                );
                sequence
            };
            let weak = Rc::downgrade(&self.0);
            let owned_root = root.to_string();
            dioxus::core::queue_effect(move || {
                if let Some(state) = weak.upgrade() {
                    let runtime = RuntimeHandle(state);
                    runtime.with_owner(&owned_root, generation, || {
                        runtime.start_owned_item_focus(owned_root.clone(), generation, sequence)
                    });
                }
            });
            // A raw native report need not modify component state. Wake the
            // owner now so the framework runs this post-render focus effect.
            dioxus::core::needs_update();
            true
        })
        .unwrap_or(false)
    }
    fn owned_item_live(
        &self,
        root: &str,
        generation: u64,
        sequence: u64,
        node: &Rc<MountedData>,
    ) -> bool {
        let Some((expected_open, popovers)) = self
            .0
            .borrow()
            .owned_item_focus
            .get(root)
            .map(|job| (job.expected_open, job.destination_popovers.clone()))
        else {
            return false;
        };
        let owner_live = self.focus_owner_registered(root, generation)
            && expected_open.is_none_or(|expected| {
                self.with_owner(root, generation, || {
                    self.entry(root)
                        .and_then(|entry| self.read_current(root, &entry))
                        == Some(expected)
                })
                .unwrap_or(false)
            });
        if !owner_live || !self.destination_popovers_visible(&popovers) {
            return false;
        }
        // Presence readers can run consumer code. Recheck identity only after
        // those callbacks, without holding a registry borrow across them.
        let state = self.0.borrow();
        state.owned_item_focus.get(root).is_some_and(|job| {
            job.sequence == sequence
                && job.generation == generation
                && if let Some(target) = &job.exit_target {
                    target
                        .live_node()
                        .is_some_and(|current| Rc::ptr_eq(&current, node))
                } else {
                    state
                        .item_mounts
                        .get(&job.key)
                        .is_some_and(|mount| Rc::ptr_eq(&mount.node, node))
                }
        })
    }

    fn start_owned_item_focus(&self, root: String, generation: u64, sequence: u64) {
        let selected = {
            let state = self.0.borrow();
            state
                .owned_item_focus
                .get(&root)
                .filter(|job| job.sequence == sequence)
                .and_then(|job| {
                    let node = if let Some(target) = &job.exit_target {
                        target.live_node()
                    } else {
                        state
                            .item_mounts
                            .get(&job.key)
                            .map(|mount| mount.node.clone())
                    }?;
                    Some((node, job.focus_native, job.exit_target.is_some()))
                })
        };
        let Some((node, focus_native, exiting)) = selected else {
            self.retire_owned_item_focus(&root, sequence);
            return;
        };
        if !self.owned_item_live(&root, generation, sequence, &node) {
            self.cancel_owned_item_focus(&root);
            return;
        }
        let callback = self
            .0
            .borrow_mut()
            .owned_item_focus
            .get_mut(&root)
            .and_then(|job| job.callback.take());
        if let Some(callback) = callback {
            callback();
        }
        if !self.owned_item_live(&root, generation, sequence, &node) {
            self.retire_owned_item_focus(&root, sequence);
            return;
        }
        let weak = Rc::downgrade(&self.0);
        let task_root = root.clone();
        let task = spawn(async move {
            let live = || {
                weak.upgrade().map(RuntimeHandle).filter(|runtime| {
                    runtime.owned_item_live(&task_root, generation, sequence, &node)
                })
            };
            let finish = || {
                if let Some(state) = weak.upgrade() {
                    RuntimeHandle(state).retire_owned_item_focus(&task_root, sequence);
                }
            };
            if live().is_none() {
                finish();
                return;
            }
            // Desktop alpha can complete focus but report a null decode error.
            // Cues and nearest scrolling follow a live attempt in either case.
            if focus_native {
                let _ = node.set_focus(true).await;
            }
            if exiting {
                finish();
                return;
            }
            let Some(runtime) = live() else {
                finish();
                return;
            };
            let visual = runtime
                .0
                .borrow()
                .owned_item_focus
                .get(&task_root)
                .and_then(|job| job.visual.clone());
            if let Some(visual) = visual {
                runtime.mark_item_visual(&visual);
            }
            drop(runtime);
            if live().is_none() {
                finish();
                return;
            }
            let _ = node
                .scroll_to_with_options(dioxus::html::ScrollToOptions {
                    behavior: dioxus::html::ScrollBehavior::Instant,
                    vertical: dioxus::html::ScrollLogicalPosition::Nearest,
                    horizontal: dioxus::html::ScrollLogicalPosition::Nearest,
                })
                .await;
            // Recheck after the await, then retire only this operation. A
            // newer reentrant intent remains owned by its own effect/task.
            let _ = live();
            finish();
        });
        if let Some(job) = self
            .0
            .borrow_mut()
            .owned_item_focus
            .get_mut(&root)
            .filter(|job| job.sequence == sequence)
        {
            job.task = Some((Rc::downgrade(&dioxus::core::Runtime::current()), task));
        }
    }
    pub(crate) fn item_visual_matches(&self, host: &HostFacts) -> bool {
        let state = self.0.borrow();
        state.item_visual_host.as_ref().is_some_and(|current| {
            current.registration_key == host.registration_key
                && current.kind == host.kind
                && current.target == host.target
                && state.item_visual_target.as_ref() == Some(&host.target)
                && Self::registered_item_live(&state, host)
        })
    }
    fn registered_item_live(state: &Runtime, host: &HostFacts) -> bool {
        state.item_kinds.get(&host.registration_key) == Some(&host.kind)
            && state
                .item_mounts
                .get(&host.registration_key)
                .is_some_and(|mount| {
                    mount.target == host.target
                        && mount
                            .renderer
                            .as_ref()
                            .is_none_or(|renderer| renderer.upgrade().is_some())
                })
    }
    /// Physical Dioxus focus is authoritative for a registered mounted item.
    /// Descriptions come from Rust components; no native capture receipt is needed.
    pub(crate) fn focus_registered_item(
        &self,
        key: &str,
        source_focusable: bool,
        part: Option<HostPartKind>,
    ) {
        let host = {
            let state = self.0.borrow();
            let Some(kind) = state.item_kinds.get(key).copied() else {
                return;
            };
            let Some(mount) = state.item_mounts.get(key) else {
                return;
            };
            if mount
                .renderer
                .as_ref()
                .is_some_and(|renderer| renderer.upgrade().is_none())
            {
                return;
            }
            if !matches!(
                (kind, part),
                (
                    ItemKind::Dropdown
                        | ItemKind::Menu
                        | ItemKind::Select
                        | ItemKind::Category
                        | ItemKind::NativeOptIn,
                    None
                ) | (ItemKind::Select, Some(HostPartKind::SelectOption))
                    | (ItemKind::Category, Some(HostPartKind::CategoryParent))
            ) {
                return;
            }
            let mut host = state
                .item_descriptions
                .get(key)
                .filter(|item| item.host.kind == kind && item.host.target == mount.target)
                .map(|item| item.host.clone())
                .unwrap_or_else(|| HostFacts {
                    root_path: Vec::new(),
                    item: ItemRef(key.into()),
                    kind,
                    target: mount.target.clone(),
                    capture: key.into(),
                    registration_key: key.into(),
                    source_focusable,
                    disabled_attribute: false,
                    hidden_attribute: false,
                    tabindex_attribute: None,
                    checked_attribute: false,
                    visible_focus: false,
                    roving_opt_in: false,
                    linked_trigger: None,
                    parts: Vec::new(),
                });
            host.source_focusable = source_focusable && kind != ItemKind::Category;
            if kind == ItemKind::Category {
                host.parts = if part == Some(HostPartKind::CategoryParent) {
                    vec![HostPartFacts {
                        kind: HostPartKind::CategoryParent,
                        capture: mount.focus_id.clone(),
                        source_focusable,
                    }]
                } else {
                    Vec::new()
                };
            }
            host
        };
        self.mark_item_visual(&host);
    }
    /// A late blur of the previous control cannot erase a newer selection.
    pub(crate) fn blur_registered_item(&self, key: &str) {
        let selected = self
            .0
            .borrow()
            .item_visual_host
            .as_ref()
            .is_some_and(|host| host.registration_key == key);
        if selected {
            self.clear_item_visual();
        }
    }
    fn set_item_visual(
        &self,
        key: Option<String>,
        target: Option<String>,
        host: Option<HostFacts>,
    ) {
        let signal = {
            let mut state = self.0.borrow_mut();
            let was_known = state.item_visual_known;
            state.item_visual_known = true;
            if was_known
                && state.item_visual == key
                && state.item_visual_target == target
                && state.item_visual_host == host
            {
                return;
            }
            state.item_visual = key.clone();
            state.item_visual_target = target;
            state.item_visual_host = host;
            state.item_visual_signal
        };
        if let Some(mut signal) = signal
            && let Ok(mut value) = signal.try_write()
        {
            *value = key;
        }
    }
    pub(crate) fn clear_item_visual(&self) {
        self.set_item_visual(None, None, None);
    }
    pub(crate) fn mark_item_visual(&self, host: &HostFacts) {
        if !self.keyboard_mode() || !Self::registered_item_live(&self.0.borrow(), host) {
            return;
        }
        // Source focusItem projects Category into its private parent; that
        // parent's marker never makes the outer Category an AD active host.
        let key = (host.source_focusable
            && matches!(
                host.kind,
                ItemKind::Dropdown | ItemKind::Select | ItemKind::Menu
            ))
        .then(|| host.registration_key.clone());
        let target = key.as_ref().map(|_| host.target.clone());
        self.set_item_visual(key, target, Some(host.clone()));
    }
    fn mounted_item_key(state: &Runtime, host: &HostFacts) -> Option<String> {
        if state.item_mounts.contains_key(&host.registration_key) {
            return Some(host.registration_key.clone());
        }
        state
            .item_mounts
            .iter()
            .find(|(_, mount)| mount.target == host.target)
            .map(|(key, _)| key.clone())
    }
    fn current_item_tabindex(state: &Runtime, key: &str) -> Option<i32> {
        state
            .item_tabindices
            .values()
            .filter_map(|values| values.get(key))
            .max_by_key(|value| value.revision)
            .map(|value| value.value)
    }
    pub(crate) fn item_tabindex(&self, key: &str) -> Option<i32> {
        let signal = self.0.borrow().item_tabindex_signal;
        let _revision = signal.and_then(|signal| signal.try_read().ok().map(|value| *value));
        Self::current_item_tabindex(&self.0.borrow(), key)
    }
    fn notify_item_tabindex(&self) {
        let (signal, revision) = {
            let mut state = self.0.borrow_mut();
            state.item_tabindex_revision = state.item_tabindex_revision.wrapping_add(1).max(1);
            (state.item_tabindex_signal, state.item_tabindex_revision)
        };
        if let Some(mut signal) = signal
            && let Ok(mut value) = signal.try_write()
        {
            *value = revision;
        }
    }
    fn clear_item_tabindices(&self, root: &str, generation: u64) {
        let removed = self
            .0
            .borrow_mut()
            .item_tabindices
            .remove(&(root.into(), generation))
            .is_some();
        if removed {
            self.notify_item_tabindex();
        }
    }
    fn set_roving_target(
        &self,
        root: &str,
        generation: u64,
        candidates: &[HostFacts],
        target: &ItemRef,
    ) -> bool {
        let mut state = self.0.borrow_mut();
        let target_key = candidates
            .iter()
            .find(|host| host.item == *target)
            .and_then(|host| Self::mounted_item_key(&state, host));
        let Some(target_key) = target_key else {
            return false;
        };
        let revision = state.item_tabindex_revision.wrapping_add(1).max(1);
        let values = candidates
            .iter()
            .filter_map(|host| {
                Self::mounted_item_key(&state, host).map(|key| {
                    let value = if key == target_key { 0 } else { -1 };
                    (key, ItemTabindex { revision, value })
                })
            })
            .collect();
        state
            .item_tabindices
            .insert((root.into(), generation), values);
        drop(state);
        self.notify_item_tabindex();
        true
    }
    fn projected_inventory(&self, mut facts: InventoryFacts) -> InventoryFacts {
        let state = self.0.borrow();
        for host in &mut facts.hosts {
            if let Some(value) = Self::mounted_item_key(&state, host)
                .and_then(|key| Self::current_item_tabindex(&state, &key))
            {
                host.tabindex_attribute = Some(value.to_string());
            }
            if state.item_visual_known {
                let key = if host.registration_key.is_empty() {
                    &host.capture
                } else {
                    &host.registration_key
                };
                host.visible_focus = host.source_focusable
                    && host.kind != ItemKind::Category
                    && state.item_visual.as_ref() == Some(key);
            }
        }
        facts
    }
    pub(crate) fn keyboard_mode(&self) -> bool {
        self.document_keyboard_mode()
            .unwrap_or_else(|| self.0.borrow().keyboard)
    }
    pub(crate) fn register(
        &self,
        id: String,
        generation: u64,
        kind: OverlayKind,
        behavior: CloseBehavior,
        callbacks: Callbacks,
    ) -> NodeHandle {
        let scope = dioxus::core::Runtime::try_current().and_then(|runtime| {
            runtime
                .try_current_scope_id()
                .map(|scope| (Rc::downgrade(&runtime), scope))
        });
        if self
            .entry(&id)
            .is_some_and(|entry| entry.generation < generation)
        {
            self.cancel_owned_item_focus(&id);
            if let Some(old) = self.entry(&id) {
                self.clear_item_tabindices(&id, old.generation);
            }
        }
        let mut runtime = self.0.borrow_mut();
        if !runtime
            .nodes
            .get(&id)
            .is_some_and(|entry| entry.generation > generation)
        {
            if let Some(old_generation) = runtime.nodes.get(&id).map(|entry| entry.generation)
                && old_generation != generation
            {
                runtime.completions.remove_root(&id, old_generation);

                runtime
                    .submenu_marks
                    .retain(|_, (root, generation)| root != &id || *generation != old_generation);
                runtime
                    .item_descriptions
                    .retain(|_, item| item.owner.as_ref() != Some(&(id.clone(), old_generation)));
            }
            let revision = runtime
                .nodes
                .get(&id)
                .map_or(0, |entry| entry.revision.wrapping_add(1));
            let suppress_trigger_restore = runtime.nodes.get(&id).is_some_and(|entry| {
                entry.generation == generation && entry.suppress_trigger_restore
            });
            let navigation_mode = runtime
                .nodes
                .get(&id)
                .filter(|entry| entry.generation == generation)
                .and_then(|entry| entry.navigation_mode.clone());
            runtime
                .registry
                .register(id.clone(), generation, kind, behavior);
            if !runtime.order.contains(&id) {
                runtime.order.push(id.clone());
            }
            let ownership = runtime
                .nodes
                .get(&id)
                .filter(|entry| entry.generation == generation)
                .map(|entry| {
                    (
                        entry.mounted,
                        entry.ownership_ready,
                        entry.ancestors.clone(),
                    )
                })
                .unwrap_or((false, false, Vec::new()));
            runtime.nodes.insert(
                id.clone(),
                Entry {
                    generation,
                    revision,
                    mounted: ownership.0,
                    ownership_ready: ownership.1,
                    ancestors: ownership.2,
                    before_operation: None,
                    kind,
                    behavior,
                    trigger_visibility_check: kind.is_dropdown(),
                    anchor_intersection: None,
                    anchor_shape: None,
                    submenu: None,
                    navigation_mode,
                    suppress_trigger_restore,
                    callbacks,
                    scope,
                },
            );
        }
        drop(runtime);
        self.join_node(&id);
        self.publish_ownership();
        NodeHandle {
            runtime: Rc::downgrade(&self.0),
            id,
            generation,
        }
    }
    #[cfg(test)]
    pub(crate) fn node_handle(&self, id: &str) -> Option<NodeHandle> {
        let generation = self.entry(id)?.generation;
        Some(NodeHandle {
            runtime: Rc::downgrade(&self.0),
            id: id.into(),
            generation,
        })
    }
    fn domain(&self) -> Option<Domain> {
        self.0.borrow().domain.clone()
    }
    fn with_owner<T>(&self, id: &str, generation: u64, f: impl FnOnce() -> T) -> Option<T> {
        let entry = self
            .entry(id)
            .filter(|entry| entry.generation == generation)?;
        match entry.scope {
            Some((owner, scope)) => owner.upgrade().map(|owner| owner.in_scope(scope, f)),
            None => Some(f()),
        }
    }
    fn join_node(&self, id: &str) {
        let Some(domain) = self.domain() else { return };
        let Some(entry) = self.entry(id) else { return };
        let generation = entry.generation;
        let owner = self.owner_id();
        let read_runtime = Rc::downgrade(&self.0);
        let request_runtime = read_runtime.clone();
        let suppress_runtime = read_runtime.clone();
        let read_id = id.to_string();
        let request_id = read_id.clone();
        let suppress_id = read_id.clone();
        domain.register(
            owner,
            id.to_string(),
            generation,
            entry.kind,
            entry.behavior,
            DomainCallbacks {
                suppress_trigger_restore: Rc::new(move || {
                    let Some(runtime) = suppress_runtime.upgrade().map(RuntimeHandle) else {
                        return false;
                    };
                    runtime.mark_trigger_restore_suppressed(&suppress_id, generation)
                }),
                read: Rc::new(move || {
                    let runtime = RuntimeHandle(read_runtime.upgrade()?);
                    runtime
                        .with_owner(&read_id, generation, || {
                            let entry = runtime.entry(&read_id)?;
                            runtime.read_current(&read_id, &entry)
                        })
                        .flatten()
                }),
                request: Rc::new(move |change| {
                    let Some(runtime) = request_runtime.upgrade().map(RuntimeHandle) else {
                        return false;
                    };
                    runtime
                        .with_owner(&request_id, generation, || {
                            runtime.request(&request_id, generation, change)
                        })
                        .unwrap_or(false)
                }),
            },
        );
        if let Some(open) = self.read_current(id, &entry) {
            domain.sync_open(id, generation, open);
        }
    }
    /// Dioxus lifetimes and authored relationships are the only graph input.
    /// Native observations are limited to read-only geometry/order/Tab facts.
    fn publish_ownership(&self) {
        let entries = self
            .0
            .borrow()
            .nodes
            .iter()
            .map(|(id, entry)| (id.clone(), entry.clone()))
            .collect::<Vec<_>>();
        let facts = entries
            .into_iter()
            .map(|(id, entry)| {
                let mut ancestors = Vec::new();
                for ancestor in &entry.ancestors {
                    let same_document = ancestor.runtime().is_some_and(|runtime| {
                        let document = runtime.0.borrow().document.clone();
                        let own_document = self.0.borrow().document.clone();
                        document.is_empty() || own_document.is_empty() || document == own_document
                    });
                    if ancestor.id != id
                        && ancestor.is_registered()
                        && same_document
                        && !ancestors.contains(&ancestor.id)
                    {
                        ancestors.push(ancestor.id.clone());
                    }
                }
                NativeFact {
                    id,
                    generation: entry.generation,
                    connected: entry.mounted,
                    discovery_ready: entry.ownership_ready,
                    assignment_ancestors: Some(ancestors.clone()),
                    trigger_ancestors: ancestors,
                }
            })
            .collect::<Vec<_>>();
        self.0.borrow_mut().registry.observe(&facts);
        if let Some(domain) = self.domain() {
            domain.observe_owner(&self.owner_id(), &facts);
        }
    }
    fn attach_document(&self, document: &str) {
        if document.is_empty() || self.0.borrow().document == document {
            return;
        }
        let old = self.domain();
        if let Some(old) = old {
            old.remove_owner(&self.owner_id());
        }
        let domain = Domain::join(document);
        if !self.0.borrow().document.is_empty() {
            let changed = {
                let mut state = self.0.borrow_mut();
                let changed = !state.item_tabindices.is_empty();
                state.item_tabindices.clear();
                changed
            };
            if changed {
                self.notify_item_tabindex();
            }
            let roots: Vec<_> = self.0.borrow().owned_item_focus.keys().cloned().collect();
            for root in roots {
                self.cancel_owned_item_focus(&root);
            }
        }
        let (ids, categories) = {
            let mut state = self.0.borrow_mut();
            // Adopting another document cancels old-document native work.
            if !state.document.is_empty() {
                state.submenu_marks.clear();

                state.item_visual_host = None;
                state.item_visual = None;
                state.item_visual_target = None;
                state.item_visual_known = false;
                state.observation = None;
            }
            state.document = document.to_string();
            state.domain = Some(domain);
            (state.order.clone(), state.categories.clone())
        };
        if let Some(domain) = self.domain() {
            let weak = Rc::downgrade(&self.0);
            domain.register_focus_reader(
                self.owner_id(),
                Rc::new(move |id| RuntimeHandle(weak.upgrade()?).mounted_focus_target(id)),
            );
        }
        for id in ids {
            self.join_node(&id);
        }
        self.publish_ownership();
        for (id, callback) in categories {
            if let Some(domain) = self.domain() {
                domain.register_category(self.owner_id(), id, callback);
            }
        }
    }
    pub(crate) fn register_category(&self, id: usize, callback: CategoryCallback) {
        let owner = dioxus::core::Runtime::try_current().and_then(|runtime| {
            runtime
                .try_current_scope_id()
                .map(|scope| (Rc::downgrade(&runtime), scope))
        });
        let state = Rc::downgrade(&self.0);
        // Preserve the original category's runtime/scope across later native
        // attachment, which can occur in another transport component's scope.
        let guarded: CategoryCallback = Rc::new(move |source, active| {
            let Some(state) = state.upgrade() else { return };
            if !state.borrow().categories.contains_key(&id) {
                return;
            }
            if let Some((runtime, scope)) = &owner {
                if let Some(runtime) = runtime.upgrade() {
                    runtime.in_scope(*scope, || callback(source, active));
                }
            } else {
                callback(source, active);
            }
        });
        self.0.borrow_mut().categories.insert(id, guarded.clone());
        if let Some(domain) = self.domain() {
            domain.register_category(self.owner_id(), id, guarded);
        }
    }
    pub(crate) fn document_keyboard_mode(&self) -> Option<bool> {
        self.domain().and_then(|domain| domain.keyboard_mode())
    }
    pub(crate) fn unregister_category(&self, id: usize) {
        self.0.borrow_mut().categories.remove(&id);
        if let Some(domain) = self.domain() {
            domain.unregister_category(&self.owner_id(), id);
        }
    }
    pub(crate) fn broadcast_category(&self, source: usize, active: &[usize]) -> bool {
        if let Some(domain) = self.domain() {
            domain.broadcast_category(source, active);
        } else {
            // Components can open before the document transport attaches. Reuse
            // their guarded Rust registrations without a second coordinator.
            let callbacks = self
                .0
                .borrow()
                .categories
                .iter()
                .filter(|(id, _)| **id != source)
                .map(|(_, callback)| callback.clone())
                .collect::<Vec<_>>();
            for callback in callbacks {
                callback(source, active);
            }
        }
        true
    }
    fn sync_domain_open(&self, id: &str, generation: u64, open: bool) {
        if let Some(domain) = self.domain() {
            domain.sync_open(id, generation, open);
        }
    }
    fn entry(&self, id: &str) -> Option<Entry> {
        self.0.borrow().nodes.get(id).cloned()
    }
    fn current(&self, id: &str, generation: u64, revision: u64) -> bool {
        self.0
            .borrow()
            .nodes
            .get(id)
            .is_some_and(|entry| entry.generation == generation && entry.revision == revision)
    }
    fn read_current(&self, id: &str, snapshot: &Entry) -> Option<bool> {
        let value = (snapshot.callbacks.read)();
        self.current(id, snapshot.generation, snapshot.revision)
            .then_some(value)
            .flatten()
    }
    fn mark_trigger_restore_suppressed(&self, id: &str, generation: u64) -> bool {
        self.with_owner(id, generation, || {
            let Some(snapshot) = self.entry(id).filter(|entry| entry.kind.is_dropdown()) else {
                return false;
            };
            if self.read_current(id, &snapshot) != Some(true) {
                return false;
            }
            let mut state = self.0.borrow_mut();
            let Some(entry) = state.nodes.get_mut(id).filter(|entry| {
                entry.generation == generation && entry.revision == snapshot.revision
            }) else {
                return false;
            };
            entry.suppress_trigger_restore = true;
            true
        })
        .unwrap_or(false)
    }

    /// Source dismissOthers after an actual trigger or host fallthrough.
    /// Closed origins are valid; a dropped/replaced registration is not.
    pub(crate) fn dismiss_dropdown_others(&self, id: &str, generation: u64) -> bool {
        self.dismiss_dropdown_set(id, generation, false)
    }

    fn dismiss_dropdown_set(&self, id: &str, generation: u64, inside: bool) -> bool {
        let admitted = self.with_owner(id, generation, || {
            let entry = self.entry(id)?;
            (entry.kind.is_dropdown() && entry.generation == generation)
                .then(|| self.read_current(id, &entry))
                .flatten()
        });
        if admitted.flatten().is_none() {
            return false;
        }
        if let Some(domain) = self.domain() {
            return if inside {
                domain.dismiss_dropdown_all(id, generation)
            } else {
                domain.dismiss_dropdown_others(id, generation)
            };
        }
        // Reconcile current Rust presence before capturing live ancestry.
        let ids = self.0.borrow().order.clone();
        for candidate in ids {
            let Some(entry) = self.entry(&candidate) else {
                continue;
            };
            if let Some(open) = self
                .with_owner(&candidate, entry.generation, || {
                    self.read_current(&candidate, &entry)
                })
                .flatten()
            {
                self.0.borrow_mut().registry.set_presence(&candidate, open);
            }
        }
        let excluded = if inside {
            Vec::new()
        } else {
            self.0.borrow().registry.dropdown_dismiss_exclusions(id)
        };
        let mut visited = std::collections::BTreeSet::new();
        loop {
            let next = self
                .0
                .borrow()
                .registry
                .next_dropdown_dismiss_other(&visited);
            let Some((candidate, candidate_generation, registration)) = next else {
                break;
            };
            visited.insert(registration);
            if excluded.contains(&candidate) {
                continue;
            }
            self.with_owner(&candidate, candidate_generation, || {
                let Some(entry) = self.entry(&candidate).filter(|entry| {
                    entry.generation == candidate_generation
                        && entry.kind.is_dropdown()
                        && (inside && candidate == id || entry.behavior.allows_outside())
                }) else {
                    return;
                };
                // Closed instances do not receive even an unchanged request:
                // that would incorrectly advance their operation revision.
                if self.read_current(&candidate, &entry) == Some(true) {
                    self.request(
                        &candidate,
                        candidate_generation,
                        Change {
                            open: false,
                            reason: if inside {
                                Reason::Inside
                            } else {
                                Reason::TriggerOther
                            },
                            restore_focus: !inside || self.keyboard_mode(),
                        },
                    );
                }
            });
        }
        true
    }

    pub(crate) fn close_and_release_focus(&self, id: &str) -> bool {
        if let Some(domain) = self.domain() {
            return domain.close_and_release_focus(id);
        }
        let Some(root) = self.0.borrow().registry.cached_root(id) else {
            return false;
        };
        let Some(entry) = self.entry(&root).filter(|entry| entry.kind.is_dropdown()) else {
            return false;
        };
        self.suppress_cached_trigger_restore(&root, &mut Vec::new());
        self.with_owner(&root, entry.generation, || {
            self.request(
                &root,
                entry.generation,
                Change {
                    open: false,
                    reason: Reason::Keyboard,
                    restore_focus: false,
                },
            )
        })
        .unwrap_or(false)
    }

    fn suppress_cached_trigger_restore(&self, id: &str, visited: &mut Vec<String>) {
        if visited.iter().any(|seen| seen == id) {
            return;
        }
        visited.push(id.to_owned());
        let Some(entry) = self.entry(id) else {
            return;
        };
        if !self.mark_trigger_restore_suppressed(id, entry.generation) {
            return;
        }
        let children = self.0.borrow().registry.cached_children(id);
        for child in children {
            self.suppress_cached_trigger_restore(&child, visited);
        }
    }
    fn request(&self, id: &str, generation: u64, change: Change) -> bool {
        let forced = change.reason == Reason::Configuration && !change.open;
        let Some(snapshot) = self.entry(id).filter(|entry| {
            entry.generation == generation && (entry.before_operation.is_none() || forced)
        }) else {
            return false;
        };
        let Some(previous) = self.read_current(id, &snapshot) else {
            return false;
        };
        let closed_popover = !previous && !change.open && snapshot.kind == OverlayKind::Popover;
        if previous == change.open && !closed_popover {
            let mut runtime = self.0.borrow_mut();
            // Even an unchanged command supersedes an older request still
            // between its before callback and apply (e.g. a child reopens its
            // still-open parent during the parent's pending close).
            if let Some(entry) = runtime.nodes.get_mut(id) {
                entry.revision = entry.revision.wrapping_add(1);
                if forced {
                    entry.before_operation = None;
                }
            }
            runtime.registry.set_presence(id, previous);
            drop(runtime);
            self.sync_domain_open(id, generation, previous);
            return true;
        }
        let operation = {
            let mut runtime = self.0.borrow_mut();
            let Some(entry) = runtime.nodes.get_mut(id).filter(|entry| {
                entry.generation == generation
                    && entry.revision == snapshot.revision
                    && (entry.before_operation.is_none() || forced)
            }) else {
                return false;
            };
            entry.revision = entry.revision.wrapping_add(1);
            entry.before_operation = Some(entry.revision);
            entry.revision
        };
        // No RefCell borrow crosses a user callback. A recursive request on the
        // same node during before is rejected, rather than recursing forever.
        let allowed = (change.reason == Reason::Configuration && !change.open)
            || (snapshot.callbacks.before)(change);
        {
            let mut runtime = self.0.borrow_mut();
            let Some(entry) = runtime
                .nodes
                .get_mut(id)
                .filter(|entry| entry.generation == generation)
            else {
                return false;
            };
            if entry.before_operation == Some(operation) {
                entry.before_operation = None;
            }
            if entry.revision != operation {
                return false;
            }
        }
        let current = Entry {
            revision: operation,
            ..snapshot.clone()
        };
        if !allowed || self.read_current(id, &current) != Some(previous) {
            return false;
        }
        if closed_popover {
            // Source Popover always emits willDismiss, even when closed, but
            // performs no child cascade, native command or completion then.
            return false;
        }
        if !change.open {
            self.close_children_with_focus(
                id,
                match change.reason {
                    Reason::Drop => Reason::Drop,
                    Reason::AnchorVisibility | Reason::AnchorVisibilityParent => {
                        Reason::AnchorVisibilityParent
                    }
                    Reason::TriggerOther => Reason::TriggerOther,
                    _ => Reason::Parent,
                },
                change.restore_focus,
            );
        }
        if !self.current(id, generation, operation)
            || self.read_current(id, &current) != Some(previous)
        {
            return false;
        }
        let applied = if previous && !change.open && snapshot.kind.is_dropdown() {
            let ordinary_cleanup = matches!(
                change.reason,
                Reason::AnchorVisibility | Reason::AnchorVisibilityParent | Reason::TriggerOther
            );
            // Each accepted source cleanup observes modality after its own
            // before/cascade, and consumes its CURRENT latch even in pointer
            // mode. Veto and stale operations never reach this point.
            let restore = if ordinary_cleanup {
                self.keyboard_mode()
            } else {
                change.restore_focus
            };
            let suppressed =
                {
                    let mut state = self.0.borrow_mut();
                    let Some(entry) = state.nodes.get_mut(id).filter(|entry| {
                        entry.generation == generation && entry.revision == operation
                    }) else {
                        return false;
                    };
                    std::mem::take(&mut entry.suppress_trigger_restore)
                };
            Change {
                restore_focus: restore
                    && !suppressed
                    && !matches!(change.reason, Reason::Configuration | Reason::Drop),
                ..change
            }
        } else {
            change
        };
        // Commit graph state before apply so a nested command in an accepted
        // callback sees the current hierarchy, not the superseded transaction.
        self.0.borrow_mut().registry.set_open(id, change.open);
        if change.open {
            if let Some(domain) = self.domain() {
                domain.presented(id, generation);
            }
        }
        self.sync_domain_open(id, generation, change.open);
        if !change.open {
            // Cancel the previous open-state intent before apply. A synchronous
            // close completion may admit a new expected-closed return target.
            self.cancel_owned_item_focus(id);
        }
        (snapshot.callbacks.apply)(applied);
        if !change.open && self.read_current(id, &current) == Some(false) {
            self.clear_item_tabindices(id, generation);
        }
        if !self.current(id, generation, operation) {
            return false;
        }
        let actual = self.read_current(id, &current);
        if let Some(open) = actual {
            self.0.borrow_mut().registry.set_presence(id, open);
            self.sync_domain_open(id, generation, open);
        }
        actual == Some(change.open)
    }
    fn request_id(&self, id: &str, change: Change) -> bool {
        let Some(entry) = self.entry(id) else {
            return false;
        };
        self.request(id, entry.generation, change)
    }
    fn close_children(&self, id: &str, reason: Reason) {
        self.close_children_with_focus(id, reason, false);
    }
    fn close_children_with_focus(&self, id: &str, reason: Reason, restore_focus: bool) {
        if let Some(domain) = self.domain() {
            domain.close_children(id, reason, restore_focus);
            return;
        }
        let children = self.0.borrow().registry.cached_children(id);
        for child in children {
            self.request_id(
                &child,
                Change {
                    open: false,
                    reason,
                    restore_focus: matches!(
                        reason,
                        Reason::AnchorVisibilityParent | Reason::TriggerOther
                    ) && self
                        .entry(&child)
                        .is_some_and(|entry| entry.kind.is_dropdown()),
                },
            );
        }
    }
    fn owns_topmost(&self, id: &str) -> bool {
        if let Some(domain) = self.domain() {
            return domain.owns_topmost(id);
        }
        let runtime = self.0.borrow();
        runtime.registry.topmost().is_some_and(|top| {
            top == id
                || runtime
                    .registry
                    .live_ancestors(&top)
                    .iter()
                    .any(|ancestor| ancestor == id)
        })
    }
    fn owns_dioxus_topmost(&self, id: &str) -> bool {
        let observed_top = self
            .domain()
            .and_then(|domain| domain.topmost())
            .or_else(|| self.0.borrow().registry.topmost());
        if observed_top.is_some() {
            return self.owns_topmost(id);
        }
        // An explicitly mounted Dioxus control can receive its first key before
        // the observation bridge publishes connectivity. Current Rust owners
        // still determine the local open layer; no native key route is needed.
        let owners = {
            let state = self.0.borrow();
            state
                .order
                .iter()
                .rev()
                .filter_map(|id| state.nodes.get(id).map(|entry| (id.clone(), entry.clone())))
                .collect::<Vec<_>>()
        };
        owners
            .into_iter()
            .find(|(id, entry)| self.read_current(id, entry) == Some(true))
            .is_some_and(|(top, _)| {
                top == id
                    || self
                        .0
                        .borrow()
                        .registry
                        .live_ancestors(&top)
                        .iter()
                        .any(|parent| parent == id)
            })
    }
    pub(crate) fn dismiss_root(&self, id: &str, reason: Reason, restore_focus: bool) -> bool {
        if let Some(domain) = self.domain() {
            return domain.dismiss_root(id, reason, restore_focus);
        }
        let root = self.0.borrow().registry.cached_root(id);
        let Some(root) = root else {
            return false;
        };
        // Nested Header Escape in the recorded fresh-small source case ends at
        // BODY. The flat Header path retains its own separately tested policy.
        let restore_focus = restore_focus
            && self
                .entry(&root)
                .is_some_and(|entry| entry.kind != OverlayKind::Header);
        self.request_id(
            &root,
            Change {
                open: false,
                reason,
                restore_focus,
            },
        )
    }
    pub(crate) fn close_submenu(&self, id: &str) {
        let parent = self.domain().map_or_else(
            || self.0.borrow().registry.cached_parent(id),
            |domain| domain.cached_parent(id),
        );
        let Some(parent) = parent else { return };
        let parent_entry = self.entry(&parent);
        let marker = parent_entry
            .as_ref()
            .and_then(|entry| self.first_submenu_marker(&parent, entry.generation));
        // Capture the parent's own inventory before forwarding is reset. Native
        // hide receipts and arbitrary-DOM return targets are unnecessary: the
        // existing Dioxus item executor validates the current open owner/mount.
        let own = parent_entry
            .as_ref()
            .and_then(|entry| self.0.borrow().owned_inventory(&parent, entry.generation));
        let host = marker.as_ref().and_then(|capture| {
            own.as_ref()?
                .hosts
                .iter()
                .find(|host| host.capture == *capture)
                .cloned()
        });
        let children = self.domain().map_or_else(
            || self.0.borrow().registry.cached_children(&parent),
            |domain| domain.cached_children(&parent),
        );
        for child in children {
            self.request_id(
                &child,
                Change {
                    open: false,
                    reason: Reason::Left,
                    restore_focus: true,
                },
            );
        }
        let (Some(capture), Some(entry)) = (marker, parent_entry) else {
            return;
        };
        self.0.borrow_mut().submenu_marks.remove(&capture);
        if let Some(access) = entry.submenu {
            self.with_owner(&parent, entry.generation, || (access.reset)());
        }
        let Some(host) = host.filter(|host| !host.disabled_attribute && !host.hidden_attribute)
        else {
            return;
        };
        let node = NodeHandle {
            runtime: Rc::downgrade(&self.0),
            id: parent.clone(),
            generation: entry.generation,
        };
        if !self.item_owner_open(&parent, entry.generation) || !node.is_item_registered(&host) {
            return;
        }
        let roving = self
            .entry(&parent)
            .is_some_and(|entry| entry.navigation_mode.as_deref() == Some("roving-tabindex"));
        if roving
            && !own
                .as_ref()
                .is_some_and(|facts| node.select_roving(facts, &host.item))
        {
            return;
        }
        node.queue_item_host_focus(&host, Rc::new(|| {}));
    }
    fn sync_open(&self, id: &str, generation: u64, open: bool) {
        let mut runtime = self.0.borrow_mut();
        let Some(entry) = runtime
            .nodes
            .get_mut(id)
            .filter(|entry| entry.generation == generation)
        else {
            return;
        };
        entry.revision = entry.revision.wrapping_add(1);
        runtime.registry.set_presence(id, open);
        drop(runtime);
        if !open {
            self.clear_item_tabindices(id, generation);
        }
        self.sync_domain_open(id, generation, open);
    }
    fn unregister(&self, id: &str, generation: u64) {
        if !self
            .entry(id)
            .is_some_and(|entry| entry.generation == generation)
        {
            return;
        }
        self.close_children(id, Reason::Drop);
        self.cancel_owned_item_focus(id);
        self.clear_item_tabindices(id, generation);
        let mut runtime = self.0.borrow_mut();
        if runtime
            .nodes
            .get(id)
            .is_some_and(|entry| entry.generation == generation)
        {
            runtime.completions.remove_root(id, generation);

            runtime
                .submenu_marks
                .retain(|_, (root, version)| root != id || *version != generation);
            runtime
                .item_descriptions
                .retain(|_, item| item.owner.as_ref() != Some(&(id.to_string(), generation)));
            runtime.nodes.remove(id);
            runtime.order.retain(|candidate| candidate != id);
            runtime.registry.unregister(id, generation);
            if let Some(domain) = &runtime.domain {
                domain.unregister(id, generation);
            }
        }
        drop(runtime);
        self.publish_ownership();
    }
    fn start_bridge(&self) {
        if self.0.borrow().bridge.is_some() {
            return;
        }
        let owner = serde_json::to_string(&self.owner_id()).expect("overlay runtime owner");
        let bridge = dioxus::document::eval(
            &crate::overlay_registry_dom::BRIDGE.replace("__OWNER__", &owner),
        );
        self.0.borrow_mut().bridge = Some(bridge);
    }
    fn set_navigation_mode(&self, id: &str, generation: u64, mode: &str) -> bool {
        if !matches!(mode, "active-descendant" | "roving-tabindex") {
            return false;
        }
        let Some(entry) = self
            .entry(id)
            .filter(|entry| entry.generation == generation && entry.kind == OverlayKind::Dropdown)
        else {
            return false;
        };
        if entry.navigation_mode.as_deref() != Some(mode) {
            self.clear_item_tabindices(id, generation);
        }
        let mut state = self.0.borrow_mut();
        let Some(entry) = state
            .nodes
            .get_mut(id)
            .filter(|entry| entry.generation == generation)
        else {
            return false;
        };
        entry.navigation_mode = Some(mode.into());
        true
    }
    #[cfg(test)]
    pub(crate) fn pending_item_focus_count(&self) -> usize {
        self.0.borrow().owned_item_focus.len()
    }
    #[cfg(test)]
    pub(crate) fn pending_completions(&self) -> usize {
        self.0.borrow().completions.len()
    }
    fn observe_document(&self, report: &Report) -> KeyDecision {
        let Some(event) = report.document_event.as_ref().filter(|event| event.valid()) else {
            return KeyDecision::default();
        };
        let domain = {
            let state = self.0.borrow();
            if report.owner != state.owner
                || report.sequence <= state.sequence
                || report.document.is_empty()
                || report.document != state.document
            {
                return KeyDecision::default();
            }
            let Some(domain) = state.domain.clone() else {
                return KeyDecision::default();
            };
            domain
        };
        let admitted = domain.accept_event(event.sequence);
        if !admitted && !domain.is_current_event(event.sequence) {
            return KeyDecision::default();
        }
        {
            let mut state = self.0.borrow_mut();
            state.sequence = report.sequence;
        }
        match &event.action {
            DocumentAction::Input { input_kind, key } => {
                if admitted {
                    domain.observe_input(input_kind, key);
                }
                if matches!(input_kind.as_str(), "mousedown" | "touchstart") {
                    self.0.borrow_mut().keyboard = false;
                    self.clear_item_visual();
                } else if let Some(keyboard) = domain.keyboard_mode() {
                    self.0.borrow_mut().keyboard = keyboard;
                }
            }
            DocumentAction::Outside {
                hosts,
                triggers,
                anchors,
            } if admitted => {
                let mut inside = hosts.clone();
                inside.extend(anchors.iter().cloned());
                domain.outside(&inside, triggers);
            }
            DocumentAction::Escape if admitted => {
                domain.escape();
            }
            _ => {}
        }
        KeyDecision::default()
    }
    fn observe(&self, report: &Report) -> KeyDecision {
        if report.kind == "document" {
            // Minimal global input is not a DOM observation and cannot attach a
            // document, disconnect nodes or replace any retained inventory.
            return self.observe_document(report);
        }
        if report.kind != "observe" {
            return KeyDecision::default();
        }
        {
            let mut runtime = self.0.borrow_mut();
            if report.owner != runtime.owner || report.sequence <= runtime.sequence {
                return KeyDecision::default();
            }
            runtime.sequence = report.sequence;
        }
        self.attach_document(&report.document);
        {
            let mut runtime = self.0.borrow_mut();
            // A snapshot supplies read-only order, focus and Tab destinations.
            // Dioxus event handlers supply all keyboard policy and origin.
            runtime.observation = Some(report.clone());
        }
        // Native open attributes may lag the Rust callback during an original
        // click. Reconcile with read(), never overwrite current state from DOM.
        let snapshots = {
            let runtime = self.0.borrow();
            runtime
                .order
                .iter()
                .filter_map(|id| {
                    runtime
                        .nodes
                        .get(id)
                        .map(|entry| (id.clone(), entry.clone()))
                })
                .collect::<Vec<_>>()
        };
        for (id, entry) in &snapshots {
            if let Some(open) = self.read_current(id, entry) {
                self.0.borrow_mut().registry.set_presence(id, open);
            }
        }
        // Submenu anchor ancestry is a current DOM fact, independent of the
        // optional visibility observer. Suppressing visibility must not disable
        // the source's submenu placement override.
        for node in &report.nodes {
            let Some(in_dropdown_item) = node.anchor_in_dropdown_item else {
                continue;
            };
            let Some(entry) = self
                .entry(&node.id)
                .filter(|entry| node.generation == entry.generation && entry.kind.is_dropdown())
            else {
                continue;
            };
            if let Some(callback) = &entry.anchor_shape {
                self.with_owner(&node.id, node.generation, || {
                    if self.read_current(&node.id, &entry) == Some(true) {
                        callback(in_dropdown_item);
                    }
                });
            }
        }
        KeyDecision::default()
    }
    fn reported_kind(&self, report: &Report, id: &str) -> Option<OverlayKind> {
        let node = report.nodes.iter().find(|node| node.id == id)?;
        self.entry(id)
            .filter(|entry| entry.generation == node.generation)
            .map(|entry| entry.kind)
            .or_else(|| self.domain()?.node_kind(id, node.generation))
    }
    /// Header's internal controls are native light-DOM buttons in this port,
    /// but live inside the source Header shadow root. Ordinary source queries
    /// do not enter that shadow root. Keep the renderer projection in Rust;
    /// the captured trigger remains separately available for reverse exits.
    fn source_tab_candidates(
        &self,
        report: &Report,
        candidates: &[crate::popover_focus::TabCandidate],
    ) -> Vec<crate::popover_focus::TabCandidate> {
        let controls = report
            .nodes
            .iter()
            .filter(|node| self.reported_kind(report, &node.id) == Some(OverlayKind::Header))
            .flat_map(|node| node.header_controls.iter())
            .collect::<Vec<_>>();
        candidates
            .iter()
            .filter(|candidate| !controls.contains(&&candidate.target))
            .cloned()
            .collect()
    }
    fn source_document_order(&self, report: &Report) -> Vec<String> {
        let Some(candidates) = &report.document_tab_candidates else {
            // Compatibility for pre-native reports and focused transport tests.
            let controls = report
                .nodes
                .iter()
                .filter(|node| self.reported_kind(report, &node.id) == Some(OverlayKind::Header))
                .flat_map(|node| node.header_controls.iter())
                .collect::<Vec<_>>();
            return report
                .tab_order
                .iter()
                .filter(|target| !controls.contains(target))
                .cloned()
                .collect();
        };
        let candidates = self
            .source_tab_candidates(report, candidates)
            .into_iter()
            .filter(|candidate| {
                !candidate.hosts.iter().any(|host| {
                    self.reported_kind(report, host)
                        .is_some_and(OverlayKind::is_dropdown)
                })
            })
            .collect::<Vec<_>>();
        crate::popover_focus::tab_order_targets(&candidates)
    }
    fn dispatch_key(
        &self,
        id: &str,
        generation: u64,
        facts: KeyFacts,
        report: &Report,
    ) -> KeyDecision {
        self.with_owner(id, generation, || {
            let Some(entry) = self
                .entry(id)
                .filter(|entry| entry.generation == generation)
            else {
                return KeyDecision::default();
            };
            if facts.default_prevented || (!facts.inside && !facts.on_trigger) {
                return KeyDecision::default();
            }
            let Some(owner_open) = self.read_current(id, &entry) else {
                return KeyDecision::default();
            };
            if matches!(facts.key.as_str(), "Escape" | "Tab") && !owner_open {
                return KeyDecision::default();
            }
            if let Some(domain) = self.domain() {
                domain.observe_input("keydown", &facts.key);
            }
            self.0.borrow_mut().keyboard = true;
            if facts.key == "Escape" {
                if !self.owns_dioxus_topmost(id) {
                    return KeyDecision::default();
                }
                self.dismiss_root(id, Reason::Escape, true);
                return KeyDecision {
                    // The shared window listener must not dismiss another
                    // owner when this handled event leaves the Dioxus root.
                    prevent: true,
                    stop: true,
                    ..KeyDecision::default()
                };
            }
            let Some(callback) = entry.callbacks.key.clone() else {
                return KeyDecision::default();
            };
            // Capture the mounted destination before close callbacks cancel old
            // focus intents or change the current graph. Admission occurs after.
            let tab_release = self.tab_release_target(id, &facts, report);
            let mut decision = callback(&facts);
            if !self.current(id, entry.generation, entry.revision)
                || self.read_current(id, &entry) != Some(true)
            {
                decision.activate = None;
            }
            if let Some((root, generation, target, popovers)) =
                tab_release.filter(|_| decision.release_focus)
                && self.queue_exit_focus(&root, generation, target, popovers)
            {
                decision.prevent = true;
            }
            decision
        })
        .unwrap_or_default()
    }
    fn tab_release_target(
        &self,
        id: &str,
        facts: &KeyFacts,
        report: &Report,
    ) -> Option<(String, u64, MountedFocusTarget, Vec<String>)> {
        if facts.key != "Tab"
            || (facts.navigation_mode.as_deref() != Some("roving-tabindex")
                && !facts.popover_ancestor)
        {
            return None;
        }
        let root = self.domain().map_or_else(
            || self.0.borrow().registry.cached_root(id),
            |domain| domain.cached_root(id),
        );
        root.and_then(|root| {
            let entry = self.entry(&root)?;
            let node = report
                .nodes
                .iter()
                .find(|node| node.id == root && node.generation == entry.generation)?;
            if self.read_current(&root, &entry) != Some(true) {
                return None;
            }
            // A linked child trigger belongs to its enclosing Dropdown,
            // although it is outside the child's own surface. Preserve
            // that registered relationship without changing event origin.
            let linked_parent = facts.on_trigger
                && self
                    .domain()
                    .map_or_else(
                        || self.0.borrow().registry.cached_parent(id),
                        |domain| domain.cached_parent(id),
                    )
                    .is_some_and(|parent| {
                        self.reported_kind(report, &parent)
                            .is_some_and(OverlayKind::is_dropdown)
                    });
            let target = if facts.shift
                && (linked_parent
                    || report.hosts.iter().any(|host| {
                        self.reported_kind(report, host)
                            .is_some_and(OverlayKind::is_dropdown)
                    })) {
                (!node.trigger_id.is_empty()).then(|| node.trigger_id.clone())
            } else {
                self.domain()
                    .and_then(|domain| {
                        let parent = domain.popover_focus_parent(&root)?;
                        let parent_node = report.nodes.iter().find(|node| node.id == parent)?;
                        let excluded = domain
                            .focus_trap_exclusions(&parent, &[], "")
                            .unwrap_or_default();
                        let excluded = if excluded.is_empty() {
                            vec![root.clone()]
                        } else {
                            excluded
                        };
                        let candidates =
                            self.source_tab_candidates(report, &parent_node.tab_candidates);
                        crate::popover_focus::tab_adjacent_target(
                            &candidates,
                            &excluded,
                            &node.trigger_id,
                            facts.shift,
                        )
                    })
                    .or_else(|| {
                        let order = self.source_document_order(report);
                        order
                            .iter()
                            .position(|id| id == &node.trigger_id)
                            .and_then(|index| {
                                let next = if facts.shift {
                                    index.checked_sub(1)
                                } else {
                                    index.checked_add(1)
                                }?;
                                order.get(next).cloned()
                            })
                    })
            }?;
            let mounted = self
                .mounted_focus_target(&target)
                .or_else(|| self.domain()?.focus_target(&target))?;
            let mut popovers = Vec::new();
            for candidate in report
                .document_tab_candidates
                .iter()
                .flatten()
                .chain(
                    report
                        .nodes
                        .iter()
                        .flat_map(|node| node.tab_candidates.iter()),
                )
                .filter(|candidate| candidate.target == target)
            {
                for host in &candidate.hosts {
                    if !popovers.contains(host)
                        && self.reported_kind(report, host) == Some(OverlayKind::Popover)
                    {
                        popovers.push(host.clone());
                    }
                }
            }
            Some((root, entry.generation, mounted, popovers))
        })
    }
    fn anchor_visibility(
        &self,
        id: &str,
        generation: u64,
        intersecting: bool,
        geometry: Option<AnchorIntersection>,
    ) {
        let Some(entry) = self.entry(id).filter(|entry| {
            entry.generation == generation
                && entry.kind.is_dropdown()
                && entry.trigger_visibility_check
        }) else {
            return;
        };
        // Dioxus owns the observed trigger and event lifecycle. Enter the
        // registered surface owner before callbacks; no DOM identity receipt or
        // native visibility transport is needed for an explicit component.
        self.with_owner(id, generation, || {
            if self.read_current(id, &entry) != Some(true) {
                return;
            }
            if intersecting {
                if let Some(geometry) = geometry.filter(|geometry| geometry.valid())
                    && let Some(callback) = &entry.anchor_intersection
                {
                    callback(geometry);
                }
            } else {
                self.request(
                    id,
                    generation,
                    Change {
                        open: false,
                        reason: Reason::AnchorVisibility,
                        restore_focus: false,
                    },
                );
            }
        });
    }
    fn input(&self, event: FormEvent) {
        event.stop_propagation();
        let Ok(report) = serde_json::from_str::<Report>(&event.value()) else {
            return;
        };
        self.observe(&report);
    }
}

impl NodeHandle {
    pub(crate) fn set_ownership(&self, ancestors: Vec<NodeHandle>, ready: bool) {
        let Some(runtime) = self.runtime() else {
            return;
        };
        let mut lineage = Vec::new();
        for ancestor in ancestors {
            if ancestor != *self && ancestor.is_registered() && !lineage.contains(&ancestor) {
                lineage.push(ancestor);
            }
        }
        let changed = {
            let mut state = runtime.0.borrow_mut();
            let Some(entry) = state
                .nodes
                .get_mut(&self.id)
                .filter(|entry| entry.generation == self.generation)
            else {
                return;
            };
            if entry.ancestors == lineage && entry.ownership_ready == ready {
                false
            } else {
                entry.ancestors = lineage;
                entry.ownership_ready = ready;
                true
            }
        };
        if changed {
            runtime.publish_ownership();
        }
    }
    pub(crate) fn set_mounted(&self) {
        let Some(runtime) = self.runtime() else {
            return;
        };
        let changed = {
            let mut state = runtime.0.borrow_mut();
            let Some(entry) = state
                .nodes
                .get_mut(&self.id)
                .filter(|entry| entry.generation == self.generation)
            else {
                return;
            };
            let changed = !entry.mounted;
            entry.mounted = true;
            changed
        };
        if changed {
            runtime.publish_ownership();
        }
    }

    pub(crate) fn owns_keyboard_topmost(&self) -> bool {
        self.runtime().is_some_and(|runtime| {
            runtime
                .entry(&self.id)
                .is_some_and(|entry| entry.generation == self.generation)
                && runtime.owns_dioxus_topmost(&self.id)
        })
    }

    /// Plan a boundary wrap for a directly delivered Dioxus Tab event.
    /// Callers apply the returned default/propagation decision to that event.
    pub(crate) fn dioxus_popover_tab(&self, event: &KeyboardEvent, enabled: bool) -> KeyDecision {
        if !enabled || !event.default_action_enabled() || event.key() != Key::Tab {
            return KeyDecision::default();
        }
        let Some(runtime) = self.runtime() else {
            return KeyDecision::default();
        };
        runtime
            .with_owner(&self.id, self.generation, || {
                let Some(entry) = runtime.entry(&self.id).filter(|entry| {
                    entry.generation == self.generation
                        && entry.kind == OverlayKind::Popover
                        && entry.mounted
                }) else {
                    return KeyDecision::default();
                };
                if runtime.read_current(&self.id, &entry) != Some(true) {
                    return KeyDecision::default();
                }
                let Some(report) = runtime.0.borrow().observation.clone() else {
                    return KeyDecision::default();
                };
                let Some(node) = report
                    .nodes
                    .iter()
                    .find(|node| node.id == self.id && node.generation == self.generation)
                else {
                    return KeyDecision::default();
                };
                let Some(domain) = runtime.domain() else {
                    return KeyDecision::default();
                };
                let Some(excluded) = domain.focus_trap_exclusions(
                    &self.id,
                    &report.active_hosts,
                    &report.active_popover,
                ) else {
                    return KeyDecision::default();
                };
                let candidates = runtime.source_tab_candidates(&report, &node.tab_candidates);
                let Some(target) = crate::popover_focus::tab_boundary_target(
                    &candidates,
                    &excluded,
                    event.modifiers().contains(Modifiers::SHIFT),
                ) else {
                    return KeyDecision::default();
                };
                let Some(target) = self.capture_registered_focus(&target) else {
                    return KeyDecision::default();
                };
                if !self.queue_registered_focus(target, Some(true)) {
                    return KeyDecision::default();
                }
                KeyDecision {
                    prevent: true,
                    stop: true,
                    ..Default::default()
                }
            })
            .unwrap_or_default()
    }
    /// A current Dioxus content surface owns its virtual click propagation.
    /// Keep browser defaults intact, including native input and form actions.
    pub(crate) fn dioxus_inside_click(&self, event: &MouseEvent) {
        let Some(runtime) = self.runtime() else {
            return;
        };
        let _ = runtime.with_owner(&self.id, self.generation, || {
            let Some(entry) = runtime
                .entry(&self.id)
                .filter(|entry| entry.generation == self.generation && entry.kind.is_dropdown())
            else {
                return;
            };
            if runtime.read_current(&self.id, &entry) != Some(true) {
                return;
            }
            event.stop_propagation();
            if event.default_action_enabled() && entry.behavior.allows_inside() {
                runtime.dismiss_dropdown_set(&self.id, self.generation, true);
            }
        });
    }
    /// Build keyboard facts from an explicit Dioxus control and the latest
    /// read-only inventory/Tab observation. Registration, not native listener
    /// installation or a captured DOM receipt, authorizes the event.
    pub(crate) fn dioxus_key_facts(
        &self,
        event: &KeyboardEvent,
        on_trigger: bool,
        focused_target: Option<String>,
        inside: bool,
    ) -> Option<KeyFacts> {
        let runtime = self.runtime()?;
        runtime
            .with_owner(&self.id, self.generation, || {
                let entry = runtime.entry(&self.id)?;
                runtime.read_current(&self.id, &entry)?;
                let navigation_root = entry
                    .submenu
                    .as_ref()
                    .and_then(|access| (access.query_root)())
                    .filter(|root| runtime.entry(root).is_some());
                let navigation_id = navigation_root.as_deref().unwrap_or(&self.id);
                let navigation_entry = runtime.entry(navigation_id)?;
                let state = runtime.0.borrow();
                let observed = state.observation.as_ref();
                let owner = observed.and_then(|report| {
                    report
                        .nodes
                        .iter()
                        .find(|node| node.id == self.id && node.generation == self.generation)
                });
                let inventory = state.owned_inventory(navigation_id, navigation_entry.generation);
                let focused = focused_target.or_else(|| {
                    if inside && !on_trigger {
                        observed
                            .map(|report| report.active_id.clone())
                            .filter(|target| !target.is_empty())
                    } else {
                        None
                    }
                });
                let physical_target = focused.as_ref().map(|target| {
                    state
                        .item_mounts
                        .values()
                        .find(|mount| mount.target == *target || mount.focus_id == *target)
                        .map_or_else(|| target.clone(), |mount| mount.focus_id.clone())
                });
                let focused_target = focused.map(|target| {
                    state
                        .item_mounts
                        .values()
                        .find(|mount| mount.focus_id == target)
                        .map_or(target.clone(), |mount| mount.target.clone())
                });
                let focusable_ids =
                    owner.map_or_else(Vec::new, |owner| owner.focusable_ids.clone());
                let focused_index = physical_target.as_ref().and_then(|target| {
                    focusable_ids
                        .iter()
                        .position(|candidate| candidate == target)
                });
                let focused_item = focused_target
                    .as_ref()
                    .and_then(|target| target.strip_prefix("ix-dropdown-item-")?.parse().ok());
                let items = Vec::new();
                drop(state);
                Some(KeyFacts {
                    navigation_mode: navigation_entry.navigation_mode,
                    key: event.key().to_string(),
                    shift: event.modifiers().contains(Modifiers::SHIFT),
                    alt: event.modifiers().contains(Modifiers::ALT),
                    ctrl: event.modifiers().contains(Modifiers::CONTROL),
                    meta: event.modifiers().contains(Modifiers::META),
                    default_prevented: !event.default_action_enabled(),
                    focused_item,
                    focused_target,
                    items,
                    focus_count: focusable_ids.len(),
                    focusable_ids,
                    focused_index,
                    on_trigger,
                    inside,
                    popover_ancestor: runtime
                        .domain()
                        .is_some_and(|domain| domain.has_popover_ancestor(&self.id)),
                    inventory: inventory.map(|facts| runtime.projected_inventory(facts)),
                })
            })
            .flatten()
    }
    /// Dispatch one Dioxus key in this surface's current owner. The snapshot is
    /// used only to select registered mounted destinations before callbacks.
    pub(crate) fn dioxus_key(&self, mut facts: KeyFacts) -> KeyDecision {
        let Some(runtime) = self.runtime() else {
            return KeyDecision::default();
        };
        let Some(entry) = runtime
            .entry(&self.id)
            .filter(|entry| entry.generation == self.generation)
        else {
            return KeyDecision::default();
        };
        facts.navigation_mode = entry
            .submenu
            .as_ref()
            .and_then(|access| (access.query_root)())
            .and_then(|root| runtime.entry(&root))
            .and_then(|entry| entry.navigation_mode)
            .or(entry.navigation_mode);
        facts.popover_ancestor = runtime
            .domain()
            .is_some_and(|domain| domain.has_popover_ancestor(&self.id));
        let mut report = runtime.0.borrow().observation.clone().unwrap_or_default();
        report.hosts = if facts.inside {
            vec![self.id.clone()]
        } else {
            Vec::new()
        };
        report.triggers = if facts.on_trigger {
            vec![self.id.clone()]
        } else {
            Vec::new()
        };
        runtime.dispatch_key(&self.id, self.generation, facts, &report)
    }
    pub(crate) fn capture_registered_focus(&self, id: &str) -> Option<MountedFocusTarget> {
        let runtime = self.runtime()?;
        runtime
            .with_owner(&self.id, self.generation, || {
                runtime.capture_registered_focus(id)
            })
            .flatten()
    }
    pub(crate) fn queue_registered_focus(
        &self,
        target: MountedFocusTarget,
        expected_open: Option<bool>,
    ) -> bool {
        self.runtime().is_some_and(|runtime| {
            runtime.queue_registered_focus(
                &self.id,
                self.generation,
                target,
                expected_open,
                Vec::new(),
            )
        })
    }
    pub(crate) fn cancel_registered_focus(&self) {
        if let Some(runtime) = self.runtime()
            && runtime
                .entry(&self.id)
                .is_some_and(|entry| entry.generation == self.generation)
        {
            runtime.cancel_owned_item_focus(&self.id);
        }
    }
    /// Revision reflected by the component's next native renderer commit.
    pub(crate) fn render_revision(&self) -> u64 {
        self.runtime()
            .and_then(|runtime| runtime.entry(&self.id))
            .filter(|entry| entry.generation == self.generation)
            .map_or(0, |entry| entry.revision)
    }
    pub(crate) fn set_navigation_mode(&self, mode: &str) -> bool {
        self.runtime()
            .is_some_and(|runtime| runtime.set_navigation_mode(&self.id, self.generation, mode))
    }
    pub(crate) async fn query_inventory(&self) -> Option<InventoryFacts> {
        self.inventory()
    }
    pub(crate) fn inventory(&self) -> Option<InventoryFacts> {
        let runtime = self.runtime()?;
        runtime
            .with_owner(&self.id, self.generation, || {
                let entry = runtime.entry(&self.id)?;
                runtime.read_current(&self.id, &entry)?;
                let facts = runtime
                    .0
                    .borrow()
                    .owned_inventory(&self.id, self.generation)?;
                Some(runtime.projected_inventory(facts))
            })
            .flatten()
    }
    pub(crate) fn select_roving(&self, facts: &InventoryFacts, target: &ItemRef) -> bool {
        let Some(runtime) = self.runtime() else {
            return false;
        };
        runtime
            .with_owner(&self.id, self.generation, || {
                if !runtime.item_owner_open(&self.id, self.generation) {
                    return false;
                }
                let inventory = crate::dropdown_inventory::Inventory::new(
                    crate::dropdown_inventory::InventoryMode::RovingTabindex,
                    runtime.projected_inventory(facts.clone()),
                );
                runtime.set_roving_target(
                    &self.id,
                    self.generation,
                    &inventory.query_hosts(),
                    target,
                )
            })
            .unwrap_or(false)
    }
    pub(crate) fn init_roving(
        &self,
        facts: &InventoryFacts,
        last: bool,
        focus_checked: bool,
    ) -> Option<HostFacts> {
        let runtime = self.runtime()?;
        runtime
            .with_owner(&self.id, self.generation, || {
                if !runtime.item_owner_open(&self.id, self.generation) {
                    return None;
                }
                let inventory = crate::dropdown_inventory::Inventory::new(
                    crate::dropdown_inventory::InventoryMode::RovingTabindex,
                    runtime.projected_inventory(facts.clone()),
                );
                let candidates = {
                    let state = runtime.0.borrow();
                    inventory
                        .query_hosts()
                        .into_iter()
                        .filter(|host| RuntimeHandle::mounted_item_key(&state, host).is_some())
                        .collect::<Vec<_>>()
                };
                let edge = if last {
                    candidates.last()
                } else {
                    candidates.first()
                }?;
                let target = if focus_checked {
                    candidates
                        .iter()
                        .find(|host| host.checked_attribute)
                        .unwrap_or(edge)
                } else {
                    edge
                };
                runtime
                    .set_roving_target(&self.id, self.generation, &candidates, &target.item)
                    .then(|| target.clone())
            })
            .flatten()
    }
    pub(crate) fn clear_roving(&self) {
        let Some(runtime) = self.runtime() else {
            return;
        };
        if runtime
            .entry(&self.id)
            .is_some_and(|entry| entry.generation == self.generation)
        {
            runtime.clear_item_tabindices(&self.id, self.generation);
        }
    }
    pub(crate) fn is_item_registered(&self, host: &HostFacts) -> bool {
        let Some(runtime) = self.runtime() else {
            return false;
        };
        runtime
            .with_owner(&self.id, self.generation, || {
                RuntimeHandle::mounted_item_key(&runtime.0.borrow(), host).is_some()
            })
            .unwrap_or(false)
    }
    pub(crate) fn queue_item_host_focus(&self, host: &HostFacts, callback: Rc<dyn Fn()>) -> bool {
        self.queue_mounted_item_focus(host.target.clone(), true, callback, Some(host.clone()))
    }
    /// Native candidate order is observational only; registered Dioxus handles
    /// remain the sole executor, including items with a distinct inner control.
    pub(crate) fn queue_tab_boundary_focus(
        &self,
        ids: &[String],
        last: bool,
        callback: Rc<dyn Fn()>,
    ) -> bool {
        let Some(runtime) = self.runtime() else {
            return false;
        };
        if !runtime.item_owner_open(&self.id, self.generation) {
            return false;
        }
        let Some(inventory) = self.inventory() else {
            return false;
        };
        let host = {
            let state = runtime.0.borrow();
            let matching = |id: &String| {
                if id.is_empty() {
                    return None;
                }
                inventory
                    .hosts
                    .iter()
                    .find(|host| {
                        !host.disabled_attribute
                            && !host.hidden_attribute
                            && RuntimeHandle::mounted_item_key(&state, host)
                                .and_then(|key| state.item_mounts.get(&key))
                                .is_some_and(|mount| mount.focus_id == *id)
                    })
                    .cloned()
            };
            if last {
                ids.iter().rev().find_map(matching)
            } else {
                ids.iter().find_map(matching)
            }
        };
        host.is_some_and(|host| self.queue_item_host_focus(&host, callback))
    }
    /// Only explicitly registered Dioxus mounted elements can receive item focus.
    /// Consumers opt in by registering their own mounted control.
    pub(crate) fn queue_item_focus(
        &self,
        target: String,
        focus_native: bool,
        callback: Rc<dyn Fn()>,
    ) -> bool {
        self.queue_mounted_item_focus(target, focus_native, callback, None)
    }
    fn queue_mounted_item_focus(
        &self,
        target: String,
        focus_native: bool,
        callback: Rc<dyn Fn()>,
        visual: Option<HostFacts>,
    ) -> bool {
        if target.is_empty() {
            return false;
        }
        let Some(runtime) = self.runtime() else {
            return false;
        };
        runtime
            .with_owner(&self.id, self.generation, || {
                let Some(entry) = runtime.entry(&self.id) else {
                    return false;
                };
                if entry.scope.is_none()
                    || runtime.read_current(&self.id, &entry) != Some(true)
                    || !runtime
                        .entry(&self.id)
                        .is_some_and(|entry| entry.generation == self.generation)
                {
                    return false;
                }
                let visual = visual.or_else(|| {
                    runtime
                        .0
                        .borrow()
                        .owned_inventory(&self.id, self.generation)
                        .and_then(|inventory| {
                            inventory
                                .hosts
                                .into_iter()
                                .find(|host| host.target == target)
                        })
                });
                let key = {
                    let state = runtime.0.borrow();
                    visual
                        .as_ref()
                        .filter(|host| state.item_mounts.contains_key(&host.registration_key))
                        .map(|host| host.registration_key.clone())
                        .or_else(|| {
                            state
                                .item_mounts
                                .iter()
                                .find(|(_, mount)| mount.target == target)
                                .map(|(key, _)| key.clone())
                        })
                };
                let Some(key) = key else {
                    return false;
                };
                runtime.cancel_owned_item_focus(&self.id);
                let sequence = {
                    let mut state = runtime.0.borrow_mut();
                    state.item_focus_sequence = state.item_focus_sequence.wrapping_add(1).max(1);
                    let sequence = state.item_focus_sequence;
                    state.owned_item_focus.insert(
                        self.id.clone(),
                        OwnedItemFocus {
                            sequence,
                            generation: self.generation,
                            key,
                            callback: Some(callback),
                            visual,
                            focus_native,
                            exit_target: None,
                            expected_open: Some(true),
                            destination_popovers: Vec::new(),
                            task: None,
                        },
                    );
                    sequence
                };
                let weak = Rc::downgrade(&runtime.0);
                let root = self.id.clone();
                let generation = self.generation;
                dioxus::core::queue_effect(move || {
                    if let Some(state) = weak.upgrade() {
                        let runtime = RuntimeHandle(state);
                        runtime.with_owner(&root, generation, || {
                            runtime.start_owned_item_focus(root.clone(), generation, sequence)
                        });
                    }
                });
                true
            })
            .unwrap_or(false)
    }
    /// Deliver changed-state notifications through the registered component's
    /// Dioxus effect queue. Admission never invokes a listener synchronously.
    pub(crate) fn queue_completion(&self, callback: Rc<dyn Fn()>) -> bool {
        let Some(runtime) = self.runtime() else {
            return false;
        };
        runtime
            .with_owner(&self.id, self.generation, || {
                let Some(entry) = runtime
                    .entry(&self.id)
                    .filter(|entry| entry.generation == self.generation)
                else {
                    return false;
                };
                // A completion belongs to a real Dioxus component. There is no
                // native transport or synchronous fallback for unowned work.
                if entry.scope.is_none() || runtime.read_current(&self.id, &entry).is_none() {
                    return false;
                }
                let ticket = runtime.0.borrow_mut().completions.admit(
                    self.id.clone(),
                    self.generation,
                    callback,
                );
                let node = self.clone();
                dioxus::core::queue_effect(move || {
                    let Some(runtime) = node.runtime() else {
                        return;
                    };
                    let callback =
                        runtime
                            .0
                            .borrow_mut()
                            .completions
                            .take(ticket, &node.id, node.generation);
                    if let Some(callback) = callback {
                        runtime.with_owner(&node.id, node.generation, || {
                            let Some(entry) = runtime
                                .entry(&node.id)
                                .filter(|entry| entry.generation == node.generation)
                            else {
                                return;
                            };
                            if runtime.read_current(&node.id, &entry).is_some() {
                                callback();
                            }
                        });
                    }
                });
                // An async native completion can arrive without another
                // component state change. Wake this owner to flush its effect.
                dioxus::core::needs_update();
                true
            })
            .unwrap_or(false)
    }
    pub(crate) fn set_anchor_shape_handler(&self, handler: Rc<dyn Fn(bool)>) {
        let Some(runtime) = self.runtime() else {
            return;
        };
        if let Some(entry) = runtime.0.borrow_mut().nodes.get_mut(&self.id)
            && entry.generation == self.generation
        {
            entry.anchor_shape = Some(handler);
        }
    }
    /// Deliver visibility from a registered Dioxus trigger's `onvisible`.
    /// The component binding owns trigger lifetime; this handle additionally
    /// rejects dropped/replaced surfaces and dispatches in the surface owner.
    pub(crate) fn observe_anchor_visibility(
        &self,
        intersecting: bool,
        geometry: Option<AnchorIntersection>,
    ) {
        if let Some(runtime) = self.runtime() {
            runtime.anchor_visibility(&self.id, self.generation, intersecting, geometry);
        }
    }
    pub(crate) fn set_anchor_intersection_handler(&self, handler: AnchorIntersectionCallback) {
        let Some(runtime) = self.runtime() else {
            return;
        };
        if let Some(entry) = runtime.0.borrow_mut().nodes.get_mut(&self.id)
            && entry.generation == self.generation
        {
            entry.anchor_intersection = Some(handler);
        }
    }
    pub(crate) fn set_trigger_visibility_check(&self, enabled: bool) {
        let Some(runtime) = self.runtime() else {
            return;
        };
        let mut state = runtime.0.borrow_mut();
        if let Some(entry) = state.nodes.get_mut(&self.id)
            && entry.generation == self.generation
        {
            entry.trigger_visibility_check = enabled;
        }
    }
    pub(crate) fn runtime(&self) -> Option<RuntimeHandle> {
        self.runtime.upgrade().map(RuntimeHandle)
    }
    pub(crate) fn id(&self) -> &str {
        &self.id
    }
    /// Execute admitted work in the registered component's runtime/scope.
    /// Queued effects must not inherit a transient invoker's scope.
    pub(crate) fn with_owner<T>(&self, f: impl FnOnce() -> T) -> Option<T> {
        self.runtime()?.with_owner(&self.id, self.generation, f)
    }
    pub(crate) fn request(&self, open: bool, reason: Reason, restore_focus: bool) -> bool {
        self.runtime().is_some_and(|runtime| {
            // Public handles may be invoked from a short-lived trigger or a
            // different VDOM. Apply can spawn admitted native work: that task
            // belongs to the registered component, not the invoking scope.
            runtime
                .with_owner(&self.id, self.generation, || {
                    runtime.request(
                        &self.id,
                        self.generation,
                        Change {
                            open,
                            reason,
                            restore_focus,
                        },
                    )
                })
                .unwrap_or(false)
        })
    }
    pub(crate) fn is_registered(&self) -> bool {
        self.runtime().is_some_and(|runtime| {
            runtime
                .entry(&self.id)
                .is_some_and(|entry| entry.generation == self.generation)
        })
    }
    pub(crate) fn generation(&self) -> u64 {
        self.generation
    }
    pub(crate) fn prepare_present(&self) -> bool {
        let Some(runtime) = self.runtime() else {
            return false;
        };
        if !runtime
            .entry(&self.id)
            .is_some_and(|entry| entry.generation == self.generation)
        {
            return false;
        }
        if let Some(domain) = runtime.domain() {
            return domain.prepare_present(&self.id);
        }
        // Before native document attachment, retain controller behavior within
        // this root. Cross-root bootstrap remains a separately tracked boundary.
        let excluded = runtime.0.borrow().registry.cached_path_including(&self.id);
        let candidates = runtime.0.borrow().order.clone();
        for id in candidates {
            if excluded.contains(&id) {
                continue;
            }
            if runtime.entry(&id).is_some_and(|entry| {
                entry.kind == OverlayKind::Popover && entry.behavior.allows_outside()
            }) {
                runtime.request_id(
                    &id,
                    Change {
                        open: false,
                        reason: Reason::Outside,
                        restore_focus: true,
                    },
                );
            }
        }
        runtime
            .entry(&self.id)
            .is_some_and(|entry| entry.generation == self.generation)
    }
    pub(crate) fn did_present(&self) {
        if let Some(runtime) = self.runtime() {
            if runtime
                .entry(&self.id)
                .is_some_and(|entry| entry.generation == self.generation)
            {
                runtime.0.borrow_mut().registry.set_presence(&self.id, true);
                if let Some(domain) = runtime.domain() {
                    domain.presented(&self.id, self.generation);
                }
            }
        }
    }
    pub(crate) fn sync_open(&self, open: bool) {
        if let Some(runtime) = self.runtime() {
            runtime.sync_open(&self.id, self.generation, open);
        }
    }
    pub(crate) fn set_close_behavior(&self, behavior: CloseBehavior) {
        if let Some(runtime) = self.runtime() {
            let mut runtime = runtime.0.borrow_mut();
            if let Some(entry) = runtime
                .nodes
                .get_mut(&self.id)
                .filter(|entry| entry.generation == self.generation)
            {
                entry.behavior = behavior;
                runtime.registry.set_close_behavior(&self.id, behavior);
                if let Some(domain) = &runtime.domain {
                    domain.set_close_behavior(&self.id, self.generation, behavior);
                }
            }
        }
    }
    pub(crate) fn close_children(&self, reason: Reason) {
        if let Some(runtime) = self.runtime() {
            if runtime
                .entry(&self.id)
                .is_some_and(|entry| entry.generation == self.generation)
            {
                runtime.close_children(&self.id, reason);
            }
        }
    }
    pub(crate) fn unregister(&self) {
        if let Some(runtime) = self.runtime() {
            runtime.unregister(&self.id, self.generation);
        }
    }
}

pub(crate) fn use_overlay_runtime() -> RuntimeHandle {
    use_root_context(RuntimeHandle::new)
}

#[component]
pub(crate) fn OverlayTransport(runtime: RuntimeHandle) -> Element {
    let owner = runtime.owner_id();
    let mounted = runtime.clone();
    let input = runtime.clone();
    rsx! { input { r#type:"hidden", "data-ix-overlay-transport":owner,
        onmounted:move |_| mounted.start_bridge(),
        oninput:move |event| input.input(event),
    } }
}

#[path = "submenu_runtime.rs"]
mod submenu_runtime;

#[cfg(test)]
#[path = "submenu_runtime_tests.rs"]
mod submenu_runtime_tests;

#[cfg(test)]
#[path = "overlay_assignment_tests.rs"]
mod overlay_assignment_tests;

#[cfg(test)]
#[path = "item_visual_transport_tests.rs"]
mod item_visual_transport_tests;

#[derive(Clone, Default, Deserialize)]
struct Report {
    owner: String,
    sequence: u64,
    kind: String,
    #[serde(default)]
    document: String,
    #[serde(default)]
    document_event: Option<DocumentEvent>,
    #[serde(default)]
    active_id: String,
    #[serde(default)]
    hosts: Vec<String>,
    #[serde(default)]
    triggers: Vec<String>,
    #[serde(default)]
    nodes: Vec<NodeFacts>,
    #[serde(default)]
    tab_order: Vec<String>,
    #[serde(default)]
    document_tab_candidates: Option<Vec<crate::popover_focus::TabCandidate>>,
    #[serde(default)]
    active_hosts: Vec<String>,
    #[serde(default)]
    active_popover: String,
}
#[derive(Clone, Deserialize)]
struct DocumentEvent {
    sequence: u64,
    #[serde(flatten)]
    action: DocumentAction,
}
impl DocumentEvent {
    fn valid(&self) -> bool {
        self.sequence != 0
            && match &self.action {
                DocumentAction::Input { input_kind, .. } => {
                    matches!(input_kind.as_str(), "keydown" | "mousedown" | "touchstart")
                }
                _ => true,
            }
    }
}
#[derive(Clone, Deserialize)]
#[serde(tag = "action", rename_all = "lowercase")]
enum DocumentAction {
    Outside {
        #[serde(default)]
        hosts: Vec<String>,
        #[serde(default)]
        triggers: Vec<String>,
        #[serde(default)]
        anchors: Vec<String>,
    },
    Escape,
    Input {
        input_kind: String,
        #[serde(default)]
        key: String,
    },
}
#[derive(Clone, Deserialize)]
struct NodeFacts {
    id: String,
    generation: u64,
    #[serde(default)]
    trigger_id: String,
    #[serde(default)]
    anchor_in_dropdown_item: Option<bool>,
    #[serde(default)]
    header_controls: Vec<String>,
    #[serde(default)]
    item_order: Vec<String>,
    #[serde(default)]
    focusable_ids: Vec<String>,
    #[serde(default)]
    tab_candidates: Vec<crate::popover_focus::TabCandidate>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use dioxus::core::current_scope_id;
    use std::cell::Cell;

    #[test]
    fn category_coordination_before_document_attach_allows_reentrant_removal() {
        let runtime = RuntimeHandle::new();
        let calls = Rc::new(RefCell::new(Vec::new()));
        let source_calls = calls.clone();
        runtime.register_category(1, Rc::new(move |_, _| source_calls.borrow_mut().push(1)));
        let recipient_calls = calls.clone();
        let owner = runtime.clone();
        runtime.register_category(
            2,
            Rc::new(move |source, active| {
                assert_eq!((source, active), (1, &[1][..]));
                recipient_calls.borrow_mut().push(2);
                owner.unregister_category(2);
            }),
        );
        assert!(runtime.broadcast_category(1, &[1]));
        assert!(runtime.broadcast_category(1, &[1]));
        assert_eq!(*calls.borrow(), [2]);
    }

    #[derive(Clone)]
    struct RequestTaskFixture(Rc<RequestTaskState>);
    impl PartialEq for RequestTaskFixture {
        fn eq(&self, other: &Self) -> bool {
            Rc::ptr_eq(&self.0, &other.0)
        }
    }
    #[derive(Default)]
    struct RequestTaskState {
        node: RefCell<Option<NodeHandle>>,
        caller: Cell<Option<ScopeId>>,
        owner: Cell<Option<ScopeId>>,
        show_caller: Cell<Option<Signal<bool>>>,
        caller_dropped: Cell<bool>,
        open: Cell<bool>,
        complete: Cell<bool>,
        release: Cell<bool>,
        waker: RefCell<Option<std::task::Waker>>,
    }
    #[derive(Clone, Copy)]
    struct RequestTaskContext(&'static str);

    #[component]
    fn RequestTaskOwner(fixture: RequestTaskFixture) -> Element {
        use_context_provider(|| RequestTaskContext("owner"));
        let runtime = use_hook(RuntimeHandle::new);
        let registered = fixture.clone();
        let node = use_hook(move || {
            let scope = current_scope_id();
            registered.0.owner.set(Some(scope));
            registered.0.open.set(true);
            let read = registered.clone();
            let before = registered.clone();
            let apply = registered.clone();
            let node = runtime.register(
                "caller-task-overlay".into(),
                1,
                OverlayKind::Popover,
                CloseBehavior::Both,
                Callbacks {
                    read: Rc::new(move || {
                        assert_eq!(current_scope_id(), scope);
                        assert_eq!(consume_context::<RequestTaskContext>().0, "owner");
                        Some(read.0.open.get())
                    }),
                    before: Rc::new(move |_| {
                        assert_eq!(current_scope_id(), before.0.owner.get().unwrap());
                        assert_eq!(consume_context::<RequestTaskContext>().0, "owner");
                        true
                    }),
                    apply: Rc::new(move |change| {
                        assert_eq!(current_scope_id(), scope);
                        apply.0.open.set(change.open);
                        let pending = apply.clone();
                        spawn(async move {
                            std::future::poll_fn(|cx| {
                                if pending.0.release.get() {
                                    std::task::Poll::Ready(())
                                } else {
                                    *pending.0.waker.borrow_mut() = Some(cx.waker().clone());
                                    std::task::Poll::Pending
                                }
                            })
                            .await;
                            assert_eq!(current_scope_id(), scope);
                            assert_eq!(consume_context::<RequestTaskContext>().0, "owner");
                            pending.0.complete.set(true);
                        });
                    }),
                    key: None,
                },
            );
            *registered.0.node.borrow_mut() = Some(node.clone());
            node
        });
        use_drop(move || node.unregister());
        rsx! { div {} }
    }

    #[component]
    fn RequestTaskCaller(fixture: RequestTaskFixture) -> Element {
        use_context_provider(|| RequestTaskContext("caller"));
        fixture.0.caller.set(Some(current_scope_id()));
        use_drop(move || fixture.0.caller_dropped.set(true));
        rsx! { div {} }
    }

    fn request_task_root(fixture: RequestTaskFixture) -> Element {
        let show = use_signal(|| true);
        fixture.0.show_caller.set(Some(show));
        rsx! {
            RequestTaskOwner { fixture: fixture.clone() }
            if show() { RequestTaskCaller { fixture: fixture.clone() } }
        }
    }

    #[test]
    fn direct_request_native_work_survives_caller_scope_teardown() {
        let fixture = RequestTaskFixture(Rc::new(RequestTaskState::default()));
        let mut dom = VirtualDom::new_with_props(request_task_root, fixture.clone());
        dom.rebuild_in_place();
        let node = fixture.0.node.borrow().clone().unwrap();
        let caller = fixture.0.caller.get().unwrap();
        assert_ne!(fixture.0.owner.get(), Some(caller));
        dom.in_scope(caller, || {
            assert!(node.request(false, Reason::Trigger, true));
            assert_eq!(current_scope_id(), caller, "dispatch restores its caller");
            assert_eq!(consume_context::<RequestTaskContext>().0, "caller");
        });
        assert!(!fixture.0.open.get());
        dom.render_immediate_to_vec();
        assert!(
            fixture.0.waker.borrow().is_some(),
            "accepted work reached await"
        );
        dom.in_scope(ScopeId::APP, || {
            let mut show = fixture.0.show_caller.get().unwrap();
            show.set(false);
        });
        dom.render_immediate_to_vec();
        assert!(fixture.0.caller_dropped.get());
        assert!(!fixture.0.complete.get());
        fixture.0.release.set(true);
        fixture.0.waker.borrow_mut().take().unwrap().wake();
        dom.render_immediate_to_vec();
        assert!(
            fixture.0.complete.get(),
            "owner task survived caller teardown"
        );
        dom.in_scope(fixture.0.owner.get().unwrap(), || node.unregister());
        assert!(
            !node.request(false, Reason::Trigger, true),
            "stale registration is rejected"
        );
        assert!(
            node.with_owner(|| panic!("stale owner cannot run work"))
                .is_none()
        );
    }

    struct Model {
        open: Rc<Cell<bool>>,
        veto: Rc<Cell<bool>>,
        node: NodeHandle,
    }
    fn model(
        runtime: &RuntimeHandle,
        id: &str,
        kind: OverlayKind,
        log: Rc<RefCell<Vec<String>>>,
    ) -> Model {
        let open = Rc::new(Cell::new(false));
        let veto = Rc::new(Cell::new(false));
        let read = open.clone();
        let apply = open.clone();
        let before_veto = veto.clone();
        let before_log = log.clone();
        let apply_log = log.clone();
        let before_id = id.to_string();
        let apply_id = id.to_string();
        let node = runtime.register(
            id.to_string(),
            1,
            kind,
            CloseBehavior::Both,
            Callbacks {
                read: Rc::new(move || Some(read.get())),
                before: Rc::new(move |change| {
                    before_log
                        .borrow_mut()
                        .push(format!("before:{before_id}:{}", change.open));
                    !before_veto.get()
                }),
                apply: Rc::new(move |change| {
                    apply.set(change.open);
                    apply_log
                        .borrow_mut()
                        .push(format!("apply:{apply_id}:{}", change.open));
                }),
                key: None,
            },
        );
        node.set_ownership(Vec::new(), true);
        node.set_mounted();
        Model { open, veto, node }
    }
    fn assign(runtime: &RuntimeHandle, id: &str, ancestors: &[&str], ready: bool) {
        let handle = |id: &str| NodeHandle {
            runtime: Rc::downgrade(&runtime.0),
            id: id.into(),
            generation: runtime.entry(id).unwrap().generation,
        };
        let node = handle(id);
        node.set_ownership(ancestors.iter().map(|id| handle(id)).collect(), ready);
        node.set_mounted();
    }
    #[test]
    fn authored_ownership_ignores_native_graph_claims_and_guards_actual_lifetimes() {
        let runtime = RuntimeHandle::new();
        let log = Rc::new(RefCell::new(Vec::new()));
        let parent = model(&runtime, "owned-parent", OverlayKind::Dropdown, log.clone());
        let child = model(&runtime, "owned-child", OverlayKind::Dropdown, log.clone());
        child.node.set_ownership(
            vec![parent.node.clone(), child.node.clone(), parent.node.clone()],
            true,
        );
        assert_eq!(
            runtime
                .0
                .borrow()
                .registry
                .cached_parent("owned-child")
                .as_deref(),
            Some("owned-parent")
        );
        runtime.observe(&report(
            &runtime,
            1,
            "observe",
            serde_json::json!([
                {"id":"owned-child","generation":1,"connected":false,"discovery_ready":false,
                    "trigger_ancestors":["fabricated"],"assignment_ancestors":[]}
            ]),
        ));
        runtime.observe(&report(&runtime, 2, "observe", serde_json::json!([])));
        assert_eq!(
            runtime
                .0
                .borrow()
                .registry
                .cached_parent("owned-child")
                .as_deref(),
            Some("owned-parent")
        );
        let same = runtime.register(
            "owned-child".into(),
            1,
            OverlayKind::Dropdown,
            CloseBehavior::Both,
            runtime.entry("owned-child").unwrap().callbacks,
        );
        assert!(same == child.node);
        assert_eq!(
            runtime
                .0
                .borrow()
                .registry
                .cached_parent("owned-child")
                .as_deref(),
            Some("owned-parent")
        );
        let replacement = runtime.register(
            "owned-parent".into(),
            2,
            OverlayKind::Dropdown,
            CloseBehavior::Both,
            runtime.entry("owned-parent").unwrap().callbacks,
        );
        replacement.set_ownership(Vec::new(), true);
        replacement.set_mounted();
        assert!(
            runtime
                .0
                .borrow()
                .registry
                .cached_parent("owned-child")
                .is_none(),
            "a same-ID replacement cannot inherit a stale parent handle"
        );
        parent.node.set_ownership(Vec::new(), true);
        child.node.set_ownership(vec![replacement.clone()], true);
        assert_eq!(
            runtime
                .0
                .borrow()
                .registry
                .cached_parent("owned-child")
                .as_deref(),
            Some("owned-parent")
        );
        let foreign = RuntimeHandle::new();
        let copied = model(&foreign, "owned-parent", OverlayKind::Dropdown, log);
        assert!(
            copied.node != parent.node,
            "runtime identity belongs to handle equality"
        );
        child.node.unregister();
        assert!(
            runtime
                .0
                .borrow()
                .registry
                .cached_children("owned-parent")
                .is_empty()
        );
    }

    #[derive(Clone, Default)]
    struct ScopeFixture(Rc<ScopeFixtureState>);
    #[derive(Default)]
    struct ScopeFixtureState {
        switch: Cell<Option<Signal<bool>>>,
        seen: RefCell<Vec<String>>,
    }
    impl PartialEq for ScopeFixture {
        fn eq(&self, other: &Self) -> bool {
            Rc::ptr_eq(&self.0, &other.0)
        }
    }
    #[component]
    fn ScopeFixtureChild(state: ScopeFixture) -> Element {
        let ancestors = use_overlay_scope(None);
        *state.0.seen.borrow_mut() = ancestors.iter().map(|node| node.id.clone()).collect();
        rsx! { div {} }
    }
    fn scope_fixture_root(state: ScopeFixture) -> Element {
        let runtime = use_hook(RuntimeHandle::new);
        let log = Rc::new(RefCell::new(Vec::new()));
        let nodes = use_hook(move || {
            (
                model(&runtime, "scope-first", OverlayKind::Header, log.clone()).node,
                model(&runtime, "scope-second", OverlayKind::Header, log).node,
            )
        });
        let switch = use_signal(|| false);
        state.0.switch.set(Some(switch));
        use_overlay_scope(Some(if switch() { nodes.1 } else { nodes.0 }));
        rsx! { ScopeFixtureChild { state } }
    }
    fn scope_override_fixture_root(state: ScopeFixture) -> Element {
        let runtime = use_hook(RuntimeHandle::new);
        let log = Rc::new(RefCell::new(Vec::new()));
        let nodes = use_hook(move || {
            (
                model(&runtime, "scope-inner", OverlayKind::Popover, log.clone()).node,
                model(&runtime, "scope-effective-parent", OverlayKind::Header, log).node,
            )
        });
        let lexical = use_overlay_ancestors();
        assert!(lexical.is_empty());
        use_overlay_scope_with_ancestors(Some(nodes.0), vec![nodes.1.clone(), nodes.1]);
        rsx! { ScopeFixtureChild { state } }
    }
    #[test]
    fn explicit_surface_lineage_reaches_body_without_lexical_parent_discovery() {
        let state = ScopeFixture::default();
        let mut dom = VirtualDom::new_with_props(scope_override_fixture_root, state.clone());
        dom.rebuild_in_place();
        assert_eq!(
            *state.0.seen.borrow(),
            ["scope-inner", "scope-effective-parent"]
        );
    }
    #[test]
    fn reactive_overlay_scope_tracks_parent_updates_without_consuming_own_provider() {
        let state = ScopeFixture::default();
        let mut dom = VirtualDom::new_with_props(scope_fixture_root, state.clone());
        dom.rebuild_in_place();
        assert_eq!(*state.0.seen.borrow(), ["scope-first"]);
        dom.in_runtime(|| state.0.switch.get().unwrap().set(true));
        for _ in 0..4 {
            dom.render_immediate_to_vec();
        }
        assert_eq!(*state.0.seen.borrow(), ["scope-second"]);
        dom.in_runtime(|| state.0.switch.get().unwrap().set(false));
        for _ in 0..4 {
            dom.render_immediate_to_vec();
        }
        assert_eq!(*state.0.seen.borrow(), ["scope-first"]);
    }

    fn report(
        runtime: &RuntimeHandle,
        sequence: u64,
        kind: &str,
        nodes: serde_json::Value,
    ) -> Report {
        serde_json::from_value(serde_json::json!({
            "owner":runtime.owner_id(),"sequence":sequence,"kind":kind,"nodes":nodes,
        }))
        .unwrap()
    }
    fn nodes(runtime: &RuntimeHandle, ancestors: &[&str]) -> serde_json::Value {
        assign(runtime, "child", ancestors, true);
        serde_json::json!([
            {"id":"header","generation":1},
            {"id":"child","generation":1}
        ])
    }

    struct TypedMouse;
    impl dioxus::html::InteractionLocation for TypedMouse {
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
    impl dioxus::html::InteractionElementOffset for TypedMouse {
        fn element_coordinates(&self) -> dioxus::html::geometry::ElementPoint {
            Default::default()
        }
    }
    impl dioxus::html::ModifiersInteraction for TypedMouse {
        fn modifiers(&self) -> Modifiers {
            Modifiers::empty()
        }
    }
    impl dioxus::html::PointerInteraction for TypedMouse {
        fn trigger_button(&self) -> Option<dioxus::html::input_data::MouseButton> {
            None
        }
        fn held_buttons(&self) -> dioxus::html::input_data::MouseButtonSet {
            Default::default()
        }
    }
    impl dioxus::html::HasMouseData for TypedMouse {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }
    pub(super) fn typed_mouse() -> MouseEvent {
        Event::new(Rc::new(dioxus::html::MouseData::new(TypedMouse)), true)
    }

    #[test]
    fn dioxus_inside_click_keeps_browser_defaults_and_current_peer_policies() {
        for behavior in [
            CloseBehavior::Both,
            CloseBehavior::Inside,
            CloseBehavior::Outside,
            CloseBehavior::None,
        ] {
            let runtime = RuntimeHandle::new();
            let log = Rc::new(RefCell::new(Vec::new()));
            let origin = model(&runtime, "origin", OverlayKind::Dropdown, log.clone());
            origin.node.set_close_behavior(behavior);
            origin.open.set(true);
            let peers = [
                model(&runtime, "both", OverlayKind::Dropdown, log.clone()),
                model(&runtime, "outside", OverlayKind::Dropdown, log.clone()),
                model(&runtime, "inside", OverlayKind::Dropdown, log.clone()),
                model(&runtime, "none", OverlayKind::Dropdown, log.clone()),
            ];
            peers[1].node.set_close_behavior(CloseBehavior::Outside);
            peers[2].node.set_close_behavior(CloseBehavior::Inside);
            peers[3].node.set_close_behavior(CloseBehavior::None);
            for peer in &peers {
                peer.open.set(true);
            }
            let click = typed_mouse();
            origin.node.dioxus_inside_click(&click);
            assert!(
                click.default_action_enabled(),
                "native input/form defaults remain enabled"
            );
            assert!(
                !click.propagates(),
                "the content owns virtual propagation for every policy"
            );
            assert_eq!(origin.open.get(), !behavior.allows_inside());
            assert_eq!(
                peers.map(|peer| peer.open.get()),
                if behavior.allows_inside() {
                    [false, false, true, true]
                } else {
                    [true; 4]
                }
            );
            assert_eq!(
                log.borrow()
                    .iter()
                    .filter(|entry| entry.as_str() == "before:origin:false")
                    .count(),
                if behavior.allows_inside() { 1 } else { 0 }
            );
        }
    }

    #[test]
    fn dioxus_inside_click_honors_consumer_prevention_veto_and_current_owner() {
        let runtime = RuntimeHandle::new();
        let log = Rc::new(RefCell::new(Vec::new()));
        let origin = model(&runtime, "origin", OverlayKind::Header, log.clone());
        let peer = model(&runtime, "peer", OverlayKind::Dropdown, log.clone());
        origin.open.set(true);
        peer.open.set(true);
        let prevented = typed_mouse();
        prevented.prevent_default();
        origin.node.dioxus_inside_click(&prevented);
        assert!(!prevented.default_action_enabled() && !prevented.propagates());
        assert!(origin.open.get() && peer.open.get() && log.borrow().is_empty());

        origin.veto.set(true);
        let click = typed_mouse();
        origin.node.dioxus_inside_click(&click);
        assert!(click.default_action_enabled() && !click.propagates());
        assert!(
            origin.open.get() && !peer.open.get(),
            "origin veto does not veto an eligible peer"
        );
        assert_eq!(
            log.borrow()
                .iter()
                .filter(|entry| entry.as_str() == "before:origin:false")
                .count(),
            1
        );
        origin.open.set(false);
        let closed = typed_mouse();
        origin.node.dioxus_inside_click(&closed);
        assert!(closed.default_action_enabled() && closed.propagates());
        origin.open.set(true);
        peer.open.set(true);
        let callbacks = runtime.entry("origin").unwrap().callbacks;
        let replacement = runtime.register(
            "origin".into(),
            2,
            OverlayKind::Header,
            CloseBehavior::Both,
            callbacks,
        );
        let stale = typed_mouse();
        origin.node.dioxus_inside_click(&stale);
        assert!(stale.default_action_enabled() && stale.propagates() && peer.open.get());
        replacement.unregister();
        let removed = typed_mouse();
        replacement.dioxus_inside_click(&removed);
        assert!(removed.default_action_enabled() && removed.propagates() && peer.open.get());
    }

    #[test]
    fn dioxus_surface_stop_does_not_replace_window_outside_coordination() {
        let runtime = RuntimeHandle::new();
        let log = Rc::new(RefCell::new(Vec::new()));
        let origin = model(&runtime, "origin", OverlayKind::Dropdown, log.clone());
        let peer = model(&runtime, "peer", OverlayKind::Dropdown, log);
        origin.node.set_close_behavior(CloseBehavior::Outside);
        origin.open.set(true);
        peer.open.set(true);
        let click = typed_mouse();
        origin.node.dioxus_inside_click(&click);
        assert!(click.default_action_enabled() && !click.propagates());
        assert!(origin.open.get() && peer.open.get());
        runtime.input(event(
            serde_json::json!({"owner":runtime.owner_id(),"document":runtime.owner_id(),
            "sequence":1,"kind":"observe",
            "nodes":[{"id":"origin","generation":1},
                {"id":"peer","generation":1}]}),
        ));
        let outside = event(
            serde_json::json!({"owner":runtime.owner_id(),"document":runtime.owner_id(),
            "sequence":2,"kind":"document","document_event":{"sequence":1,"action":"outside","hosts":["origin"]}}),
        );
        runtime.input(outside);
        assert!(origin.open.get() && !peer.open.get());
    }

    #[test]
    fn parent_request_child_veto_and_more_outside_are_not_atomic() {
        let runtime = RuntimeHandle::new();
        let log = Rc::new(RefCell::new(Vec::new()));
        let header = model(&runtime, "header", OverlayKind::Header, log.clone());
        let child = model(&runtime, "child", OverlayKind::Dropdown, log.clone());
        runtime.observe(&report(
            &runtime,
            1,
            "observe",
            nodes(&runtime, &["header"]),
        ));
        assert!(header.node.request(true, Reason::Trigger, false));
        assert!(child.node.request(true, Reason::Trigger, false));
        log.borrow_mut().clear();
        child.veto.set(true);
        assert!(header.node.request(false, Reason::Trigger, false));
        assert_eq!(
            &*log.borrow(),
            &[
                "before:header:false",
                "before:child:false",
                "apply:header:false"
            ]
        );
        assert!(!header.open.get() && child.open.get());
        child.veto.set(false);
        assert!(header.node.request(true, Reason::Trigger, false));
        header.veto.set(true);
        assert!(!header.node.request(false, Reason::Trigger, false));
        assert!(runtime.dismiss_dropdown_others(header.node.id(), header.node.generation()));
        assert!(
            header.open.get() && !child.open.get(),
            "the explicit More peer pass remains independent of its toggle veto"
        );
    }

    #[test]
    fn current_parent_controls_escape_and_header_focus_adapter() {
        let runtime = RuntimeHandle::new();
        let log = Rc::new(RefCell::new(Vec::new()));
        let header = model(&runtime, "header", OverlayKind::Header, log.clone());
        let child = model(&runtime, "child", OverlayKind::Dropdown, log);
        runtime.observe(&report(&runtime, 1, "observe", nodes(&runtime, &[])));
        runtime.observe(&report(
            &runtime,
            2,
            "observe",
            nodes(&runtime, &["header"]),
        ));
        assert!(header.node.request(true, Reason::Trigger, false));
        assert_eq!(
            runtime
                .0
                .borrow()
                .registry
                .cached_parent("child")
                .as_deref(),
            Some("header")
        );
        assert!(header.node.owns_keyboard_topmost());
        assert!(!child.node.owns_keyboard_topmost());
        assert!(child.node.request(true, Reason::Trigger, false));
        assert!(child.node.owns_keyboard_topmost());
        child.veto.set(true);
        assert!(header.node.request(false, Reason::Trigger, false));
        child.veto.set(false);
        assert!(header.node.request(true, Reason::Trigger, false));
        assert!(
            !child.node.owns_keyboard_topmost(),
            "newer topmost Header must retain flat Escape authority"
        );
        assert!(header.node.owns_keyboard_topmost());
        runtime
            .0
            .borrow_mut()
            .nodes
            .get_mut("child")
            .unwrap()
            .callbacks
            .key = Some(Rc::new(|_| panic!("newer topmost Header owns this Escape")));
        let decision = child.node.dioxus_key(KeyFacts {
            key: "Escape".into(),
            on_trigger: true,
            ..Default::default()
        });
        assert!(!decision.stop && !decision.prevent);
        assert!(header.open.get() && child.open.get());
        assert!(child.node.request(false, Reason::Trigger, false));
        assert!(child.node.request(true, Reason::Trigger, false));
        let decision = child.node.dioxus_key(KeyFacts {
            key: "Escape".into(),
            on_trigger: true,
            ..Default::default()
        });
        assert!(
            decision.prevent && decision.stop,
            "handled Escape cancels shared native delivery"
        );
        // Install a callback without changing registration generation or graph.
        runtime
            .0
            .borrow_mut()
            .nodes
            .get_mut("child")
            .unwrap()
            .callbacks
            .key = Some(Rc::new(|_| {
            panic!("closed Escape/Tab cannot reach child policy")
        }));
        let decision = child.node.dioxus_key(KeyFacts {
            key: "Escape".into(),
            on_trigger: true,
            ..Default::default()
        });
        assert!(!decision.stop && !decision.prevent);
        let decision = child.node.dioxus_key(KeyFacts {
            key: "Tab".into(),
            on_trigger: true,
            ..Default::default()
        });
        assert!(!decision.stop && !decision.prevent);
        assert!(
            !header.open.get() && !child.open.get(),
            "current assignment routes child Escape through its enclosing Header"
        );
        header.node.unregister();
        child.node.unregister();
        let log = Rc::new(RefCell::new(Vec::new()));
        let header = model(&runtime, "header", OverlayKind::Header, log.clone());
        let child = model(&runtime, "child", OverlayKind::Dropdown, log);
        runtime.observe(&report(
            &runtime,
            3,
            "observe",
            nodes(&runtime, &["header"]),
        ));
        assert!(header.node.request(true, Reason::Trigger, false));
        assert!(child.node.request(true, Reason::Trigger, false));
        assert!(runtime.dismiss_root("child", Reason::Escape, true));
        assert!(!header.open.get() && !child.open.get());
    }

    #[test]
    fn submenu_bound_after_parent_open_still_closes_through_its_current_parent() {
        let runtime = RuntimeHandle::new();
        let log = Rc::new(RefCell::new(Vec::new()));
        let parent = model(&runtime, "late-parent", OverlayKind::Dropdown, log.clone());
        let child = model(&runtime, "late-child", OverlayKind::Dropdown, log);
        assert!(parent.node.request(true, Reason::Trigger, false));
        child.node.set_ownership(vec![parent.node.clone()], false);
        let mut observed = report(
            &runtime,
            1,
            "observe",
            serde_json::json!([
                {"id":"late-parent","generation":1},
                {"id":"late-child","generation":1}
            ]),
        );
        runtime.observe(&observed);
        assert!(
            runtime
                .0
                .borrow()
                .registry
                .cached_children("late-parent")
                .is_empty()
        );
        observed.sequence = 2;
        child.node.set_ownership(vec![parent.node.clone()], true);
        runtime.observe(&observed);
        assert_eq!(
            runtime.0.borrow().registry.cached_children("late-parent"),
            ["late-child"]
        );
        assert!(child.node.request(true, Reason::Keyboard, false));
        runtime.close_submenu(child.node.id());
        assert!(parent.open.get(), "return keeps the enclosing menu open");
        assert!(
            !child.open.get(),
            "a newly bound child belongs to its open parent"
        );
    }

    #[test]
    fn before_recursion_aba_and_forced_configuration_reject_superseded_work() {
        let runtime = RuntimeHandle::new();
        let open = Rc::new(Cell::new(true));
        let pending = Rc::new(RefCell::new(None::<NodeHandle>));
        let edit = Rc::new(Cell::new(true));
        let force_inside = Rc::new(Cell::new(false));
        let before_force = force_inside.clone();
        let read = open.clone();
        let before_node = pending.clone();
        let before_open = open.clone();
        let before_edit = edit.clone();
        let apply = open.clone();
        let applied = Rc::new(Cell::new(0));
        let count = applied.clone();
        let node = runtime.register(
            "node".into(),
            1,
            OverlayKind::Dropdown,
            CloseBehavior::Both,
            Callbacks {
                read: Rc::new(move || Some(read.get())),
                before: Rc::new(move |_| {
                    let node = before_node.borrow().clone().unwrap();
                    assert!(
                        !node.request(false, Reason::Trigger, false),
                        "recursive before must be rejected"
                    );
                    if before_force.replace(false) {
                        assert!(node.request(false, Reason::Configuration, false));
                        return true;
                    }
                    if before_edit.replace(false) {
                        before_open.set(false);
                        node.sync_open(false);
                        before_open.set(true);
                        node.sync_open(true);
                    }
                    true
                }),
                apply: Rc::new(move |change| {
                    apply.set(change.open);
                    count.set(count.get() + 1);
                }),
                key: None,
            },
        );
        *pending.borrow_mut() = Some(node.clone());
        assert!(
            !node.request(false, Reason::Trigger, false),
            "ABA invalidates older command"
        );
        assert!(open.get() && applied.get() == 0);
        assert!(
            node.request(false, Reason::Trigger, false),
            "superseded before does not leave recursion guard stuck"
        );
        force_inside.set(true);
        assert!(
            !node.request(true, Reason::Trigger, false),
            "forced unchanged close supersedes pending opener"
        );
        assert!(!open.get());
        open.set(true);
        node.sync_open(true);
        assert!(
            node.request(false, Reason::Configuration, false),
            "configuration bypasses veto callback"
        );
        assert!(!open.get());

        let runtime = RuntimeHandle::new();
        let log = Rc::new(RefCell::new(Vec::new()));
        let parent = model(&runtime, "header", OverlayKind::Header, log.clone());
        let child_open = Rc::new(Cell::new(false));
        let read = child_open.clone();
        let apply = child_open.clone();
        let reopen = parent.node.clone();
        let child = runtime.register(
            "child".into(),
            1,
            OverlayKind::Dropdown,
            CloseBehavior::Both,
            Callbacks {
                read: Rc::new(move || Some(read.get())),
                before: Rc::new(move |change| {
                    if !change.open {
                        assert!(reopen.request(true, Reason::Trigger, false));
                    }
                    true
                }),
                apply: Rc::new(move |change| apply.set(change.open)),
                key: None,
            },
        );
        runtime.observe(&report(
            &runtime,
            1,
            "observe",
            nodes(&runtime, &["header"]),
        ));
        assert!(parent.node.request(true, Reason::Trigger, false));
        assert!(child.request(true, Reason::Trigger, false));
        log.borrow_mut().clear();
        assert!(
            !parent.node.request(false, Reason::Trigger, false),
            "unchanged reopen from child supersedes pending parent close"
        );
        assert!(parent.open.get() && !child_open.get());
        assert!(!log.borrow().contains(&"apply:header:false".into()));
    }

    struct RawForm(String);
    impl dioxus::html::HasFileData for RawForm {
        fn files(&self) -> Vec<dioxus::html::FileData> {
            Vec::new()
        }
    }
    impl dioxus::html::HasFormData for RawForm {
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
    fn event(value: serde_json::Value) -> FormEvent {
        Event::new(
            Rc::new(dioxus::html::FormData::new(RawForm(value.to_string()))),
            true,
        )
    }
    struct TypedKeyboard(Key, Modifiers);
    impl dioxus::html::ModifiersInteraction for TypedKeyboard {
        fn modifiers(&self) -> Modifiers {
            self.1
        }
    }
    impl dioxus::html::HasKeyboardData for TypedKeyboard {
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
    pub(super) fn typed_keyboard(key: &str, modifiers: Modifiers) -> KeyboardEvent {
        let key = key.parse().unwrap_or_else(|_| Key::Character(key.into()));
        Event::new(
            Rc::new(dioxus::html::KeyboardData::new(TypedKeyboard(
                key, modifiers,
            ))),
            true,
        )
    }
    #[test]
    fn dioxus_key_dispatch_uses_current_registration_without_native_listener_readiness() {
        let runtime = RuntimeHandle::new();
        let root = model(
            &runtime,
            "typed",
            OverlayKind::Dropdown,
            Rc::new(RefCell::new(Vec::new())),
        );
        root.open.set(true);
        let calls = Rc::new(Cell::new(0));
        let called = calls.clone();
        runtime
            .0
            .borrow_mut()
            .nodes
            .get_mut("typed")
            .unwrap()
            .callbacks
            .key = Some(Rc::new(move |_| {
            called.set(called.get() + 1);
            KeyDecision {
                prevent: true,
                activate: Some("registered-item".into()),
                ..Default::default()
            }
        }));
        let key = typed_keyboard("Enter", Modifiers::empty());
        let facts = root.node.dioxus_key_facts(&key, true, None, false).unwrap();
        assert!(facts.inventory.as_ref().unwrap().hosts.is_empty());
        let decision = root.node.dioxus_key(facts);
        assert!(decision.prevent);
        assert_eq!(decision.activate.as_deref(), Some("registered-item"));
        assert_eq!(calls.get(), 1);

        let prevented = typed_keyboard("Enter", Modifiers::empty());
        prevented.prevent_default();
        let facts = root
            .node
            .dioxus_key_facts(&prevented, true, None, false)
            .unwrap();
        assert!(!root.node.dioxus_key(facts).prevent);
        assert_eq!(calls.get(), 1);

        // Legacy native key reports cannot dispatch policy a second time.
        let native = report(
            &runtime,
            1,
            "key",
            serde_json::json!([
                {"id":"typed","generation":1,"on_trigger":true}
            ]),
        );
        runtime.observe(&native);
        assert_eq!(calls.get(), 1);
        let replacement = runtime.register(
            "typed".into(),
            2,
            OverlayKind::Dropdown,
            CloseBehavior::Both,
            runtime.entry("typed").unwrap().callbacks,
        );
        assert!(
            root.node
                .dioxus_key_facts(&key, true, None, false)
                .is_none()
        );
        assert!(
            !root
                .node
                .dioxus_key(KeyFacts {
                    key: "Enter".into(),
                    on_trigger: true,
                    ..Default::default()
                })
                .prevent
        );
        assert_eq!(calls.get(), 1);
        replacement.unregister();
    }

    #[test]
    fn dioxus_key_cannot_activate_after_its_callback_removes_the_owner() {
        let runtime = RuntimeHandle::new();
        let root = model(
            &runtime,
            "activation-owner",
            OverlayKind::Dropdown,
            Rc::new(RefCell::new(Vec::new())),
        );
        root.open.set(true);
        let called = runtime.clone();
        runtime
            .0
            .borrow_mut()
            .nodes
            .get_mut(root.node.id())
            .unwrap()
            .callbacks
            .key = Some(Rc::new(move |_| {
            called.unregister("activation-owner", 1);
            KeyDecision {
                prevent: true,
                activate: Some("old-control".into()),
                ..Default::default()
            }
        }));
        let decision = root.node.dioxus_key(KeyFacts {
            key: "Enter".into(),
            inside: true,
            ..Default::default()
        });
        assert!(decision.prevent && decision.activate.is_none());
    }

    fn mixed_inventory_facts() -> InventoryFacts {
        serde_json::from_value(serde_json::json!({"items_host":"actual-items-host",
            "hosts":[
                {"item":"key-a","kind":"dropdown","target":"a","capture":"key-a","registration_key":"key-a"},
                {"item":"key-b","kind":"menu","target":"b","capture":"key-b","registration_key":"key-b","tabindex_attribute":"4"}
            ],"navigation_order":["key-a","key-b"],"query_order":["key-a","key-b"]})).unwrap()
    }
    fn describe_inventory(
        runtime: &RuntimeHandle,
        root: &str,
        generation: u64,
        facts: &InventoryFacts,
    ) {
        for host in &facts.hosts {
            runtime.register_owned_item_description(
                &host.registration_key,
                Some((root.into(), generation)),
                host.clone(),
            );
        }
    }
    #[test]
    fn owned_inventory_uses_registered_members_props_and_native_order_only() {
        let runtime = RuntimeHandle::new();
        let root = model(
            &runtime,
            "owned-items",
            OverlayKind::Dropdown,
            Rc::new(RefCell::new(Vec::new())),
        );
        let raw = mixed_inventory_facts();
        describe_inventory(&runtime, "owned-items", 1, &raw);
        let mut other = raw.hosts[0].clone();
        other.registration_key = "other-owner".into();
        runtime.register_owned_item_description("other-owner", Some(("foreign".into(), 1)), other);
        runtime.input(event(serde_json::json!({"owner":runtime.owner_id(),"sequence":1,"kind":"observe",
            "nodes":[{"id":"owned-items","generation":1,
                "item_order":["unknown-native-control","key-b","key-b","other-owner"],
                "inventory":{"items_host":"untrusted","hosts":[{"item":"native","kind":"dropdown","target":"native","capture":"native"}],
                "navigation_order":["native"],"query_order":["native"]}}]})));
        let inventory = root.node.inventory().unwrap();
        assert_eq!(
            inventory.query_order,
            [ItemRef("key-b".into()), ItemRef("key-a".into())]
        );
        assert_eq!(inventory.hosts.len(), 2);
        assert!(inventory.slots.is_none());
        let mut changed = raw.hosts[0].clone();
        changed.disabled_attribute = true;
        changed.checked_attribute = true;
        changed.target = "updated-rust-target".into();
        runtime.register_owned_item_description("key-a", Some(("owned-items".into(), 1)), changed);
        let inventory = root.node.inventory().unwrap();
        let updated = inventory
            .hosts
            .iter()
            .find(|host| host.registration_key == "key-a")
            .unwrap();
        assert!(updated.disabled_attribute && updated.checked_attribute);
        assert_eq!(updated.target, "updated-rust-target");
        assert_eq!(updated.item.0, "key-a");
        runtime.unregister_item_cue("key-b");
        assert_eq!(root.node.inventory().unwrap().hosts.len(), 1);
        root.node.unregister();
        assert!(root.node.inventory().is_none());
        let replacement = runtime.register(
            "owned-items".into(),
            2,
            OverlayKind::Dropdown,
            CloseBehavior::None,
            Callbacks {
                read: Rc::new(|| Some(false)),
                before: Rc::new(|_| true),
                apply: Rc::new(|_| {}),
                key: None,
            },
        );
        // An old-generation publisher cannot enroll an item in a replacement root.
        describe_inventory(&runtime, "owned-items", 1, &raw);
        assert!(replacement.inventory().unwrap().hosts.is_empty());
    }

    #[test]
    fn description_prop_updates_preserve_completed_cue_and_identity_replacement_clears_it() {
        let runtime = RuntimeHandle::new();
        let _root = model(
            &runtime,
            "description-cue",
            OverlayKind::Dropdown,
            Rc::new(RefCell::new(Vec::new())),
        );
        let mut host = mixed_inventory_facts().hosts.remove(0);
        runtime.register_owned_item_description(
            "key-a",
            Some(("description-cue".into(), 1)),
            host.clone(),
        );
        install_item_mock(&runtime, &host, ItemMountMock::default());
        runtime.0.borrow_mut().keyboard = true;
        runtime.focus_registered_item("key-a", true, None);
        assert_eq!(runtime.item_visual_key().as_deref(), Some("key-a"));
        host.checked_attribute = true;
        host.tabindex_attribute = Some("0".into());
        runtime.register_owned_item_description(
            "key-a",
            Some(("description-cue".into(), 1)),
            host.clone(),
        );
        assert!(runtime.item_visual_matches(&host));
        assert!(
            runtime
                .0
                .borrow()
                .item_visual_host
                .as_ref()
                .unwrap()
                .checked_attribute
        );
        host.hidden_attribute = true;
        runtime.register_owned_item_description(
            "key-a",
            Some(("description-cue".into(), 1)),
            host.clone(),
        );
        assert!(
            runtime.item_visual_matches(&host),
            "inline panel closure preserves a completed cue"
        );
        host.target = "replacement-control".into();
        runtime.register_owned_item_description("key-a", Some(("description-cue".into(), 1)), host);
        assert_eq!(runtime.item_visual_key(), None);
    }

    fn document_value(
        runtime: &RuntimeHandle,
        sequence: u64,
        document: &str,
        payload: serde_json::Value,
    ) -> serde_json::Value {
        serde_json::json!({"owner":runtime.owner_id(),"sequence":sequence,"document":document,
            "kind":"document","document_event":payload})
    }
    #[test]
    fn document_input_keeps_snapshots_and_clears_each_broadcast_owners_cue() {
        let first = RuntimeHandle::new();
        let second = RuntimeHandle::new();
        let log = Rc::new(RefCell::new(Vec::new()));
        let _root = model(&first, "document-root", OverlayKind::Dropdown, log.clone());
        let _child = model(&first, "document-child", OverlayKind::Dropdown, log.clone());
        let _peer = model(&second, "document-peer", OverlayKind::Dropdown, log);
        assign(&first, "document-child", &["document-root"], true);
        let document = "document snapshot and local cue broadcast";
        let raw = mixed_inventory_facts();
        describe_inventory(&first, "document-root", 1, &raw);
        describe_inventory(&second, "document-peer", 1, &raw);
        first.input(event(serde_json::json!({"owner":first.owner_id(),"sequence":1,"document":document,"kind":"observe",
            "nodes":[{"id":"document-root","generation":1,
                "item_order":["key-a","key-b"]},
                {"id":"document-child","generation":1}]})));
        second.input(event(serde_json::json!({"owner":second.owner_id(),"sequence":1,"document":document,"kind":"observe",
            "nodes":[{"id":"document-peer","generation":1,"item_order":["key-a","key-b"]}]})));
        let snapshots = {
            let state = first.0.borrow();
            state.item_descriptions.clone()
        };
        for runtime in [&first, &second] {
            install_item_mock(runtime, &raw.hosts[0], ItemMountMock::default());
            runtime.input(event(document_value(
                runtime,
                2,
                document,
                serde_json::json!({
                "sequence":1,"action":"input","input_kind":"keydown","key":"Tab"}),
            )));
            runtime.focus_registered_item("key-a", true, None);
            assert_eq!(runtime.item_visual_key().as_deref(), Some("key-a"));
        }
        for runtime in [&first, &second] {
            runtime.input(event(document_value(
                runtime,
                3,
                document,
                serde_json::json!({
                "sequence":2,"action":"input","input_kind":"mousedown"}),
            )));
            assert_eq!(runtime.item_visual_key(), None);
            assert!(!runtime.keyboard_mode() && !runtime.0.borrow().keyboard);
        }
        first.input(event(document_value(
            &first,
            4,
            document,
            serde_json::json!({
            "sequence":3,"action":"input","input_kind":"keydown","key":"Tab"}),
        )));
        first.focus_registered_item("key-a", true, None);
        first.input(event(document_value(
            &first,
            99,
            document,
            serde_json::json!({
            "sequence":2,"action":"input","input_kind":"touchstart"}),
        )));
        assert_eq!(
            first.item_visual_key().as_deref(),
            Some("key-a"),
            "older input cannot clear a newer cue"
        );
        let state = first.0.borrow();
        assert_eq!(
            state.sequence, 4,
            "stale shared IDs do not consume the owner watermark"
        );
        assert_eq!(state.observation.as_ref().unwrap().sequence, 1);
        assert_eq!(snapshots, state.item_descriptions.clone());
        drop(state);
        assert_eq!(
            first
                .domain()
                .unwrap()
                .cached_parent("document-child")
                .as_deref(),
            Some("document-root")
        );
    }

    #[test]
    fn document_auth_and_schema_rejection_cannot_consume_or_handoff_observations() {
        let runtime = RuntimeHandle::new();
        let _root = model(
            &runtime,
            "document-auth",
            OverlayKind::Dropdown,
            Rc::new(RefCell::new(Vec::new())),
        );
        let document = "document authentication";
        runtime.input(event(serde_json::json!({"owner":runtime.owner_id(),"sequence":1,"document":document,"kind":"observe",
            "nodes":[{"id":"document-auth","generation":1}]})));
        let valid = document_value(
            &runtime,
            99,
            document,
            serde_json::json!({
            "sequence":1,"action":"input","input_kind":"keydown","key":"Tab"}),
        );
        let mut wrong_owner = valid.clone();
        wrong_owner["owner"] = "foreign-owner".into();
        let mut wrong_document = valid.clone();
        wrong_document["document"] = "another-document".into();
        let mut zero = valid.clone();
        zero["document_event"]["sequence"] = 0.into();
        let mut invalid_action = valid.clone();
        invalid_action["document_event"]["action"] = "unsupported".into();
        let mut invalid_input = valid.clone();
        invalid_input["document_event"]["input_kind"] = "scroll".into();
        let mut stale_owner = valid.clone();
        stale_owner["sequence"] = 1.into();
        for rejected in [
            wrong_owner,
            wrong_document,
            zero,
            invalid_action,
            invalid_input,
            stale_owner,
        ] {
            runtime.input(event(rejected));
            let state = runtime.0.borrow();
            assert_eq!(state.sequence, 1);
            assert_eq!(state.document, document);
            assert_eq!(state.observation.as_ref().unwrap().sequence, 1);
            assert!(state.nodes.contains_key("document-auth"));
        }
        let mut accepted = valid;
        accepted["sequence"] = 2.into();
        runtime.input(event(accepted));
        assert_eq!(runtime.document_keyboard_mode(), Some(true));
        runtime.input(event(serde_json::json!({"owner":runtime.owner_id(),"sequence":3,"document":"authorized next document","kind":"observe",
            "nodes":[{"id":"document-auth","generation":1}]})));
        runtime.input(event(document_value(
            &runtime,
            100,
            document,
            serde_json::json!({"sequence":100,"action":"escape"}),
        )));
        assert_eq!(runtime.0.borrow().sequence, 3);
        assert_eq!(runtime.0.borrow().document, "authorized next document");
    }
    #[derive(Clone, Default)]
    struct PublicationFixture {
        runtime: Rc<RefCell<Option<RuntimeHandle>>>,
        visible: Rc<Cell<Option<Signal<bool>>>>,
    }
    impl PartialEq for PublicationFixture {
        fn eq(&self, other: &Self) -> bool {
            Rc::ptr_eq(&self.runtime, &other.runtime)
        }
    }
    fn publication_root(fixture: PublicationFixture) -> Element {
        let runtime = use_overlay_runtime();
        *fixture.runtime.borrow_mut() = Some(runtime.clone());
        let visible = use_signal(|| true);
        fixture.visible.set(Some(visible));
        rsx! {
            if visible() {
                crate::Dropdown { label: "Publication", suppress_trigger_visibility_check: true,
                    crate::DropdownItem { id: "publication-dropdown", label: "Dropdown" }
                    crate::SelectItem { id: "publication-select", value: "select", label: "Select", class: "consumer-publication-select" }
                    crate::MenuItem { id: "publication-menu", label: "Menu" }
                    crate::MenuCategory { id: "publication-category", label: "Category", span { "Child" } }
                }
            }
            OverlayTransport { runtime }
        }
    }
    struct PublicationConverter;
    macro_rules! publication_converter {
        (enum Event { $(#[convert = $convert:ident] #[events = [$($events:tt)*]] $(#[raw = [$($raw:ident),* $(,)?]])? $group:ident($data:ident),)* }) => {
            impl dioxus::html::HtmlEventConverter for PublicationConverter {
                $(fn $convert(&self,event:&dioxus::html::PlatformEventData)->dioxus::html::$data {let _=event;publication_converter!(@data $group,event)})*
            }
        };
        (@data Form,$event:ident)=>{dioxus::html::FormData::new(RawForm($event.downcast::<String>().unwrap().clone()))};
        (@data $other:ident,$event:ident)=>{panic!("Unexpected publication test event")};
    }
    dioxus::html::with_html_event_groups!(publication_converter);
    fn publication_dispatch(
        dom: &VirtualDom,
        transport: dioxus::core::ElementId,
        name: &str,
        value: serde_json::Value,
    ) -> bool {
        let data: Rc<dyn std::any::Any> = Rc::new(dioxus::html::PlatformEventData::new(Box::new(
            value.to_string(),
        )));
        let original = Event::new(data, true);
        dom.runtime()
            .handle_event(name, original.clone(), transport);
        !original.default_action_enabled()
    }
    fn publication_attr(tag: &str, name: &str) -> Option<String> {
        let rest = tag.split_once(&format!(" {name}="))?.1;
        if let Some(quoted) = rest.strip_prefix('"') {
            Some(quoted.split('"').next()?.to_string())
        } else {
            Some(rest.split([' ', '>']).next()?.to_string())
        }
    }
    fn publication_tag<'a>(html: &'a str, id: &str) -> &'a str {
        html.split('>')
            .find(|tag| publication_attr(tag, "id").as_deref() == Some(id))
            .unwrap_or_else(|| panic!("Missing rendered source host {id}"))
    }
    fn publication_category_parent(html: &str) -> &str {
        html.split('>')
            .find(|tag| {
                publication_attr(tag, "data-ix-item-part").as_deref() == Some("category-parent")
            })
            .expect("rendered Category parent control")
    }
    fn publication_cues(html: &str, expected: Option<&str>, category_active: bool) {
        assert!(
            publication_attr(publication_tag(html, "publication-select"), "class").is_some_and(
                |class| class
                    .split_whitespace()
                    .any(|name| name == "consumer-publication-select")
            ),
            "consumer class survives Dioxus cue updates"
        );
        for id in [
            "publication-dropdown",
            "publication-select",
            "publication-menu",
        ] {
            assert_eq!(
                publication_attr(publication_tag(html, id), "data-active").as_deref(),
                Some(if expected == Some(id) {
                    "true"
                } else {
                    "false"
                }),
                "rendered cue on {id}, expected {expected:?}"
            );
            assert_eq!(
                publication_attr(publication_tag(html, id), "class")
                    .is_some_and(|class| class.split_whitespace().any(|name| name == "ix-focused")),
                expected == Some(id),
                "Dioxus-rendered focus class on {id}"
            );
        }
        assert_eq!(
            publication_attr(publication_tag(html, "publication-category"), "data-active"),
            None,
            "Category's outer host never gains the private parent's marker"
        );
        let select_inner = html
            .split('>')
            .find(|tag| {
                publication_attr(tag, "class")
                    .is_some_and(|class| class.contains("ix-select-item-option"))
            })
            .unwrap();
        assert_eq!(
            publication_attr(select_inner, "data-active").as_deref(),
            Some(if expected == Some("publication-select") {
                "true"
            } else {
                "false"
            }),
            "Select's generated inner cue follows its source host"
        );
        assert_eq!(
            publication_attr(select_inner, "class")
                .is_some_and(|class| class.split_whitespace().any(|name| name == "ix-focused")),
            expected == Some("publication-select")
        );
        let category_parent = publication_category_parent(html);
        assert_eq!(
            publication_attr(category_parent, "data-active").as_deref(),
            Some(if category_active { "true" } else { "false" })
        );
        assert_eq!(
            publication_attr(category_parent, "class")
                .is_some_and(|class| class.split_whitespace().any(|name| name == "ix-focused")),
            category_active,
            "Category cue belongs to its physical parent control"
        );
        let root = html
            .split('>')
            .find(|tag| {
                publication_attr(tag, "aria-label").as_deref() == Some("Publication")
                    && publication_attr(tag, "data-ix-overlay-root").is_some()
            })
            .unwrap();
        assert_eq!(
            publication_attr(root, "aria-activedescendant").as_deref(),
            expected
        );
    }
    #[test]
    fn actual_vdom_item_effect_publishes_mixed_host_cues_without_another_input() {
        let _guard = crate::test_event_converter_guard();
        dioxus::html::set_event_converter(Box::new(PublicationConverter));
        let fixture = PublicationFixture::default();
        let mut dom = VirtualDom::new_with_props(publication_root, fixture.clone());
        let edits = dom.rebuild_to_vec();
        let runtime = fixture.runtime.borrow().clone().unwrap();
        assert!(
            runtime.0.borrow().item_visual_signal.is_some(),
            "real VDOM signal path"
        );
        let mut current = dioxus::core::ElementId::ROOT;
        let mut transport = None;
        for edit in edits.edits {
            match edit {
                dioxus::core::Mutation::SetId { id } | dioxus::core::Mutation::PushId { id } => {
                    current = id
                }
                dioxus::core::Mutation::SetAttribute {
                    name,
                    value: dioxus::core::AttributeValue::Text(value),
                    ..
                } if name == "data-ix-overlay-transport" && value == runtime.owner_id() => {
                    transport = Some(current)
                }
                _ => {}
            }
        }
        let transport = transport.expect("actual rendered overlay transport");
        let initial = dioxus_ssr::render(&dom);
        let root_tag = initial
            .split('>')
            .find(|tag| {
                publication_attr(tag, "aria-label").as_deref() == Some("Publication")
                    && publication_attr(tag, "data-ix-overlay-root").is_some()
            })
            .unwrap();
        let root = publication_attr(root_tag, "data-ix-overlay-root").unwrap();
        let generation: u64 = publication_attr(root_tag, "data-ix-overlay-generation")
            .unwrap()
            .parse()
            .unwrap();
        let node = NodeHandle {
            runtime: Rc::downgrade(&runtime.0),
            id: root.clone(),
            generation,
        };
        dom.in_runtime(|| assert!(node.request(true, Reason::Trigger, false)));
        for _ in 0..4 {
            dom.render_immediate_to_vec();
        }
        let html = dioxus_ssr::render(&dom);
        let hosts = [
            ("publication-dropdown", "dropdown"),
            ("publication-select", "select"),
            ("publication-menu", "menu"),
            ("publication-category", "category"),
        ].into_iter().map(|(id,kind)| {
            let tag = publication_tag(&html,id);
            serde_json::json!({"item":publication_attr(tag,"data-ix-item-key").unwrap(),"capture":publication_attr(tag,"data-ix-item-key").unwrap(),
                "kind":kind,"target":id,"registration_key":publication_attr(tag,"data-ix-item-key").unwrap(),
                "source_focusable":kind != "category",
                "parts":if kind == "category" { vec![serde_json::json!({
                    "kind":"category-parent", "capture":publication_attr(publication_category_parent(&html),"id").unwrap(),
                    "source_focusable":true
                })] } else { Vec::new() }})
        }).collect::<Vec<_>>();
        publication_cues(&html, None, false);
        let order = hosts
            .iter()
            .map(|host| host["item"].clone())
            .collect::<Vec<_>>();
        for host in &hosts {
            let facts: HostFacts = serde_json::from_value(host.clone()).unwrap();
            install_item_mock(&runtime, &facts, ItemMountMock::default());
            runtime.register_owned_item_description(
                &facts.registration_key,
                Some((root.clone(), generation)),
                facts.clone(),
            );
        }
        let report = serde_json::json!({"owner":runtime.owner_id(),"sequence":1,
            "kind":"observe",
            "nodes":[{"id":root,"generation":generation,
                "mode":"active-descendant","inside":true,"on_trigger":true,
                "item_order":order}]});
        assert!(!publication_dispatch(&dom, transport, "input", report));
        let node = NodeHandle {
            runtime: Rc::downgrade(&runtime.0),
            id: root.clone(),
            generation,
        };
        let key = typed_keyboard("Home", Modifiers::empty());
        let facts = node.dioxus_key_facts(&key, true, None, false).unwrap();
        assert!(node.dioxus_key(facts).prevent);
        assert_eq!(runtime.pending_item_focus_count(), 1);
        settle_item_effects(&mut dom);
        assert_eq!(runtime.pending_item_focus_count(), 0);
        publication_cues(
            &dioxus_ssr::render(&dom),
            Some("publication-dropdown"),
            false,
        );
        // Publish through real owner effects and mounted methods, without a
        // bridge receipt or a later keyboard/focusin event.
        for id in [
            "publication-select",
            "publication-menu",
            "publication-category",
        ] {
            let facts: HostFacts = serde_json::from_value(
                hosts
                    .iter()
                    .find(|host| host["target"] == id)
                    .unwrap()
                    .clone(),
            )
            .unwrap();
            dom.in_runtime(|| assert!(node.queue_item_host_focus(&facts, Rc::new(|| {}))));
            assert_eq!(runtime.pending_item_focus_count(), 1);
            for _ in 0..4 {
                dom.render_immediate_to_vec();
            }
            publication_cues(
                &dioxus_ssr::render(&dom),
                (id != "publication-category").then_some(id),
                id == "publication-category",
            );
        }
        let facts: HostFacts = serde_json::from_value(hosts[1].clone()).unwrap();
        let called = Rc::new(Cell::new(false));
        let callback = called.clone();
        dom.in_runtime(|| {
            assert!(node.queue_item_host_focus(&facts, Rc::new(move || callback.set(true))))
        });
        dom.in_runtime(|| fixture.visible.get().unwrap().set(false));
        for _ in 0..4 {
            dom.render_immediate_to_vec();
        }
        assert!(!called.get());
        assert!(runtime.0.borrow().owned_item_focus.is_empty());
        assert!(!dioxus_ssr::render(&dom).contains("publication-dropdown"));
    }
    #[test]
    fn registered_mixed_inventory_uses_latest_roving_state_before_a_render_commit() {
        let runtime = RuntimeHandle::new();
        let root = model(
            &runtime,
            "mixed-root",
            OverlayKind::Dropdown,
            Rc::new(RefCell::new(Vec::new())),
        );
        assert!(root.node.request(true, Reason::Trigger, false));
        assert!(root.node.set_navigation_mode("roving-tabindex"));
        let selected = root.node.clone();
        runtime
            .0
            .borrow_mut()
            .nodes
            .get_mut("mixed-root")
            .unwrap()
            .callbacks
            .key = Some(Rc::new(move |facts| {
            let raw = facts.inventory.clone().unwrap();
            let inventory = crate::dropdown_inventory::Inventory::new(
                crate::dropdown_inventory::InventoryMode::RovingTabindex,
                raw.clone(),
            );
            let target = inventory
                .navigate(&facts.key, inventory.active().as_ref())
                .unwrap();
            assert!(selected.select_roving(&raw, &target));
            KeyDecision {
                prevent: true,
                ..KeyDecision::default()
            }
        }));
        let raw = mixed_inventory_facts();
        for host in &raw.hosts {
            install_item_mock(&runtime, host, ItemMountMock::default());
        }
        describe_inventory(&runtime, "mixed-root", 1, &raw);
        for (sequence, key) in [(1, "End"), (2, "ArrowUp")] {
            let input = event(
                serde_json::json!({"owner":runtime.owner_id(),"sequence":sequence,"kind":"observe",
                "hosts":["mixed-root"],"nodes":[{"id":"mixed-root","generation":1,
                    "mode":"roving-tabindex","inside":true,"item_order":["key-a","key-b"]}]}),
            );
            runtime.input(input);
            let key = typed_keyboard(key, Modifiers::empty());
            let facts = root.node.dioxus_key_facts(&key, false, None, true).unwrap();
            assert!(root.node.dioxus_key(facts).prevent);
        }
        assert_eq!(runtime.item_tabindex("key-a"), Some(0));
        assert_eq!(runtime.item_tabindex("key-b"), Some(-1));
        let logical = root.node.inventory().unwrap();
        assert_eq!(logical.hosts[0].tabindex_attribute.as_deref(), Some("0"));
        assert_eq!(logical.hosts[1].tabindex_attribute.as_deref(), Some("-1"));
        assert_eq!(
            raw.hosts[0].tabindex_attribute, None,
            "the authored Rust description remains unchanged"
        );
        assert_eq!(raw.hosts[1].tabindex_attribute.as_deref(), Some("4"));
        assert!(root.node.request(false, Reason::Trigger, false));
        assert_eq!(runtime.item_tabindex("key-a"), None);
        assert_eq!(runtime.item_tabindex("key-b"), None);
    }

    #[test]
    fn registered_focus_uses_typed_live_control_and_keyboard_modality() {
        let runtime = RuntimeHandle::new();
        let raw = mixed_inventory_facts();
        let host = &raw.hosts[0];
        runtime.register_item_cue(host.registration_key.clone(), host.kind);
        runtime.attach_document("mixed-visual-runtime-test");
        runtime.input(event(
            serde_json::json!({"owner":runtime.owner_id(),"sequence":1,
            "document":"mixed-visual-runtime-test","kind":"document",
            "document_event":{"sequence":1,"action":"input","input_kind":"keydown","key":"Tab"}}),
        ));
        runtime.focus_registered_item(&host.registration_key, true, None);
        assert_eq!(
            runtime.item_visual_key(),
            None,
            "kind registration without a mounted control cannot focus"
        );
        install_item_mock(&runtime, host, ItemMountMock::default());
        runtime.focus_registered_item(&host.registration_key, true, None);
        assert_eq!(runtime.item_visual_key().as_deref(), Some("key-a"));
        let mut copy = host.clone();
        copy.capture = "another-native-capture".into();
        assert!(runtime.item_visual_matches(&copy));
        // Reusing a typed registration cannot silently change its kind.
        runtime.register_item_cue(host.registration_key.clone(), ItemKind::Select);
        copy.kind = ItemKind::Select;
        assert!(!runtime.item_visual_matches(&copy));
        runtime.focus_registered_item(&host.registration_key, false, None);
        assert_eq!(runtime.item_visual_key(), None);
        runtime.input(event(
            serde_json::json!({"owner":runtime.owner_id(),"sequence":2,
            "document":"mixed-visual-runtime-test","kind":"document",
            "document_event":{"sequence":2,"action":"input","input_kind":"mousedown"}}),
        ));
        runtime.focus_registered_item(&host.registration_key, true, None);
        assert_eq!(runtime.item_visual_key(), None);
    }

    #[derive(Clone, Default)]
    pub(super) struct ItemMountMock {
        events: Rc<RefCell<Vec<(&'static str, ScopeId)>>>,
        pub(super) pending: Rc<Cell<bool>>,
        pub(super) waker: Rc<RefCell<Option<std::task::Waker>>>,
        fail_focus: bool,
    }
    impl dioxus::html::RenderedElementBacking for ItemMountMock {
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
            Box::pin(async move {
                mock.events.borrow_mut().push(("focus", current_scope_id()));
                std::future::poll_fn(|cx| {
                    if mock.pending.get() {
                        *mock.waker.borrow_mut() = Some(cx.waker().clone());
                        std::task::Poll::Pending
                    } else {
                        std::task::Poll::Ready(())
                    }
                })
                .await;
                if mock.fail_focus {
                    Err(dioxus::html::MountedError::NotSupported)
                } else {
                    Ok(())
                }
            })
        }
        fn scroll_to(
            &self,
            options: dioxus::html::ScrollToOptions,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = dioxus::html::MountedResult<()>>>>
        {
            assert_eq!(options.behavior, dioxus::html::ScrollBehavior::Instant);
            assert!(matches!(
                options.vertical,
                dioxus::html::ScrollLogicalPosition::Nearest
            ));
            assert!(matches!(
                options.horizontal,
                dioxus::html::ScrollLogicalPosition::Nearest
            ));
            let events = self.events.clone();
            Box::pin(async move {
                events.borrow_mut().push(("scroll", current_scope_id()));
                Ok(())
            })
        }
    }
    pub(super) fn settle_item_effects(dom: &mut VirtualDom) {
        for _ in 0..4 {
            dom.render_immediate_to_vec();
        }
    }
    pub(super) fn install_item_mock(
        runtime: &RuntimeHandle,
        host: &HostFacts,
        mock: ItemMountMock,
    ) -> Weak<MountedData> {
        runtime.register_item_cue(host.registration_key.clone(), host.kind);
        let node = Rc::new(MountedData::new(mock));
        let weak = Rc::downgrade(&node);
        runtime.register_item_mount(&host.registration_key, &host.target, node);
        weak
    }
    #[test]
    fn tab_boundary_uses_registered_physical_control_order_and_skips_ineligible_hosts() {
        let fixture = RequestTaskFixture(Rc::new(RequestTaskState::default()));
        let mut dom = VirtualDom::new_with_props(request_task_root, fixture.clone());
        dom.rebuild_in_place();
        let node = fixture.0.node.borrow().clone().unwrap();
        let runtime = node.runtime().unwrap();
        let owner = fixture.0.owner.get().unwrap();
        let mut raw = mixed_inventory_facts();
        raw.hosts[0].kind = ItemKind::Menu;
        raw.hosts[1].kind = ItemKind::Category;
        raw.hosts[1].source_focusable = false;
        for (item, disabled, hidden) in [
            ("custom", false, false),
            ("disabled", true, false),
            ("hidden", false, true),
            ("unregistered", false, false),
        ] {
            let mut host = raw.hosts[0].clone();
            host.item = ItemRef(item.into());
            host.target = item.into();
            host.capture = item.into();
            host.registration_key = format!("key-{item}");
            host.kind = ItemKind::NativeOptIn;
            host.roving_opt_in = true;
            host.disabled_attribute = disabled;
            host.hidden_attribute = hidden;
            raw.hosts.push(host);
        }
        let menu = ItemMountMock::default();
        let category = ItemMountMock::default();
        let custom = ItemMountMock::default();
        for (host, focus_id, mock) in [
            (&raw.hosts[0], "a-control", menu.clone()),
            (&raw.hosts[1], "b-parent", category.clone()),
            (&raw.hosts[2], "custom", custom.clone()),
            (&raw.hosts[3], "disabled", ItemMountMock::default()),
            (&raw.hosts[4], "hidden", ItemMountMock::default()),
        ] {
            runtime.register_item_cue(host.registration_key.clone(), host.kind);
            runtime.register_item_control_mount(
                &host.registration_key,
                &host.target,
                focus_id,
                Rc::new(MountedData::new(mock)),
            );
        }
        // A registered element outside the current inventory is not a target.
        runtime.register_item_mount(
            "orphan",
            "orphan",
            Rc::new(MountedData::new(ItemMountMock::default())),
        );
        describe_inventory(&runtime, &node.id, node.generation, &raw);
        dom.in_scope(fixture.0.owner.get().unwrap(), || {
            runtime.input(event(
                serde_json::json!({"owner":runtime.owner_id(),"sequence":1,"kind":"observe",
            "nodes":[{"id":node.id,"generation":1,"item_order":raw.hosts.iter().map(|host| host.registration_key.clone()).collect::<Vec<_>>()}]}),
            ))
        });
        let ids = [
            "unregistered",
            "disabled",
            "hidden",
            "orphan",
            "b-parent",
            "custom",
            "a-control",
        ]
        .map(str::to_string);
        assert!(node.queue_tab_boundary_focus(&ids, false, Rc::new(|| {})));
        settle_item_effects(&mut dom);
        assert_eq!(
            *category.events.borrow(),
            [("focus", owner), ("scroll", owner)]
        );
        assert!(menu.events.borrow().is_empty());
        assert!(node.queue_tab_boundary_focus(&ids, true, Rc::new(|| {})));
        settle_item_effects(&mut dom);
        assert_eq!(*menu.events.borrow(), [("focus", owner), ("scroll", owner)]);
        assert!(node.queue_tab_boundary_focus(&["custom".into()], false, Rc::new(|| {})));
        settle_item_effects(&mut dom);
        assert_eq!(
            *custom.events.borrow(),
            [("focus", owner), ("scroll", owner)]
        );
        assert!(!node.queue_tab_boundary_focus(
            &["a".into(), "b".into()],
            false,
            Rc::new(|| panic!("logical IDs are not physical candidates"))
        ));
    }

    #[test]
    fn tab_boundary_metadata_replacement_dismissal_and_owner_drop_cancel_pending_work() {
        let fixture = RequestTaskFixture(Rc::new(RequestTaskState::default()));
        let mut dom = VirtualDom::new_with_props(request_task_root, fixture.clone());
        dom.rebuild_in_place();
        let node = fixture.0.node.borrow().clone().unwrap();
        let runtime = node.runtime().unwrap();
        let raw = mixed_inventory_facts();
        let host = raw.hosts[0].clone();
        let mock = ItemMountMock::default();
        let mounted = Rc::new(MountedData::new(mock.clone()));
        runtime.register_item_cue(host.registration_key.clone(), host.kind);
        runtime.register_item_control_mount(
            &host.registration_key,
            &host.target,
            "old-control",
            mounted.clone(),
        );
        describe_inventory(&runtime, &node.id, node.generation, &raw);
        dom.in_scope(fixture.0.owner.get().unwrap(), || {
            runtime.input(event(
                serde_json::json!({"owner":runtime.owner_id(),"sequence":1,"kind":"observe",
            "nodes":[{"id":node.id,"generation":1,"item_order":raw.hosts.iter().map(|host| host.registration_key.clone()).collect::<Vec<_>>()}]}),
            ))
        });
        let captured = Rc::new(());
        let released = Rc::downgrade(&captured);
        assert!(node.queue_tab_boundary_focus(
            &["old-control".into()],
            false,
            Rc::new(move || {
                let _ = &captured;
                panic!("replaced physical metadata");
            })
        ));
        runtime.register_item_control_mount(
            &host.registration_key,
            &host.target,
            "new-control",
            mounted,
        );
        assert!(released.upgrade().is_none());
        assert!(!node.queue_tab_boundary_focus(&["old-control".into()], false, Rc::new(|| {})));
        settle_item_effects(&mut dom);
        assert!(mock.events.borrow().is_empty());
        assert!(node.queue_tab_boundary_focus(
            &["new-control".into()],
            false,
            Rc::new(|| panic!("dismissed owner"))
        ));
        assert!(node.request(false, Reason::Trigger, false));
        settle_item_effects(&mut dom);
        assert!(mock.events.borrow().is_empty());
        assert!(!node.queue_tab_boundary_focus(&["new-control".into()], false, Rc::new(|| {})));
        fixture.0.open.set(true);
        let stale = NodeHandle {
            generation: node.generation + 1,
            ..node.clone()
        };
        assert!(!stale.queue_tab_boundary_focus(&["new-control".into()], false, Rc::new(|| {})));
        assert!(node.queue_tab_boundary_focus(
            &["new-control".into()],
            false,
            Rc::new(|| panic!("removed owner"))
        ));
        node.unregister();
        settle_item_effects(&mut dom);
        assert!(mock.events.borrow().is_empty());
        assert_eq!(runtime.pending_item_focus_count(), 0);
    }

    #[test]
    fn key_facts_preserve_existing_native_focusable_candidate_order() {
        let runtime = RuntimeHandle::new();
        let root = model(
            &runtime,
            "tab-report",
            OverlayKind::Dropdown,
            Rc::new(RefCell::new(Vec::new())),
        );
        root.open.set(true);
        let delivered = Rc::new(RefCell::new(Vec::new()));
        let captured = delivered.clone();
        runtime
            .0
            .borrow_mut()
            .nodes
            .get_mut("tab-report")
            .unwrap()
            .callbacks
            .key = Some(Rc::new(move |facts| {
            *captured.borrow_mut() = facts.focusable_ids.clone();
            KeyDecision::default()
        }));
        let mut report = report(
            &runtime,
            1,
            "key",
            serde_json::json!([
                {"id":"tab-report","generation":1,"inside":true,"on_trigger":true,
                    "focusable_ids":["b-parent","custom","a-control"]}
            ]),
        );
        report.kind = "observe".into();
        runtime.observe(&report);
        let keyboard = typed_keyboard("Tab", Modifiers::SHIFT);
        let facts = root
            .node
            .dioxus_key_facts(&keyboard, false, Some("custom".into()), true)
            .unwrap();
        assert_eq!(facts.focused_index, Some(1));
        assert!(facts.shift && !facts.on_trigger && facts.inside);
        root.node.dioxus_key(facts);
        assert_eq!(*delivered.borrow(), ["b-parent", "custom", "a-control"]);
    }

    #[test]
    fn mounted_item_effect_uses_latest_intent_owner_focus_and_nearest_scroll_even_on_focus_error() {
        let fixture = RequestTaskFixture(Rc::new(RequestTaskState::default()));
        let mut dom = VirtualDom::new_with_props(request_task_root, fixture.clone());
        dom.rebuild_in_place();
        let node = fixture.0.node.borrow().clone().unwrap();
        let runtime = node.runtime().unwrap();
        let owner = fixture.0.owner.get().unwrap();
        let caller = fixture.0.caller.get().unwrap();
        let hosts = mixed_inventory_facts().hosts;
        let first = ItemMountMock::default();
        let second = ItemMountMock {
            fail_focus: true,
            ..Default::default()
        };
        install_item_mock(&runtime, &hosts[0], first.clone());
        install_item_mock(&runtime, &hosts[1], second.clone());
        runtime.0.borrow_mut().keyboard = true;
        let released = Rc::new(());
        let weak_callback = Rc::downgrade(&released);
        dom.in_scope(caller, || {
            assert!(node.queue_item_host_focus(
                &hosts[0],
                Rc::new(move || {
                    let _ = &released;
                    panic!("superseded");
                })
            ));
            assert!(node.queue_item_host_focus(
                &hosts[1],
                Rc::new(move || {
                    assert_eq!(current_scope_id(), owner);
                    assert_eq!(consume_context::<RequestTaskContext>().0, "owner");
                })
            ));
        });
        assert!(
            weak_callback.upgrade().is_none(),
            "latest intent releases superseded callbacks"
        );
        assert_eq!(
            runtime.pending_item_focus_count(),
            1,
            "latest mounted intent is the only pending operation"
        );
        assert!(
            second.events.borrow().is_empty(),
            "admission is asynchronous"
        );
        dom.in_scope(ScopeId::APP, || {
            fixture.0.show_caller.get().unwrap().set(false)
        });
        settle_item_effects(&mut dom);
        assert!(fixture.0.caller_dropped.get());
        assert!(first.events.borrow().is_empty());
        assert_eq!(
            *second.events.borrow(),
            [("focus", owner), ("scroll", owner)]
        );
        assert_eq!(runtime.item_visual_key().as_deref(), Some("key-b"));
        assert!(runtime.0.borrow().owned_item_focus.is_empty());
        assert!(node.queue_item_focus(hosts[0].target.clone(), false, Rc::new(|| {})));
        settle_item_effects(&mut dom);
        assert_eq!(
            *first.events.borrow(),
            [("scroll", owner)],
            "active descendant navigation can scroll without native focus"
        );
    }
    #[test]
    fn mounted_item_unmount_replacement_close_and_generation_cancel_pending_work() {
        let fixture = RequestTaskFixture(Rc::new(RequestTaskState::default()));
        let mut dom = VirtualDom::new_with_props(request_task_root, fixture.clone());
        dom.rebuild_in_place();
        let node = fixture.0.node.borrow().clone().unwrap();
        let runtime = node.runtime().unwrap();
        let host = mixed_inventory_facts().hosts[0].clone();
        let pending = ItemMountMock::default();
        pending.pending.set(true);
        let weak_mount = install_item_mock(&runtime, &host, pending.clone());
        assert!(node.queue_item_host_focus(&host, Rc::new(|| {})));
        settle_item_effects(&mut dom);
        assert_eq!(pending.events.borrow().len(), 1);
        runtime.unregister_item_cue(&host.registration_key);
        assert!(
            weak_mount.upgrade().is_none(),
            "unmount releases the handle and suspended future"
        );
        pending.pending.set(false);
        pending.waker.borrow_mut().take().unwrap().wake();
        settle_item_effects(&mut dom);
        assert_eq!(
            pending.events.borrow().len(),
            1,
            "unmounted item never scrolls"
        );
        let old = ItemMountMock::default();
        let weak_old = install_item_mock(&runtime, &host, old.clone());
        let retained = Rc::new(());
        let released = Rc::downgrade(&retained);
        assert!(node.queue_item_host_focus(
            &host,
            Rc::new(move || {
                let _ = &retained;
                panic!("replaced mount");
            })
        ));
        let replacement = ItemMountMock::default();
        install_item_mock(&runtime, &host, replacement.clone());
        assert!(weak_old.upgrade().is_none());
        assert!(released.upgrade().is_none());
        settle_item_effects(&mut dom);
        assert!(old.events.borrow().is_empty());
        assert!(replacement.events.borrow().is_empty());
        assert!(node.queue_item_host_focus(&host, Rc::new(|| panic!("dismissed"))));
        assert!(node.request(false, Reason::Trigger, false));
        settle_item_effects(&mut dom);
        assert!(replacement.events.borrow().is_empty());
        assert!(!node.queue_item_host_focus(&host, Rc::new(|| panic!("closed"))));
        fixture.0.open.set(true);
        assert!(node.queue_item_host_focus(&host, Rc::new(|| panic!("old generation"))));
        dom.in_scope(fixture.0.owner.get().unwrap(), || {
            runtime.register(
                node.id.clone(),
                2,
                OverlayKind::Popover,
                CloseBehavior::Both,
                Callbacks {
                    read: Rc::new(|| Some(true)),
                    before: Rc::new(|_| true),
                    apply: Rc::new(|_| {}),
                    key: None,
                },
            );
        });
        settle_item_effects(&mut dom);
        assert!(replacement.events.borrow().is_empty());
        assert!(runtime.0.borrow().owned_item_focus.is_empty());
    }
    #[test]
    fn mounted_item_reentrant_new_intent_cannot_focus_or_publish_the_old_item() {
        let fixture = RequestTaskFixture(Rc::new(RequestTaskState::default()));
        let mut dom = VirtualDom::new_with_props(request_task_root, fixture.clone());
        dom.rebuild_in_place();
        let node = fixture.0.node.borrow().clone().unwrap();
        let runtime = node.runtime().unwrap();
        let hosts = mixed_inventory_facts().hosts;
        let first = ItemMountMock::default();
        let second = ItemMountMock::default();
        install_item_mock(&runtime, &hosts[0], first.clone());
        install_item_mock(&runtime, &hosts[1], second.clone());
        runtime.0.borrow_mut().keyboard = true;
        let nested = node.clone();
        let target = hosts[1].clone();
        assert!(node.queue_item_host_focus(
            &hosts[0],
            Rc::new(move || {
                assert!(nested.queue_item_host_focus(&target, Rc::new(|| {})));
            })
        ));
        settle_item_effects(&mut dom);
        assert!(first.events.borrow().is_empty());
        assert_eq!(second.events.borrow().len(), 2);
        assert_eq!(runtime.item_visual_key().as_deref(), Some("key-b"));
    }
    #[test]
    fn item_focus_requires_explicit_mount_registration_and_rejection_keeps_the_current_intent() {
        let fixture = RequestTaskFixture(Rc::new(RequestTaskState::default()));
        let mut dom = VirtualDom::new_with_props(request_task_root, fixture.clone());
        dom.rebuild_in_place();
        let node = fixture.0.node.borrow().clone().unwrap();
        let runtime = node.runtime().unwrap();
        let released = Rc::new(());
        let weak = Rc::downgrade(&released);
        assert!(!node.queue_item_focus(
            "unregistered".into(),
            true,
            Rc::new(move || {
                let _ = &released;
                panic!("unregistered");
            })
        ));
        assert!(weak.upgrade().is_none());
        assert_eq!(runtime.pending_item_focus_count(), 0);
        let mock = ItemMountMock::default();
        let mut host = mixed_inventory_facts().hosts[0].clone();
        install_item_mock(&runtime, &host, mock.clone());
        // An explicitly registered consumer control resolves by target even
        // when the reported native host has no generated registration key.
        host.registration_key.clear();
        assert!(node.queue_item_host_focus(&host, Rc::new(|| {})));
        assert!(!node.queue_item_focus(
            "unregistered".into(),
            true,
            Rc::new(|| panic!("unregistered"))
        ));
        assert_eq!(runtime.pending_item_focus_count(), 1);
        settle_item_effects(&mut dom);
        assert_eq!(mock.events.borrow().len(), 2);
        assert_eq!(runtime.pending_item_focus_count(), 0);
        runtime.unregister_item_cue("key-a");
        assert!(!node.queue_item_host_focus(&host, Rc::new(|| panic!("unmounted"))));
    }

    #[test]
    fn completion_effect_wakes_an_idle_owner_without_another_state_change() {
        use std::{
            future::Future,
            pin::pin,
            task::{Context, Poll, Waker},
        };
        let fixture = RequestTaskFixture(Rc::new(RequestTaskState::default()));
        let mut dom = VirtualDom::new_with_props(request_task_root, fixture.clone());
        dom.rebuild_in_place();
        settle_item_effects(&mut dom);
        let node = fixture.0.node.borrow().clone().unwrap();
        let mut context = Context::from_waker(Waker::noop());
        assert!(matches!(
            pin!(dom.wait_for_work()).poll(&mut context),
            Poll::Pending
        ));
        let delivered = Rc::new(Cell::new(false));
        let callback = delivered.clone();
        let owner = fixture.0.owner.get().unwrap();
        assert!(node.queue_completion(Rc::new(move || {
            assert_eq!(current_scope_id(), owner);
            callback.set(true);
        })));
        assert!(!delivered.get(), "completion stays asynchronous");
        assert!(
            matches!(
                pin!(dom.wait_for_work()).poll(&mut context),
                Poll::Ready(())
            ),
            "admission itself requests renderer work"
        );
        assert!(
            !delivered.get(),
            "the renderer has not flushed the queued effect yet"
        );
        dom.render_immediate_to_vec();
        // Pinned Dioxus drains post-render effects on its following idle
        // scheduler pass, rather than at the end of render_immediate itself.
        assert!(matches!(
            pin!(dom.wait_for_work()).poll(&mut context),
            Poll::Pending
        ));
        assert!(delivered.get());
    }
    #[test]
    fn completion_effects_restore_registered_owner_and_allow_reentrant_writes() {
        let fixture = RequestTaskFixture(Rc::new(RequestTaskState::default()));
        let mut dom = VirtualDom::new_with_props(request_task_root, fixture.clone());
        dom.rebuild_in_place();
        let node = fixture.0.node.borrow().clone().unwrap();
        let runtime = node.runtime().unwrap();
        let owner = fixture.0.owner.get().unwrap();
        let caller = fixture.0.caller.get().unwrap();
        let delivered = Rc::new(RefCell::new(Vec::new()));
        let first_log = delivered.clone();
        let nested_node = node.clone();
        dom.in_scope(caller, || {
            assert!(node.queue_completion(Rc::new(move || {
                assert_eq!(current_scope_id(), owner);
                assert_eq!(consume_context::<RequestTaskContext>().0, "owner");
                first_log.borrow_mut().push(1);
                let nested_log = first_log.clone();
                assert!(nested_node.queue_completion(Rc::new(move || {
                    assert_eq!(current_scope_id(), owner);
                    nested_log.borrow_mut().push(3);
                })));
            })));
            let second_log = delivered.clone();
            assert!(node.queue_completion(Rc::new(move || {
                assert_eq!(current_scope_id(), owner);
                second_log.borrow_mut().push(2);
            })));
        });
        assert!(
            delivered.borrow().is_empty(),
            "request admission never delivers synchronously"
        );
        assert_eq!(runtime.pending_completions(), 2);
        dom.in_scope(ScopeId::APP, || {
            let mut show = fixture.0.show_caller.get().unwrap();
            show.set(false);
        });
        for _ in 0..3 {
            dom.render_immediate_to_vec();
        }
        assert!(fixture.0.caller_dropped.get());
        assert_eq!(*delivered.borrow(), [1, 2, 3]);
        assert_eq!(runtime.pending_completions(), 0);
        let dropped_log = delivered.clone();
        assert!(node.queue_completion(Rc::new(move || dropped_log.borrow_mut().push(4))));
        assert_eq!(runtime.pending_completions(), 1);
        drop(dom);
        assert_eq!(runtime.pending_completions(), 0);
        assert_eq!(*delivered.borrow(), [1, 2, 3]);
        assert!(!node.queue_completion(Rc::new(|| panic!("dropped owner"))));
    }

    #[test]
    fn completion_effect_cannot_retarget_replaced_registration() {
        let fixture = RequestTaskFixture(Rc::new(RequestTaskState::default()));
        let mut dom = VirtualDom::new_with_props(request_task_root, fixture.clone());
        dom.rebuild_in_place();
        let old = fixture.0.node.borrow().clone().unwrap();
        let runtime = old.runtime().unwrap();
        let owner = fixture.0.owner.get().unwrap();
        let delivered = Rc::new(Cell::new(0));
        let first = delivered.clone();
        assert!(old.queue_completion(Rc::new(move || first.set(first.get() + 1))));
        let replacement = dom.in_scope(owner, || {
            runtime.register(
                old.id.clone(),
                2,
                OverlayKind::Popover,
                CloseBehavior::Both,
                Callbacks {
                    read: Rc::new(|| Some(false)),
                    before: Rc::new(|_| true),
                    apply: Rc::new(|_| {}),
                    key: None,
                },
            )
        });
        assert_eq!(runtime.pending_completions(), 0);
        assert!(!old.queue_completion(Rc::new(|| panic!("old generation"))));
        let current = delivered.clone();
        assert!(replacement.queue_completion(Rc::new(move || current.set(current.get() + 10))));
        assert_eq!(delivered.get(), 0);
        dom.render_immediate_to_vec();
        dom.process_events();
        assert_eq!(delivered.get(), 10);
        assert_eq!(runtime.pending_completions(), 0);
        replacement.unregister();
    }
    #[test]
    fn outside_tab_mounted_exit_runs_after_close_without_item_scroll_or_cue() {
        let fixture = RequestTaskFixture(Rc::new(RequestTaskState::default()));
        let mut dom = VirtualDom::new_with_props(request_task_root, fixture.clone());
        dom.rebuild_in_place();
        let node = fixture.0.node.borrow().clone().unwrap();
        let runtime = node.runtime().unwrap();
        let owner = fixture.0.owner.get().unwrap();
        let destination = ItemMountMock::default();
        runtime.register_item_mount(
            "outside-after",
            "after",
            Rc::new(MountedData::new(destination.clone())),
        );
        let snapshot = runtime.mounted_focus_target("after").unwrap();
        assert!(node.request(false, Reason::Keyboard, false));
        assert!(runtime.queue_exit_focus(node.id(), node.generation, snapshot, Vec::new()));
        assert!(destination.events.borrow().is_empty());
        settle_item_effects(&mut dom);
        assert_eq!(*destination.events.borrow(), [("focus", owner)]);
        assert_eq!(runtime.item_visual_key(), None);
        assert_eq!(runtime.pending_item_focus_count(), 0);
        assert!(runtime.mounted_focus_target("unregistered").is_none());
        node.unregister();
        assert!(runtime.0.borrow().nodes.is_empty());
    }
    #[test]
    fn runtime_capture_uses_registered_physical_ids_and_keeps_cleanup_identity_after_removal() {
        let runtime = RuntimeHandle::new();
        let mounted = Rc::new(MountedData::new(ItemMountMock::default()));
        runtime.register_item_control_mount(
            "capture-key",
            "logical-host",
            "physical-control",
            mounted.clone(),
        );
        assert!(runtime.capture_registered_focus("unregistered").is_none());
        assert!(runtime.capture_registered_focus("logical-host").is_none());
        let captured = runtime
            .capture_registered_focus("physical-control")
            .unwrap();
        assert!(captured.matches_node(&mounted));
        assert!(Rc::ptr_eq(&captured.live_node().unwrap(), &mounted));
        runtime.unregister_item_cue("capture-key");
        assert!(
            runtime
                .capture_registered_focus("physical-control")
                .is_none()
        );
        assert!(captured.live_node().is_none());
        assert!(
            captured.matches_node(&mounted),
            "cleanup still identifies the removed invoker"
        );
        let replacement = Rc::new(MountedData::new(ItemMountMock::default()));
        runtime.register_item_control_mount(
            "capture-key",
            "logical-host",
            "physical-control",
            replacement.clone(),
        );
        assert!(!captured.matches_node(&replacement));
        assert!(captured.live_node().is_none());
        assert!(
            runtime
                .capture_registered_focus("physical-control")
                .unwrap()
                .matches_node(&replacement)
        );
    }
    #[test]
    fn accepted_close_cancels_prior_focus_but_preserves_return_admitted_by_apply() {
        let fixture = RequestTaskFixture(Rc::new(RequestTaskState::default()));
        let mut dom = VirtualDom::new_with_props(request_task_root, fixture.clone());
        dom.rebuild_in_place();
        let node = fixture.0.node.borrow().clone().unwrap();
        let runtime = node.runtime().unwrap();
        let owner = fixture.0.owner.get().unwrap();
        let old = ItemMountMock::default();
        let returned = ItemMountMock::default();
        runtime.register_item_mount(
            "close-old",
            "close-old",
            Rc::new(MountedData::new(old.clone())),
        );
        runtime.register_item_mount(
            "close-return",
            "close-return",
            Rc::new(MountedData::new(returned.clone())),
        );
        let return_target = node.capture_registered_focus("close-return").unwrap();
        dom.in_scope(owner, || {
            let mut callbacks = runtime.entry(node.id()).unwrap().callbacks;
            let apply = callbacks.apply.clone();
            let closing = node.clone();
            callbacks.apply = Rc::new(move |change| {
                apply(change);
                if !change.open {
                    assert!(closing.queue_registered_focus(return_target.clone(), Some(false)));
                }
            });
            runtime.register(
                node.id().into(),
                node.generation,
                OverlayKind::Popover,
                CloseBehavior::Both,
                callbacks,
            );
        });
        assert!(node.queue_registered_focus(
            node.capture_registered_focus("close-old").unwrap(),
            Some(true)
        ));
        assert!(node.request(false, Reason::Trigger, true));
        assert_eq!(
            runtime.pending_item_focus_count(),
            1,
            "the new close return remains admitted"
        );
        settle_item_effects(&mut dom);
        assert!(old.events.borrow().is_empty());
        assert_eq!(*returned.events.borrow(), [("focus", owner)]);
        assert_eq!(runtime.pending_item_focus_count(), 0);
    }
    #[test]
    fn registered_focus_respects_expected_visibility_and_explicit_cancellation() {
        let fixture = RequestTaskFixture(Rc::new(RequestTaskState::default()));
        let mut dom = VirtualDom::new_with_props(request_task_root, fixture.clone());
        dom.rebuild_in_place();
        let node = fixture.0.node.borrow().clone().unwrap();
        let runtime = node.runtime().unwrap();
        let mock = ItemMountMock::default();
        runtime.register_item_mount(
            "registered-control",
            "control",
            Rc::new(MountedData::new(mock.clone())),
        );
        let captured = node.capture_registered_focus("control").unwrap();
        assert!(!node.queue_registered_focus(captured.clone(), Some(false)));
        assert!(node.queue_registered_focus(captured.clone(), Some(true)));
        node.cancel_registered_focus();
        settle_item_effects(&mut dom);
        assert!(mock.events.borrow().is_empty());
        assert!(node.request(false, Reason::Keyboard, false));
        assert!(!node.queue_registered_focus(captured.clone(), Some(true)));
        assert!(node.queue_registered_focus(captured.clone(), Some(false)));
        settle_item_effects(&mut dom);
        assert_eq!(
            *mock.events.borrow(),
            [("focus", fixture.0.owner.get().unwrap())]
        );
        mock.events.borrow_mut().clear();
        assert!(node.queue_registered_focus(captured, Some(false)));
        fixture.0.open.set(true);
        settle_item_effects(&mut dom);
        assert!(
            mock.events.borrow().is_empty(),
            "changed owner visibility rejects stale focus"
        );
        assert!(node.capture_registered_focus("unregistered").is_none());
    }
    #[test]
    fn outside_tab_capture_cannot_follow_a_replacement_or_retain_unmounted_destination() {
        let fixture = RequestTaskFixture(Rc::new(RequestTaskState::default()));
        let mut dom = VirtualDom::new_with_props(request_task_root, fixture.clone());
        dom.rebuild_in_place();
        let node = fixture.0.node.borrow().clone().unwrap();
        let runtime = node.runtime().unwrap();
        let old = Rc::new(MountedData::new(ItemMountMock::default()));
        let weak_old = Rc::downgrade(&old);
        runtime.register_item_mount("outside-after", "after", old);
        let snapshot = runtime.mounted_focus_target("after").unwrap();
        assert!(node.request(false, Reason::Keyboard, false));
        let replacement = ItemMountMock::default();
        runtime.register_item_mount(
            "outside-after",
            "after",
            Rc::new(MountedData::new(replacement.clone())),
        );
        assert!(
            weak_old.upgrade().is_none(),
            "the captured destination is weak"
        );
        assert!(!runtime.queue_exit_focus(node.id(), node.generation, snapshot, Vec::new()));
        let current = runtime.mounted_focus_target("after").unwrap();
        assert!(runtime.queue_exit_focus(node.id(), node.generation, current, Vec::new()));
        runtime.unregister_item_cue("outside-after");
        settle_item_effects(&mut dom);
        assert!(replacement.events.borrow().is_empty());
        assert_eq!(runtime.pending_item_focus_count(), 0);
    }
    #[derive(Clone, Default)]
    struct RovingView(Rc<RovingViewState>);
    #[derive(Default)]
    struct RovingViewState {
        runtime: RefCell<Option<RuntimeHandle>>,
        node: RefCell<Option<NodeHandle>>,
        authored: Cell<Option<Signal<i32>>>,
        open: Cell<bool>,
    }
    fn roving_view(fixture: RovingView) -> Element {
        let runtime = use_hook(RuntimeHandle::new);
        let authored = use_signal(|| 7);
        fixture.0.authored.set(Some(authored));
        *fixture.0.runtime.borrow_mut() = Some(runtime.clone());
        let registered = fixture.clone();
        let registry = runtime.clone();
        let node = use_hook(move || {
            registered.0.open.set(true);
            let read = registered.clone();
            let apply = registered.clone();
            let node = registry.register(
                "reactive-roving".into(),
                1,
                OverlayKind::Dropdown,
                CloseBehavior::Both,
                Callbacks {
                    read: Rc::new(move || Some(read.0.open.get())),
                    before: Rc::new(|_| true),
                    apply: Rc::new(move |change| apply.0.open.set(change.open)),
                    key: None,
                },
            );
            for host in &mixed_inventory_facts().hosts {
                install_item_mock(&registry, host, ItemMountMock::default());
            }
            *registered.0.node.borrow_mut() = Some(node.clone());
            node
        });
        let cleanup = runtime.clone();
        use_drop(move || {
            node.unregister();
            cleanup.unregister_item_cue("key-a");
            cleanup.unregister_item_cue("key-b");
        });
        let first = runtime.item_tabindex("key-a").unwrap_or(*authored.read());
        let second = runtime.item_tabindex("key-b").unwrap_or(4);
        rsx! { button { id: "a", tabindex: first, "A" } button { id: "b", tabindex: second, "B" } }
    }
    fn roving_rendered_tabindex(dom: &VirtualDom, id: &str) -> String {
        let html = dioxus_ssr::render(dom);
        publication_attr(publication_tag(&html, id), "tabindex").unwrap()
    }
    #[test]
    fn roving_revision_renders_overrides_then_restores_current_authored_props() {
        let fixture = RovingView::default();
        let mut dom = VirtualDom::new_with_props(roving_view, fixture.clone());
        dom.rebuild_in_place();
        let runtime = fixture.0.runtime.borrow().clone().unwrap();
        let node = fixture.0.node.borrow().clone().unwrap();
        let raw = mixed_inventory_facts();
        assert_eq!(roving_rendered_tabindex(&dom, "a"), "7");
        assert_eq!(roving_rendered_tabindex(&dom, "b"), "4");
        assert_eq!(node.init_roving(&raw, true, false).unwrap().target, "b");
        assert_eq!(runtime.item_tabindex("key-b"), Some(0));
        assert_eq!(
            roving_rendered_tabindex(&dom, "b"),
            "4",
            "logical state precedes the Dioxus commit"
        );
        settle_item_effects(&mut dom);
        assert_eq!(roving_rendered_tabindex(&dom, "a"), "-1");
        assert_eq!(roving_rendered_tabindex(&dom, "b"), "0");
        dom.in_scope(ScopeId::APP, || fixture.0.authored.get().unwrap().set(9));
        settle_item_effects(&mut dom);
        assert_eq!(roving_rendered_tabindex(&dom, "a"), "-1");
        node.clear_roving();
        settle_item_effects(&mut dom);
        assert_eq!(
            roving_rendered_tabindex(&dom, "a"),
            "9",
            "clear uses the current authored prop rather than a raw baseline"
        );
        assert_eq!(roving_rendered_tabindex(&dom, "b"), "4");
        assert!(node.select_roving(&raw, &ItemRef("key-a".into())));
        assert!(node.request(false, Reason::Trigger, false));
        settle_item_effects(&mut dom);
        assert_eq!(roving_rendered_tabindex(&dom, "a"), "9");
        assert_eq!(runtime.item_tabindex("key-b"), None);
    }
    #[test]
    fn roving_initialization_skips_disabled_and_unregistered_items_and_supports_custom_mounts() {
        let runtime = RuntimeHandle::new();
        let root = model(
            &runtime,
            "roving-root",
            OverlayKind::Dropdown,
            Rc::new(RefCell::new(Vec::new())),
        );
        root.open.set(true);
        let mut raw = mixed_inventory_facts();
        for host in &raw.hosts {
            install_item_mock(&runtime, host, ItemMountMock::default());
        }
        let mut disabled = raw.hosts[0].clone();
        disabled.item = ItemRef("disabled".into());
        disabled.capture = "disabled".into();
        disabled.target = "disabled".into();
        disabled.registration_key = "disabled-key".into();
        disabled.disabled_attribute = true;
        install_item_mock(&runtime, &disabled, ItemMountMock::default());
        let mut custom = disabled.clone();
        custom.item = ItemRef("custom".into());
        custom.capture = "custom".into();
        custom.target = "custom".into();
        custom.registration_key.clear();
        custom.kind = ItemKind::NativeOptIn;
        custom.disabled_attribute = false;
        custom.roving_opt_in = true;
        custom.checked_attribute = true;
        runtime.register_item_mount(
            "custom-key",
            "custom",
            Rc::new(MountedData::new(ItemMountMock::default())),
        );
        let mut unregistered = custom.clone();
        unregistered.item = ItemRef("unregistered".into());
        unregistered.target = "unregistered".into();
        unregistered.capture = "unregistered".into();
        raw.hosts.extend([disabled, custom, unregistered]);
        raw.query_order = raw.hosts.iter().map(|host| host.item.clone()).collect();
        raw.navigation_order = raw.query_order.clone();
        assert_eq!(
            root.node.init_roving(&raw, false, true).unwrap().target,
            "custom"
        );
        assert_eq!(runtime.item_tabindex("custom-key"), Some(0));
        assert_eq!(runtime.item_tabindex("key-a"), Some(-1));
        assert_eq!(runtime.item_tabindex("disabled-key"), None);
        assert!(!root.node.select_roving(&raw, &ItemRef("disabled".into())));
        assert!(
            !root
                .node
                .select_roving(&raw, &ItemRef("unregistered".into()))
        );
        assert_eq!(runtime.item_tabindex("custom-key"), Some(0));
    }
    #[test]
    fn roving_owner_layers_survive_other_owner_clear_and_vetoed_nested_close() {
        let runtime = RuntimeHandle::new();
        let log = Rc::new(RefCell::new(Vec::new()));
        let parent = model(
            &runtime,
            "parent-roving",
            OverlayKind::Dropdown,
            log.clone(),
        );
        let child = model(&runtime, "child-roving", OverlayKind::Dropdown, log.clone());
        child.node.set_ownership(vec![parent.node.clone()], true);
        let raw = mixed_inventory_facts();
        for host in &raw.hosts {
            install_item_mock(&runtime, host, ItemMountMock::default());
        }
        runtime.input(event(
            serde_json::json!({"owner":runtime.owner_id(),"sequence":1,"kind":"observe","nodes":[
            {"id":"parent-roving","generation":1,"item_order":["key-a","key-b"]},
            {"id":"child-roving","generation":1,"item_order":["key-a","key-b"]}]}),
        ));
        assert!(parent.node.request(true, Reason::Trigger, false));
        assert!(child.node.request(true, Reason::Trigger, false));
        assert_eq!(
            runtime.0.borrow().registry.cached_children("parent-roving"),
            ["child-roving"],
            "the fixture must establish trigger discovery before exercising the vetoed nested close"
        );
        assert!(child.node.select_roving(&raw, &ItemRef("key-a".into())));
        assert!(parent.node.select_roving(&raw, &ItemRef("key-b".into())));
        assert_eq!(runtime.item_tabindex("key-a"), Some(-1));
        child.veto.set(true);
        log.borrow_mut().clear();
        assert!(parent.node.request(false, Reason::Trigger, false));
        assert!(!parent.open.get() && child.open.get());
        assert_eq!(
            *log.borrow(),
            [
                "before:parent-roving:false",
                "before:child-roving:false",
                "apply:parent-roving:false"
            ]
        );
        assert_eq!(
            runtime.item_tabindex("key-a"),
            Some(0),
            "clearing the parent reveals the still-open child's own state"
        );
        assert_eq!(runtime.item_tabindex("key-b"), Some(-1));
        child.node.clear_roving();
        assert_eq!(runtime.item_tabindex("key-a"), None);
        assert_eq!(runtime.item_tabindex("key-b"), None);
    }
    #[test]
    fn roving_mode_generation_mount_replacement_and_drop_release_owned_overrides() {
        let runtime = RuntimeHandle::new();
        let root = model(
            &runtime,
            "owned-roving",
            OverlayKind::Dropdown,
            Rc::new(RefCell::new(Vec::new())),
        );
        root.open.set(true);
        let raw = mixed_inventory_facts();
        for host in &raw.hosts {
            install_item_mock(&runtime, host, ItemMountMock::default());
        }
        assert!(root.node.set_navigation_mode("roving-tabindex"));
        assert!(root.node.init_roving(&raw, false, false).is_some());
        assert!(root.node.set_navigation_mode("active-descendant"));
        assert_eq!(runtime.item_tabindex("key-a"), None);
        assert!(root.node.set_navigation_mode("roving-tabindex"));
        assert!(root.node.select_roving(&raw, &ItemRef("key-a".into())));
        install_item_mock(&runtime, &raw.hosts[0], ItemMountMock::default());
        assert_eq!(
            runtime.item_tabindex("key-a"),
            None,
            "a replacement mount cannot inherit the previous handle's override"
        );
        assert_eq!(runtime.item_tabindex("key-b"), Some(-1));
        runtime.unregister_item_cue("key-b");
        assert_eq!(runtime.item_tabindex("key-b"), None);
        install_item_mock(&runtime, &raw.hosts[1], ItemMountMock::default());
        assert!(root.node.init_roving(&raw, false, false).is_some());
        let callbacks = runtime.entry("owned-roving").unwrap().callbacks;
        let replacement = runtime.register(
            "owned-roving".into(),
            2,
            OverlayKind::Dropdown,
            CloseBehavior::Both,
            callbacks,
        );
        assert_eq!(runtime.item_tabindex("key-a"), None);
        assert!(replacement.init_roving(&raw, true, false).is_some());
        root.node.clear_roving();
        assert_eq!(
            runtime.item_tabindex("key-b"),
            Some(0),
            "the stale generation cannot clear its replacement"
        );
        replacement.unregister();
        assert_eq!(runtime.item_tabindex("key-a"), None);
        assert_eq!(runtime.item_tabindex("key-b"), None);
        assert!(runtime.0.borrow().item_tabindices.is_empty());
    }
}

#[cfg(test)]
#[path = "overlay_domain_runtime_tests.rs"]
mod document_tests;
